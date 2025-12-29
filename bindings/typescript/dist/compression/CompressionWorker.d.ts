import { CompressionOptions, CompressionProgress } from './types';
export declare class CompressionWorkerClient {
    private worker;
    private pendingRequests;
    private nextRequestId;
    constructor();
    private initWorker;
    private handleMessage;
    private handleError;
    compress(data: Uint8Array, options?: CompressionOptions, onProgress?: (progress: CompressionProgress) => void): Promise<any>;
    decompress(data: Uint8Array, onProgress?: (progress: CompressionProgress) => void): Promise<any>;
    terminate(): void;
    private generateRequestId;
    get pendingCount(): number;
    get isReady(): boolean;
}
export declare const compressionWorker: CompressionWorkerClient;
export default CompressionWorkerClient;
//# sourceMappingURL=CompressionWorker.d.ts.map