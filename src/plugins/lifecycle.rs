//! Plugin lifecycle management
//!
//! Manages the complete lifecycle of a plugin from initialization through
//! activation, running, suspension, and termination.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Lifecycle errors
#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error("Invalid state transition: {from:?} -> {to:?}")]
    InvalidTransition { from: PluginState, to: PluginState },

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Plugin not initialized")]
    NotInitialized,

    #[error("Plugin already running")]
    AlreadyRunning,

    #[error("Plugin not running")]
    NotRunning,
}

pub type LifecycleResult<T> = Result<T, LifecycleError>;

/// Plugin lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginState {
    /// Plugin is being loaded
    Loading,

    /// Plugin is loaded but not yet initialized
    Loaded,

    /// Plugin is initializing
    Initializing,

    /// Plugin is initialized and ready to run
    Ready,

    /// Plugin is starting up
    Starting,

    /// Plugin is running normally
    Running,

    /// Plugin is paused/suspended
    Suspended,

    /// Plugin is stopping
    Stopping,

    /// Plugin has stopped
    Stopped,

    /// Plugin encountered an error
    Error,

    /// Plugin is being unloaded
    Unloading,

    /// Plugin has been unloaded
    Unloaded,
}

impl PluginState {
    /// Check if transition to another state is valid
    pub fn can_transition_to(&self, target: PluginState) -> bool {
        use PluginState::*;

        match (self, target) {
            // Loading can go to Loaded or Error
            (Loading, Loaded) | (Loading, Error) => true,

            // Loaded can go to Initializing or Unloading
            (Loaded, Initializing) | (Loaded, Unloading) => true,

            // Initializing can go to Ready or Error
            (Initializing, Ready) | (Initializing, Error) => true,

            // Ready can go to Starting or Unloading
            (Ready, Starting) | (Ready, Unloading) => true,

            // Starting can go to Running or Error
            (Starting, Running) | (Starting, Error) => true,

            // Running can go to Suspended, Stopping, or Error
            (Running, Suspended) | (Running, Stopping) | (Running, Error) => true,

            // Suspended can go back to Running or to Stopping
            (Suspended, Running) | (Suspended, Stopping) => true,

            // Stopping can go to Stopped or Error
            (Stopping, Stopped) | (Stopping, Error) => true,

            // Stopped can go to Starting (restart) or Unloading
            (Stopped, Starting) | (Stopped, Unloading) => true,

            // Error can go to Stopping or Unloading (recovery)
            (Error, Stopping) | (Error, Unloading) => true,

            // Unloading can go to Unloaded
            (Unloading, Unloaded) => true,

            // Stay in same state is always valid
            (a, b) if a == b => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    /// Check if state is active (can process requests)
    pub fn is_active(&self) -> bool {
        matches!(self, PluginState::Running)
    }

    /// Check if state is transitional
    pub fn is_transitional(&self) -> bool {
        matches!(
            self,
            PluginState::Loading
                | PluginState::Initializing
                | PluginState::Starting
                | PluginState::Stopping
                | PluginState::Unloading
        )
    }

    /// Check if state is terminal
    pub fn is_terminal(&self) -> bool {
        matches!(self, PluginState::Unloaded | PluginState::Error)
    }
}

/// Lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub from_state: PluginState,
    pub to_state: PluginState,
    pub message: Option<String>,
}

/// Plugin lifecycle manager
#[derive(Debug)]
pub struct PluginLifecycle {
    /// Plugin identifier
    plugin_id: String,

    /// Current state
    state: PluginState,

    /// State history
    history: Vec<LifecycleEvent>,

    /// Time when current state was entered
    state_entered_at: Instant,

    /// Total running time
    total_running_time: Duration,

    /// Number of restarts
    restart_count: u32,

    /// Number of errors
    error_count: u32,
}

impl PluginLifecycle {
    /// Create a new lifecycle manager
    pub fn new(plugin_id: String) -> Self {
        Self {
            plugin_id,
            state: PluginState::Loading,
            history: Vec::new(),
            state_entered_at: Instant::now(),
            total_running_time: Duration::default(),
            restart_count: 0,
            error_count: 0,
        }
    }

    /// Get current state
    pub fn state(&self) -> PluginState {
        self.state
    }

    /// Check if plugin is active
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Get plugin ID
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Transition to a new state
    pub fn transition(&mut self, target_state: PluginState) -> LifecycleResult<()> {
        self.transition_with_message(target_state, None)
    }

    /// Transition to a new state with a message
    pub fn transition_with_message(
        &mut self,
        target_state: PluginState,
        message: Option<String>,
    ) -> LifecycleResult<()> {
        // Check if transition is valid
        if !self.state.can_transition_to(target_state) {
            return Err(LifecycleError::InvalidTransition {
                from: self.state,
                to: target_state,
            });
        }

        // Record time in previous state
        let time_in_state = self.state_entered_at.elapsed();
        if self.state == PluginState::Running {
            self.total_running_time += time_in_state;
        }

        // Update counters
        if target_state == PluginState::Error {
            self.error_count += 1;
        }

        if target_state == PluginState::Starting && self.state == PluginState::Stopped {
            self.restart_count += 1;
        }

        // Create event
        let event = LifecycleEvent {
            timestamp: chrono::Utc::now(),
            from_state: self.state,
            to_state: target_state,
            message,
        };

        // Update state
        let old_state = self.state;
        self.state = target_state;
        self.state_entered_at = Instant::now();

        // Record event
        self.history.push(event);

        log::info!(
            "Plugin {} transitioned: {:?} -> {:?}",
            self.plugin_id,
            old_state,
            target_state
        );

        Ok(())
    }

    /// Initialize the plugin
    pub fn initialize(&mut self) -> LifecycleResult<()> {
        self.transition(PluginState::Initializing)?;
        self.transition(PluginState::Ready)
    }

    /// Start the plugin
    pub fn start(&mut self) -> LifecycleResult<()> {
        self.transition(PluginState::Starting)?;
        self.transition(PluginState::Running)
    }

    /// Stop the plugin
    pub fn stop(&mut self) -> LifecycleResult<()> {
        self.transition(PluginState::Stopping)?;
        self.transition(PluginState::Stopped)
    }

    /// Suspend the plugin
    pub fn suspend(&mut self) -> LifecycleResult<()> {
        if !self.is_active() {
            return Err(LifecycleError::NotRunning);
        }
        self.transition(PluginState::Suspended)
    }

    /// Resume the plugin
    pub fn resume(&mut self) -> LifecycleResult<()> {
        if self.state != PluginState::Suspended {
            return Err(LifecycleError::OperationFailed(
                "Plugin is not suspended".to_string(),
            ));
        }
        self.transition(PluginState::Running)
    }

    /// Mark plugin as having an error
    pub fn error(&mut self, message: String) -> LifecycleResult<()> {
        self.transition_with_message(PluginState::Error, Some(message))
    }

    /// Unload the plugin
    pub fn unload(&mut self) -> LifecycleResult<()> {
        self.transition(PluginState::Unloading)?;
        self.transition(PluginState::Unloaded)
    }

    /// Get lifecycle statistics
    pub fn stats(&self) -> LifecycleStats {
        LifecycleStats {
            current_state: self.state,
            time_in_current_state: self.state_entered_at.elapsed(),
            total_running_time: self.total_running_time,
            restart_count: self.restart_count,
            error_count: self.error_count,
            total_transitions: self.history.len(),
        }
    }

    /// Get state history
    pub fn history(&self) -> &[LifecycleEvent] {
        &self.history
    }

    /// Get recent history (last N events)
    pub fn recent_history(&self, count: usize) -> Vec<LifecycleEvent> {
        let start = self.history.len().saturating_sub(count);
        self.history[start..].to_vec()
    }

    /// Clear history (keep only recent events)
    pub fn clear_old_history(&mut self, keep_count: usize) {
        if self.history.len() > keep_count {
            let drain_count = self.history.len() - keep_count;
            self.history.drain(0..drain_count);
        }
    }
}

/// Lifecycle statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleStats {
    pub current_state: PluginState,
    pub time_in_current_state: Duration,
    pub total_running_time: Duration,
    pub restart_count: u32,
    pub error_count: u32,
    pub total_transitions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        assert!(PluginState::Loading.can_transition_to(PluginState::Loaded));
        assert!(PluginState::Loaded.can_transition_to(PluginState::Initializing));
        assert!(PluginState::Ready.can_transition_to(PluginState::Starting));
        assert!(PluginState::Running.can_transition_to(PluginState::Suspended));

        assert!(!PluginState::Loading.can_transition_to(PluginState::Running));
        assert!(!PluginState::Unloaded.can_transition_to(PluginState::Running));
    }

    #[test]
    fn test_lifecycle_flow() {
        let mut lifecycle = PluginLifecycle::new("test-plugin".to_string());

        assert_eq!(lifecycle.state(), PluginState::Loading);

        lifecycle.transition(PluginState::Loaded).unwrap();
        lifecycle.initialize().unwrap();
        assert_eq!(lifecycle.state(), PluginState::Ready);

        lifecycle.start().unwrap();
        assert!(lifecycle.is_active());

        lifecycle.suspend().unwrap();
        assert!(!lifecycle.is_active());

        lifecycle.resume().unwrap();
        assert!(lifecycle.is_active());

        lifecycle.stop().unwrap();
        assert_eq!(lifecycle.state(), PluginState::Stopped);
    }

    #[test]
    fn test_invalid_transition() {
        let mut lifecycle = PluginLifecycle::new("test-plugin".to_string());

        let result = lifecycle.transition(PluginState::Running);
        assert!(result.is_err());
    }

    #[test]
    fn test_state_properties() {
        assert!(PluginState::Running.is_active());
        assert!(!PluginState::Stopped.is_active());

        assert!(PluginState::Loading.is_transitional());
        assert!(!PluginState::Running.is_transitional());

        assert!(PluginState::Unloaded.is_terminal());
        assert!(!PluginState::Running.is_terminal());
    }
}
