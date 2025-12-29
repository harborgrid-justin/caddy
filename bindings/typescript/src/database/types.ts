/**
 * CADDY Database Layer - TypeScript Type Definitions
 *
 * Provides comprehensive type-safe interfaces for database operations,
 * caching, migrations, and administrative functions.
 */

/**
 * Database configuration options
 */
export interface DatabaseConfig {
  /** Primary database URL */
  primaryUrl: string;

  /** Read replica URLs for load balancing */
  replicaUrls?: string[];

  /** Connection pool configuration */
  poolConfig?: ConnectionPoolConfig;

  /** Cache configuration */
  cacheConfig?: CacheConfig;

  /** Replication configuration */
  replicationConfig?: ReplicationConfig;

  /** Sharding configuration */
  shardingConfig?: ShardingConfig;

  /** Backup configuration */
  backupConfig?: BackupConfig;

  /** Enable debug logging */
  debug?: boolean;
}

/**
 * Connection pool configuration
 */
export interface ConnectionPoolConfig {
  /** Minimum number of connections */
  minConnections?: number;

  /** Maximum number of connections */
  maxConnections?: number;

  /** Connection timeout in milliseconds */
  connectTimeout?: number;

  /** Idle timeout in milliseconds */
  idleTimeout?: number;

  /** Maximum connection lifetime in milliseconds */
  maxLifetime?: number;

  /** Health check interval in milliseconds */
  healthCheckInterval?: number;
}

/**
 * Cache configuration
 */
export interface CacheConfig {
  /** Enable L1 (memory) cache */
  enableL1?: boolean;

  /** L1 cache capacity (number of entries) */
  l1Capacity?: number;

  /** L1 cache TTL in milliseconds */
  l1Ttl?: number;

  /** Enable L2 (disk) cache */
  enableL2?: boolean;

  /** L2 cache directory */
  l2Directory?: string;

  /** L2 cache maximum size in bytes */
  l2MaxSize?: number;

  /** Enable L3 (distributed) cache */
  enableL3?: boolean;

  /** L3 Redis URL */
  l3RedisUrl?: string;

  /** Enable compression for cached values */
  enableCompression?: boolean;

  /** Compression threshold in bytes */
  compressionThreshold?: number;
}

/**
 * Replication configuration
 */
export interface ReplicationConfig {
  /** Replication role */
  role: 'master' | 'slave' | 'standalone';

  /** Master URL (if slave) */
  masterUrl?: string;

  /** Replica URLs (if master) */
  replicaUrls?: string[];

  /** Replication lag threshold in milliseconds */
  lagThreshold?: number;

  /** Enable automatic failover */
  enableAutoFailover?: boolean;
}

/**
 * Sharding configuration
 */
export interface ShardingConfig {
  /** Sharding strategy */
  strategy: 'hash' | 'range' | 'consistent-hash' | 'directory';

  /** Shard URLs */
  shardUrls: string[];

  /** Virtual nodes for consistent hashing */
  virtualNodes?: number;

  /** Enable cross-shard transactions */
  enableCrossShardTx?: boolean;
}

/**
 * Backup configuration
 */
export interface BackupConfig {
  /** Backup directory */
  backupDir: string;

  /** Enable compression */
  enableCompression?: boolean;

  /** Enable encryption */
  enableEncryption?: boolean;

  /** Maximum number of backups to retain */
  maxBackups?: number;

  /** Automatic backup interval in milliseconds (null = disabled) */
  autoBackupInterval?: number | null;

  /** Enable point-in-time recovery */
  enablePitr?: boolean;
}

/**
 * Query filter for type-safe queries
 */
export interface QueryFilter {
  /** Field name */
  field: string;

  /** Operator */
  operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'in' | 'like' | 'between';

  /** Value(s) */
  value: any;
}

/**
 * Query options
 */
export interface QueryOptions {
  /** Filters to apply */
  where?: QueryFilter[];

  /** Fields to order by */
  orderBy?: Array<{ field: string; direction: 'asc' | 'desc' }>;

  /** Limit number of results */
  limit?: number;

  /** Offset for pagination */
  offset?: number;

  /** Fields to select (default: all) */
  select?: string[];

  /** Use cache for this query */
  useCache?: boolean;

  /** Cache TTL in milliseconds */
  cacheTtl?: number;
}

/**
 * Query result
 */
export interface QueryResult<T> {
  /** Result data */
  data: T[];

  /** Total count (before pagination) */
  total: number;

  /** Whether this result came from cache */
  fromCache: boolean;

  /** Execution time in milliseconds */
  executionTime: number;
}

/**
 * Migration definition
 */
export interface Migration {
  /** Migration version (timestamp) */
  version: number;

  /** Migration name */
  name: string;

  /** Migration description */
  description: string;

  /** SQL to execute (up) */
  up: string;

  /** SQL to rollback (down) */
  down?: string;

  /** Applied timestamp */
  appliedAt?: string;

  /** Whether this migration is applied */
  isApplied?: boolean;
}

/**
 * Migration status
 */
export interface MigrationStatus {
  /** Total number of migrations */
  total: number;

  /** Number of applied migrations */
  applied: number;

  /** Number of pending migrations */
  pending: number;

  /** Migration history */
  history: Migration[];

  /** Pending migrations */
  pendingMigrations: Migration[];
}

/**
 * Backup metadata
 */
export interface BackupMetadata {
  /** Backup ID */
  id: string;

  /** Backup type */
  type: 'full' | 'incremental' | 'differential';

  /** Creation timestamp */
  createdAt: string;

  /** Backup size in bytes */
  sizeBytes: number;

  /** Compressed size (if compressed) */
  compressedSize?: number;

  /** Whether encrypted */
  isEncrypted: boolean;

  /** Checksum */
  checksum: string;

  /** Parent backup ID (for incremental) */
  parentId?: string;

  /** Database version */
  dbVersion: string;
}

/**
 * Cache statistics
 */
export interface CacheStats {
  /** L1 hits */
  l1Hits: number;

  /** L1 misses */
  l1Misses: number;

  /** L1 size (number of entries) */
  l1Size: number;

  /** L2 hits */
  l2Hits: number;

  /** L2 misses */
  l2Misses: number;

  /** L2 size in bytes */
  l2Size: number;

  /** L3 hits */
  l3Hits: number;

  /** L3 misses */
  l3Misses: number;

  /** Total hits */
  totalHits: number;

  /** Total misses */
  totalMisses: number;

  /** Hit rate (0-1) */
  hitRate: number;

  /** Average get time in microseconds */
  avgGetTimeUs: number;

  /** Average set time in microseconds */
  avgSetTimeUs: number;
}

/**
 * Connection pool statistics
 */
export interface PoolStats {
  /** Total connections created */
  totalConnections: number;

  /** Active connections */
  activeConnections: number;

  /** Idle connections */
  idleConnections: number;

  /** Total queries executed */
  totalQueries: number;

  /** Total errors */
  totalErrors: number;

  /** Average query time in microseconds */
  avgQueryTimeUs: number;

  /** Last health check */
  lastHealthCheck?: string;

  /** Health status */
  isHealthy: boolean;
}

/**
 * Database statistics
 */
export interface DatabaseStats {
  /** Connection pool stats */
  pool: PoolStats;

  /** Cache stats */
  cache: CacheStats;

  /** Replication stats */
  replication?: ReplicationStats;

  /** Sharding stats */
  sharding?: ShardingStats;

  /** Backup stats */
  backup?: BackupStats;
}

/**
 * Replication statistics
 */
export interface ReplicationStats {
  /** Number of replicas */
  replicaCount: number;

  /** Number of healthy replicas */
  healthyReplicas: number;

  /** Average replication lag in milliseconds */
  avgLagMs: number;

  /** Maximum replication lag in milliseconds */
  maxLagMs: number;

  /** Total replications */
  totalReplications: number;

  /** Total errors */
  totalErrors: number;

  /** Success rate (0-1) */
  successRate: number;
}

/**
 * Sharding statistics
 */
export interface ShardingStats {
  /** Total shards */
  totalShards: number;

  /** Available shards */
  availableShards: number;

  /** Total shard lookups */
  totalLookups: number;

  /** Cross-shard queries */
  crossShardQueries: number;

  /** Rebalancing operations */
  rebalanceCount: number;
}

/**
 * Backup statistics
 */
export interface BackupStats {
  /** Total backups */
  totalBackups: number;

  /** Total backup size in bytes */
  totalBackupSize: number;

  /** Total restores */
  totalRestores: number;

  /** Last backup time */
  lastBackup?: string;

  /** Last restore time */
  lastRestore?: string;

  /** Average backup time in milliseconds */
  avgBackupTime: number;
}

/**
 * Spatial bounding box
 */
export interface SpatialBBox {
  /** Minimum point [x, y, z] */
  min: [number, number, number];

  /** Maximum point [x, y, z] */
  max: [number, number, number];
}

/**
 * Spatial entity for indexing
 */
export interface SpatialEntity {
  /** Entity ID */
  id: number;

  /** Bounding box */
  bbox: SpatialBBox;

  /** Metadata */
  metadata?: Record<string, string>;
}

/**
 * Health check result
 */
export interface HealthCheckResult {
  /** Overall health status */
  healthy: boolean;

  /** Component health status */
  components: {
    database?: boolean;
    cache?: boolean;
    replication?: boolean;
    sharding?: boolean;
  };

  /** Response time in milliseconds */
  responseTime: number;

  /** Timestamp */
  timestamp: string;

  /** Error messages */
  errors?: string[];
}

/**
 * Transaction options
 */
export interface TransactionOptions {
  /** Isolation level */
  isolationLevel?: 'read-uncommitted' | 'read-committed' | 'repeatable-read' | 'serializable';

  /** Transaction timeout in milliseconds */
  timeout?: number;

  /** Read-only transaction */
  readOnly?: boolean;
}

/**
 * Query plan for optimization analysis
 */
export interface QueryPlan {
  /** Original query */
  query: string;

  /** Optimized query */
  optimizedQuery: string;

  /** Estimated cost */
  estimatedCost: number;

  /** Estimated rows */
  estimatedRows: number;

  /** Optimization hints applied */
  hints: string[];

  /** Index suggestions */
  indexSuggestions: string[];

  /** Uses spatial index */
  usesSpatialIndex: boolean;

  /** Is cacheable */
  cacheable: boolean;

  /** Execution strategy */
  strategy: 'direct' | 'batch' | 'parallel' | 'cached';
}

/**
 * Error types
 */
export type DatabaseErrorType =
  | 'connection-pool'
  | 'query-execution'
  | 'migration'
  | 'replication'
  | 'sharding'
  | 'backup'
  | 'cache'
  | 'spatial-index'
  | 'serialization';

/**
 * Database error
 */
export interface DatabaseError {
  /** Error type */
  type: DatabaseErrorType;

  /** Error message */
  message: string;

  /** Stack trace */
  stack?: string;

  /** Additional details */
  details?: Record<string, any>;
}
