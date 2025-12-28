//! Tenant Context Management
//!
//! Provides thread-local tenant context, async task propagation, and tenant isolation.
//! Implements a hierarchical tenant model: Organization -> Workspace -> Project

use std::cell::RefCell;
use std::sync::Arc;
use parking_lot::RwLock;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Tenant context errors
#[derive(Error, Debug)]
pub enum ContextError {
    #[error("No tenant context available")]
    NoContext,

    #[error("Invalid tenant hierarchy: {0}")]
    InvalidHierarchy(String),

    #[error("Context already set")]
    AlreadySet,

    #[error("Permission denied for tenant: {0}")]
    PermissionDenied(String),
}

pub type ContextResult<T> = Result<T, ContextError>;

/// Tenant identifier with hierarchical structure
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId {
    /// Organization ID (top level)
    pub org_id: Uuid,
    /// Workspace ID (optional, within org)
    pub workspace_id: Option<Uuid>,
    /// Project ID (optional, within workspace)
    pub project_id: Option<Uuid>,
}

impl TenantId {
    /// Create a new organization-level tenant ID
    pub fn new_org(org_id: Uuid) -> Self {
        Self {
            org_id,
            workspace_id: None,
            project_id: None,
        }
    }

    /// Create a new workspace-level tenant ID
    pub fn new_workspace(org_id: Uuid, workspace_id: Uuid) -> Self {
        Self {
            org_id,
            workspace_id: Some(workspace_id),
            project_id: None,
        }
    }

    /// Create a new project-level tenant ID
    pub fn new_project(org_id: Uuid, workspace_id: Uuid, project_id: Uuid) -> Self {
        Self {
            org_id,
            workspace_id: Some(workspace_id),
            project_id: Some(project_id),
        }
    }

    /// Get the hierarchy level (1=org, 2=workspace, 3=project)
    pub fn level(&self) -> u8 {
        if self.project_id.is_some() {
            3
        } else if self.workspace_id.is_some() {
            2
        } else {
            1
        }
    }

    /// Check if this tenant is a parent or ancestor of another
    pub fn is_ancestor_of(&self, other: &TenantId) -> bool {
        if self.org_id != other.org_id {
            return false;
        }

        match (self.workspace_id, other.workspace_id) {
            (None, _) => true, // Org is ancestor of everything in org
            (Some(ws1), Some(ws2)) if ws1 == ws2 => {
                // Same workspace, check project
                self.project_id.is_none() || self.project_id == other.project_id
            }
            (Some(_), None) => false,
            (Some(_), Some(_)) => false,
        }
    }

    /// Get the organization-level tenant ID
    pub fn org_tenant(&self) -> TenantId {
        Self::new_org(self.org_id)
    }

    /// Get the workspace-level tenant ID (if applicable)
    pub fn workspace_tenant(&self) -> Option<TenantId> {
        self.workspace_id.map(|ws_id| Self::new_workspace(self.org_id, ws_id))
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "org:{}", self.org_id)?;
        if let Some(ws) = self.workspace_id {
            write!(f, "/ws:{}", ws)?;
        }
        if let Some(proj) = self.project_id {
            write!(f, "/proj:{}", proj)?;
        }
        Ok(())
    }
}

/// Tenant context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    /// Tenant identifier
    pub tenant_id: TenantId,
    /// User ID associated with this context
    pub user_id: Option<Uuid>,
    /// Request/correlation ID for tracing
    pub request_id: Uuid,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Context creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TenantContext {
    /// Create a new tenant context
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            user_id: None,
            request_id: Uuid::new_v4(),
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create a new context with user ID
    pub fn with_user(tenant_id: TenantId, user_id: Uuid) -> Self {
        Self {
            tenant_id,
            user_id: Some(user_id),
            request_id: Uuid::new_v4(),
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: Uuid) -> Self {
        self.request_id = request_id;
        self
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

// Thread-local storage for tenant context
thread_local! {
    static TENANT_CONTEXT: RefCell<Option<TenantContext>> = RefCell::new(None);
}

/// Global tenant context registry for async task propagation
static CONTEXT_REGISTRY: once_cell::sync::Lazy<DashMap<Uuid, Arc<TenantContext>>> =
    once_cell::sync::Lazy::new(DashMap::new);

/// Set the current tenant context for this thread
pub fn set_context(context: TenantContext) -> ContextResult<()> {
    TENANT_CONTEXT.with(|ctx| {
        let mut ctx_ref = ctx.borrow_mut();
        if ctx_ref.is_some() {
            return Err(ContextError::AlreadySet);
        }

        // Register in global registry for async propagation
        let request_id = context.request_id;
        CONTEXT_REGISTRY.insert(request_id, Arc::new(context.clone()));

        *ctx_ref = Some(context);
        Ok(())
    })
}

/// Get the current tenant context
pub fn get_context() -> ContextResult<TenantContext> {
    TENANT_CONTEXT.with(|ctx| {
        ctx.borrow()
            .as_ref()
            .cloned()
            .ok_or(ContextError::NoContext)
    })
}

/// Get the current tenant ID
pub fn get_tenant_id() -> ContextResult<TenantId> {
    get_context().map(|ctx| ctx.tenant_id)
}

/// Clear the current tenant context
pub fn clear_context() {
    TENANT_CONTEXT.with(|ctx| {
        if let Some(context) = ctx.borrow_mut().take() {
            // Remove from global registry
            CONTEXT_REGISTRY.remove(&context.request_id);
        }
    });
}

/// Execute a function with a specific tenant context
pub fn with_context<F, R>(context: TenantContext, f: F) -> ContextResult<R>
where
    F: FnOnce() -> R,
{
    set_context(context)?;
    let result = f();
    clear_context();
    Ok(result)
}

/// Context guard that ensures context cleanup on drop
pub struct ContextGuard {
    request_id: Uuid,
}

impl ContextGuard {
    /// Create a new context guard
    pub fn new(context: TenantContext) -> ContextResult<Self> {
        let request_id = context.request_id;
        set_context(context)?;
        Ok(Self { request_id })
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        clear_context();
    }
}

/// Async-aware context propagation
pub mod async_context {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    /// Future wrapper that propagates tenant context
    pub struct TenantFuture<F> {
        inner: F,
        context: Arc<TenantContext>,
        context_set: bool,
    }

    impl<F> TenantFuture<F> {
        /// Create a new tenant future with the given context
        pub fn new(future: F, context: TenantContext) -> Self {
            Self {
                inner: future,
                context: Arc::new(context),
                context_set: false,
            }
        }
    }

    impl<F: Future> Future for TenantFuture<F> {
        type Output = F::Output;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = unsafe { self.get_unchecked_mut() };

            // Set context before polling
            if !this.context_set {
                let _ = set_context((*this.context).clone());
                this.context_set = true;
            }

            // Poll the inner future
            let inner = unsafe { Pin::new_unchecked(&mut this.inner) };
            let result = inner.poll(cx);

            // Clear context if complete
            if result.is_ready() {
                clear_context();
            }

            result
        }
    }

    impl<F> Drop for TenantFuture<F> {
        fn drop(&mut self) {
            if self.context_set {
                clear_context();
            }
        }
    }

    /// Extension trait for futures to add tenant context
    pub trait TenantFutureExt: Future + Sized {
        /// Run this future with the current tenant context
        fn with_tenant_context(self) -> ContextResult<TenantFuture<Self>> {
            let context = get_context()?;
            Ok(TenantFuture::new(self, context))
        }

        /// Run this future with a specific tenant context
        fn with_specific_context(self, context: TenantContext) -> TenantFuture<Self> {
            TenantFuture::new(self, context)
        }
    }

    impl<F: Future> TenantFutureExt for F {}

    /// Spawn an async task with tenant context propagation
    pub fn spawn_with_context<F>(context: TenantContext, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let tenant_future = TenantFuture::new(future, context);
        tokio::spawn(tenant_future)
    }
}

/// Tenant context switching for administrative operations
pub struct ContextSwitcher {
    original_context: Option<TenantContext>,
    restored: bool,
}

impl ContextSwitcher {
    /// Switch to a new tenant context, saving the current one
    pub fn switch_to(new_context: TenantContext) -> ContextResult<Self> {
        let original_context = TENANT_CONTEXT.with(|ctx| ctx.borrow_mut().take());
        set_context(new_context)?;
        Ok(Self {
            original_context,
            restored: false,
        })
    }

    /// Restore the original context
    pub fn restore(mut self) {
        self.restored = true;
        clear_context();
        if let Some(ctx) = self.original_context.take() {
            let _ = set_context(ctx);
        }
    }
}

impl Drop for ContextSwitcher {
    fn drop(&mut self) {
        if !self.restored {
            clear_context();
            if let Some(ctx) = self.original_context.take() {
                let _ = set_context(ctx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_id_hierarchy() {
        let org_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let proj_id = Uuid::new_v4();

        let org_tenant = TenantId::new_org(org_id);
        let ws_tenant = TenantId::new_workspace(org_id, ws_id);
        let proj_tenant = TenantId::new_project(org_id, ws_id, proj_id);

        assert_eq!(org_tenant.level(), 1);
        assert_eq!(ws_tenant.level(), 2);
        assert_eq!(proj_tenant.level(), 3);

        assert!(org_tenant.is_ancestor_of(&ws_tenant));
        assert!(org_tenant.is_ancestor_of(&proj_tenant));
        assert!(ws_tenant.is_ancestor_of(&proj_tenant));
        assert!(!proj_tenant.is_ancestor_of(&ws_tenant));
    }

    #[test]
    fn test_context_set_and_get() {
        clear_context(); // Ensure clean state

        let tenant_id = TenantId::new_org(Uuid::new_v4());
        let context = TenantContext::new(tenant_id.clone());

        assert!(set_context(context).is_ok());

        let retrieved = get_context().unwrap();
        assert_eq!(retrieved.tenant_id, tenant_id);

        clear_context();
        assert!(get_context().is_err());
    }

    #[test]
    fn test_context_guard() {
        clear_context();

        let tenant_id = TenantId::new_org(Uuid::new_v4());
        let context = TenantContext::new(tenant_id.clone());

        {
            let _guard = ContextGuard::new(context).unwrap();
            assert!(get_context().is_ok());
        }

        // Context should be cleared after guard is dropped
        assert!(get_context().is_err());
    }

    #[test]
    fn test_with_context() {
        clear_context();

        let tenant_id = TenantId::new_org(Uuid::new_v4());
        let context = TenantContext::new(tenant_id.clone());

        let result = with_context(context, || {
            let ctx = get_context().unwrap();
            ctx.tenant_id.clone()
        }).unwrap();

        assert_eq!(result, tenant_id);
        assert!(get_context().is_err()); // Context cleared after execution
    }

    #[test]
    fn test_context_switcher() {
        clear_context();

        let tenant1 = TenantId::new_org(Uuid::new_v4());
        let tenant2 = TenantId::new_org(Uuid::new_v4());

        let ctx1 = TenantContext::new(tenant1.clone());
        let ctx2 = TenantContext::new(tenant2.clone());

        set_context(ctx1).unwrap();

        {
            let _switcher = ContextSwitcher::switch_to(ctx2).unwrap();
            let current = get_context().unwrap();
            assert_eq!(current.tenant_id, tenant2);
        }

        // Original context should be restored
        let current = get_context().unwrap();
        assert_eq!(current.tenant_id, tenant1);

        clear_context();
    }

    #[tokio::test]
    async fn test_async_context_propagation() {
        clear_context();

        let tenant_id = TenantId::new_org(Uuid::new_v4());
        let context = TenantContext::new(tenant_id.clone());

        async fn test_async_fn() -> TenantId {
            get_tenant_id().unwrap()
        }

        use async_context::TenantFutureExt;

        set_context(context).unwrap();
        let result = test_async_fn().with_tenant_context().unwrap().await;

        assert_eq!(result, tenant_id);
        clear_context();
    }
}
