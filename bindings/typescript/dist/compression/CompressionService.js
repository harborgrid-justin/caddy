import { CompressionAlgorithm, CompressionLevel, calculateThroughput, } from './types';
export class CompressionService {
    constructor(settings) {
        this.maxCacheSize = 100 * 1024 * 1024;
        this.currentCacheSize = 0;
        this.settings = {
            defaultAlgorithm: CompressionAlgorithm.Adaptive,
            defaultLevel: CompressionLevel.Balanced,
            enableAutoDetection: true,
            useParallelForLargeFiles: true,
            largeFileThreshold: 10 * 1024 * 1024,
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
            algorithmUsage: {},
        };
        this.compressionCache = new Map();
    }
    async compress(data, options) {
        const startTime = performance.now();
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
        const algorithm = options?.algorithm || this.selectAlgorithm(data);
        const level = options?.level || this.settings.defaultLevel;
        let compressed;
        let algorithmName;
        try {
            const result = await this.compressWithAlgorithm(data, algorithm, level, options);
            compressed = result.data;
            algorithmName = algorithm;
        }
        catch (error) {
            console.error(`Compression failed with ${algorithm}:`, error);
            const fallback = await this.compressWithAlgorithm(data, CompressionAlgorithm.LZ4Custom, level, options);
            compressed = fallback.data;
            algorithmName = CompressionAlgorithm.LZ4Custom;
        }
        const endTime = performance.now();
        const compressionTimeMs = endTime - startTime;
        const stats = {
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
        if (this.settings.trackStatistics) {
            this.updateCompressionMetrics(stats, algorithm);
        }
        this.cacheCompression(cacheKey, compressed, stats);
        return {
            data: compressed,
            stats,
            originalSize: data.length,
            compressedSize: compressed.length,
        };
    }
    async decompress(data) {
        const startTime = performance.now();
        const algorithm = this.detectAlgorithm(data);
        const decompressed = await this.decompressWithAlgorithm(data, algorithm);
        const endTime = performance.now();
        const decompressionTimeMs = endTime - startTime;
        const stats = {
            originalSize: decompressed.length,
            compressedSize: data.length,
            ratio: data.length / decompressed.length,
            compressionTimeMs: 0,
            decompressionTimeMs,
            algorithm: algorithm,
            metadata: {},
        };
        if (this.settings.trackStatistics) {
            this.updateDecompressionMetrics(stats);
        }
        return {
            data: decompressed,
            stats,
        };
    }
    async compressFile(file, options) {
        const data = new Uint8Array(await file.arrayBuffer());
        if (data.length > this.settings.largeFileThreshold &&
            this.settings.useParallelForLargeFiles) {
            return this.compressLargeFile(data, options);
        }
        return this.compress(data, options);
    }
    getSettings() {
        return { ...this.settings };
    }
    updateSettings(settings) {
        this.settings = { ...this.settings, ...settings };
    }
    getMetrics() {
        return { ...this.performanceMetrics };
    }
    resetMetrics() {
        this.performanceMetrics = {
            totalFilesCompressed: 0,
            totalFilesDecompressed: 0,
            totalBytesCompressed: 0,
            totalBytesDecompressed: 0,
            averageCompressionRatio: 0,
            averageCompressionSpeed: 0,
            averageDecompressionSpeed: 0,
            algorithmUsage: {},
        };
    }
    clearCache() {
        this.compressionCache.clear();
        this.currentCacheSize = 0;
    }
    async compressLargeFile(data, options) {
        return this.compressWithAlgorithm(data, CompressionAlgorithm.Parallel, options?.level || this.settings.defaultLevel, options);
    }
    selectAlgorithm(data) {
        if (!this.settings.enableAutoDetection) {
            return this.settings.defaultAlgorithm;
        }
        return CompressionAlgorithm.Adaptive;
    }
    async compressWithAlgorithm(data, algorithm, level, options) {
        const startTime = performance.now();
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
    async decompressWithAlgorithm(data, algorithm) {
        return this.simulateDecompression(data, algorithm);
    }
    detectAlgorithm(data) {
        if (data.length < 4) {
            return 'unknown';
        }
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
    async simulateCompression(data, algorithm, level) {
        const ratio = this.getExpectedRatio(algorithm);
        const targetSize = Math.floor(data.length * ratio);
        const compressed = new Uint8Array(targetSize + 8);
        const view = new DataView(compressed.buffer);
        view.setUint32(0, this.getAlgorithmMagic(algorithm), true);
        view.setUint32(4, data.length, true);
        for (let i = 0; i < targetSize; i++) {
            compressed[i + 8] = data[i % data.length];
        }
        return compressed;
    }
    async simulateDecompression(data, algorithm) {
        const view = new DataView(data.buffer);
        const originalSize = view.getUint32(4, true);
        const decompressed = new Uint8Array(originalSize);
        const compressedData = data.slice(8);
        for (let i = 0; i < originalSize; i++) {
            decompressed[i] = compressedData[i % compressedData.length];
        }
        return decompressed;
    }
    getExpectedRatio(algorithm) {
        switch (algorithm) {
            case CompressionAlgorithm.LZ4Custom:
                return 0.4;
            case CompressionAlgorithm.Delta:
                return 0.3;
            case CompressionAlgorithm.Mesh:
                return 0.2;
            case CompressionAlgorithm.Dictionary:
                return 0.35;
            case CompressionAlgorithm.Parallel:
                return 0.4;
            case CompressionAlgorithm.Adaptive:
                return 0.35;
            default:
                return 0.5;
        }
    }
    getAlgorithmMagic(algorithm) {
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
    updateCompressionMetrics(stats, algorithm) {
        this.performanceMetrics.totalFilesCompressed++;
        this.performanceMetrics.totalBytesCompressed += stats.originalSize;
        const totalRatio = this.performanceMetrics.averageCompressionRatio *
            (this.performanceMetrics.totalFilesCompressed - 1);
        this.performanceMetrics.averageCompressionRatio =
            (totalRatio + stats.ratio) / this.performanceMetrics.totalFilesCompressed;
        const speed = calculateThroughput(stats);
        const totalSpeed = this.performanceMetrics.averageCompressionSpeed *
            (this.performanceMetrics.totalFilesCompressed - 1);
        this.performanceMetrics.averageCompressionSpeed =
            (totalSpeed + speed) / this.performanceMetrics.totalFilesCompressed;
        this.performanceMetrics.algorithmUsage[algorithm] =
            (this.performanceMetrics.algorithmUsage[algorithm] || 0) + 1;
    }
    updateDecompressionMetrics(stats) {
        this.performanceMetrics.totalFilesDecompressed++;
        this.performanceMetrics.totalBytesDecompressed += stats.originalSize;
        if (stats.decompressionTimeMs) {
            const speed = (stats.originalSize / 1000000) / (stats.decompressionTimeMs / 1000);
            const totalSpeed = this.performanceMetrics.averageDecompressionSpeed *
                (this.performanceMetrics.totalFilesDecompressed - 1);
            this.performanceMetrics.averageDecompressionSpeed =
                (totalSpeed + speed) / this.performanceMetrics.totalFilesDecompressed;
        }
    }
    getCacheKey(data) {
        let hash = 0;
        const sample = data.slice(0, Math.min(1024, data.length));
        for (let i = 0; i < sample.length; i++) {
            hash = ((hash << 5) - hash + sample[i]) | 0;
        }
        return `${hash}_${data.length}`;
    }
    cacheCompression(key, compressed, stats) {
        const size = compressed.length;
        while (this.currentCacheSize + size > this.maxCacheSize &&
            this.compressionCache.size > 0) {
            const firstKey = this.compressionCache.keys().next().value;
            if (firstKey !== undefined) {
                const entry = this.compressionCache.get(firstKey);
                if (entry) {
                    this.currentCacheSize -= entry.compressed.length;
                    this.compressionCache.delete(firstKey);
                }
            }
        }
        this.compressionCache.set(key, {
            compressed,
            stats,
            timestamp: Date.now(),
        });
        this.currentCacheSize += size;
    }
}
export const compressionService = new CompressionService();
export default CompressionService;
//# sourceMappingURL=CompressionService.js.map