/**
 * CADDY v0.4.0 - User Management Module
 *
 * Enterprise-grade user management system with:
 * - Complete user lifecycle management
 * - RBAC with role inheritance
 * - Team management and hierarchy
 * - SSO integration (SAML, OAuth, LDAP)
 * - Activity tracking and audit logging
 * - Bulk operations with progress tracking
 * - GDPR compliance features
 * - Real-time updates
 */

// ============================================================================
// Type Exports
// ============================================================================

export type {
  // Core User Types
  User,
  UserStatus,
  UserMetadata,
  UserPreferences,
  UserSecuritySettings,
  GDPRConsent,

  // RBAC Types
  Role,
  Permission,
  PermissionAction,
  PermissionScope,
  PermissionCondition,
  RoleConstraint,
  RoleConstraintType,
  RoleAssignment,

  // Team Types
  Team,
  TeamType,
  TeamMember,
  TeamRole,
  TeamSettings,

  // SSO Types
  SSOProvider,
  SSOProviderType,
  SSOConfig,
  SAMLConfig,
  OAuthConfig,
  LDAPConfig,
  AttributeMapping,
  ProvisioningConfig,

  // Activity & Audit Types
  ActivityLog,
  ActivityCategory,
  ActivitySeverity,
  ActivityDetails,
  ChangeRecord,
  UserSession,

  // Invitation Types
  UserInvitation,
  InvitationStatus,

  // Bulk Operation Types
  BulkOperation,
  BulkOperationType,
  BulkOperationStatus,
  BulkOperationError,
  ImportUserData,

  // API Request/Response Types
  ListUsersRequest,
  ListUsersResponse,
  CreateUserRequest,
  UpdateUserRequest,
  UserFilterOptions,
  PermissionCheckRequest,
  PermissionCheckResponse,

  // Analytics Types
  UserStatistics,
  UserActivitySummary,

  // Event Types
  UserEvent,
  UserEventType,

  // Utility Types
  DeepPartial,
  SortDirection,
  PaginationParams,
  SortParams,
  SearchParams,
} from './types';

// ============================================================================
// Error Exports
// ============================================================================

export {
  UserManagementError,
  PermissionDeniedError,
  UserNotFoundError,
  RoleNotFoundError,
  TeamNotFoundError,
  ValidationError,
} from './types';

// ============================================================================
// Hook Exports
// ============================================================================

export {
  // Configuration
  configureUserManagement,

  // User Hooks
  useUsers,
  useUser,
  useCreateUser,

  // Role & Permission Hooks
  useRoles,
  usePermissions,

  // Team Hooks
  useTeams,

  // Activity & Session Hooks
  useUserActivity,
  useUserSessions,
  useUserActivity,

  // Invitation Hooks
  useInvitations,

  // Bulk Operation Hooks
  useBulkOperations,

  // SSO Hooks
  useSSOProviders,

  // Statistics Hooks
  useUserStatistics,

  // Real-time Hooks
  useRealtimeUserEvents,

  // Utility Hooks
  useDebounce,
  useLocalStorage,
} from './UserHooks';

// ============================================================================
// Component Exports
// ============================================================================

export { UserList } from './UserList';
export { UserProfile } from './UserProfile';
export { UserCreate } from './UserCreate';
export { UserRoles } from './UserRoles';
export { UserPermissions } from './UserPermissions';
export { TeamManagement } from './TeamManagement';
export { UserActivity } from './UserActivity';
export { UserBulkActions } from './UserBulkActions';
export { SSOConfiguration } from './SSOConfiguration';

// ============================================================================
// Default Exports
// ============================================================================

import { UserList } from './UserList';
import { UserProfile } from './UserProfile';
import { UserCreate } from './UserCreate';
import { UserRoles } from './UserRoles';
import { UserPermissions } from './UserPermissions';
import { TeamManagement } from './TeamManagement';
import { UserActivity } from './UserActivity';
import { UserBulkActions } from './UserBulkActions';
import { SSOConfiguration } from './SSOConfiguration';
import { configureUserManagement } from './UserHooks';

export default {
  // Components
  UserList,
  UserProfile,
  UserCreate,
  UserRoles,
  UserPermissions,
  TeamManagement,
  UserActivity,
  UserBulkActions,
  SSOConfiguration,

  // Configuration
  configure: configureUserManagement,
};

// ============================================================================
// Module Metadata
// ============================================================================

export const MODULE_INFO = {
  name: 'CADDY User Management',
  version: '0.4.0',
  description: 'Enterprise-grade user management system',
  features: [
    'User lifecycle management',
    'RBAC with inheritance',
    'Team management',
    'SSO integration',
    'Activity tracking',
    'Bulk operations',
    'GDPR compliance',
    'Real-time updates',
  ],
  author: 'CADDY Platform Team',
  license: 'Proprietary',
};

// ============================================================================
// Usage Examples (for documentation)
// ============================================================================

/**
 * @example Basic Usage
 * ```tsx
 * import { UserList, UserProfile, configureUserManagement } from '@caddy/users';
 *
 * // Configure the module
 * configureUserManagement({
 *   apiUrl: 'https://api.example.com',
 *   wsUrl: 'wss://api.example.com',
 *   token: 'your-auth-token',
 *   tenantId: 'your-tenant-id',
 * });
 *
 * // Use components
 * function App() {
 *   return (
 *     <div>
 *       <UserList onUserSelect={(user) => console.log(user)} />
 *     </div>
 *   );
 * }
 * ```
 *
 * @example Using Hooks
 * ```tsx
 * import { useUsers, useCreateUser } from '@caddy/users';
 *
 * function UserManagement() {
 *   const { data, loading, error } = useUsers({ page: 1, pageSize: 25 });
 *   const { createUser } = useCreateUser();
 *
 *   const handleCreate = async () => {
 *     await createUser({
 *       username: 'john.doe',
 *       email: 'john@example.com',
 *       firstName: 'John',
 *       lastName: 'Doe',
 *     });
 *   };
 *
 *   return <div>{/* Your UI *\/}</div>;
 * }
 * ```
 *
 * @example RBAC Management
 * ```tsx
 * import { UserRoles, UserPermissions } from '@caddy/users';
 *
 * function RBACManagement({ userId }: { userId: string }) {
 *   return (
 *     <div>
 *       <UserRoles userId={userId} />
 *       <UserPermissions userId={userId} />
 *     </div>
 *   );
 * }
 * ```
 *
 * @example SSO Configuration
 * ```tsx
 * import { SSOConfiguration } from '@caddy/users';
 *
 * function SSOSettings() {
 *   return (
 *     <SSOConfiguration
 *       onProviderCreate={(provider) => console.log('Created:', provider)}
 *     />
 *   );
 * }
 * ```
 *
 * @example Bulk Operations
 * ```tsx
 * import { UserBulkActions } from '@caddy/users';
 *
 * function BulkImport() {
 *   return (
 *     <UserBulkActions
 *       onOperationComplete={(op) => console.log('Operation complete:', op)}
 *     />
 *   );
 * }
 * ```
 */
