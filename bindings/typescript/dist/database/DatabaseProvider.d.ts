import React, { ReactNode } from 'react';
import { CacheManager } from './CacheManager';
import { DatabaseConfig, QueryOptions, QueryResult, DatabaseStats, MigrationStatus, BackupMetadata, HealthCheckResult } from './types';
interface DatabaseContextValue {
    isConnected: boolean;
    isLoading: boolean;
    error: Error | null;
    cache: CacheManager;
    query: <T = any>(table: string, options?: QueryOptions) => Promise<QueryResult<T>>;
    raw: <T = any>(sql: string, params?: any[]) => Promise<QueryResult<T>>;
    insert: <T = any>(table: string, data: Partial<T>) => Promise<T>;
    update: <T = any>(table: string, id: number | string, data: Partial<T>) => Promise<T>;
    delete: (table: string, id: number | string) => Promise<void>;
    getStats: () => Promise<DatabaseStats>;
    runMigrations: () => Promise<void>;
    getMigrationStatus: () => Promise<MigrationStatus>;
    createBackup: () => Promise<string>;
    listBackups: () => Promise<BackupMetadata[]>;
    restoreBackup: (backupId: string) => Promise<void>;
    healthCheck: () => Promise<HealthCheckResult>;
    invalidateCache: (pattern?: string | RegExp) => void;
}
interface DatabaseProviderProps {
    config: DatabaseConfig;
    children: ReactNode;
    apiEndpoint?: string;
    autoConnect?: boolean;
    debug?: boolean;
}
export declare function DatabaseProvider({ config, children, apiEndpoint, autoConnect, debug, }: DatabaseProviderProps): React.JSX.Element;
export declare function useDatabase(): DatabaseContextValue;
export declare function useQuery<T = any>(table: string, options?: QueryOptions): {
    data: T[] | null;
    total: number;
    isLoading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
};
export declare function useMutation<T = any>(table: string): {
    create: (data: Partial<T>) => Promise<T | null>;
    update: (id: number | string, data: Partial<T>) => Promise<T | null>;
    delete: (id: number | string) => Promise<boolean>;
    isLoading: boolean;
    error: Error | null;
};
export declare function useDatabaseStats(refreshInterval?: number): {
    stats: DatabaseStats | null;
    isLoading: boolean;
    error: Error | null;
    refetch: () => Promise<void>;
};
export {};
//# sourceMappingURL=DatabaseProvider.d.ts.map