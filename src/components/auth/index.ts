/**
 * CADDY Enterprise Authentication Components
 * v0.3.0
 *
 * Complete authentication system with SSO, RBAC, MFA, and session management
 */

// Components
export { SSOConfiguration } from './SSOConfiguration';
export { RoleManager } from './RoleManager';
export { MFASetup } from './MFASetup';
export { SessionManager } from './SessionManager';

// Types
export type {
  // SSO Types
  SsoProvider,
  SsoConfig,
  SsoTestResult,
  SSOConfigurationProps,

  // RBAC Types
  Action,
  ResourceType,
  BuiltInRole,
  Permission,
  Role,
  RoleAssignment,
  RoleManagerProps,

  // MFA Types
  MfaMethod,
  TotpConfig,
  SmsConfig,
  EmailConfig,
  Fido2Credential,
  BackupCode,
  MfaEnrollment,
  MfaVerificationRequest,
  MfaResult,
  MFASetupProps,

  // Session Types
  Session,
  SessionStats,
  SessionManagerProps,

  // Audit Types
  AuditEventType,
  AuditSeverity,
  AuditEvent,
  AuditReport,
  AnomalyDetection,

  // User Context
  UserContext,

  // Configuration
  AuthConfig,

  // API Response Types
  AuthResult,
  ApiResponse,
} from './types';
