/**
 * CompressionWorker - Web Worker for async compression
 *
 * Handles compression/decompression in a background thread
 * to prevent blocking the main UI thread.
 */

import {
  CompressionWorkerMessage,
  CompressionWorkerMessageType,
  CompressionOptions,
  CompressionProgress,
} from './types';
import { CompressionService } from './CompressionService';

/**
 * Worker state
 */
let compressionService: CompressionService | null = null;
let activeOperations = new Map<string, AbortController>();

/**
 * Initialize compression service
 */
function initService(): CompressionService {
  if (!compressionService) {
    compressionService = new CompressionService();
  }
  return compressionService;
}

/**
 * Handle incoming messages
 */
self.onmessage = async (event: MessageEvent<CompressionWorkerMessage>) => {
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
  } catch (error) {
    sendError(id, error instanceof Error ? error.message : String(error));
  }
};

/**
 * Handle compression request
 */
async function handleCompress(
  id: string,
  payload: { data: Uint8Array; options?: CompressionOptions }
): Promise<void> {
  const service = initService();
  const { data, options } = payload;

  // Create abort controller
  const controller = new AbortController();
  activeOperations.set(id, controller);

  try {
    // Send progress update - analyzing
    sendProgress(id, {
      id,
      progress: 10,
      phase: 'analyzing',
      bytesProcessed: 0,
      totalBytes: data.length,
    });

    // Compress data
    sendProgress(id, {
      id,
      progress: 30,
      phase: 'compressing',
      bytesProcessed: 0,
      totalBytes: data.length,
    });

    const result = await service.compress(data, options);

    // Send progress update - finalizing
    sendProgress(id, {
      id,
      progress: 90,
      phase: 'finalizing',
      bytesProcessed: data.length,
      totalBytes: data.length,
    });

    // Send result
    self.postMessage({
      type: CompressionWorkerMessageType.CompressResult,
      id,
      payload: {
        data: result.data,
        stats: result.stats,
        originalSize: result.originalSize,
        compressedSize: result.compressedSize,
      },
    } as CompressionWorkerMessage);
  } catch (error) {
    sendError(id, error instanceof Error ? error.message : String(error));
  } finally {
    activeOperations.delete(id);
  }
}

/**
 * Handle decompression request
 */
async function handleDecompress(
  id: string,
  payload: { data: Uint8Array }
): Promise<void> {
  const service = initService();
  const { data } = payload;

  // Create abort controller
  const controller = new AbortController();
  activeOperations.set(id, controller);

  try {
    // Send progress update
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

    // Send result
    self.postMessage({
      type: CompressionWorkerMessageType.DecompressResult,
      id,
      payload: {
        data: result.data,
        stats: result.stats,
      },
    } as CompressionWorkerMessage);
  } catch (error) {
    sendError(id, error instanceof Error ? error.message : String(error));
  } finally {
    activeOperations.delete(id);
  }
}

/**
 * Send progress update
 */
function sendProgress(id: string, progress: CompressionProgress): void {
  self.postMessage({
    type: CompressionWorkerMessageType.Progress,
    id,
    payload: progress,
  } as CompressionWorkerMessage);
}

/**
 * Send error message
 */
function sendError(id: string, error: string): void {
  self.postMessage({
    type: CompressionWorkerMessageType.Error,
    id,
    error,
  } as CompressionWorkerMessage);
}

/**
 * Worker client for managing compression operations
 */
export class CompressionWorkerClient {
  private worker: Worker | null = null;
  private pendingRequests = new Map<
    string,
    {
      resolve: (value: any) => void;
      reject: (error: Error) => void;
      onProgress?: (progress: CompressionProgress) => void;
    }
  >();
  private nextRequestId = 0;

  constructor() {
    this.initWorker();
  }

  /**
   * Initialize worker
   */
  private initWorker(): void {
    try {
      // Create worker from this file
      this.worker = new Worker(new URL('./CompressionWorker.ts', import.meta.url), {
        type: 'module',
      });

      this.worker.onmessage = this.handleMessage.bind(this);
      this.worker.onerror = this.handleError.bind(this);
    } catch (error) {
      console.error('Failed to initialize compression worker:', error);
    }
  }

  /**
   * Handle message from worker
   */
  private handleMessage(event: MessageEvent<CompressionWorkerMessage>): void {
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
          request.onProgress(payload as CompressionProgress);
        }
        break;
    }
  }

  /**
   * Handle worker error
   */
  private handleError(error: ErrorEvent): void {
    console.error('Compression worker error:', error);

    // Reject all pending requests
    for (const [id, request] of this.pendingRequests.entries()) {
      request.reject(new Error(`Worker error: ${error.message}`));
    }
    this.pendingRequests.clear();

    // Attempt to restart worker
    this.terminate();
    this.initWorker();
  }

  /**
   * Compress data using worker
   */
  async compress(
    data: Uint8Array,
    options?: CompressionOptions,
    onProgress?: (progress: CompressionProgress) => void
  ): Promise<any> {
    if (!this.worker) {
      throw new Error('Worker not initialized');
    }

    const id = this.generateRequestId();

    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject, onProgress });

      this.worker!.postMessage({
        type: CompressionWorkerMessageType.Compress,
        id,
        payload: { data, options },
      } as CompressionWorkerMessage);
    });
  }

  /**
   * Decompress data using worker
   */
  async decompress(
    data: Uint8Array,
    onProgress?: (progress: CompressionProgress) => void
  ): Promise<any> {
    if (!this.worker) {
      throw new Error('Worker not initialized');
    }

    const id = this.generateRequestId();

    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject, onProgress });

      this.worker!.postMessage({
        type: CompressionWorkerMessageType.Decompress,
        id,
        payload: { data },
      } as CompressionWorkerMessage);
    });
  }

  /**
   * Terminate worker
   */
  terminate(): void {
    if (this.worker) {
      this.worker.terminate();
      this.worker = null;
    }

    // Reject all pending requests
    for (const [id, request] of this.pendingRequests.entries()) {
      request.reject(new Error('Worker terminated'));
    }
    this.pendingRequests.clear();
  }

  /**
   * Generate unique request ID
   */
  private generateRequestId(): string {
    return `req_${this.nextRequestId++}_${Date.now()}`;
  }

  /**
   * Get number of pending requests
   */
  get pendingCount(): number {
    return this.pendingRequests.size;
  }

  /**
   * Check if worker is ready
   */
  get isReady(): boolean {
    return this.worker !== null;
  }
}

/**
 * Export singleton worker client
 */
export const compressionWorker = new CompressionWorkerClient();

/**
 * Export default
 */
export default CompressionWorkerClient;
