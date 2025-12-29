export interface User {
    id: string;
    tenantId: string;
    username: string;
    email: string;
    firstName: string;
    lastName: string;
    displayName: string;
    avatar?: string;
    phoneNumber?: string;
    timezone: string;
    locale: string;
    status: UserStatus;
    roles: string[];
    teams: string[];
    attributes: Record<string, any>;
    metadata: UserMetadata;
    preferences: UserPreferences;
    securitySettings: UserSecuritySettings;
    gdprConsent: GDPRConsent;
    createdAt: string;
    updatedAt: string;
    lastLoginAt?: string;
    lastActivityAt?: string;
    deactivatedAt?: string;
    deletedAt?: string;
}
export type UserStatus = 'active' | 'inactive' | 'pending' | 'suspended' | 'locked' | 'deactivated';
export interface UserMetadata {
    emailVerified: boolean;
    phoneVerified: boolean;
    mfaEnabled: boolean;
    ssoEnabled: boolean;
    ssoProvider?: string;
    passwordLastChanged?: string;
    failedLoginAttempts: number;
    lastFailedLoginAt?: string;
    accountLockedUntil?: string;
    source: 'manual' | 'import' | 'sso' | 'invitation';
    externalId?: string;
    department?: string;
    jobTitle?: string;
    manager?: string;
    employeeId?: string;
    costCenter?: string;
}
export interface UserPreferences {
    theme: 'light' | 'dark' | 'auto';
    language: string;
    dateFormat: string;
    timeFormat: '12h' | '24h';
    emailNotifications: boolean;
    pushNotifications: boolean;
    smsNotifications: boolean;
    weekStartsOn: 0 | 1 | 6;
    dashboardLayout?: string;
    customSettings: Record<string, any>;
}
export interface UserSecuritySettings {
    requireMfa: boolean;
    allowedIpRanges: string[];
    sessionTimeout: number;
    maxConcurrentSessions: number;
    requirePasswordChange: boolean;
    passwordExpiryDays?: number;
    allowApiAccess: boolean;
    allowMobileAccess: boolean;
}
export interface GDPRConsent {
    dataProcessing: boolean;
    marketing: boolean;
    analytics: boolean;
    thirdPartySharing: boolean;
    consentDate: string;
    consentVersion: string;
    ipAddress: string;
    userAgent: string;
}
export interface Role {
    id: string;
    tenantId: string;
    name: string;
    displayName: string;
    description: string;
    type: 'system' | 'custom';
    level: number;
    parentRoles: string[];
    permissions: Permission[];
    constraints: RoleConstraint[];
    isInheritable: boolean;
    isSystem: boolean;
    metadata: {
        color?: string;
        icon?: string;
        category?: string;
    };
    createdAt: string;
    updatedAt: string;
    createdBy: string;
    updatedBy: string;
}
export interface Permission {
    id: string;
    resource: string;
    action: PermissionAction;
    scope: PermissionScope;
    conditions?: PermissionCondition[];
    effect: 'allow' | 'deny';
    priority: number;
}
export type PermissionAction = 'create' | 'read' | 'update' | 'delete' | 'list' | 'execute' | 'manage' | 'approve' | 'publish' | '*';
export type PermissionScope = 'own' | 'team' | 'department' | 'organization' | 'tenant' | 'global';
export interface PermissionCondition {
    field: string;
    operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'in' | 'nin' | 'contains';
    value: any;
}
export interface RoleConstraint {
    type: RoleConstraintType;
    config: Record<string, any>;
}
export type RoleConstraintType = 'time_window' | 'date_range' | 'ip_whitelist' | 'location' | 'mfa_required' | 'approval_required' | 'max_sessions';
export interface RoleAssignment {
    id: string;
    userId: string;
    roleId: string;
    tenantId: string;
    scope?: string;
    conditions?: Record<string, any>;
    expiresAt?: string;
    assignedBy: string;
    assignedAt: string;
    revokedBy?: string;
    revokedAt?: string;
}
export interface Team {
    id: string;
    tenantId: string;
    name: string;
    displayName: string;
    description: string;
    type: TeamType;
    parentTeamId?: string;
    path: string;
    level: number;
    members: TeamMember[];
    settings: TeamSettings;
    metadata: {
        department?: string;
        location?: string;
        costCenter?: string;
        manager?: string;
    };
    createdAt: string;
    updatedAt: string;
    createdBy: string;
}
export type TeamType = 'department' | 'project' | 'functional' | 'virtual' | 'organization';
export interface TeamMember {
    userId: string;
    teamRole: TeamRole;
    isPrimary: boolean;
    joinedAt: string;
    invitedBy: string;
    permissions: string[];
}
export type TeamRole = 'owner' | 'admin' | 'manager' | 'member' | 'viewer' | 'guest';
export interface TeamSettings {
    visibility: 'public' | 'private' | 'secret';
    joinPolicy: 'open' | 'approval' | 'invitation';
    allowMemberInvite: boolean;
    maxMembers?: number;
    inheritParentPermissions: boolean;
    notificationPreferences: {
        memberJoined: boolean;
        memberLeft: boolean;
        roleChanged: boolean;
    };
}
export interface SSOProvider {
    id: string;
    tenantId: string;
    name: string;
    displayName: string;
    type: SSOProviderType;
    enabled: boolean;
    config: SSOConfig;
    attributeMapping: AttributeMapping;
    provisioning: ProvisioningConfig;
    metadata: {
        logoUrl?: string;
        supportEmail?: string;
        documentation?: string;
    };
    createdAt: string;
    updatedAt: string;
}
export type SSOProviderType = 'saml' | 'oauth' | 'oidc' | 'ldap' | 'azure_ad' | 'google' | 'okta';
export type SSOConfig = SAMLConfig | OAuthConfig | LDAPConfig;
export interface SAMLConfig {
    type: 'saml';
    entityId: string;
    ssoUrl: string;
    sloUrl?: string;
    certificate: string;
    signRequests: boolean;
    encryptAssertions: boolean;
    nameIdFormat: 'email' | 'persistent' | 'transient';
    authnContext: string[];
    attributeStatements: Record<string, string>;
}
export interface OAuthConfig {
    type: 'oauth' | 'oidc';
    clientId: string;
    clientSecret: string;
    authorizationUrl: string;
    tokenUrl: string;
    userInfoUrl?: string;
    jwksUrl?: string;
    scopes: string[];
    responseType: string;
    grantType: string;
    usePKCE: boolean;
}
export interface LDAPConfig {
    type: 'ldap';
    host: string;
    port: number;
    useTLS: boolean;
    bindDN: string;
    bindPassword: string;
    baseDN: string;
    userSearchFilter: string;
    groupSearchFilter?: string;
    attributes: {
        username: string;
        email: string;
        firstName: string;
        lastName: string;
        memberOf?: string;
    };
    groupMembershipAttribute?: string;
}
export interface AttributeMapping {
    userId: string;
    username: string;
    email: string;
    firstName: string;
    lastName: string;
    displayName?: string;
    groups?: string;
    roles?: string;
    department?: string;
    customAttributes: Record<string, string>;
}
export interface ProvisioningConfig {
    enabled: boolean;
    createUsers: boolean;
    updateUsers: boolean;
    deactivateUsers: boolean;
    syncGroups: boolean;
    syncRoles: boolean;
    defaultRoles: string[];
    defaultTeams: string[];
}
export interface ActivityLog {
    id: string;
    tenantId: string;
    userId: string;
    actorId: string;
    actorType: 'user' | 'system' | 'api';
    action: string;
    resource: string;
    resourceId?: string;
    category: ActivityCategory;
    severity: ActivitySeverity;
    status: 'success' | 'failure' | 'pending';
    details: ActivityDetails;
    metadata: {
        ipAddress: string;
        userAgent: string;
        location?: string;
        deviceId?: string;
        sessionId?: string;
    };
    timestamp: string;
}
export type ActivityCategory = 'authentication' | 'authorization' | 'user_management' | 'role_management' | 'team_management' | 'sso' | 'data_access' | 'configuration' | 'security';
export type ActivitySeverity = 'info' | 'warning' | 'error' | 'critical';
export interface ActivityDetails {
    description: string;
    changes?: ChangeRecord[];
    errorMessage?: string;
    errorCode?: string;
    duration?: number;
    additionalData?: Record<string, any>;
}
export interface ChangeRecord {
    field: string;
    oldValue?: any;
    newValue?: any;
}
export interface UserSession {
    id: string;
    userId: string;
    tenantId: string;
    deviceId: string;
    deviceName: string;
    deviceType: 'desktop' | 'mobile' | 'tablet' | 'unknown';
    browser: string;
    os: string;
    ipAddress: string;
    location?: {
        country: string;
        region: string;
        city: string;
    };
    status: 'active' | 'expired' | 'terminated';
    lastActivityAt: string;
    createdAt: string;
    expiresAt: string;
    terminatedAt?: string;
    terminatedBy?: string;
    terminationReason?: string;
}
export interface UserInvitation {
    id: string;
    tenantId: string;
    email: string;
    firstName?: string;
    lastName?: string;
    roles: string[];
    teams: string[];
    invitedBy: string;
    status: InvitationStatus;
    token: string;
    message?: string;
    expiresAt: string;
    createdAt: string;
    acceptedAt?: string;
    revokedAt?: string;
    revokedBy?: string;
}
export type InvitationStatus = 'pending' | 'accepted' | 'expired' | 'revoked';
export interface BulkOperation {
    id: string;
    tenantId: string;
    type: BulkOperationType;
    status: BulkOperationStatus;
    initiatedBy: string;
    totalItems: number;
    processedItems: number;
    successfulItems: number;
    failedItems: number;
    errors: BulkOperationError[];
    fileUrl?: string;
    resultUrl?: string;
    startedAt: string;
    completedAt?: string;
    metadata: Record<string, any>;
}
export type BulkOperationType = 'import_users' | 'export_users' | 'update_users' | 'deactivate_users' | 'assign_roles' | 'remove_roles' | 'add_to_teams' | 'remove_from_teams';
export type BulkOperationStatus = 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled';
export interface BulkOperationError {
    row: number;
    userId?: string;
    field?: string;
    error: string;
    severity: 'error' | 'warning';
}
export interface ImportUserData {
    username: string;
    email: string;
    firstName: string;
    lastName: string;
    roles?: string[];
    teams?: string[];
    department?: string;
    jobTitle?: string;
    manager?: string;
    attributes?: Record<string, any>;
}
export interface ListUsersRequest {
    page?: number;
    pageSize?: number;
    search?: string;
    status?: UserStatus[];
    roles?: string[];
    teams?: string[];
    department?: string;
    sortBy?: string;
    sortOrder?: 'asc' | 'desc';
    filters?: Record<string, any>;
}
export interface ListUsersResponse {
    users: User[];
    total: number;
    page: number;
    pageSize: number;
    totalPages: number;
    hasMore: boolean;
}
export interface CreateUserRequest {
    username: string;
    email: string;
    firstName: string;
    lastName: string;
    password?: string;
    roles?: string[];
    teams?: string[];
    attributes?: Record<string, any>;
    sendInvitation?: boolean;
    skipEmailVerification?: boolean;
}
export interface UpdateUserRequest {
    firstName?: string;
    lastName?: string;
    displayName?: string;
    email?: string;
    phoneNumber?: string;
    timezone?: string;
    locale?: string;
    avatar?: string;
    status?: UserStatus;
    roles?: string[];
    teams?: string[];
    attributes?: Record<string, any>;
    preferences?: Partial<UserPreferences>;
    securitySettings?: Partial<UserSecuritySettings>;
}
export interface UserFilterOptions {
    status?: UserStatus[];
    roles?: string[];
    teams?: string[];
    departments?: string[];
    dateRange?: {
        field: 'createdAt' | 'lastLoginAt' | 'lastActivityAt';
        start: string;
        end: string;
    };
    customFilters?: Record<string, any>;
}
export interface PermissionCheckRequest {
    userId: string;
    resource: string;
    action: PermissionAction;
    scope?: PermissionScope;
    context?: Record<string, any>;
}
export interface PermissionCheckResponse {
    allowed: boolean;
    reason?: string;
    matchedPermissions: Permission[];
    deniedBy?: Permission;
}
export interface UserStatistics {
    total: number;
    active: number;
    inactive: number;
    pending: number;
    suspended: number;
    locked: number;
    byRole: Record<string, number>;
    byTeam: Record<string, number>;
    byDepartment: Record<string, number>;
    bySource: Record<string, number>;
    mfaEnabled: number;
    ssoEnabled: number;
    recentLogins: number;
    newThisMonth: number;
    growthRate: number;
}
export interface UserActivitySummary {
    userId: string;
    loginCount: number;
    lastLogin: string;
    sessionCount: number;
    averageSessionDuration: number;
    totalActions: number;
    topActions: Array<{
        action: string;
        count: number;
    }>;
    topResources: Array<{
        resource: string;
        count: number;
    }>;
    securityEvents: number;
    failedLogins: number;
}
export interface UserEvent {
    type: UserEventType;
    userId: string;
    tenantId: string;
    data: any;
    timestamp: string;
}
export type UserEventType = 'user_created' | 'user_updated' | 'user_deleted' | 'user_login' | 'user_logout' | 'user_status_changed' | 'role_assigned' | 'role_removed' | 'team_joined' | 'team_left' | 'permission_changed' | 'session_started' | 'session_ended' | 'mfa_enabled' | 'mfa_disabled';
export declare class UserManagementError extends Error {
    code: string;
    statusCode?: number | undefined;
    details?: any | undefined;
    constructor(message: string, code: string, statusCode?: number | undefined, details?: any | undefined);
}
export declare class PermissionDeniedError extends UserManagementError {
    constructor(message: string, details?: any);
}
export declare class UserNotFoundError extends UserManagementError {
    constructor(userId: string);
}
export declare class RoleNotFoundError extends UserManagementError {
    constructor(roleId: string);
}
export declare class TeamNotFoundError extends UserManagementError {
    constructor(teamId: string);
}
export declare class ValidationError extends UserManagementError {
    fields?: Record<string, string[]> | undefined;
    constructor(message: string, fields?: Record<string, string[]> | undefined);
}
export type DeepPartial<T> = {
    [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};
export type SortDirection = 'asc' | 'desc';
export interface PaginationParams {
    page: number;
    pageSize: number;
}
export interface SortParams {
    sortBy: string;
    sortOrder: SortDirection;
}
export interface SearchParams {
    search: string;
    searchFields?: string[];
}
//# sourceMappingURL=types.d.ts.map