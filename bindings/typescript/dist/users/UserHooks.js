import { useState, useEffect, useCallback, useRef } from 'react';
let globalConfig = {
    apiUrl: '/api',
};
export const configureUserManagement = (config) => {
    globalConfig = { ...globalConfig, ...config };
};
class UserManagementClient {
    async request(endpoint, options = {}) {
        const headers = {
            'Content-Type': 'application/json',
            ...(options.headers || {}),
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
    async getUsers(params) {
        const queryString = new URLSearchParams(Object.entries(params).reduce((acc, [key, value]) => {
            if (value !== undefined && value !== null) {
                acc[key] = Array.isArray(value) ? value.join(',') : String(value);
            }
            return acc;
        }, {})).toString();
        return this.request(`/users?${queryString}`);
    }
    async getUser(userId) {
        return this.request(`/users/${userId}`);
    }
    async createUser(data) {
        return this.request('/users', {
            method: 'POST',
            body: JSON.stringify(data),
        });
    }
    async updateUser(userId, data) {
        return this.request(`/users/${userId}`, {
            method: 'PATCH',
            body: JSON.stringify(data),
        });
    }
    async deleteUser(userId) {
        return this.request(`/users/${userId}`, {
            method: 'DELETE',
        });
    }
    async deactivateUser(userId) {
        return this.request(`/users/${userId}/deactivate`, {
            method: 'POST',
        });
    }
    async reactivateUser(userId) {
        return this.request(`/users/${userId}/reactivate`, {
            method: 'POST',
        });
    }
    async getRoles() {
        return this.request('/roles');
    }
    async getRole(roleId) {
        return this.request(`/roles/${roleId}`);
    }
    async assignRole(userId, roleId) {
        return this.request(`/users/${userId}/roles`, {
            method: 'POST',
            body: JSON.stringify({ roleId }),
        });
    }
    async removeRole(userId, roleId) {
        return this.request(`/users/${userId}/roles/${roleId}`, {
            method: 'DELETE',
        });
    }
    async createRole(data) {
        return this.request('/roles', {
            method: 'POST',
            body: JSON.stringify(data),
        });
    }
    async updateRole(roleId, data) {
        return this.request(`/roles/${roleId}`, {
            method: 'PATCH',
            body: JSON.stringify(data),
        });
    }
    async deleteRole(roleId) {
        return this.request(`/roles/${roleId}`, {
            method: 'DELETE',
        });
    }
    async checkPermission(data) {
        return this.request('/permissions/check', {
            method: 'POST',
            body: JSON.stringify(data),
        });
    }
    async getUserPermissions(userId) {
        return this.request(`/users/${userId}/permissions`);
    }
    async getTeams() {
        return this.request('/teams');
    }
    async getTeam(teamId) {
        return this.request(`/teams/${teamId}`);
    }
    async createTeam(data) {
        return this.request('/teams', {
            method: 'POST',
            body: JSON.stringify(data),
        });
    }
    async updateTeam(teamId, data) {
        return this.request(`/teams/${teamId}`, {
            method: 'PATCH',
            body: JSON.stringify(data),
        });
    }
    async deleteTeam(teamId) {
        return this.request(`/teams/${teamId}`, {
            method: 'DELETE',
        });
    }
    async addTeamMember(teamId, userId, role) {
        return this.request(`/teams/${teamId}/members`, {
            method: 'POST',
            body: JSON.stringify({ userId, role }),
        });
    }
    async removeTeamMember(teamId, userId) {
        return this.request(`/teams/${teamId}/members/${userId}`, {
            method: 'DELETE',
        });
    }
    async getUserActivity(userId, params) {
        const queryString = params
            ? new URLSearchParams(params).toString()
            : '';
        return this.request(`/users/${userId}/activity${queryString ? `?${queryString}` : ''}`);
    }
    async getUserSessions(userId) {
        return this.request(`/users/${userId}/sessions`);
    }
    async terminateSession(sessionId) {
        return this.request(`/sessions/${sessionId}`, {
            method: 'DELETE',
        });
    }
    async sendInvitation(data) {
        return this.request('/invitations', {
            method: 'POST',
            body: JSON.stringify(data),
        });
    }
    async getInvitations() {
        return this.request('/invitations');
    }
    async revokeInvitation(invitationId) {
        return this.request(`/invitations/${invitationId}`, {
            method: 'DELETE',
        });
    }
    async resendInvitation(invitationId) {
        return this.request(`/invitations/${invitationId}/resend`, {
            method: 'POST',
        });
    }
    async createBulkOperation(type, data) {
        return this.request('/bulk-operations', {
            method: 'POST',
            body: JSON.stringify({ type, data }),
        });
    }
    async getBulkOperation(operationId) {
        return this.request(`/bulk-operations/${operationId}`);
    }
    async getBulkOperations() {
        return this.request('/bulk-operations');
    }
    async getSSOProviders() {
        return this.request('/sso/providers');
    }
    async getSSOProvider(providerId) {
        return this.request(`/sso/providers/${providerId}`);
    }
    async createSSOProvider(data) {
        return this.request('/sso/providers', {
            method: 'POST',
            body: JSON.stringify(data),
        });
    }
    async updateSSOProvider(providerId, data) {
        return this.request(`/sso/providers/${providerId}`, {
            method: 'PATCH',
            body: JSON.stringify(data),
        });
    }
    async deleteSSOProvider(providerId) {
        return this.request(`/sso/providers/${providerId}`, {
            method: 'DELETE',
        });
    }
    async testSSOProvider(providerId) {
        return this.request(`/sso/providers/${providerId}/test`, { method: 'POST' });
    }
    async getUserStatistics() {
        return this.request('/users/statistics');
    }
    async getUserActivitySummary(userId) {
        return this.request(`/users/${userId}/activity/summary`);
    }
    async exportUsers(params) {
        const queryString = new URLSearchParams(params).toString();
        const response = await fetch(`${globalConfig.apiUrl}/users/export?${queryString}`, {
            headers: {
                Authorization: `Bearer ${globalConfig.token}`,
                'X-Tenant-ID': globalConfig.tenantId || '',
            },
        });
        return response.blob();
    }
}
const client = new UserManagementClient();
export function useUsers(params = {}) {
    const [data, setData] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const fetchUsers = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const result = await client.getUsers(params);
            setData(result);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, [JSON.stringify(params)]);
    useEffect(() => {
        fetchUsers();
    }, [fetchUsers]);
    return { data, loading, error, refetch: fetchUsers };
}
export function useUser(userId) {
    const [user, setUser] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
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
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, [userId]);
    useEffect(() => {
        fetchUser();
    }, [fetchUser]);
    const updateUser = useCallback(async (data) => {
        if (!userId)
            return;
        const updated = await client.updateUser(userId, data);
        setUser(updated);
        return updated;
    }, [userId]);
    return { user, loading, error, refetch: fetchUser, updateUser };
}
export function useCreateUser() {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState(null);
    const createUser = useCallback(async (data) => {
        try {
            setLoading(true);
            setError(null);
            const user = await client.createUser(data);
            return user;
        }
        catch (err) {
            setError(err);
            throw err;
        }
        finally {
            setLoading(false);
        }
    }, []);
    return { createUser, loading, error };
}
export function useRoles() {
    const [roles, setRoles] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const fetchRoles = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const result = await client.getRoles();
            setRoles(result);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, []);
    useEffect(() => {
        fetchRoles();
    }, [fetchRoles]);
    const assignRole = useCallback(async (userId, roleId) => {
        await client.assignRole(userId, roleId);
    }, []);
    const removeRole = useCallback(async (userId, roleId) => {
        await client.removeRole(userId, roleId);
    }, []);
    return { roles, loading, error, refetch: fetchRoles, assignRole, removeRole };
}
export function usePermissions(userId) {
    const [permissions, setPermissions] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
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
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, [userId]);
    useEffect(() => {
        fetchPermissions();
    }, [fetchPermissions]);
    const checkPermission = useCallback(async (resource, action) => {
        if (!userId)
            return { allowed: false };
        return client.checkPermission({ userId, resource, action });
    }, [userId]);
    return { permissions, loading, error, refetch: fetchPermissions, checkPermission };
}
export function useTeams() {
    const [teams, setTeams] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const fetchTeams = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const result = await client.getTeams();
            setTeams(result);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, []);
    useEffect(() => {
        fetchTeams();
    }, [fetchTeams]);
    const createTeam = useCallback(async (data) => {
        const team = await client.createTeam(data);
        setTeams((prev) => [...prev, team]);
        return team;
    }, []);
    const updateTeam = useCallback(async (teamId, data) => {
        const team = await client.updateTeam(teamId, data);
        setTeams((prev) => prev.map((t) => (t.id === teamId ? team : t)));
        return team;
    }, []);
    const deleteTeam = useCallback(async (teamId) => {
        await client.deleteTeam(teamId);
        setTeams((prev) => prev.filter((t) => t.id !== teamId));
    }, []);
    const addMember = useCallback(async (teamId, userId, role) => {
        await client.addTeamMember(teamId, userId, role);
        await fetchTeams();
    }, [fetchTeams]);
    const removeMember = useCallback(async (teamId, userId) => {
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
export function useUserActivity(userId, limit = 50) {
    const [activity, setActivity] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
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
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, [userId, limit]);
    useEffect(() => {
        fetchActivity();
    }, [fetchActivity]);
    return { activity, loading, error, refetch: fetchActivity };
}
export function useUserSessions(userId) {
    const [sessions, setSessions] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
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
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, [userId]);
    useEffect(() => {
        fetchSessions();
    }, [fetchSessions]);
    const terminateSession = useCallback(async (sessionId) => {
        await client.terminateSession(sessionId);
        await fetchSessions();
    }, [fetchSessions]);
    return { sessions, loading, error, refetch: fetchSessions, terminateSession };
}
export function useInvitations() {
    const [invitations, setInvitations] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const fetchInvitations = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const result = await client.getInvitations();
            setInvitations(result);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, []);
    useEffect(() => {
        fetchInvitations();
    }, [fetchInvitations]);
    const sendInvitation = useCallback(async (data) => {
        const invitation = await client.sendInvitation(data);
        setInvitations((prev) => [...prev, invitation]);
        return invitation;
    }, []);
    const revokeInvitation = useCallback(async (invitationId) => {
        await client.revokeInvitation(invitationId);
        await fetchInvitations();
    }, [fetchInvitations]);
    const resendInvitation = useCallback(async (invitationId) => {
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
    const [operations, setOperations] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const fetchOperations = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const result = await client.getBulkOperations();
            setOperations(result);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, []);
    useEffect(() => {
        fetchOperations();
    }, [fetchOperations]);
    const createOperation = useCallback(async (type, data) => {
        const operation = await client.createBulkOperation(type, data);
        setOperations((prev) => [...prev, operation]);
        return operation;
    }, []);
    const getOperation = useCallback(async (operationId) => {
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
    const [providers, setProviders] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const fetchProviders = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const result = await client.getSSOProviders();
            setProviders(result);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, []);
    useEffect(() => {
        fetchProviders();
    }, [fetchProviders]);
    const createProvider = useCallback(async (data) => {
        const provider = await client.createSSOProvider(data);
        setProviders((prev) => [...prev, provider]);
        return provider;
    }, []);
    const updateProvider = useCallback(async (providerId, data) => {
        const provider = await client.updateSSOProvider(providerId, data);
        setProviders((prev) => prev.map((p) => (p.id === providerId ? provider : p)));
        return provider;
    }, []);
    const deleteProvider = useCallback(async (providerId) => {
        await client.deleteSSOProvider(providerId);
        setProviders((prev) => prev.filter((p) => p.id !== providerId));
    }, []);
    const testProvider = useCallback(async (providerId) => {
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
    const [statistics, setStatistics] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const fetchStatistics = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const result = await client.getUserStatistics();
            setStatistics(result);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, []);
    useEffect(() => {
        fetchStatistics();
    }, [fetchStatistics]);
    return { statistics, loading, error, refetch: fetchStatistics };
}
export function useRealtimeUserEvents(onEvent, filters) {
    const wsRef = useRef(null);
    const [connected, setConnected] = useState(false);
    useEffect(() => {
        if (!globalConfig.wsUrl)
            return;
        const ws = new WebSocket(`${globalConfig.wsUrl}/users/events?token=${globalConfig.token}`);
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
            }
            catch (err) {
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
export function useDebounce(value, delay) {
    const [debouncedValue, setDebouncedValue] = useState(value);
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
export function useLocalStorage(key, initialValue) {
    const [storedValue, setStoredValue] = useState(() => {
        try {
            const item = window.localStorage.getItem(key);
            return item ? JSON.parse(item) : initialValue;
        }
        catch (error) {
            return initialValue;
        }
    });
    const setValue = useCallback((value) => {
        try {
            const valueToStore = value instanceof Function ? value(storedValue) : value;
            setStoredValue(valueToStore);
            window.localStorage.setItem(key, JSON.stringify(valueToStore));
        }
        catch (error) {
            console.error('Error saving to localStorage:', error);
        }
    }, [key, storedValue]);
    return [storedValue, setValue];
}
//# sourceMappingURL=UserHooks.js.map