/**
 * CADDY v0.4.0 - User Management React Hooks
 *
 * Custom React hooks for user management operations including:
 * - User CRUD operations with caching
 * - Real-time user status updates
 * - Role and permission management
 * - Team operations
 * - Activity tracking
 * - SSO integration
 */

import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import {
  User,
  Role,
  Team,
  Permission,
  ActivityLog,
  UserSession,
  UserInvitation,
  BulkOperation,
  SSOProvider,
  ListUsersRequest,
  ListUsersResponse,
  CreateUserRequest,
  UpdateUserRequest,
  PermissionCheckRequest,
  PermissionCheckResponse,
  UserStatistics,
  UserActivitySummary,
  UserEvent,
  UserStatus,
  TeamMember,
} from './types';

// ============================================================================
// Configuration
// ============================================================================

interface UserManagementConfig {
  apiUrl: string;
  wsUrl?: string;
  token?: string;
  tenantId?: string;
}

let globalConfig: UserManagementConfig = {
  apiUrl: '/api',
};

export const configureUserManagement = (config: Partial<UserManagementConfig>) => {
  globalConfig = { ...globalConfig, ...config };
};

// ============================================================================
// API Client
// ============================================================================

class UserManagementClient {
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string> || {}),
    };

    if (globalConfig.token) {
      headers['Authorization'] = `Bearer ${globalConfig.token}`;
    }

    if (globalConfig.tenantId) {
      headers['X-Tenant-ID'] = globalConfig.tenantId;
    }

    const response = await fetch(`${globalConfig.apiUrl}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({
        message: response.statusText,
      }));
      throw new Error(error.message || 'Request failed');
    }

    return response.json();
  }

  // User operations
  async getUsers(params: ListUsersRequest): Promise<ListUsersResponse> {
    const queryString = new URLSearchParams(
      Object.entries(params).reduce((acc, [key, value]) => {
        if (value !== undefined && value !== null) {
          acc[key] = Array.isArray(value) ? value.join(',') : String(value);
        }
        return acc;
      }, {} as Record<string, string>)
    ).toString();

    return this.request<ListUsersResponse>(
      `/users?${queryString}`
    );
  }

  async getUser(userId: string): Promise<User> {
    return this.request<User>(`/users/${userId}`);
  }

  async createUser(data: CreateUserRequest): Promise<User> {
    return this.request<User>('/users', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateUser(userId: string, data: UpdateUserRequest): Promise<User> {
    return this.request<User>(`/users/${userId}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deleteUser(userId: string): Promise<void> {
    return this.request<void>(`/users/${userId}`, {
      method: 'DELETE',
    });
  }

  async deactivateUser(userId: string): Promise<User> {
    return this.request<User>(`/users/${userId}/deactivate`, {
      method: 'POST',
    });
  }

  async reactivateUser(userId: string): Promise<User> {
    return this.request<User>(`/users/${userId}/reactivate`, {
      method: 'POST',
    });
  }

  // Role operations
  async getRoles(): Promise<Role[]> {
    return this.request<Role[]>('/roles');
  }

  async getRole(roleId: string): Promise<Role> {
    return this.request<Role>(`/roles/${roleId}`);
  }

  async assignRole(userId: string, roleId: string): Promise<void> {
    return this.request<void>(`/users/${userId}/roles`, {
      method: 'POST',
      body: JSON.stringify({ roleId }),
    });
  }

  async removeRole(userId: string, roleId: string): Promise<void> {
    return this.request<void>(`/users/${userId}/roles/${roleId}`, {
      method: 'DELETE',
    });
  }

  async createRole(data: Partial<Role>): Promise<Role> {
    return this.request<Role>('/roles', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateRole(roleId: string, data: Partial<Role>): Promise<Role> {
    return this.request<Role>(`/roles/${roleId}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deleteRole(roleId: string): Promise<void> {
    return this.request<void>(`/roles/${roleId}`, {
      method: 'DELETE',
    });
  }

  // Permission operations
  async checkPermission(data: PermissionCheckRequest): Promise<PermissionCheckResponse> {
    return this.request<PermissionCheckResponse>('/permissions/check', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getUserPermissions(userId: string): Promise<Permission[]> {
    return this.request<Permission[]>(`/users/${userId}/permissions`);
  }

  // Team operations
  async getTeams(): Promise<Team[]> {
    return this.request<Team[]>('/teams');
  }

  async getTeam(teamId: string): Promise<Team> {
    return this.request<Team>(`/teams/${teamId}`);
  }

  async createTeam(data: Partial<Team>): Promise<Team> {
    return this.request<Team>('/teams', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateTeam(teamId: string, data: Partial<Team>): Promise<Team> {
    return this.request<Team>(`/teams/${teamId}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deleteTeam(teamId: string): Promise<void> {
    return this.request<void>(`/teams/${teamId}`, {
      method: 'DELETE',
    });
  }

  async addTeamMember(teamId: string, userId: string, role: string): Promise<void> {
    return this.request<void>(`/teams/${teamId}/members`, {
      method: 'POST',
      body: JSON.stringify({ userId, role }),
    });
  }

  async removeTeamMember(teamId: string, userId: string): Promise<void> {
    return this.request<void>(`/teams/${teamId}/members/${userId}`, {
      method: 'DELETE',
    });
  }

  // Activity operations
  async getUserActivity(
    userId: string,
    params?: { limit?: number; offset?: number }
  ): Promise<ActivityLog[]> {
    const queryString = params
      ? new URLSearchParams(params as any).toString()
      : '';
    return this.request<ActivityLog[]>(
      `/users/${userId}/activity${queryString ? `?${queryString}` : ''}`
    );
  }

  async getUserSessions(userId: string): Promise<UserSession[]> {
    return this.request<UserSession[]>(`/users/${userId}/sessions`);
  }

  async terminateSession(sessionId: string): Promise<void> {
    return this.request<void>(`/sessions/${sessionId}`, {
      method: 'DELETE',
    });
  }

  // Invitation operations
  async sendInvitation(data: Partial<UserInvitation>): Promise<UserInvitation> {
    return this.request<UserInvitation>('/invitations', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getInvitations(): Promise<UserInvitation[]> {
    return this.request<UserInvitation[]>('/invitations');
  }

  async revokeInvitation(invitationId: string): Promise<void> {
    return this.request<void>(`/invitations/${invitationId}`, {
      method: 'DELETE',
    });
  }

  async resendInvitation(invitationId: string): Promise<void> {
    return this.request<void>(`/invitations/${invitationId}/resend`, {
      method: 'POST',
    });
  }

  // Bulk operations
  async createBulkOperation(type: string, data: any): Promise<BulkOperation> {
    return this.request<BulkOperation>('/bulk-operations', {
      method: 'POST',
      body: JSON.stringify({ type, data }),
    });
  }

  async getBulkOperation(operationId: string): Promise<BulkOperation> {
    return this.request<BulkOperation>(`/bulk-operations/${operationId}`);
  }

  async getBulkOperations(): Promise<BulkOperation[]> {
    return this.request<BulkOperation[]>('/bulk-operations');
  }

  // SSO operations
  async getSSOProviders(): Promise<SSOProvider[]> {
    return this.request<SSOProvider[]>('/sso/providers');
  }

  async getSSOProvider(providerId: string): Promise<SSOProvider> {
    return this.request<SSOProvider>(`/sso/providers/${providerId}`);
  }

  async createSSOProvider(data: Partial<SSOProvider>): Promise<SSOProvider> {
    return this.request<SSOProvider>('/sso/providers', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateSSOProvider(
    providerId: string,
    data: Partial<SSOProvider>
  ): Promise<SSOProvider> {
    return this.request<SSOProvider>(`/sso/providers/${providerId}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deleteSSOProvider(providerId: string): Promise<void> {
    return this.request<void>(`/sso/providers/${providerId}`, {
      method: 'DELETE',
    });
  }

  async testSSOProvider(providerId: string): Promise<{ success: boolean; message: string }> {
    return this.request<{ success: boolean; message: string }>(
      `/sso/providers/${providerId}/test`,
      { method: 'POST' }
    );
  }

  // Statistics
  async getUserStatistics(): Promise<UserStatistics> {
    return this.request<UserStatistics>('/users/statistics');
  }

  async getUserActivitySummary(userId: string): Promise<UserActivitySummary> {
    return this.request<UserActivitySummary>(`/users/${userId}/activity/summary`);
  }

  // Export operations
  async exportUsers(params: ListUsersRequest): Promise<Blob> {
    const queryString = new URLSearchParams(params as any).toString();
    const response = await fetch(
      `${globalConfig.apiUrl}/users/export?${queryString}`,
      {
        headers: {
          Authorization: `Bearer ${globalConfig.token}`,
          'X-Tenant-ID': globalConfig.tenantId || '',
        },
      }
    );
    return response.blob();
  }
}

const client = new UserManagementClient();

// ============================================================================
// Custom Hooks
// ============================================================================

export function useUsers(params: ListUsersRequest = {}) {
  const [data, setData] = useState<ListUsersResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchUsers = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await client.getUsers(params);
      setData(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [JSON.stringify(params)]);

  useEffect(() => {
    fetchUsers();
  }, [fetchUsers]);

  return { data, loading, error, refetch: fetchUsers };
}

export function useUser(userId: string | null) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchUser = useCallback(async () => {
    if (!userId) {
      setUser(null);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const result = await client.getUser(userId);
      setUser(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    fetchUser();
  }, [fetchUser]);

  const updateUser = useCallback(
    async (data: UpdateUserRequest) => {
      if (!userId) return;
      const updated = await client.updateUser(userId, data);
      setUser(updated);
      return updated;
    },
    [userId]
  );

  return { user, loading, error, refetch: fetchUser, updateUser };
}

export function useCreateUser() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const createUser = useCallback(async (data: CreateUserRequest) => {
    try {
      setLoading(true);
      setError(null);
      const user = await client.createUser(data);
      return user;
    } catch (err) {
      setError(err as Error);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return { createUser, loading, error };
}

export function useRoles() {
  const [roles, setRoles] = useState<Role[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchRoles = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await client.getRoles();
      setRoles(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchRoles();
  }, [fetchRoles]);

  const assignRole = useCallback(async (userId: string, roleId: string) => {
    await client.assignRole(userId, roleId);
  }, []);

  const removeRole = useCallback(async (userId: string, roleId: string) => {
    await client.removeRole(userId, roleId);
  }, []);

  return { roles, loading, error, refetch: fetchRoles, assignRole, removeRole };
}

export function usePermissions(userId: string | null) {
  const [permissions, setPermissions] = useState<Permission[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchPermissions = useCallback(async () => {
    if (!userId) {
      setPermissions([]);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const result = await client.getUserPermissions(userId);
      setPermissions(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    fetchPermissions();
  }, [fetchPermissions]);

  const checkPermission = useCallback(
    async (resource: string, action: string) => {
      if (!userId) return { allowed: false };
      return client.checkPermission({ userId, resource, action });
    },
    [userId]
  );

  return { permissions, loading, error, refetch: fetchPermissions, checkPermission };
}

export function useTeams() {
  const [teams, setTeams] = useState<Team[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchTeams = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await client.getTeams();
      setTeams(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTeams();
  }, [fetchTeams]);

  const createTeam = useCallback(async (data: Partial<Team>) => {
    const team = await client.createTeam(data);
    setTeams((prev) => [...prev, team]);
    return team;
  }, []);

  const updateTeam = useCallback(async (teamId: string, data: Partial<Team>) => {
    const team = await client.updateTeam(teamId, data);
    setTeams((prev) => prev.map((t) => (t.id === teamId ? team : t)));
    return team;
  }, []);

  const deleteTeam = useCallback(async (teamId: string) => {
    await client.deleteTeam(teamId);
    setTeams((prev) => prev.filter((t) => t.id !== teamId));
  }, []);

  const addMember = useCallback(async (teamId: string, userId: string, role: string) => {
    await client.addTeamMember(teamId, userId, role);
    await fetchTeams();
  }, [fetchTeams]);

  const removeMember = useCallback(async (teamId: string, userId: string) => {
    await client.removeTeamMember(teamId, userId);
    await fetchTeams();
  }, [fetchTeams]);

  return {
    teams,
    loading,
    error,
    refetch: fetchTeams,
    createTeam,
    updateTeam,
    deleteTeam,
    addMember,
    removeMember,
  };
}

export function useUserActivity(userId: string | null, limit: number = 50) {
  const [activity, setActivity] = useState<ActivityLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchActivity = useCallback(async () => {
    if (!userId) {
      setActivity([]);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const result = await client.getUserActivity(userId, { limit });
      setActivity(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [userId, limit]);

  useEffect(() => {
    fetchActivity();
  }, [fetchActivity]);

  return { activity, loading, error, refetch: fetchActivity };
}

export function useUserSessions(userId: string | null) {
  const [sessions, setSessions] = useState<UserSession[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchSessions = useCallback(async () => {
    if (!userId) {
      setSessions([]);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const result = await client.getUserSessions(userId);
      setSessions(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    fetchSessions();
  }, [fetchSessions]);

  const terminateSession = useCallback(
    async (sessionId: string) => {
      await client.terminateSession(sessionId);
      await fetchSessions();
    },
    [fetchSessions]
  );

  return { sessions, loading, error, refetch: fetchSessions, terminateSession };
}

export function useInvitations() {
  const [invitations, setInvitations] = useState<UserInvitation[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchInvitations = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await client.getInvitations();
      setInvitations(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchInvitations();
  }, [fetchInvitations]);

  const sendInvitation = useCallback(
    async (data: Partial<UserInvitation>) => {
      const invitation = await client.sendInvitation(data);
      setInvitations((prev) => [...prev, invitation]);
      return invitation;
    },
    []
  );

  const revokeInvitation = useCallback(async (invitationId: string) => {
    await client.revokeInvitation(invitationId);
    await fetchInvitations();
  }, [fetchInvitations]);

  const resendInvitation = useCallback(async (invitationId: string) => {
    await client.resendInvitation(invitationId);
  }, []);

  return {
    invitations,
    loading,
    error,
    refetch: fetchInvitations,
    sendInvitation,
    revokeInvitation,
    resendInvitation,
  };
}

export function useBulkOperations() {
  const [operations, setOperations] = useState<BulkOperation[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchOperations = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await client.getBulkOperations();
      setOperations(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchOperations();
  }, [fetchOperations]);

  const createOperation = useCallback(async (type: string, data: any) => {
    const operation = await client.createBulkOperation(type, data);
    setOperations((prev) => [...prev, operation]);
    return operation;
  }, []);

  const getOperation = useCallback(async (operationId: string) => {
    return client.getBulkOperation(operationId);
  }, []);

  return {
    operations,
    loading,
    error,
    refetch: fetchOperations,
    createOperation,
    getOperation,
  };
}

export function useSSOProviders() {
  const [providers, setProviders] = useState<SSOProvider[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchProviders = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await client.getSSOProviders();
      setProviders(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  const createProvider = useCallback(async (data: Partial<SSOProvider>) => {
    const provider = await client.createSSOProvider(data);
    setProviders((prev) => [...prev, provider]);
    return provider;
  }, []);

  const updateProvider = useCallback(
    async (providerId: string, data: Partial<SSOProvider>) => {
      const provider = await client.updateSSOProvider(providerId, data);
      setProviders((prev) => prev.map((p) => (p.id === providerId ? provider : p)));
      return provider;
    },
    []
  );

  const deleteProvider = useCallback(async (providerId: string) => {
    await client.deleteSSOProvider(providerId);
    setProviders((prev) => prev.filter((p) => p.id !== providerId));
  }, []);

  const testProvider = useCallback(async (providerId: string) => {
    return client.testSSOProvider(providerId);
  }, []);

  return {
    providers,
    loading,
    error,
    refetch: fetchProviders,
    createProvider,
    updateProvider,
    deleteProvider,
    testProvider,
  };
}

export function useUserStatistics() {
  const [statistics, setStatistics] = useState<UserStatistics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchStatistics = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await client.getUserStatistics();
      setStatistics(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStatistics();
  }, [fetchStatistics]);

  return { statistics, loading, error, refetch: fetchStatistics };
}

export function useRealtimeUserEvents(
  onEvent: (event: UserEvent) => void,
  filters?: { userId?: string; eventTypes?: string[] }
) {
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    if (!globalConfig.wsUrl) return;

    const ws = new WebSocket(
      `${globalConfig.wsUrl}/users/events?token=${globalConfig.token}`
    );

    ws.onopen = () => {
      setConnected(true);
      if (filters) {
        ws.send(JSON.stringify({ type: 'subscribe', filters }));
      }
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        onEvent(data);
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.onclose = () => {
      setConnected(false);
    };

    wsRef.current = ws;

    return () => {
      ws.close();
    };
  }, [globalConfig.wsUrl, globalConfig.token, onEvent, JSON.stringify(filters)]);

  return { connected };
}

export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}

export function useLocalStorage<T>(
  key: string,
  initialValue: T
): [T, (value: T | ((val: T) => T)) => void] {
  const [storedValue, setStoredValue] = useState<T>(() => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initialValue;
    } catch (error) {
      return initialValue;
    }
  });

  const setValue = useCallback(
    (value: T | ((val: T) => T)) => {
      try {
        const valueToStore = value instanceof Function ? value(storedValue) : value;
        setStoredValue(valueToStore);
        window.localStorage.setItem(key, JSON.stringify(valueToStore));
      } catch (error) {
        console.error('Error saving to localStorage:', error);
      }
    },
    [key, storedValue]
  );

  return [storedValue, setValue];
}
