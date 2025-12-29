import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { CacheManager } from './CacheManager';
const DatabaseContext = createContext(null);
export function DatabaseProvider({ config, children, apiEndpoint = '/api/database', autoConnect = true, debug = false, }) {
    const [isConnected, setIsConnected] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);
    const [cache] = useState(() => new CacheManager(config.cacheConfig));
    const connect = useCallback(async () => {
        if (isConnected)
            return;
        setIsLoading(true);
        setError(null);
        try {
            const response = await fetch(`${apiEndpoint}/connect`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(config),
            });
            if (!response.ok) {
                throw new Error(`Connection failed: ${response.statusText}`);
            }
            setIsConnected(true);
            if (debug)
                console.log('Database connected');
        }
        catch (err) {
            const error = err instanceof Error ? err : new Error(String(err));
            setError(error);
            if (debug)
                console.error('Database connection error:', error);
        }
        finally {
            setIsLoading(false);
        }
    }, [apiEndpoint, config, debug, isConnected]);
    useEffect(() => {
        if (autoConnect) {
            connect();
        }
    }, [autoConnect, connect]);
    const query = useCallback(async (table, options) => {
        const cacheKey = options?.useCache !== false
            ? `query:${table}:${JSON.stringify(options)}`
            : null;
        if (cacheKey) {
            const cached = await cache.get(cacheKey);
            if (cached) {
                if (debug)
                    console.log('Cache hit:', cacheKey);
                return cached;
            }
        }
        const response = await fetch(`${apiEndpoint}/query`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ table, options }),
        });
        if (!response.ok) {
            throw new Error(`Query failed: ${response.statusText}`);
        }
        const result = await response.json();
        if (cacheKey && options?.useCache !== false) {
            await cache.set(cacheKey, result, options?.cacheTtl);
        }
        return result;
    }, [apiEndpoint, cache, debug]);
    const raw = useCallback(async (sql, params) => {
        const response = await fetch(`${apiEndpoint}/raw`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ sql, params }),
        });
        if (!response.ok) {
            throw new Error(`Query failed: ${response.statusText}`);
        }
        return response.json();
    }, [apiEndpoint]);
    const insert = useCallback(async (table, data) => {
        const response = await fetch(`${apiEndpoint}/insert`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ table, data }),
        });
        if (!response.ok) {
            throw new Error(`Insert failed: ${response.statusText}`);
        }
        const result = await response.json();
        cache.invalidatePattern(new RegExp(`^query:${table}:`));
        return result;
    }, [apiEndpoint, cache]);
    const update = useCallback(async (table, id, data) => {
        const response = await fetch(`${apiEndpoint}/update`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ table, id, data }),
        });
        if (!response.ok) {
            throw new Error(`Update failed: ${response.statusText}`);
        }
        const result = await response.json();
        cache.invalidatePattern(new RegExp(`^query:${table}:`));
        return result;
    }, [apiEndpoint, cache]);
    const deleteRecord = useCallback(async (table, id) => {
        const response = await fetch(`${apiEndpoint}/delete`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ table, id }),
        });
        if (!response.ok) {
            throw new Error(`Delete failed: ${response.statusText}`);
        }
        cache.invalidatePattern(new RegExp(`^query:${table}:`));
    }, [apiEndpoint, cache]);
    const getStats = useCallback(async () => {
        const response = await fetch(`${apiEndpoint}/stats`);
        if (!response.ok) {
            throw new Error(`Failed to get stats: ${response.statusText}`);
        }
        return response.json();
    }, [apiEndpoint]);
    const runMigrations = useCallback(async () => {
        const response = await fetch(`${apiEndpoint}/migrations/run`, {
            method: 'POST',
        });
        if (!response.ok) {
            throw new Error(`Migration failed: ${response.statusText}`);
        }
    }, [apiEndpoint]);
    const getMigrationStatus = useCallback(async () => {
        const response = await fetch(`${apiEndpoint}/migrations/status`);
        if (!response.ok) {
            throw new Error(`Failed to get migration status: ${response.statusText}`);
        }
        return response.json();
    }, [apiEndpoint]);
    const createBackup = useCallback(async () => {
        const response = await fetch(`${apiEndpoint}/backup/create`, {
            method: 'POST',
        });
        if (!response.ok) {
            throw new Error(`Backup failed: ${response.statusText}`);
        }
        const result = await response.json();
        return result.backupId;
    }, [apiEndpoint]);
    const listBackups = useCallback(async () => {
        const response = await fetch(`${apiEndpoint}/backup/list`);
        if (!response.ok) {
            throw new Error(`Failed to list backups: ${response.statusText}`);
        }
        return response.json();
    }, [apiEndpoint]);
    const restoreBackup = useCallback(async (backupId) => {
        const response = await fetch(`${apiEndpoint}/backup/restore`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ backupId }),
        });
        if (!response.ok) {
            throw new Error(`Restore failed: ${response.statusText}`);
        }
    }, [apiEndpoint]);
    const healthCheck = useCallback(async () => {
        const response = await fetch(`${apiEndpoint}/health`);
        if (!response.ok) {
            throw new Error(`Health check failed: ${response.statusText}`);
        }
        return response.json();
    }, [apiEndpoint]);
    const invalidateCache = useCallback((pattern) => {
        if (pattern) {
            cache.invalidatePattern(pattern);
        }
        else {
            cache.clear();
        }
    }, [cache]);
    const value = {
        isConnected,
        isLoading,
        error,
        cache,
        query,
        raw,
        insert,
        update,
        delete: deleteRecord,
        getStats,
        runMigrations,
        getMigrationStatus,
        createBackup,
        listBackups,
        restoreBackup,
        healthCheck,
        invalidateCache,
    };
    return (React.createElement(DatabaseContext.Provider, { value: value }, children));
}
export function useDatabase() {
    const context = useContext(DatabaseContext);
    if (!context) {
        throw new Error('useDatabase must be used within a DatabaseProvider');
    }
    return context;
}
export function useQuery(table, options) {
    const { query: queryFn } = useDatabase();
    const [data, setData] = useState(null);
    const [total, setTotal] = useState(0);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState(null);
    const fetchData = useCallback(async () => {
        setIsLoading(true);
        setError(null);
        try {
            const result = await queryFn(table, options);
            setData(result.data);
            setTotal(result.total);
        }
        catch (err) {
            const error = err instanceof Error ? err : new Error(String(err));
            setError(error);
            setData(null);
        }
        finally {
            setIsLoading(false);
        }
    }, [queryFn, table, options]);
    useEffect(() => {
        fetchData();
    }, [fetchData]);
    return { data, total, isLoading, error, refetch: fetchData };
}
export function useMutation(table) {
    const { insert, update, delete: deleteFn } = useDatabase();
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);
    const create = useCallback(async (data) => {
        setIsLoading(true);
        setError(null);
        try {
            const result = await insert(table, data);
            return result;
        }
        catch (err) {
            const error = err instanceof Error ? err : new Error(String(err));
            setError(error);
            return null;
        }
        finally {
            setIsLoading(false);
        }
    }, [insert, table]);
    const modify = useCallback(async (id, data) => {
        setIsLoading(true);
        setError(null);
        try {
            const result = await update(table, id, data);
            return result;
        }
        catch (err) {
            const error = err instanceof Error ? err : new Error(String(err));
            setError(error);
            return null;
        }
        finally {
            setIsLoading(false);
        }
    }, [update, table]);
    const remove = useCallback(async (id) => {
        setIsLoading(true);
        setError(null);
        try {
            await deleteFn(table, id);
            return true;
        }
        catch (err) {
            const error = err instanceof Error ? err : new Error(String(err));
            setError(error);
            return false;
        }
        finally {
            setIsLoading(false);
        }
    }, [deleteFn, table]);
    return { create, update: modify, delete: remove, isLoading, error };
}
export function useDatabaseStats(refreshInterval) {
    const { getStats } = useDatabase();
    const [stats, setStats] = useState(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState(null);
    const fetchStats = useCallback(async () => {
        setIsLoading(true);
        setError(null);
        try {
            const result = await getStats();
            setStats(result);
        }
        catch (err) {
            const error = err instanceof Error ? err : new Error(String(err));
            setError(error);
        }
        finally {
            setIsLoading(false);
        }
    }, [getStats]);
    useEffect(() => {
        fetchStats();
        if (refreshInterval) {
            const interval = setInterval(fetchStats, refreshInterval);
            return () => clearInterval(interval);
        }
        return undefined;
    }, [fetchStats, refreshInterval]);
    return { stats, isLoading, error, refetch: fetchStats };
}
//# sourceMappingURL=DatabaseProvider.js.map