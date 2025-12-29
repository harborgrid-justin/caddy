import { CompressionOptions, CompressionResult, DecompressionResult, CompressionSettings, CompressionPerformanceMetrics } from './types';
export declare class CompressionService {
    private settings;
    private performanceMetrics;
    private compressionCache;
    private maxCacheSize;
    private currentCacheSize;
    constructor(settings?: Partial<CompressionSettings>);
    compress(data: Uint8Array, options?: CompressionOptions): Promise<CompressionResult>;
    decompress(data: Uint8Array): Promise<DecompressionResult>;
    compressFile(file: File, options?: CompressionOptions): Promise<CompressionResult>;
    getSettings(): CompressionSettings;
    updateSettings(settings: Partial<CompressionSettings>): void;
    getMetrics(): CompressionPerformanceMetrics;
    resetMetrics(): void;
    clearCache(): void;
    private compressLargeFile;
    private selectAlgorithm;
    private compressWithAlgorithm;
    private decompressWithAlgorithm;
    private detectAlgorithm;
    private simulateCompression;
    private simulateDecompression;
    private getExpectedRatio;
    private getAlgorithmMagic;
    private updateCompressionMetrics;
    private updateDecompressionMetrics;
    private getCacheKey;
    private cacheCompression;
}
export declare const compressionService: CompressionService;
export default CompressionService;
//# sourceMappingURL=CompressionService.d.ts.map