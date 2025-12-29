export { UserManagementError, PermissionDeniedError, UserNotFoundError, RoleNotFoundError, TeamNotFoundError, ValidationError, } from './types';
export { configureUserManagement, useUsers, useUser, useCreateUser, useRoles, usePermissions, useTeams, useUserActivity, useUserSessions, useUserActivity, useInvitations, useBulkOperations, useSSOProviders, useUserStatistics, useRealtimeUserEvents, useDebounce, useLocalStorage, } from './UserHooks';
export { UserList } from './UserList';
export { UserProfile } from './UserProfile';
export { UserCreate } from './UserCreate';
export { UserRoles } from './UserRoles';
export { UserPermissions } from './UserPermissions';
export { TeamManagement } from './TeamManagement';
export { UserActivity } from './UserActivity';
export { UserBulkActions } from './UserBulkActions';
export { SSOConfiguration } from './SSOConfiguration';
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
    UserList,
    UserProfile,
    UserCreate,
    UserRoles,
    UserPermissions,
    TeamManagement,
    UserActivity,
    UserBulkActions,
    SSOConfiguration,
    configure: configureUserManagement,
};
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
//# sourceMappingURL=index.js.map