import { CompressionWorkerMessageType, } from './types';
import { CompressionService } from './CompressionService';
let compressionService = null;
let activeOperations = new Map();
function initService() {
    if (!compressionService) {
        compressionService = new CompressionService();
    }
    return compressionService;
}
self.onmessage = async (event) => {
    const { type, id, payload } = event.data;
    try {
        switch (type) {
            case CompressionWorkerMessageType.Compress:
                await handleCompress(id, payload);
                break;
            case CompressionWorkerMessageType.Decompress:
                await handleDecompress(id, payload);
                break;
            default:
                sendError(id, `Unknown message type: ${type}`);
        }
    }
    catch (error) {
        sendError(id, error instanceof Error ? error.message : String(error));
    }
};
async function handleCompress(id, payload) {
    const service = initService();
    const { data, options } = payload;
    const controller = new AbortController();
    activeOperations.set(id, controller);
    try {
        sendProgress(id, {
            id,
            progress: 10,
            phase: 'analyzing',
            bytesProcessed: 0,
            totalBytes: data.length,
        });
        sendProgress(id, {
            id,
            progress: 30,
            phase: 'compressing',
            bytesProcessed: 0,
            totalBytes: data.length,
        });
        const result = await service.compress(data, options);
        sendProgress(id, {
            id,
            progress: 90,
            phase: 'finalizing',
            bytesProcessed: data.length,
            totalBytes: data.length,
        });
        self.postMessage({
            type: CompressionWorkerMessageType.CompressResult,
            id,
            payload: {
                data: result.data,
                stats: result.stats,
                originalSize: result.originalSize,
                compressedSize: result.compressedSize,
            },
        });
    }
    catch (error) {
        sendError(id, error instanceof Error ? error.message : String(error));
    }
    finally {
        activeOperations.delete(id);
    }
}
async function handleDecompress(id, payload) {
    const service = initService();
    const { data } = payload;
    const controller = new AbortController();
    activeOperations.set(id, controller);
    try {
        sendProgress(id, {
            id,
            progress: 20,
            phase: 'decompressing',
            bytesProcessed: 0,
            totalBytes: data.length,
        });
        const result = await service.decompress(data);
        sendProgress(id, {
            id,
            progress: 90,
            phase: 'finalizing',
            bytesProcessed: data.length,
            totalBytes: data.length,
        });
        self.postMessage({
            type: CompressionWorkerMessageType.DecompressResult,
            id,
            payload: {
                data: result.data,
                stats: result.stats,
            },
        });
    }
    catch (error) {
        sendError(id, error instanceof Error ? error.message : String(error));
    }
    finally {
        activeOperations.delete(id);
    }
}
function sendProgress(id, progress) {
    self.postMessage({
        type: CompressionWorkerMessageType.Progress,
        id,
        payload: progress,
    });
}
function sendError(id, error) {
    self.postMessage({
        type: CompressionWorkerMessageType.Error,
        id,
        error,
    });
}
export class CompressionWorkerClient {
    constructor() {
        this.worker = null;
        this.pendingRequests = new Map();
        this.nextRequestId = 0;
        this.initWorker();
    }
    initWorker() {
        try {
            this.worker = new Worker(new URL('./CompressionWorker.ts', import.meta.url), {
                type: 'module',
            });
            this.worker.onmessage = this.handleMessage.bind(this);
            this.worker.onerror = this.handleError.bind(this);
        }
        catch (error) {
            console.error('Failed to initialize compression worker:', error);
        }
    }
    handleMessage(event) {
        const { type, id, payload, error } = event.data;
        const request = this.pendingRequests.get(id);
        if (!request) {
            return;
        }
        switch (type) {
            case CompressionWorkerMessageType.CompressResult:
            case CompressionWorkerMessageType.DecompressResult:
                this.pendingRequests.delete(id);
                request.resolve(payload);
                break;
            case CompressionWorkerMessageType.Error:
                this.pendingRequests.delete(id);
                request.reject(new Error(error || 'Unknown error'));
                break;
            case CompressionWorkerMessageType.Progress:
                if (request.onProgress) {
                    request.onProgress(payload);
                }
                break;
        }
    }
    handleError(error) {
        console.error('Compression worker error:', error);
        for (const [id, request] of this.pendingRequests.entries()) {
            request.reject(new Error(`Worker error: ${error.message}`));
        }
        this.pendingRequests.clear();
        this.terminate();
        this.initWorker();
    }
    async compress(data, options, onProgress) {
        if (!this.worker) {
            throw new Error('Worker not initialized');
        }
        const id = this.generateRequestId();
        return new Promise((resolve, reject) => {
            this.pendingRequests.set(id, { resolve, reject, onProgress });
            this.worker.postMessage({
                type: CompressionWorkerMessageType.Compress,
                id,
                payload: { data, options },
            });
        });
    }
    async decompress(data, onProgress) {
        if (!this.worker) {
            throw new Error('Worker not initialized');
        }
        const id = this.generateRequestId();
        return new Promise((resolve, reject) => {
            this.pendingRequests.set(id, { resolve, reject, onProgress });
            this.worker.postMessage({
                type: CompressionWorkerMessageType.Decompress,
                id,
                payload: { data },
            });
        });
    }
    terminate() {
        if (this.worker) {
            this.worker.terminate();
            this.worker = null;
        }
        for (const [id, request] of this.pendingRequests.entries()) {
            request.reject(new Error('Worker terminated'));
        }
        this.pendingRequests.clear();
    }
    generateRequestId() {
        return `req_${this.nextRequestId++}_${Date.now()}`;
    }
    get pendingCount() {
        return this.pendingRequests.size;
    }
    get isReady() {
        return this.worker !== null;
    }
}
export const compressionWorker = new CompressionWorkerClient();
export default CompressionWorkerClient;
//# sourceMappingURL=CompressionWorker.js.map