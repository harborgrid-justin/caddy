/**
 * CADDY Enterprise Authentication Types
 * TypeScript definitions for authentication system
 */

// ============================================================================
// SSO Types
// ============================================================================

export type SsoProvider =
  | 'saml2'
  | 'oauth2'
  | 'oidc'
  | 'active_directory'
  | 'ldap'
  | 'google_workspace'
  | 'azure_ad'
  | 'okta';

export interface SsoConfig {
  provider: SsoProvider;
  provider_name: string;
  client_id: string;
  client_secret?: string;
  authorization_endpoint?: string;
  token_endpoint?: string;
  userinfo_endpoint?: string;
  jwks_uri?: string;
  issuer?: string;
  redirect_uri: string;
  scopes: string[];
  saml_entity_id?: string;
  saml_sso_url?: string;
  saml_certificate?: string;
  ldap_url?: string;
  ldap_bind_dn?: string;
  ldap_bind_password?: string;
  ldap_base_dn?: string;
  ldap_user_filter?: string;
  attribute_mapping: Record<string, string>;
  auto_provision: boolean;
  default_role?: string;
  enabled: boolean;
}

export interface SsoTestResult {
  success: boolean;
  error?: string;
  user_info?: Record<string, any>;
}

// ============================================================================
// RBAC Types
// ============================================================================

export type Action =
  | 'create'
  | 'read'
  | 'update'
  | 'delete'
  | 'execute'
  | 'share'
  | 'export'
  | 'import'
  | 'approve'
  | 'publish'
  | 'archive'
  | 'restore';

export type ResourceType =
  | 'project'
  | 'drawing'
  | 'model'
  | 'layer'
  | 'template'
  | 'user'
  | 'role'
  | 'team'
  | 'organization'
  | 'settings'
  | 'audit_log'
  | 'report'
  | 'plugin'
  | 'workflow';

export type BuiltInRole =
  | 'admin'
  | 'manager'
  | 'editor'
  | 'viewer'
  | 'auditor'
  | 'guest';

export interface Permission {
  resource_type: ResourceType;
  action: Action;
  resource_id?: string;
  conditions?: Record<string, string>;
}

export interface Role {
  id: string;
  name: string;
  description: string;
  built_in?: BuiltInRole;
  permissions: Permission[];
  parent_roles: string[];
  organization_id?: string;
  created_by: string;
  created_at: string;
  updated_at: string;
  is_active: boolean;
}

export interface RoleAssignment {
  user_id: string;
  role_id: string;
  assigned_by: string;
  assigned_at: string;
}

// ============================================================================
// MFA Types
// ============================================================================

export type MfaMethod = 'totp' | 'sms' | 'email' | 'fido2' | 'backup_code';

export interface TotpConfig {
  secret?: string;
  algorithm: string;
  digits: number;
  period: number;
  issuer: string;
  account_name: string;
  provisioning_url: string;
}

export interface SmsConfig {
  phone_number?: string;
  country_code: string;
  masked_number: string;
  provider: string;
}

export interface EmailConfig {
  email: string;
  masked_email: string;
}

export interface Fido2Credential {
  credential_id: string;
  public_key: number[];
  authenticator_type: string;
  device_name: string;
  registered_at: string;
  last_used?: string;
  counter: number;
}

export interface BackupCode {
  code?: string;
  used: boolean;
  used_at?: string;
}

export interface MfaEnrollment {
  id: string;
  user_id: string;
  method: MfaMethod;
  totp_config?: TotpConfig;
  sms_config?: SmsConfig;
  email_config?: EmailConfig;
  fido2_credentials: Fido2Credential[];
  backup_codes: BackupCode[];
  enabled: boolean;
  is_primary: boolean;
  enrolled_at: string;
  last_verified?: string;
}

export interface MfaVerificationRequest {
  user_id: string;
  method: MfaMethod;
  code: string;
  remember_device: boolean;
  device_fingerprint?: string;
}

export interface MfaResult {
  success: boolean;
  error?: string;
  device_token?: string;
  verified_at: string;
}

// ============================================================================
// Session Types
// ============================================================================

export interface Session {
  id: string;
  user_id: string;
  ip_address: string;
  user_agent: string;
  device_fingerprint?: string;
  device_name?: string;
  created_at: string;
  last_accessed: string;
  expires_at: string;
  is_active: boolean;
  revoked: boolean;
  revoked_at?: string;
  revoked_reason?: string;
  mfa_verified: boolean;
  mfa_verified_at?: string;
}

export interface SessionStats {
  total_sessions: number;
  active_sessions: number;
  expired_sessions: number;
  revoked_sessions: number;
  sessions_by_device: Record<string, number>;
  sessions_by_ip: Record<string, number>;
}

// ============================================================================
// Audit Types
// ============================================================================

export type AuditEventType =
  // Authentication
  | 'login_success'
  | 'login_failure'
  | 'logout'
  | 'password_changed'
  | 'password_reset_requested'
  | 'password_reset_completed'
  // MFA
  | 'mfa_enabled'
  | 'mfa_disabled'
  | 'mfa_verified'
  | 'mfa_failed'
  | 'backup_code_used'
  // Session
  | 'session_created'
  | 'session_expired'
  | 'session_revoked'
  | 'session_refreshed'
  // Authorization
  | 'permission_granted'
  | 'permission_denied'
  | 'role_assigned'
  | 'role_revoked'
  | 'role_created'
  | 'role_modified'
  | 'role_deleted'
  // Data access
  | 'sensitive_data_accessed'
  | 'data_exported'
  | 'data_imported'
  | 'data_deleted'
  | 'data_modified'
  // Administrative
  | 'user_created'
  | 'user_modified'
  | 'user_deleted'
  | 'user_locked'
  | 'user_unlocked'
  | 'organization_created'
  | 'organization_modified'
  | 'settings_changed'
  // Security
  | 'suspicious_activity'
  | 'anomaly_detected'
  | 'brute_force_attempt'
  | 'ip_blocked'
  | 'account_compromised'
  | 'security_alert_triggered'
  // SSO
  | 'sso_login_success'
  | 'sso_login_failure'
  | 'sso_configured'
  | 'sso_disabled';

export type AuditSeverity = 'info' | 'warning' | 'error' | 'critical';

export interface AuditEvent {
  id: string;
  event_type: AuditEventType;
  severity: AuditSeverity;
  user_id?: string;
  username?: string;
  target_user_id?: string;
  resource_type?: string;
  resource_id?: string;
  ip_address: string;
  user_agent: string;
  session_id?: string;
  description: string;
  metadata: Record<string, any>;
  timestamp: string;
  organization_id?: string;
  success: boolean;
  error_message?: string;
  geo_location?: string;
  risk_score?: number;
}

export interface AuditReport {
  start_time: string;
  end_time: string;
  total_events: number;
  failed_events: number;
  security_events: number;
  unique_users: number;
  unique_ips: number;
  event_counts: Record<AuditEventType, number>;
  severity_counts: Record<AuditSeverity, number>;
  top_users: [string, number][];
  top_ips: [string, number][];
}

export interface AnomalyDetection {
  anomaly_detected: boolean;
  anomaly_type?: string;
  risk_score: number;
  reasons: string[];
  recommended_action?: string;
}

// ============================================================================
// User Context
// ============================================================================

export interface UserContext {
  user_id: string;
  username: string;
  email: string;
  roles: string[];
  permissions: string[];
  organization_id?: string;
  department?: string;
  ip_address: string;
  user_agent: string;
  session_id: string;
  authenticated_at: string;
}

// ============================================================================
// Auth Configuration
// ============================================================================

export interface AuthConfig {
  session_timeout: number;
  refresh_token_lifetime: number;
  max_concurrent_sessions: number;
  max_failed_attempts: number;
  lockout_duration: number;
  password_min_length: number;
  password_require_uppercase: boolean;
  password_require_lowercase: boolean;
  password_require_numbers: boolean;
  password_require_special: boolean;
  mfa_required_for_admin: boolean;
  mfa_required_for_all: boolean;
  rate_limit_enabled: boolean;
  rate_limit_requests: number;
  rate_limit_window_secs: number;
  anomaly_detection_enabled: boolean;
  jwt_issuer: string;
  jwt_audience: string;
}

// ============================================================================
// API Response Types
// ============================================================================

export interface AuthResult {
  success: boolean;
  user_id?: string;
  session_token?: string;
  refresh_token?: string;
  error?: string;
  mfa_required: boolean;
  mfa_methods: string[];
  expires_at?: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

// ============================================================================
// Component Props
// ============================================================================

export interface SSOConfigurationProps {
  onSave?: (config: SsoConfig) => void;
  onCancel?: () => void;
  initialConfig?: Partial<SsoConfig>;
  mode?: 'create' | 'edit';
}

export interface RoleManagerProps {
  organizationId?: string;
  onRoleCreated?: (role: Role) => void;
  onRoleUpdated?: (role: Role) => void;
  onRoleDeleted?: (roleId: string) => void;
}

export interface MFASetupProps {
  userId: string;
  onComplete?: (enrollment: MfaEnrollment) => void;
  onCancel?: () => void;
  availableMethods?: MfaMethod[];
}

export interface SessionManagerProps {
  userId?: string;
  showAllSessions?: boolean;
  onSessionRevoked?: (sessionId: string) => void;
}
