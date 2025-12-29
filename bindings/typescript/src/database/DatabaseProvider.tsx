/**
 * CADDY Database - React Context and Hooks
 *
 * Provides React hooks and context for database operations.
 */

import React, { createContext, useContext, useEffect, useState, useCallback, ReactNode } from 'react';
import { CacheManager } from './CacheManager';
import { QueryBuilder } from './QueryBuilder';
import {
  DatabaseConfig,
  QueryOptions,
  QueryResult,
  DatabaseStats,
  Migration,
  MigrationStatus,
  BackupMetadata,
  HealthCheckResult,
} from './types';

/**
 * Database context value
 */
interface DatabaseContextValue {
  /** Whether the database is connected */
  isConnected: boolean;

  /** Whether the database is loading */
  isLoading: boolean;

  /** Error if any */
  error: Error | null;

  /** Cache manager instance */
  cache: CacheManager;

  /** Execute a query */
  query: <T = any>(
    table: string,
    options?: QueryOptions
  ) => Promise<QueryResult<T>>;

  /** Execute a raw SQL query */
  raw: <T = any>(sql: string, params?: any[]) => Promise<QueryResult<T>>;

  /** Insert a record */
  insert: <T = any>(table: string, data: Partial<T>) => Promise<T>;

  /** Update records */
  update: <T = any>(
    table: string,
    id: number | string,
    data: Partial<T>
  ) => Promise<T>;

  /** Delete records */
  delete: (table: string, id: number | string) => Promise<void>;

  /** Get database statistics */
  getStats: () => Promise<DatabaseStats>;

  /** Run migrations */
  runMigrations: () => Promise<void>;

  /** Get migration status */
  getMigrationStatus: () => Promise<MigrationStatus>;

  /** Create a backup */
  createBackup: () => Promise<string>;

  /** List backups */
  listBackups: () => Promise<BackupMetadata[]>;

  /** Restore from backup */
  restoreBackup: (backupId: string) => Promise<void>;

  /** Health check */
  healthCheck: () => Promise<HealthCheckResult>;

  /** Invalidate cache */
  invalidateCache: (pattern?: string | RegExp) => void;
}

/**
 * Database context
 */
const DatabaseContext = createContext<DatabaseContextValue | null>(null);

/**
 * Database provider props
 */
interface DatabaseProviderProps {
  /** Database configuration */
  config: DatabaseConfig;

  /** Children components */
  children: ReactNode;

  /** API endpoint for database operations */
  apiEndpoint?: string;

  /** Auto-connect on mount */
  autoConnect?: boolean;

  /** Enable debug logging */
  debug?: boolean;
}

/**
 * Database provider component
 */
export function DatabaseProvider({
  config,
  children,
  apiEndpoint = '/api/database',
  autoConnect = true,
  debug = false,
}: DatabaseProviderProps) {
  const [isConnected, setIsConnected] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [cache] = useState(() => new CacheManager(config.cacheConfig));

  // Connect to database
  const connect = useCallback(async () => {
    if (isConnected) return;

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
      if (debug) console.log('Database connected');
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      if (debug) console.error('Database connection error:', error);
    } finally {
      setIsLoading(false);
    }
  }, [apiEndpoint, config, debug, isConnected]);

  // Auto-connect on mount
  useEffect(() => {
    if (autoConnect) {
      connect();
    }
  }, [autoConnect, connect]);

  // Query function
  const query = useCallback(
    async <T = any>(table: string, options?: QueryOptions): Promise<QueryResult<T>> => {
      const cacheKey = options?.useCache !== false
        ? `query:${table}:${JSON.stringify(options)}`
        : null;

      // Check cache first
      if (cacheKey) {
        const cached = await cache.get<QueryResult<T>>(cacheKey);
        if (cached) {
          if (debug) console.log('Cache hit:', cacheKey);
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

      const result: QueryResult<T> = await response.json();

      // Cache the result
      if (cacheKey && options?.useCache !== false) {
        await cache.set(cacheKey, result, options?.cacheTtl);
      }

      return result;
    },
    [apiEndpoint, cache, debug]
  );

  // Raw SQL query
  const raw = useCallback(
    async <T = any>(sql: string, params?: any[]): Promise<QueryResult<T>> => {
      const response = await fetch(`${apiEndpoint}/raw`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sql, params }),
      });

      if (!response.ok) {
        throw new Error(`Query failed: ${response.statusText}`);
      }

      return response.json();
    },
    [apiEndpoint]
  );

  // Insert function
  const insert = useCallback(
    async <T = any>(table: string, data: Partial<T>): Promise<T> => {
      const response = await fetch(`${apiEndpoint}/insert`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ table, data }),
      });

      if (!response.ok) {
        throw new Error(`Insert failed: ${response.statusText}`);
      }

      const result = await response.json();

      // Invalidate relevant cache entries
      cache.invalidatePattern(new RegExp(`^query:${table}:`));

      return result;
    },
    [apiEndpoint, cache]
  );

  // Update function
  const update = useCallback(
    async <T = any>(
      table: string,
      id: number | string,
      data: Partial<T>
    ): Promise<T> => {
      const response = await fetch(`${apiEndpoint}/update`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ table, id, data }),
      });

      if (!response.ok) {
        throw new Error(`Update failed: ${response.statusText}`);
      }

      const result = await response.json();

      // Invalidate relevant cache entries
      cache.invalidatePattern(new RegExp(`^query:${table}:`));

      return result;
    },
    [apiEndpoint, cache]
  );

  // Delete function
  const deleteRecord = useCallback(
    async (table: string, id: number | string): Promise<void> => {
      const response = await fetch(`${apiEndpoint}/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ table, id }),
      });

      if (!response.ok) {
        throw new Error(`Delete failed: ${response.statusText}`);
      }

      // Invalidate relevant cache entries
      cache.invalidatePattern(new RegExp(`^query:${table}:`));
    },
    [apiEndpoint, cache]
  );

  // Get statistics
  const getStats = useCallback(async (): Promise<DatabaseStats> => {
    const response = await fetch(`${apiEndpoint}/stats`);
    if (!response.ok) {
      throw new Error(`Failed to get stats: ${response.statusText}`);
    }
    return response.json();
  }, [apiEndpoint]);

  // Run migrations
  const runMigrations = useCallback(async (): Promise<void> => {
    const response = await fetch(`${apiEndpoint}/migrations/run`, {
      method: 'POST',
    });
    if (!response.ok) {
      throw new Error(`Migration failed: ${response.statusText}`);
    }
  }, [apiEndpoint]);

  // Get migration status
  const getMigrationStatus = useCallback(async (): Promise<MigrationStatus> => {
    const response = await fetch(`${apiEndpoint}/migrations/status`);
    if (!response.ok) {
      throw new Error(`Failed to get migration status: ${response.statusText}`);
    }
    return response.json();
  }, [apiEndpoint]);

  // Create backup
  const createBackup = useCallback(async (): Promise<string> => {
    const response = await fetch(`${apiEndpoint}/backup/create`, {
      method: 'POST',
    });
    if (!response.ok) {
      throw new Error(`Backup failed: ${response.statusText}`);
    }
    const result = await response.json();
    return result.backupId;
  }, [apiEndpoint]);

  // List backups
  const listBackups = useCallback(async (): Promise<BackupMetadata[]> => {
    const response = await fetch(`${apiEndpoint}/backup/list`);
    if (!response.ok) {
      throw new Error(`Failed to list backups: ${response.statusText}`);
    }
    return response.json();
  }, [apiEndpoint]);

  // Restore backup
  const restoreBackup = useCallback(
    async (backupId: string): Promise<void> => {
      const response = await fetch(`${apiEndpoint}/backup/restore`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ backupId }),
      });
      if (!response.ok) {
        throw new Error(`Restore failed: ${response.statusText}`);
      }
    },
    [apiEndpoint]
  );

  // Health check
  const healthCheck = useCallback(async (): Promise<HealthCheckResult> => {
    const response = await fetch(`${apiEndpoint}/health`);
    if (!response.ok) {
      throw new Error(`Health check failed: ${response.statusText}`);
    }
    return response.json();
  }, [apiEndpoint]);

  // Invalidate cache
  const invalidateCache = useCallback(
    (pattern?: string | RegExp) => {
      if (pattern) {
        cache.invalidatePattern(pattern);
      } else {
        cache.clear();
      }
    },
    [cache]
  );

  const value: DatabaseContextValue = {
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

  return (
    <DatabaseContext.Provider value={value}>
      {children}
    </DatabaseContext.Provider>
  );
}

/**
 * Hook to use database context
 */
export function useDatabase(): DatabaseContextValue {
  const context = useContext(DatabaseContext);
  if (!context) {
    throw new Error('useDatabase must be used within a DatabaseProvider');
  }
  return context;
}

/**
 * Hook for querying data
 */
export function useQuery<T = any>(
  table: string,
  options?: QueryOptions
): {
  data: T[] | null;
  total: number;
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
} {
  const { query: queryFn } = useDatabase();
  const [data, setData] = useState<T[] | null>(null);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchData = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const result = await queryFn<T>(table, options);
      setData(result.data);
      setTotal(result.total);
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      setData(null);
    } finally {
      setIsLoading(false);
    }
  }, [queryFn, table, options]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { data, total, isLoading, error, refetch: fetchData };
}

/**
 * Hook for database mutations
 */
export function useMutation<T = any>(table: string) {
  const { insert, update, delete: deleteFn } = useDatabase();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const create = useCallback(
    async (data: Partial<T>): Promise<T | null> => {
      setIsLoading(true);
      setError(null);

      try {
        const result = await insert<T>(table, data);
        return result;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    [insert, table]
  );

  const modify = useCallback(
    async (id: number | string, data: Partial<T>): Promise<T | null> => {
      setIsLoading(true);
      setError(null);

      try {
        const result = await update<T>(table, id, data);
        return result;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    [update, table]
  );

  const remove = useCallback(
    async (id: number | string): Promise<boolean> => {
      setIsLoading(true);
      setError(null);

      try {
        await deleteFn(table, id);
        return true;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        return false;
      } finally {
        setIsLoading(false);
      }
    },
    [deleteFn, table]
  );

  return { create, update: modify, delete: remove, isLoading, error };
}

/**
 * Hook for database statistics
 */
export function useDatabaseStats(refreshInterval?: number) {
  const { getStats } = useDatabase();
  const [stats, setStats] = useState<DatabaseStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchStats = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const result = await getStats();
      setStats(result);
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
    } finally {
      setIsLoading(false);
    }
  }, [getStats]);

  useEffect(() => {
    fetchStats();

    if (refreshInterval) {
      const interval = setInterval(fetchStats, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [fetchStats, refreshInterval]);

  return { stats, isLoading, error, refetch: fetchStats };
}
