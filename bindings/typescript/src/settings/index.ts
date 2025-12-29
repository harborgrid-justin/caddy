/**
 * CADDY v0.4.0 Enterprise Settings Module
 * Comprehensive settings and configuration management system
 *
 * @module settings
 * @version 0.4.0
 * @license Enterprise
 */

// Main Layout
export { default as SettingsLayout } from './SettingsLayout';

// Settings Components
export { default as GeneralSettings } from './GeneralSettings';
export { default as SecuritySettings } from './SecuritySettings';
export { default as NotificationSettings } from './NotificationSettings';
export { default as IntegrationSettings } from './IntegrationSettings';
export { default as BillingSettings } from './BillingSettings';
export { default as TeamSettings } from './TeamSettings';
export { default as AdvancedSettings } from './AdvancedSettings';
export { default as SettingsSearch } from './SettingsSearch';

// Type Exports
export type {
  // Base Types
  SettingsBase,

  // General Settings
  GeneralSettings as GeneralSettingsData,
  BrandingSettings,
  MaintenanceSettings,

  // Security Settings
  SecuritySettings as SecuritySettingsData,
  PasswordPolicy,
  TwoFactorAuthSettings,
  SSOSettings,
  SSOProvider,
  SessionManagement,
  IPWhitelistSettings,
  AuditLogSettings,

  // Notification Settings
  NotificationSettings as NotificationSettingsData,
  EmailNotificationSettings,
  EmailTemplate,
  SMSNotificationSettings,
  PushNotificationSettings,
  InAppNotificationSettings,
  NotificationChannel,
  NotificationCondition,

  // Integration Settings
  IntegrationSettings as IntegrationSettingsData,
  Integration,
  IntegrationCredentials,
  Webhook,
  APILimits,

  // Billing Settings
  BillingSettings as BillingSettingsData,
  Subscription,
  SubscriptionAddon,
  PaymentMethod,
  BillingAddress,
  Invoice,
  InvoiceItem,
  UsageMetrics,

  // Team Settings
  TeamSettings as TeamSettingsData,
  TeamMember,
  Role,
  Permission,
  PermissionCondition,
  Invitation,
  TeamGroup,

  // Advanced Settings
  AdvancedSettings as AdvancedSettingsData,
  APIKey,
  AdvancedWebhook,
  RetryPolicy,
  WebhookFilter,
  WebhookDelivery,
  CustomDomain,
  DNSRecord,
  CORSSettings,
  LoggingSettings,
  LogDestination,
  PerformanceSettings,

  // Form and UI Types
  ValidationError,
  FormState,
  SettingsHistory,
  SettingsChange,
  ConfirmationDialog,
  AutoSaveState,
  UndoRedoState,
  SearchResult,
  SearchMatch,
  SettingsTab,
  ToastNotification,
} from './types';
