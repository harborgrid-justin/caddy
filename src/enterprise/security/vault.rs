// CADDY v0.1.5 - Secrets Vault
// Secure storage and management of secrets with versioning and access control

use crate::enterprise::security::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use zeroize::ZeroizeOnDrop;

/// Secret value (automatically zeroized on drop)
#[derive(Clone, ZeroizeOnDrop, Serialize, Deserialize)]
pub struct SecretValue {
    #[serde(with = "hex")]
    data: Vec<u8>,
    pub content_type: String,
    #[zeroize(skip)]
    pub created_at: i64,
}

impl SecretValue {
    pub fn new(data: Vec<u8>, content_type: String) -> Self {
        Self {
            data,
            content_type,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn as_string(&self) -> SecurityResult<String> {
        String::from_utf8(self.data.clone())
            .map_err(|e| SecurityError::InvalidInput(format!("Invalid UTF-8: {}", e)))
    }
}

/// Secret version
#[derive(Clone, Serialize, Deserialize)]
pub struct SecretVersion {
    pub version: u32,
    #[serde(with = "hex")]
    encrypted_value: Vec<u8>,
    #[serde(with = "hex")]
    nonce: Vec<u8>,
    pub metadata: SecretMetadata,
    pub created_at: i64,
    pub created_by: String,
    pub is_active: bool,
}

/// Secret metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub description: String,
    pub tags: HashMap<String, String>,
    pub rotation_interval_days: Option<u32>,
    pub expires_at: Option<i64>,
    pub content_type: String,
}

/// Complete secret with all versions
#[derive(Clone, Serialize, Deserialize)]
pub struct Secret {
    pub path: String,
    pub versions: Vec<SecretVersion>,
    pub current_version: u32,
    pub access_policy: AccessPolicy,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Access control policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub allowed_users: Vec<String>,
    pub allowed_roles: Vec<String>,
    pub allowed_services: Vec<String>,
    pub min_access_level: AccessLevel,
    pub require_mfa: bool,
}

/// Access level enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum AccessLevel {
    Read = 1,
    Write = 2,
    Admin = 3,
    Owner = 4,
}

/// Access context for authorization
#[derive(Debug, Clone)]
pub struct AccessContext {
    pub user: String,
    pub roles: Vec<String>,
    pub service: Option<String>,
    pub access_level: AccessLevel,
    pub mfa_verified: bool,
}

/// Secret rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    pub enabled: bool,
    pub interval_days: u32,
    pub notify_before_days: u32,
    pub auto_rotate: bool,
    pub rotation_lambda: Option<String>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: i64,
    pub action: AuditAction,
    pub secret_path: String,
    pub user: String,
    pub success: bool,
    pub details: HashMap<String, String>,
}

/// Audit action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    Read,
    Write,
    Delete,
    Rotate,
    UpdatePolicy,
    VersionRollback,
}

/// Main secrets vault
pub struct Vault {
    secrets: Arc<RwLock<HashMap<String, Secret>>>,
    encryption_key: Arc<RwLock<Vec<u8>>>,
    rotation_configs: Arc<RwLock<HashMap<String, RotationConfig>>>,
    audit_log: Arc<RwLock<Vec<AuditEntry>>>,
}

impl Vault {
    /// Create a new vault
    pub fn new(master_password: &str) -> SecurityResult<Self> {
        let encryption_key = Self::derive_encryption_key(master_password)?;

        Ok(Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
            encryption_key: Arc::new(RwLock::new(encryption_key)),
            rotation_configs: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Store a secret
    pub fn put_secret(
        &self,
        path: &str,
        value: SecretValue,
        access_policy: AccessPolicy,
        context: &AccessContext,
    ) -> SecurityResult<u32> {
        // Check authorization
        self.check_access(path, context, AccessLevel::Write)?;

        let encryption_key = self.encryption_key.read()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        // Encrypt the secret value
        let (encrypted_value, nonce) = self.encrypt_value(value.data(), &encryption_key)?;

        let version = SecretVersion {
            version: 1,
            encrypted_value,
            nonce,
            metadata: SecretMetadata {
                description: String::new(),
                tags: HashMap::new(),
                rotation_interval_days: None,
                expires_at: None,
                content_type: value.content_type.clone(),
            },
            created_at: chrono::Utc::now().timestamp(),
            created_by: context.user.clone(),
            is_active: true,
        };

        let mut secrets = self.secrets.write()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        let current_version = if let Some(existing) = secrets.get_mut(path) {
            // Update existing secret with new version
            let new_version = existing.current_version + 1;
            let mut new_version_obj = version;
            new_version_obj.version = new_version;

            // Deactivate old version
            if let Some(old) = existing.versions.iter_mut().find(|v| v.version == existing.current_version) {
                old.is_active = false;
            }

            existing.versions.push(new_version_obj);
            existing.current_version = new_version;
            existing.updated_at = chrono::Utc::now().timestamp();

            new_version
        } else {
            // Create new secret
            let secret = Secret {
                path: path.to_string(),
                versions: vec![version],
                current_version: 1,
                access_policy,
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
            };

            secrets.insert(path.to_string(), secret);
            1
        };

        // Log audit entry
        self.log_audit(AuditEntry {
            timestamp: chrono::Utc::now().timestamp(),
            action: AuditAction::Write,
            secret_path: path.to_string(),
            user: context.user.clone(),
            success: true,
            details: HashMap::new(),
        })?;

        Ok(current_version)
    }

    /// Retrieve a secret
    pub fn get_secret(
        &self,
        path: &str,
        version: Option<u32>,
        context: &AccessContext,
    ) -> SecurityResult<SecretValue> {
        // Check authorization
        self.check_access(path, context, AccessLevel::Read)?;

        let secrets = self.secrets.read()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        let secret = secrets.get(path)
            .ok_or_else(|| SecurityError::InvalidInput(format!("Secret not found: {}", path)))?;

        let version_num = version.unwrap_or(secret.current_version);

        let secret_version = secret.versions.iter()
            .find(|v| v.version == version_num)
            .ok_or_else(|| SecurityError::InvalidInput(format!("Version not found: {}", version_num)))?;

        let encryption_key = self.encryption_key.read()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        // Decrypt the secret value
        let decrypted = self.decrypt_value(
            &secret_version.encrypted_value,
            &secret_version.nonce,
            &encryption_key,
        )?;

        // Log audit entry
        self.log_audit(AuditEntry {
            timestamp: chrono::Utc::now().timestamp(),
            action: AuditAction::Read,
            secret_path: path.to_string(),
            user: context.user.clone(),
            success: true,
            details: HashMap::new(),
        })?;

        Ok(SecretValue::new(decrypted, secret_version.metadata.content_type.clone()))
    }

    /// List secret versions
    pub fn list_versions(&self, path: &str, context: &AccessContext) -> SecurityResult<Vec<u32>> {
        self.check_access(path, context, AccessLevel::Read)?;

        let secrets = self.secrets.read()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        let secret = secrets.get(path)
            .ok_or_else(|| SecurityError::InvalidInput(format!("Secret not found: {}", path)))?;

        Ok(secret.versions.iter().map(|v| v.version).collect())
    }

    /// Delete a secret
    pub fn delete_secret(&self, path: &str, context: &AccessContext) -> SecurityResult<()> {
        self.check_access(path, context, AccessLevel::Admin)?;

        let mut secrets = self.secrets.write()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        secrets.remove(path)
            .ok_or_else(|| SecurityError::InvalidInput(format!("Secret not found: {}", path)))?;

        // Log audit entry
        self.log_audit(AuditEntry {
            timestamp: chrono::Utc::now().timestamp(),
            action: AuditAction::Delete,
            secret_path: path.to_string(),
            user: context.user.clone(),
            success: true,
            details: HashMap::new(),
        })?;

        Ok(())
    }

    /// Rotate a secret
    pub fn rotate_secret(
        &self,
        path: &str,
        new_value: SecretValue,
        context: &AccessContext,
    ) -> SecurityResult<u32> {
        self.check_access(path, context, AccessLevel::Write)?;

        let version = self.put_secret(path, new_value, self.get_access_policy(path)?, context)?;

        // Log audit entry
        self.log_audit(AuditEntry {
            timestamp: chrono::Utc::now().timestamp(),
            action: AuditAction::Rotate,
            secret_path: path.to_string(),
            user: context.user.clone(),
            success: true,
            details: HashMap::new(),
        })?;

        Ok(version)
    }

    /// Set rotation configuration
    pub fn set_rotation_config(&self, path: &str, config: RotationConfig) -> SecurityResult<()> {
        let mut configs = self.rotation_configs.write()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        configs.insert(path.to_string(), config);
        Ok(())
    }

    /// Check if secret needs rotation
    pub fn needs_rotation(&self, path: &str) -> SecurityResult<bool> {
        let secrets = self.secrets.read()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        let secret = secrets.get(path)
            .ok_or_else(|| SecurityError::InvalidInput(format!("Secret not found: {}", path)))?;

        let configs = self.rotation_configs.read()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        if let Some(config) = configs.get(path) {
            if !config.enabled || !config.auto_rotate {
                return Ok(false);
            }

            let now = chrono::Utc::now().timestamp();
            let age_days = (now - secret.updated_at) / 86400;

            Ok(age_days >= config.interval_days as i64)
        } else {
            Ok(false)
        }
    }

    /// Update access policy
    pub fn update_access_policy(
        &self,
        path: &str,
        policy: AccessPolicy,
        context: &AccessContext,
    ) -> SecurityResult<()> {
        self.check_access(path, context, AccessLevel::Admin)?;

        let mut secrets = self.secrets.write()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        let secret = secrets.get_mut(path)
            .ok_or_else(|| SecurityError::InvalidInput(format!("Secret not found: {}", path)))?;

        secret.access_policy = policy;

        // Log audit entry
        self.log_audit(AuditEntry {
            timestamp: chrono::Utc::now().timestamp(),
            action: AuditAction::UpdatePolicy,
            secret_path: path.to_string(),
            user: context.user.clone(),
            success: true,
            details: HashMap::new(),
        })?;

        Ok(())
    }

    /// List all secret paths
    pub fn list_secrets(&self, context: &AccessContext) -> SecurityResult<Vec<String>> {
        let secrets = self.secrets.read()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        let mut paths: Vec<String> = secrets.keys()
            .filter(|path| self.check_access(path, context, AccessLevel::Read).is_ok())
            .cloned()
            .collect();

        paths.sort();
        Ok(paths)
    }

    /// Get audit log
    pub fn get_audit_log(&self, limit: Option<usize>) -> SecurityResult<Vec<AuditEntry>> {
        let audit_log = self.audit_log.read()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        let entries = if let Some(n) = limit {
            audit_log.iter().rev().take(n).cloned().collect()
        } else {
            audit_log.clone()
        };

        Ok(entries)
    }

    // Helper methods

    fn check_access(
        &self,
        path: &str,
        context: &AccessContext,
        required_level: AccessLevel,
    ) -> SecurityResult<()> {
        let secrets = self.secrets.read()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        if let Some(secret) = secrets.get(path) {
            let policy = &secret.access_policy;

            // Check access level
            if context.access_level < required_level {
                return Err(SecurityError::AccessDenied(
                    format!("Insufficient access level: required {:?}, have {:?}",
                            required_level, context.access_level)
                ));
            }

            // Check MFA requirement
            if policy.require_mfa && !context.mfa_verified {
                return Err(SecurityError::AccessDenied("MFA verification required".to_string()));
            }

            // Check user/role/service authorization
            let is_authorized = policy.allowed_users.contains(&context.user)
                || context.roles.iter().any(|r| policy.allowed_roles.contains(r))
                || context.service.as_ref().map_or(false, |s| policy.allowed_services.contains(s));

            if !is_authorized {
                return Err(SecurityError::AccessDenied("User not authorized".to_string()));
            }
        }

        Ok(())
    }

    fn get_access_policy(&self, path: &str) -> SecurityResult<AccessPolicy> {
        let secrets = self.secrets.read()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        let secret = secrets.get(path)
            .ok_or_else(|| SecurityError::InvalidInput(format!("Secret not found: {}", path)))?;

        Ok(secret.access_policy.clone())
    }

    fn derive_encryption_key(password: &str) -> SecurityResult<Vec<u8>> {
        // Simulate PBKDF2 key derivation
        let mut key = vec![0u8; 32];
        let password_bytes = password.as_bytes();
        let salt = b"caddy_vault_salt_v1";

        for i in 0..32 {
            let mut val = password_bytes[i % password_bytes.len()];
            for _ in 0..100000 {
                val = val.wrapping_mul(251).wrapping_add(salt[i % salt.len()]);
            }
            key[i] = val;
        }

        Ok(key)
    }

    fn encrypt_value(&self, value: &[u8], key: &[u8]) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
        // Generate nonce
        let mut nonce = vec![0u8; 12];
        self.fill_random(&mut nonce)?;

        // Simulate AES-256-GCM encryption
        let mut encrypted = value.to_vec();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= key[i % key.len()] ^ nonce[i % nonce.len()];
        }

        Ok((encrypted, nonce))
    }

    fn decrypt_value(&self, encrypted: &[u8], nonce: &[u8], key: &[u8]) -> SecurityResult<Vec<u8>> {
        // Simulate AES-256-GCM decryption
        let mut decrypted = encrypted.to_vec();
        for (i, byte) in decrypted.iter_mut().enumerate() {
            *byte ^= key[i % key.len()] ^ nonce[i % nonce.len()];
        }

        Ok(decrypted)
    }

    fn log_audit(&self, entry: AuditEntry) -> SecurityResult<()> {
        let mut audit_log = self.audit_log.write()
            .map_err(|e| SecurityError::AccessDenied(format!("Lock error: {}", e)))?;

        audit_log.push(entry);
        Ok(())
    }

    fn fill_random(&self, buf: &mut [u8]) -> SecurityResult<()> {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};

        let rs = RandomState::new();
        for (i, byte) in buf.iter_mut().enumerate() {
            let mut hasher = rs.build_hasher();
            (i, std::time::SystemTime::now()).hash(&mut hasher);
            *byte = (hasher.finish() & 0xFF) as u8;
        }
        Ok(())
    }
}

mod hex {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex_encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex_decode(&s).map_err(serde::de::Error::custom)
    }

    fn hex_encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("Odd hex string length".to_string());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_storage() {
        let vault = Vault::new("test_password").unwrap();
        let context = AccessContext {
            user: "test_user".to_string(),
            roles: vec!["admin".to_string()],
            service: None,
            access_level: AccessLevel::Owner,
            mfa_verified: true,
        };

        let policy = AccessPolicy {
            allowed_users: vec!["test_user".to_string()],
            allowed_roles: vec![],
            allowed_services: vec![],
            min_access_level: AccessLevel::Read,
            require_mfa: false,
        };

        let secret = SecretValue::new(b"my_secret_value".to_vec(), "text/plain".to_string());
        vault.put_secret("test/secret", secret, policy, &context).unwrap();

        let retrieved = vault.get_secret("test/secret", None, &context).unwrap();
        assert_eq!(b"my_secret_value", retrieved.data());
    }

    #[test]
    fn test_secret_versioning() {
        let vault = Vault::new("test_password").unwrap();
        let context = AccessContext {
            user: "test_user".to_string(),
            roles: vec!["admin".to_string()],
            service: None,
            access_level: AccessLevel::Owner,
            mfa_verified: true,
        };

        let policy = AccessPolicy {
            allowed_users: vec!["test_user".to_string()],
            allowed_roles: vec![],
            allowed_services: vec![],
            min_access_level: AccessLevel::Read,
            require_mfa: false,
        };

        let v1 = SecretValue::new(b"version1".to_vec(), "text/plain".to_string());
        let v2 = SecretValue::new(b"version2".to_vec(), "text/plain".to_string());

        vault.put_secret("test/versioned", v1, policy.clone(), &context).unwrap();
        vault.put_secret("test/versioned", v2, policy, &context).unwrap();

        let versions = vault.list_versions("test/versioned", &context).unwrap();
        assert_eq!(vec![1, 2], versions);
    }
}
