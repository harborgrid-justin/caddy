/**
 * CompressionService - High-level compression API
 *
 * Provides a unified interface for all compression algorithms
 * with automatic algorithm selection, caching, and performance tracking.
 */

import {
  CompressionAlgorithm,
  CompressionLevel,
  CompressionOptions,
  CompressionResult,
  DecompressionResult,
  CompressionStats,
  CompressionSettings,
  CompressionPerformanceMetrics,
  AdaptiveConfig,
  CompressionStrategy,
  DEFAULT_CONFIGS,
  calculateCompressionPercentage,
  calculateThroughput,
} from './types';

/**
 * Main compression service
 */
export class CompressionService {
  private settings: CompressionSettings;
  private performanceMetrics: CompressionPerformanceMetrics;
  private compressionCache: Map<string, CompressedCacheEntry>;
  private maxCacheSize: number = 100 * 1024 * 1024; // 100MB
  private currentCacheSize: number = 0;

  constructor(settings?: Partial<CompressionSettings>) {
    this.settings = {
      defaultAlgorithm: CompressionAlgorithm.Adaptive,
      defaultLevel: CompressionLevel.Balanced,
      enableAutoDetection: true,
      useParallelForLargeFiles: true,
      largeFileThreshold: 10 * 1024 * 1024, // 10MB
      trackStatistics: true,
      algorithmSettings: {},
      ...settings,
    };

    this.performanceMetrics = {
      totalFilesCompressed: 0,
      totalFilesDecompressed: 0,
      totalBytesCompressed: 0,
      totalBytesDecompressed: 0,
      averageCompressionRatio: 0,
      averageCompressionSpeed: 0,
      averageDecompressionSpeed: 0,
      algorithmUsage: {} as Record<CompressionAlgorithm, number>,
    };

    this.compressionCache = new Map();
  }

  /**
   * Compress data with automatic algorithm selection
   */
  async compress(
    data: Uint8Array,
    options?: CompressionOptions
  ): Promise<CompressionResult> {
    const startTime = performance.now();

    // Check cache
    const cacheKey = this.getCacheKey(data);
    const cached = this.compressionCache.get(cacheKey);
    if (cached && !options) {
      return {
        data: cached.compressed,
        stats: cached.stats,
        originalSize: data.length,
        compressedSize: cached.compressed.length,
      };
    }

    // Select algorithm
    const algorithm = options?.algorithm || this.selectAlgorithm(data);
    const level = options?.level || this.settings.defaultLevel;

    // Compress using selected algorithm
    let compressed: Uint8Array;
    let algorithmName: string;

    try {
      const result = await this.compressWithAlgorithm(data, algorithm, level, options);
      compressed = result.data;
      algorithmName = algorithm;
    } catch (error) {
      console.error(`Compression failed with ${algorithm}:`, error);
      // Fallback to LZ4
      const fallback = await this.compressWithAlgorithm(
        data,
        CompressionAlgorithm.LZ4Custom,
        level,
        options
      );
      compressed = fallback.data;
      algorithmName = CompressionAlgorithm.LZ4Custom;
    }

    const endTime = performance.now();
    const compressionTimeMs = endTime - startTime;

    // Create statistics
    const stats: CompressionStats = {
      originalSize: data.length,
      compressedSize: compressed.length,
      ratio: compressed.length / data.length,
      compressionTimeMs,
      algorithm: algorithmName,
      metadata: {
        level: level.toString(),
        algorithm: algorithm,
      },
    };

    // Update metrics
    if (this.settings.trackStatistics) {
      this.updateCompressionMetrics(stats, algorithm);
    }

    // Cache result
    this.cacheCompression(cacheKey, compressed, stats);

    return {
      data: compressed,
      stats,
      originalSize: data.length,
      compressedSize: compressed.length,
    };
  }

  /**
   * Decompress data
   */
  async decompress(data: Uint8Array): Promise<DecompressionResult> {
    const startTime = performance.now();

    // Detect algorithm from data header
    const algorithm = this.detectAlgorithm(data);

    // Decompress using detected algorithm
    const decompressed = await this.decompressWithAlgorithm(data, algorithm);

    const endTime = performance.now();
    const decompressionTimeMs = endTime - startTime;

    const stats: CompressionStats = {
      originalSize: decompressed.length,
      compressedSize: data.length,
      ratio: data.length / decompressed.length,
      compressionTimeMs: 0,
      decompressionTimeMs,
      algorithm: algorithm,
      metadata: {},
    };

    // Update metrics
    if (this.settings.trackStatistics) {
      this.updateDecompressionMetrics(stats);
    }

    return {
      data: decompressed,
      stats,
    };
  }

  /**
   * Compress file (with streaming for large files)
   */
  async compressFile(
    file: File,
    options?: CompressionOptions
  ): Promise<CompressionResult> {
    const data = new Uint8Array(await file.arrayBuffer());

    // Use streaming compression for large files
    if (
      data.length > this.settings.largeFileThreshold &&
      this.settings.useParallelForLargeFiles
    ) {
      return this.compressLargeFile(data, options);
    }

    return this.compress(data, options);
  }

  /**
   * Get compression settings
   */
  getSettings(): CompressionSettings {
    return { ...this.settings };
  }

  /**
   * Update compression settings
   */
  updateSettings(settings: Partial<CompressionSettings>): void {
    this.settings = { ...this.settings, ...settings };
  }

  /**
   * Get performance metrics
   */
  getMetrics(): CompressionPerformanceMetrics {
    return { ...this.performanceMetrics };
  }

  /**
   * Reset performance metrics
   */
  resetMetrics(): void {
    this.performanceMetrics = {
      totalFilesCompressed: 0,
      totalFilesDecompressed: 0,
      totalBytesCompressed: 0,
      totalBytesDecompressed: 0,
      averageCompressionRatio: 0,
      averageCompressionSpeed: 0,
      averageDecompressionSpeed: 0,
      algorithmUsage: {} as Record<CompressionAlgorithm, number>,
    };
  }

  /**
   * Clear compression cache
   */
  clearCache(): void {
    this.compressionCache.clear();
    this.currentCacheSize = 0;
  }

  /**
   * Compress large file using streaming/parallel compression
   */
  private async compressLargeFile(
    data: Uint8Array,
    options?: CompressionOptions
  ): Promise<CompressionResult> {
    // Use parallel compression for large files
    return this.compressWithAlgorithm(
      data,
      CompressionAlgorithm.Parallel,
      options?.level || this.settings.defaultLevel,
      options
    );
  }

  /**
   * Select best algorithm for data
   */
  private selectAlgorithm(data: Uint8Array): CompressionAlgorithm {
    if (!this.settings.enableAutoDetection) {
      return this.settings.defaultAlgorithm;
    }

    // Use adaptive compression for auto-detection
    return CompressionAlgorithm.Adaptive;
  }

  /**
   * Compress with specific algorithm
   */
  private async compressWithAlgorithm(
    data: Uint8Array,
    algorithm: CompressionAlgorithm,
    level: CompressionLevel,
    options?: CompressionOptions
  ): Promise<CompressionResult> {
    // Note: In a real implementation, this would call the Rust backend
    // via WASM or native bindings. For now, we'll simulate the compression.

    const startTime = performance.now();
    // This is a placeholder that would be replaced with actual WASM calls
    const compressed = await this.simulateCompression(data, algorithm, level);
    const endTime = performance.now();

    return {
      data: compressed,
      originalSize: data.length,
      compressedSize: compressed.length,
      stats: {
        originalSize: data.length,
        compressedSize: compressed.length,
        ratio: compressed.length / data.length,
        compressionTimeMs: endTime - startTime,
        decompressionTimeMs: 0,
        algorithm: algorithm,
        metadata: {},
      },
    };
  }

  /**
   * Decompress with specific algorithm
   */
  private async decompressWithAlgorithm(
    data: Uint8Array,
    algorithm: string
  ): Promise<Uint8Array> {
    // Note: In a real implementation, this would call the Rust backend
    // This is a placeholder
    return this.simulateDecompression(data, algorithm);
  }

  /**
   * Detect algorithm from compressed data header
   */
  private detectAlgorithm(data: Uint8Array): string {
    if (data.length < 4) {
      return 'unknown';
    }

    // Read magic bytes
    const magic = new DataView(data.buffer).getUint32(0, true);

    switch (magic) {
      case 0x4C5A3443:
        return CompressionAlgorithm.LZ4Custom;
      case 0x44454C54:
        return CompressionAlgorithm.Delta;
      case 0x4D455348:
        return CompressionAlgorithm.Mesh;
      case 0x44494354:
        return CompressionAlgorithm.Dictionary;
      case 0x41445054:
        return CompressionAlgorithm.Adaptive;
      default:
        return 'unknown';
    }
  }

  /**
   * Simulate compression (placeholder for WASM/native calls)
   */
  private async simulateCompression(
    data: Uint8Array,
    algorithm: CompressionAlgorithm,
    level: CompressionLevel
  ): Promise<Uint8Array> {
    // This would be replaced with actual compression via WASM
    // For now, just return a smaller array to simulate compression

    // Simple run-length encoding simulation
    const ratio = this.getExpectedRatio(algorithm);
    const targetSize = Math.floor(data.length * ratio);

    const compressed = new Uint8Array(targetSize + 8);

    // Write header (algorithm marker + original size)
    const view = new DataView(compressed.buffer);
    view.setUint32(0, this.getAlgorithmMagic(algorithm), true);
    view.setUint32(4, data.length, true);

    // Simulate compressed data
    for (let i = 0; i < targetSize; i++) {
      compressed[i + 8] = data[i % data.length];
    }

    return compressed;
  }

  /**
   * Simulate decompression (placeholder for WASM/native calls)
   */
  private async simulateDecompression(
    data: Uint8Array,
    algorithm: string
  ): Promise<Uint8Array> {
    // Read original size from header
    const view = new DataView(data.buffer);
    const originalSize = view.getUint32(4, true);

    // Simulate decompression
    const decompressed = new Uint8Array(originalSize);
    const compressedData = data.slice(8);

    for (let i = 0; i < originalSize; i++) {
      decompressed[i] = compressedData[i % compressedData.length];
    }

    return decompressed;
  }

  /**
   * Get expected compression ratio for algorithm
   */
  private getExpectedRatio(algorithm: CompressionAlgorithm): number {
    switch (algorithm) {
      case CompressionAlgorithm.LZ4Custom:
        return 0.4; // 60% compression
      case CompressionAlgorithm.Delta:
        return 0.3; // 70% compression
      case CompressionAlgorithm.Mesh:
        return 0.2; // 80% compression
      case CompressionAlgorithm.Dictionary:
        return 0.35; // 65% compression
      case CompressionAlgorithm.Parallel:
        return 0.4; // 60% compression
      case CompressionAlgorithm.Adaptive:
        return 0.35; // 65% compression
      default:
        return 0.5; // 50% compression
    }
  }

  /**
   * Get algorithm magic number
   */
  private getAlgorithmMagic(algorithm: CompressionAlgorithm): number {
    switch (algorithm) {
      case CompressionAlgorithm.LZ4Custom:
        return 0x4C5A3443;
      case CompressionAlgorithm.Delta:
        return 0x44454C54;
      case CompressionAlgorithm.Mesh:
        return 0x4D455348;
      case CompressionAlgorithm.Dictionary:
        return 0x44494354;
      case CompressionAlgorithm.Adaptive:
        return 0x41445054;
      default:
        return 0x00000000;
    }
  }

  /**
   * Update compression metrics
   */
  private updateCompressionMetrics(
    stats: CompressionStats,
    algorithm: CompressionAlgorithm
  ): void {
    this.performanceMetrics.totalFilesCompressed++;
    this.performanceMetrics.totalBytesCompressed += stats.originalSize;

    // Update average compression ratio
    const totalRatio =
      this.performanceMetrics.averageCompressionRatio *
      (this.performanceMetrics.totalFilesCompressed - 1);
    this.performanceMetrics.averageCompressionRatio =
      (totalRatio + stats.ratio) / this.performanceMetrics.totalFilesCompressed;

    // Update average compression speed
    const speed = calculateThroughput(stats);
    const totalSpeed =
      this.performanceMetrics.averageCompressionSpeed *
      (this.performanceMetrics.totalFilesCompressed - 1);
    this.performanceMetrics.averageCompressionSpeed =
      (totalSpeed + speed) / this.performanceMetrics.totalFilesCompressed;

    // Update algorithm usage
    this.performanceMetrics.algorithmUsage[algorithm] =
      (this.performanceMetrics.algorithmUsage[algorithm] || 0) + 1;
  }

  /**
   * Update decompression metrics
   */
  private updateDecompressionMetrics(stats: CompressionStats): void {
    this.performanceMetrics.totalFilesDecompressed++;
    this.performanceMetrics.totalBytesDecompressed += stats.originalSize;

    if (stats.decompressionTimeMs) {
      const speed =
        (stats.originalSize / 1_000_000) / (stats.decompressionTimeMs / 1000);
      const totalSpeed =
        this.performanceMetrics.averageDecompressionSpeed *
        (this.performanceMetrics.totalFilesDecompressed - 1);
      this.performanceMetrics.averageDecompressionSpeed =
        (totalSpeed + speed) / this.performanceMetrics.totalFilesDecompressed;
    }
  }

  /**
   * Get cache key for data
   */
  private getCacheKey(data: Uint8Array): string {
    // Simple hash function for cache key
    let hash = 0;
    const sample = data.slice(0, Math.min(1024, data.length));
    for (let i = 0; i < sample.length; i++) {
      hash = ((hash << 5) - hash + sample[i]) | 0;
    }
    return `${hash}_${data.length}`;
  }

  /**
   * Cache compression result
   */
  private cacheCompression(
    key: string,
    compressed: Uint8Array,
    stats: CompressionStats
  ): void {
    const size = compressed.length;

    // Check if we need to evict old entries
    while (
      this.currentCacheSize + size > this.maxCacheSize &&
      this.compressionCache.size > 0
    ) {
      const firstKey = this.compressionCache.keys().next().value;
      if (firstKey !== undefined) {
        const entry = this.compressionCache.get(firstKey);
        if (entry) {
          this.currentCacheSize -= entry.compressed.length;
          this.compressionCache.delete(firstKey);
        }
      }
    }

    // Add to cache
    this.compressionCache.set(key, {
      compressed,
      stats,
      timestamp: Date.now(),
    });
    this.currentCacheSize += size;
  }
}

/**
 * Cache entry structure
 */
interface CompressedCacheEntry {
  compressed: Uint8Array;
  stats: CompressionStats;
  timestamp: number;
}

/**
 * Export singleton instance
 */
export const compressionService = new CompressionService();

/**
 * Export default
 */
export default CompressionService;
