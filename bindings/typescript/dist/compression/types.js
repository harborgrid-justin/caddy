export var CompressionLevel;
(function (CompressionLevel) {
    CompressionLevel[CompressionLevel["Fastest"] = 1] = "Fastest";
    CompressionLevel[CompressionLevel["Fast"] = 3] = "Fast";
    CompressionLevel[CompressionLevel["Balanced"] = 5] = "Balanced";
    CompressionLevel[CompressionLevel["Best"] = 7] = "Best";
    CompressionLevel[CompressionLevel["Maximum"] = 9] = "Maximum";
})(CompressionLevel || (CompressionLevel = {}));
export var CompressionAlgorithm;
(function (CompressionAlgorithm) {
    CompressionAlgorithm["LZ4Custom"] = "lz4_custom";
    CompressionAlgorithm["Delta"] = "delta";
    CompressionAlgorithm["Mesh"] = "mesh";
    CompressionAlgorithm["Streaming"] = "streaming";
    CompressionAlgorithm["Dictionary"] = "dictionary";
    CompressionAlgorithm["Parallel"] = "parallel";
    CompressionAlgorithm["Adaptive"] = "adaptive";
})(CompressionAlgorithm || (CompressionAlgorithm = {}));
export var CompressionFormat;
(function (CompressionFormat) {
    CompressionFormat[CompressionFormat["Lz4Custom"] = 1280980035] = "Lz4Custom";
    CompressionFormat[CompressionFormat["Delta"] = 1145392212] = "Delta";
    CompressionFormat[CompressionFormat["Mesh"] = 1296388936] = "Mesh";
    CompressionFormat[CompressionFormat["Dictionary"] = 1145652052] = "Dictionary";
    CompressionFormat[CompressionFormat["Adaptive"] = 1094996052] = "Adaptive";
})(CompressionFormat || (CompressionFormat = {}));
export var DeltaDataType;
(function (DeltaDataType) {
    DeltaDataType["Mixed"] = "mixed";
    DeltaDataType["Coordinates"] = "coordinates";
    DeltaDataType["Integers"] = "integers";
    DeltaDataType["Colors"] = "colors";
    DeltaDataType["Text"] = "text";
})(DeltaDataType || (DeltaDataType = {}));
export var CompressionStrategy;
(function (CompressionStrategy) {
    CompressionStrategy["AlwaysLz4"] = "always_lz4";
    CompressionStrategy["AlwaysDelta"] = "always_delta";
    CompressionStrategy["AlwaysMesh"] = "always_mesh";
    CompressionStrategy["AlwaysDictionary"] = "always_dictionary";
    CompressionStrategy["Auto"] = "auto";
    CompressionStrategy["Fastest"] = "fastest";
    CompressionStrategy["BestRatio"] = "best_ratio";
})(CompressionStrategy || (CompressionStrategy = {}));
export var CompressionWorkerMessageType;
(function (CompressionWorkerMessageType) {
    CompressionWorkerMessageType["Compress"] = "compress";
    CompressionWorkerMessageType["Decompress"] = "decompress";
    CompressionWorkerMessageType["CompressResult"] = "compress_result";
    CompressionWorkerMessageType["DecompressResult"] = "decompress_result";
    CompressionWorkerMessageType["Error"] = "error";
    CompressionWorkerMessageType["Progress"] = "progress";
})(CompressionWorkerMessageType || (CompressionWorkerMessageType = {}));
export const DEFAULT_CONFIGS = {
    lz4Custom: {
        level: CompressionLevel.Balanced,
        dictSize: 65536,
        optimizeFloats: true,
        optimizeIds: true,
        minMatch: 4,
        maxDistance: 65535,
    },
    delta: {
        level: CompressionLevel.Balanced,
        usePrediction: true,
        useRle: true,
        contextSize: 8,
        dataType: DeltaDataType.Mixed,
    },
    mesh: {
        level: CompressionLevel.Balanced,
        positionBits: 12,
        normalBits: 10,
        uvBits: 12,
        usePrediction: true,
        encodeConnectivity: true,
    },
    streaming: {
        level: CompressionLevel.Balanced,
        chunkSize: 1024 * 1024,
        bufferSize: 64 * 1024,
        useChecksums: true,
        baseAlgorithm: 'lz4_custom',
    },
    dictionary: {
        level: CompressionLevel.Balanced,
        learnDictionary: true,
        maxDictSize: 4096,
        minStringLength: 3,
    },
    parallel: {
        level: CompressionLevel.Balanced,
        numThreads: 0,
        chunkSize: 256 * 1024,
        baseAlgorithm: 'lz4_custom',
        adaptiveChunks: true,
    },
    adaptive: {
        strategy: CompressionStrategy.Auto,
        level: CompressionLevel.Balanced,
        sampleSize: 4096,
        useParallel: true,
    },
};
export function calculateCompressionPercentage(stats) {
    return (1.0 - stats.ratio) * 100.0;
}
export function calculateThroughput(stats) {
    if (stats.compressionTimeMs === 0) {
        return 0;
    }
    return (stats.originalSize / 1000000) / (stats.compressionTimeMs / 1000);
}
export function formatFileSize(bytes) {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
        size /= 1024;
        unitIndex++;
    }
    return `${size.toFixed(2)} ${units[unitIndex]}`;
}
export function formatDuration(ms) {
    if (ms < 1000) {
        return `${ms.toFixed(0)}ms`;
    }
    else if (ms < 60000) {
        return `${(ms / 1000).toFixed(2)}s`;
    }
    else {
        const minutes = Math.floor(ms / 60000);
        const seconds = ((ms % 60000) / 1000).toFixed(0);
        return `${minutes}m ${seconds}s`;
    }
}
//# sourceMappingURL=types.js.map