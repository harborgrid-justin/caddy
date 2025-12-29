export declare enum CompressionLevel {
    Fastest = 1,
    Fast = 3,
    Balanced = 5,
    Best = 7,
    Maximum = 9
}
export declare enum CompressionAlgorithm {
    LZ4Custom = "lz4_custom",
    Delta = "delta",
    Mesh = "mesh",
    Streaming = "streaming",
    Dictionary = "dictionary",
    Parallel = "parallel",
    Adaptive = "adaptive"
}
export declare enum CompressionFormat {
    Lz4Custom = 1280980035,
    Delta = 1145392212,
    Mesh = 1296388936,
    Dictionary = 1145652052,
    Adaptive = 1094996052
}
export declare enum DeltaDataType {
    Mixed = "mixed",
    Coordinates = "coordinates",
    Integers = "integers",
    Colors = "colors",
    Text = "text"
}
export declare enum CompressionStrategy {
    AlwaysLz4 = "always_lz4",
    AlwaysDelta = "always_delta",
    AlwaysMesh = "always_mesh",
    AlwaysDictionary = "always_dictionary",
    Auto = "auto",
    Fastest = "fastest",
    BestRatio = "best_ratio"
}
export interface CompressionStats {
    originalSize: number;
    compressedSize: number;
    ratio: number;
    compressionTimeMs: number;
    decompressionTimeMs?: number;
    algorithm: string;
    metadata: Record<string, string>;
}
export interface Lz4CustomConfig {
    level: CompressionLevel;
    dictSize: number;
    optimizeFloats: boolean;
    optimizeIds: boolean;
    minMatch: number;
    maxDistance: number;
}
export interface DeltaConfig {
    level: CompressionLevel;
    usePrediction: boolean;
    useRle: boolean;
    contextSize: number;
    dataType: DeltaDataType;
}
export interface MeshCompressionConfig {
    level: CompressionLevel;
    positionBits: number;
    normalBits: number;
    uvBits: number;
    usePrediction: boolean;
    encodeConnectivity: boolean;
}
export interface StreamConfig {
    level: CompressionLevel;
    chunkSize: number;
    bufferSize: number;
    useChecksums: boolean;
    baseAlgorithm: 'lz4_custom' | 'delta' | 'raw';
}
export interface DictionaryConfig {
    level: CompressionLevel;
    learnDictionary: boolean;
    maxDictSize: number;
    minStringLength: number;
}
export interface ParallelConfig {
    level: CompressionLevel;
    numThreads: number;
    chunkSize: number;
    baseAlgorithm: 'lz4_custom' | 'delta' | 'dictionary';
    adaptiveChunks: boolean;
}
export interface AdaptiveConfig {
    strategy: CompressionStrategy;
    level: CompressionLevel;
    sampleSize: number;
    useParallel: boolean;
}
export interface Mesh {
    positions: Float32Array;
    normals: Float32Array;
    uvs: Float32Array;
    indices: Uint32Array;
}
export interface CADDictionary {
    entityTypes: string[];
    propertyNames: string[];
    commonValues: string[];
    customEntries: string[];
}
export interface CompressedData {
    format: number;
    version: number;
    flags: number;
    originalSize: number;
    data: Uint8Array;
    checksum?: number;
}
export interface CompressionOptions {
    algorithm?: CompressionAlgorithm;
    level?: CompressionLevel;
    config?: Lz4CustomConfig | DeltaConfig | MeshCompressionConfig | StreamConfig | DictionaryConfig | ParallelConfig | AdaptiveConfig;
}
export interface CompressionResult {
    data: Uint8Array;
    stats: CompressionStats;
    originalSize: number;
    compressedSize: number;
}
export interface DecompressionResult {
    data: Uint8Array;
    stats?: CompressionStats;
}
export declare enum CompressionWorkerMessageType {
    Compress = "compress",
    Decompress = "decompress",
    CompressResult = "compress_result",
    DecompressResult = "decompress_result",
    Error = "error",
    Progress = "progress"
}
export interface CompressionWorkerMessage {
    type: CompressionWorkerMessageType;
    id: string;
    payload?: any;
    error?: string;
}
export interface CompressionProgress {
    id: string;
    progress: number;
    phase: 'analyzing' | 'compressing' | 'decompressing' | 'finalizing';
    bytesProcessed: number;
    totalBytes: number;
}
export interface CompressionSettings {
    defaultAlgorithm: CompressionAlgorithm;
    defaultLevel: CompressionLevel;
    enableAutoDetection: boolean;
    useParallelForLargeFiles: boolean;
    largeFileThreshold: number;
    trackStatistics: boolean;
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
export interface CompressionPerformanceMetrics {
    totalFilesCompressed: number;
    totalFilesDecompressed: number;
    totalBytesCompressed: number;
    totalBytesDecompressed: number;
    averageCompressionRatio: number;
    averageCompressionSpeed: number;
    averageDecompressionSpeed: number;
    algorithmUsage: Record<CompressionAlgorithm, number>;
}
export declare const DEFAULT_CONFIGS: {
    lz4Custom: Lz4CustomConfig;
    delta: DeltaConfig;
    mesh: MeshCompressionConfig;
    streaming: StreamConfig;
    dictionary: DictionaryConfig;
    parallel: ParallelConfig;
    adaptive: AdaptiveConfig;
};
export declare function calculateCompressionPercentage(stats: CompressionStats): number;
export declare function calculateThroughput(stats: CompressionStats): number;
export declare function formatFileSize(bytes: number): string;
export declare function formatDuration(ms: number): string;
//# sourceMappingURL=types.d.ts.map