export interface DatabaseConfig {
    primaryUrl: string;
    replicaUrls?: string[];
    poolConfig?: ConnectionPoolConfig;
    cacheConfig?: CacheConfig;
    replicationConfig?: ReplicationConfig;
    shardingConfig?: ShardingConfig;
    backupConfig?: BackupConfig;
    debug?: boolean;
}
export interface ConnectionPoolConfig {
    minConnections?: number;
    maxConnections?: number;
    connectTimeout?: number;
    idleTimeout?: number;
    maxLifetime?: number;
    healthCheckInterval?: number;
}
export interface CacheConfig {
    enableL1?: boolean;
    l1Capacity?: number;
    l1Ttl?: number;
    enableL2?: boolean;
    l2Directory?: string;
    l2MaxSize?: number;
    enableL3?: boolean;
    l3RedisUrl?: string;
    enableCompression?: boolean;
    compressionThreshold?: number;
}
export interface ReplicationConfig {
    role: 'master' | 'slave' | 'standalone';
    masterUrl?: string;
    replicaUrls?: string[];
    lagThreshold?: number;
    enableAutoFailover?: boolean;
}
export interface ShardingConfig {
    strategy: 'hash' | 'range' | 'consistent-hash' | 'directory';
    shardUrls: string[];
    virtualNodes?: number;
    enableCrossShardTx?: boolean;
}
export interface BackupConfig {
    backupDir: string;
    enableCompression?: boolean;
    enableEncryption?: boolean;
    maxBackups?: number;
    autoBackupInterval?: number | null;
    enablePitr?: boolean;
}
export interface QueryFilter {
    field: string;
    operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'in' | 'like' | 'between';
    value: any;
}
export interface QueryOptions {
    where?: QueryFilter[];
    orderBy?: Array<{
        field: string;
        direction: 'asc' | 'desc';
    }>;
    limit?: number;
    offset?: number;
    select?: string[];
    useCache?: boolean;
    cacheTtl?: number;
}
export interface QueryResult<T> {
    data: T[];
    total: number;
    fromCache: boolean;
    executionTime: number;
}
export interface Migration {
    version: number;
    name: string;
    description: string;
    up: string;
    down?: string;
    appliedAt?: string;
    isApplied?: boolean;
}
export interface MigrationStatus {
    total: number;
    applied: number;
    pending: number;
    history: Migration[];
    pendingMigrations: Migration[];
}
export interface BackupMetadata {
    id: string;
    type: 'full' | 'incremental' | 'differential';
    createdAt: string;
    sizeBytes: number;
    compressedSize?: number;
    isEncrypted: boolean;
    checksum: string;
    parentId?: string;
    dbVersion: string;
}
export interface CacheStats {
    l1Hits: number;
    l1Misses: number;
    l1Size: number;
    l2Hits: number;
    l2Misses: number;
    l2Size: number;
    l3Hits: number;
    l3Misses: number;
    totalHits: number;
    totalMisses: number;
    hitRate: number;
    avgGetTimeUs: number;
    avgSetTimeUs: number;
}
export interface PoolStats {
    totalConnections: number;
    activeConnections: number;
    idleConnections: number;
    totalQueries: number;
    totalErrors: number;
    avgQueryTimeUs: number;
    lastHealthCheck?: string;
    isHealthy: boolean;
}
export interface DatabaseStats {
    pool: PoolStats;
    cache: CacheStats;
    replication?: ReplicationStats;
    sharding?: ShardingStats;
    backup?: BackupStats;
}
export interface ReplicationStats {
    replicaCount: number;
    healthyReplicas: number;
    avgLagMs: number;
    maxLagMs: number;
    totalReplications: number;
    totalErrors: number;
    successRate: number;
}
export interface ShardingStats {
    totalShards: number;
    availableShards: number;
    totalLookups: number;
    crossShardQueries: number;
    rebalanceCount: number;
}
export interface BackupStats {
    totalBackups: number;
    totalBackupSize: number;
    totalRestores: number;
    lastBackup?: string;
    lastRestore?: string;
    avgBackupTime: number;
}
export interface SpatialBBox {
    min: [number, number, number];
    max: [number, number, number];
}
export interface SpatialEntity {
    id: number;
    bbox: SpatialBBox;
    metadata?: Record<string, string>;
}
export interface HealthCheckResult {
    healthy: boolean;
    components: {
        database?: boolean;
        cache?: boolean;
        replication?: boolean;
        sharding?: boolean;
    };
    responseTime: number;
    timestamp: string;
    errors?: string[];
}
export interface TransactionOptions {
    isolationLevel?: 'read-uncommitted' | 'read-committed' | 'repeatable-read' | 'serializable';
    timeout?: number;
    readOnly?: boolean;
}
export interface QueryPlan {
    query: string;
    optimizedQuery: string;
    estimatedCost: number;
    estimatedRows: number;
    hints: string[];
    indexSuggestions: string[];
    usesSpatialIndex: boolean;
    cacheable: boolean;
    strategy: 'direct' | 'batch' | 'parallel' | 'cached';
}
export type DatabaseErrorType = 'connection-pool' | 'query-execution' | 'migration' | 'replication' | 'sharding' | 'backup' | 'cache' | 'spatial-index' | 'serialization';
export interface DatabaseError {
    type: DatabaseErrorType;
    message: string;
    stack?: string;
    details?: Record<string, any>;
}
//# sourceMappingURL=types.d.ts.map