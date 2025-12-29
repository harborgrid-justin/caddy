import { User, Role, Team, Permission, ActivityLog, UserSession, UserInvitation, BulkOperation, SSOProvider, ListUsersRequest, ListUsersResponse, CreateUserRequest, UpdateUserRequest, PermissionCheckResponse, UserStatistics, UserEvent } from './types';
interface UserManagementConfig {
    apiUrl: string;
    wsUrl?: string;
    token?: string;
    tenantId?: string;
}
export declare const configureUserManagement: (config: Partial<UserManagementConfig>) => void;
export declare function useUsers(params?: ListUsersRequest): {
    data: ListUsersResponse | null;
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
};
export declare function useUser(userId: string | null): {
    user: User | null;
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
    updateUser: (data: UpdateUserRequest) => Promise<User | undefined>;
};
export declare function useCreateUser(): {
    createUser: (data: CreateUserRequest) => Promise<User>;
    loading: boolean;
    error: Error | null;
};
export declare function useRoles(): {
    roles: Role[];
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
    assignRole: (userId: string, roleId: string) => Promise<void>;
    removeRole: (userId: string, roleId: string) => Promise<void>;
};
export declare function usePermissions(userId: string | null): {
    permissions: Permission[];
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
    checkPermission: (resource: string, action: string) => Promise<PermissionCheckResponse | {
        allowed: boolean;
    }>;
};
export declare function useTeams(): {
    teams: Team[];
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
    createTeam: (data: Partial<Team>) => Promise<Team>;
    updateTeam: (teamId: string, data: Partial<Team>) => Promise<Team>;
    deleteTeam: (teamId: string) => Promise<void>;
    addMember: (teamId: string, userId: string, role: string) => Promise<void>;
    removeMember: (teamId: string, userId: string) => Promise<void>;
};
export declare function useUserActivity(userId: string | null, limit?: number): {
    activity: ActivityLog[];
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
};
export declare function useUserSessions(userId: string | null): {
    sessions: UserSession[];
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
    terminateSession: (sessionId: string) => Promise<void>;
};
export declare function useInvitations(): {
    invitations: UserInvitation[];
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
    sendInvitation: (data: Partial<UserInvitation>) => Promise<UserInvitation>;
    revokeInvitation: (invitationId: string) => Promise<void>;
    resendInvitation: (invitationId: string) => Promise<void>;
};
export declare function useBulkOperations(): {
    operations: BulkOperation[];
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
    createOperation: (type: string, data: any) => Promise<BulkOperation>;
    getOperation: (operationId: string) => Promise<BulkOperation>;
};
export declare function useSSOProviders(): {
    providers: SSOProvider[];
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
    createProvider: (data: Partial<SSOProvider>) => Promise<SSOProvider>;
    updateProvider: (providerId: string, data: Partial<SSOProvider>) => Promise<SSOProvider>;
    deleteProvider: (providerId: string) => Promise<void>;
    testProvider: (providerId: string) => Promise<{
        success: boolean;
        message: string;
    }>;
};
export declare function useUserStatistics(): {
    statistics: UserStatistics | null;
    loading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
};
export declare function useRealtimeUserEvents(onEvent: (event: UserEvent) => void, filters?: {
    userId?: string;
    eventTypes?: string[];
}): {
    connected: boolean;
};
export declare function useDebounce<T>(value: T, delay: number): T;
export declare function useLocalStorage<T>(key: string, initialValue: T): [T, (value: T | ((val: T) => T)) => void];
export {};
//# sourceMappingURL=UserHooks.d.ts.map