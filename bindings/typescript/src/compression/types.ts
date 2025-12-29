/**
 * TypeScript Type Definitions for CADDY Compression System
 *
 * Enterprise-grade compression types for CAD file formats.
 */

/**
 * Compression level enumeration
 */
export enum CompressionLevel {
  /** Fastest compression (level 1) */
  Fastest = 1,
  /** Fast compression (level 3) */
  Fast = 3,
  /** Balanced compression (level 5) - default */
  Balanced = 5,
  /** Best compression (level 7) */
  Best = 7,
  /** Maximum compression (level 9) */
  Maximum = 9,
}

/**
 * Compression algorithm types
 */
export enum CompressionAlgorithm {
  /** Custom LZ4 variant optimized for CAD geometry */
  LZ4Custom = 'lz4_custom',
  /** Delta encoding for versioned CAD data */
  Delta = 'delta',
  /** Draco-inspired mesh compression */
  Mesh = 'mesh',
  /** Streaming compression for large files */
  Streaming = 'streaming',
  /** Domain-specific dictionary compression */
  Dictionary = 'dictionary',
  /** Multi-threaded parallel compression */
  Parallel = 'parallel',
  /** Adaptive algorithm selection */
  Adaptive = 'adaptive',
}

/**
 * Compression format identifier
 */
export enum CompressionFormat {
  /** Custom LZ4 variant */
  Lz4Custom = 0x4C5A3443,
  /** Delta encoding */
  Delta = 0x44454C54,
  /** Mesh compression */
  Mesh = 0x4D455348,
  /** Dictionary compression */
  Dictionary = 0x44494354,
  /** Adaptive multi-algorithm */
  Adaptive = 0x41445054,
}

/**
 * Data type hint for delta encoding optimization
 */
export enum DeltaDataType {
  /** Mixed data (default) */
  Mixed = 'mixed',
  /** Floating point coordinates */
  Coordinates = 'coordinates',
  /** Integer IDs/indices */
  Integers = 'integers',
  /** Color/material data */
  Colors = 'colors',
  /** Text/metadata */
  Text = 'text',
}

/**
 * Compression strategy for adaptive compression
 */
export enum CompressionStrategy {
  /** Always use LZ4 custom */
  AlwaysLz4 = 'always_lz4',
  /** Always use delta encoding */
  AlwaysDelta = 'always_delta',
  /** Always use mesh compression */
  AlwaysMesh = 'always_mesh',
  /** Always use dictionary compression */
  AlwaysDictionary = 'always_dictionary',
  /** Automatically select best algorithm */
  Auto = 'auto',
  /** Fastest possible compression */
  Fastest = 'fastest',
  /** Best compression ratio */
  BestRatio = 'best_ratio',
}

/**
 * Compression statistics
 */
export interface CompressionStats {
  /** Original uncompressed size in bytes */
  originalSize: number;
  /** Compressed size in bytes */
  compressedSize: number;
  /** Compression ratio (0.0 to 1.0) */
  ratio: number;
  /** Compression time in milliseconds */
  compressionTimeMs: number;
  /** Decompression time in milliseconds (if available) */
  decompressionTimeMs?: number;
  /** Algorithm used */
  algorithm: string;
  /** Additional metadata */
  metadata: Record<string, string>;
}

/**
 * LZ4 custom configuration
 */
export interface Lz4CustomConfig {
  /** Compression level */
  level: CompressionLevel;
  /** Dictionary size (power of 2, max 64KB) */
  dictSize: number;
  /** Enable float optimization */
  optimizeFloats: boolean;
  /** Enable entity ID optimization */
  optimizeIds: boolean;
  /** Minimum match length */
  minMatch: number;
  /** Maximum match distance */
  maxDistance: number;
}

/**
 * Delta encoding configuration
 */
export interface DeltaConfig {
  /** Compression level */
  level: CompressionLevel;
  /** Use predictive delta (vs simple XOR) */
  usePrediction: boolean;
  /** Enable run-length encoding for zeros */
  useRle: boolean;
  /** Context window size for prediction */
  contextSize: number;
  /** Data type hint for optimization */
  dataType: DeltaDataType;
}

/**
 * Mesh compression configuration
 */
export interface MeshCompressionConfig {
  /** Compression level */
  level: CompressionLevel;
  /** Position quantization bits (8-14) */
  positionBits: number;
  /** Normal quantization bits (8-12) */
  normalBits: number;
  /** UV quantization bits (10-14) */
  uvBits: number;
  /** Enable prediction for positions */
  usePrediction: boolean;
  /** Enable connectivity encoding */
  encodeConnectivity: boolean;
}

/**
 * Streaming compression configuration
 */
export interface StreamConfig {
  /** Compression level */
  level: CompressionLevel;
  /** Chunk size for streaming (in bytes) */
  chunkSize: number;
  /** Buffer size for I/O operations */
  bufferSize: number;
  /** Enable checksums for each chunk */
  useChecksums: boolean;
  /** Base algorithm to use for chunks */
  baseAlgorithm: 'lz4_custom' | 'delta' | 'raw';
}

/**
 * Dictionary compression configuration
 */
export interface DictionaryConfig {
  /** Compression level */
  level: CompressionLevel;
  /** Use dynamic dictionary learning */
  learnDictionary: boolean;
  /** Maximum dictionary size */
  maxDictSize: number;
  /** Minimum string length to consider */
  minStringLength: number;
}

/**
 * Parallel compression configuration
 */
export interface ParallelConfig {
  /** Compression level */
  level: CompressionLevel;
  /** Number of threads (0 = auto-detect) */
  numThreads: number;
  /** Chunk size for parallel processing */
  chunkSize: number;
  /** Base compressor to use */
  baseAlgorithm: 'lz4_custom' | 'delta' | 'dictionary';
  /** Enable adaptive chunk sizing */
  adaptiveChunks: boolean;
}

/**
 * Adaptive compression configuration
 */
export interface AdaptiveConfig {
  /** Strategy selection mode */
  strategy: CompressionStrategy;
  /** Compression level */
  level: CompressionLevel;
  /** Sample size for analysis (bytes) */
  sampleSize: number;
  /** Enable parallel compression when beneficial */
  useParallel: boolean;
}

/**
 * 3D Mesh data structure
 */
export interface Mesh {
  /** Vertex positions (x, y, z triplets) */
  positions: Float32Array;
  /** Vertex normals (x, y, z triplets) */
  normals: Float32Array;
  /** Texture coordinates (u, v pairs) */
  uvs: Float32Array;
  /** Triangle indices */
  indices: Uint32Array;
}

/**
 * CAD-specific dictionary
 */
export interface CADDictionary {
  /** Common CAD entity type names */
  entityTypes: string[];
  /** Common property names */
  propertyNames: string[];
  /** Common string values */
  commonValues: string[];
  /** Custom entries */
  customEntries: string[];
}

/**
 * Compressed data container
 */
export interface CompressedData {
  /** Format identifier */
  format: number;
  /** Version number */
  version: number;
  /** Flags (reserved for future use) */
  flags: number;
  /** Original uncompressed size */
  originalSize: number;
  /** Compressed payload */
  data: Uint8Array;
  /** Optional checksum (CRC32 or similar) */
  checksum?: number;
}

/**
 * Compression options for general use
 */
export interface CompressionOptions {
  /** Algorithm to use */
  algorithm?: CompressionAlgorithm;
  /** Compression level */
  level?: CompressionLevel;
  /** Algorithm-specific configuration */
  config?:
    | Lz4CustomConfig
    | DeltaConfig
    | MeshCompressionConfig
    | StreamConfig
    | DictionaryConfig
    | ParallelConfig
    | AdaptiveConfig;
}

/**
 * Compression result
 */
export interface CompressionResult {
  /** Compressed data */
  data: Uint8Array;
  /** Compression statistics */
  stats: CompressionStats;
  /** Original size */
  originalSize: number;
  /** Compressed size */
  compressedSize: number;
}

/**
 * Decompression result
 */
export interface DecompressionResult {
  /** Decompressed data */
  data: Uint8Array;
  /** Decompression statistics */
  stats?: CompressionStats;
}

/**
 * Compression worker message types
 */
export enum CompressionWorkerMessageType {
  /** Compress data request */
  Compress = 'compress',
  /** Decompress data request */
  Decompress = 'decompress',
  /** Compression result */
  CompressResult = 'compress_result',
  /** Decompression result */
  DecompressResult = 'decompress_result',
  /** Error occurred */
  Error = 'error',
  /** Progress update */
  Progress = 'progress',
}

/**
 * Compression worker message
 */
export interface CompressionWorkerMessage {
  /** Message type */
  type: CompressionWorkerMessageType;
  /** Request/response ID */
  id: string;
  /** Message payload */
  payload?: any;
  /** Error message (if type is Error) */
  error?: string;
}

/**
 * Compression progress event
 */
export interface CompressionProgress {
  /** Request ID */
  id: string;
  /** Progress percentage (0-100) */
  progress: number;
  /** Current phase */
  phase: 'analyzing' | 'compressing' | 'decompressing' | 'finalizing';
  /** Bytes processed */
  bytesProcessed: number;
  /** Total bytes */
  totalBytes: number;
}

/**
 * Compression settings state
 */
export interface CompressionSettings {
  /** Default algorithm */
  defaultAlgorithm: CompressionAlgorithm;
  /** Default compression level */
  defaultLevel: CompressionLevel;
  /** Enable auto-detection */
  enableAutoDetection: boolean;
  /** Use parallel compression for large files */
  useParallelForLargeFiles: boolean;
  /** Large file threshold (bytes) */
  largeFileThreshold: number;
  /** Enable compression statistics tracking */
  trackStatistics: boolean;
  /** Algorithm-specific settings */
  algorithmSettings: {
    lz4Custom?: Partial<Lz4CustomConfig>;
    delta?: Partial<DeltaConfig>;
    mesh?: Partial<MeshCompressionConfig>;
    streaming?: Partial<StreamConfig>;
    dictionary?: Partial<DictionaryConfig>;
    parallel?: Partial<ParallelConfig>;
    adaptive?: Partial<AdaptiveConfig>;
  };
}

/**
 * Compression performance metrics
 */
export interface CompressionPerformanceMetrics {
  /** Total files compressed */
  totalFilesCompressed: number;
  /** Total files decompressed */
  totalFilesDecompressed: number;
  /** Total bytes compressed */
  totalBytesCompressed: number;
  /** Total bytes decompressed */
  totalBytesDecompressed: number;
  /** Average compression ratio */
  averageCompressionRatio: number;
  /** Average compression speed (MB/s) */
  averageCompressionSpeed: number;
  /** Average decompression speed (MB/s) */
  averageDecompressionSpeed: number;
  /** Algorithm usage statistics */
  algorithmUsage: Record<CompressionAlgorithm, number>;
}

/**
 * Default compression configurations
 */
export const DEFAULT_CONFIGS = {
  lz4Custom: {
    level: CompressionLevel.Balanced,
    dictSize: 65536,
    optimizeFloats: true,
    optimizeIds: true,
    minMatch: 4,
    maxDistance: 65535,
  } as Lz4CustomConfig,

  delta: {
    level: CompressionLevel.Balanced,
    usePrediction: true,
    useRle: true,
    contextSize: 8,
    dataType: DeltaDataType.Mixed,
  } as DeltaConfig,

  mesh: {
    level: CompressionLevel.Balanced,
    positionBits: 12,
    normalBits: 10,
    uvBits: 12,
    usePrediction: true,
    encodeConnectivity: true,
  } as MeshCompressionConfig,

  streaming: {
    level: CompressionLevel.Balanced,
    chunkSize: 1024 * 1024, // 1MB
    bufferSize: 64 * 1024,  // 64KB
    useChecksums: true,
    baseAlgorithm: 'lz4_custom' as const,
  } as StreamConfig,

  dictionary: {
    level: CompressionLevel.Balanced,
    learnDictionary: true,
    maxDictSize: 4096,
    minStringLength: 3,
  } as DictionaryConfig,

  parallel: {
    level: CompressionLevel.Balanced,
    numThreads: 0, // Auto-detect
    chunkSize: 256 * 1024, // 256KB
    baseAlgorithm: 'lz4_custom' as const,
    adaptiveChunks: true,
  } as ParallelConfig,

  adaptive: {
    strategy: CompressionStrategy.Auto,
    level: CompressionLevel.Balanced,
    sampleSize: 4096,
    useParallel: true,
  } as AdaptiveConfig,
};

/**
 * Helper function to calculate compression percentage
 */
export function calculateCompressionPercentage(stats: CompressionStats): number {
  return (1.0 - stats.ratio) * 100.0;
}

/**
 * Helper function to calculate throughput in MB/s
 */
export function calculateThroughput(stats: CompressionStats): number {
  if (stats.compressionTimeMs === 0) {
    return 0;
  }
  return (stats.originalSize / 1_000_000) / (stats.compressionTimeMs / 1000);
}

/**
 * Helper function to format file size
 */
export function formatFileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

/**
 * Helper function to format duration
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms.toFixed(0)}ms`;
  } else if (ms < 60000) {
    return `${(ms / 1000).toFixed(2)}s`;
  } else {
    const minutes = Math.floor(ms / 60000);
    const seconds = ((ms % 60000) / 1000).toFixed(0);
    return `${minutes}m ${seconds}s`;
  }
}
