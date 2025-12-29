// CADDY - Enterprise CAD System
// TypeScript I/O Provider - React Context
// Agent 9 - Import/Export Pipeline Specialist

import React, { createContext, useContext, useState, useCallback, useRef } from 'react';
import {
  FileFormat,
  ImportOptions,
  ExportOptions,
  ImportResult,
  ExportResult,
  BatchConversionJob,
  BatchConversionResult,
  ValidationResult,
  IOEvent,
  IOEventType,
  ProgressCallback,
  DEFAULT_EXPORT_OPTIONS,
} from './types';

/**
 * Import/Export context interface
 */
interface ImportExportContextType {
  // Import methods
  importFile: (file: File, options?: ImportOptions) => Promise<ImportResult>;
  importFiles: (files: File[], options?: ImportOptions) => Promise<ImportResult[]>;

  // Export methods
  exportFile: (format: FileFormat, options?: ExportOptions) => Promise<ExportResult>;

  // Batch conversion
  createBatchJob: (job: BatchConversionJob) => string;
  executeBatchJob: (jobId: string) => Promise<BatchConversionResult>;
  cancelBatchJob: (jobId: string) => void;

  // Validation
  validateDocument: () => Promise<ValidationResult>;
  repairDocument: (issues: ValidationResult) => Promise<boolean>;

  // State
  isImporting: boolean;
  isExporting: boolean;
  progress: number;
  currentOperation: string | null;
  lastImportResult: ImportResult | null;
  lastExportResult: ExportResult | null;

  // Events
  addEventListener: (type: IOEventType, callback: (event: IOEvent) => void) => void;
  removeEventListener: (type: IOEventType, callback: (event: IOEvent) => void) => void;
}

/**
 * Create context
 */
const ImportExportContext = createContext<ImportExportContextType | undefined>(undefined);

/**
 * Provider props
 */
interface ImportExportProviderProps {
  children: React.ReactNode;
  onImportComplete?: (result: ImportResult) => void;
  onExportComplete?: (result: ExportResult) => void;
  onError?: (error: Error) => void;
}

/**
 * Import/Export Provider Component
 */
export const ImportExportProvider: React.FC<ImportExportProviderProps> = ({
  children,
  onImportComplete,
  onExportComplete,
  onError,
}) => {
  // State
  const [isImporting, setIsImporting] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [progress, setProgress] = useState(0);
  const [currentOperation, setCurrentOperation] = useState<string | null>(null);
  const [lastImportResult, setLastImportResult] = useState<ImportResult | null>(null);
  const [lastExportResult, setLastExportResult] = useState<ExportResult | null>(null);

  // Event listeners
  const eventListeners = useRef<Map<IOEventType, Set<(event: IOEvent) => void>>>(new Map());

  // Batch jobs
  const batchJobs = useRef<Map<string, BatchConversionJob>>(new Map());

  /**
   * Emit event
   */
  const emitEvent = useCallback((type: IOEventType, payload: any) => {
    const event: IOEvent = {
      type,
      payload,
      timestamp: Date.now(),
    };

    const listeners = eventListeners.current.get(type);
    if (listeners) {
      listeners.forEach(callback => callback(event));
    }
  }, []);

  /**
   * Add event listener
   */
  const addEventListener = useCallback((
    type: IOEventType,
    callback: (event: IOEvent) => void
  ) => {
    if (!eventListeners.current.has(type)) {
      eventListeners.current.set(type, new Set());
    }
    eventListeners.current.get(type)!.add(callback);
  }, []);

  /**
   * Remove event listener
   */
  const removeEventListener = useCallback((
    type: IOEventType,
    callback: (event: IOEvent) => void
  ) => {
    const listeners = eventListeners.current.get(type);
    if (listeners) {
      listeners.delete(callback);
    }
  }, []);

  /**
   * Import single file
   */
  const importFile = useCallback(async (
    file: File,
    options: ImportOptions = {}
  ): Promise<ImportResult> => {
    setIsImporting(true);
    setProgress(0);
    setCurrentOperation(`Importing ${file.name}...`);

    const startTime = Date.now();

    try {
      emitEvent(IOEventType.IMPORT_START, { fileName: file.name });

      // Detect format if not specified
      const format = options.format || detectFormat(file.name);

      // Read file
      const fileData = await readFileAsArrayBuffer(file);

      setProgress(25);

      // Call Rust backend via WebAssembly or API
      const result = await invokeImport(fileData, format, options);

      setProgress(75);

      // Validate if requested
      if (options.validateInput) {
        setCurrentOperation('Validating import...');
        const validationResult = await validateImportedData(result);

        if (!validationResult.valid && options.repairGeometry) {
          setCurrentOperation('Repairing geometry...');
          await repairImportedData(result);
        }
      }

      setProgress(100);

      const importResult: ImportResult = {
        success: true,
        fileName: file.name,
        format,
        entityCount: result.entityCount || 0,
        layerCount: result.layerCount || 0,
        warnings: result.warnings || [],
        errors: [],
        duration: Date.now() - startTime,
        fileSize: file.size,
        preview: result.preview,
      };

      setLastImportResult(importResult);
      emitEvent(IOEventType.IMPORT_COMPLETE, importResult);

      if (onImportComplete) {
        onImportComplete(importResult);
      }

      return importResult;
    } catch (error) {
      const importResult: ImportResult = {
        success: false,
        fileName: file.name,
        format: options.format || FileFormat.AUTO_DETECT,
        entityCount: 0,
        layerCount: 0,
        warnings: [],
        errors: [error instanceof Error ? error.message : 'Unknown error'],
        duration: Date.now() - startTime,
        fileSize: file.size,
      };

      emitEvent(IOEventType.IMPORT_ERROR, { error: error instanceof Error ? error.message : error });

      if (onError && error instanceof Error) {
        onError(error);
      }

      return importResult;
    } finally {
      setIsImporting(false);
      setProgress(0);
      setCurrentOperation(null);
    }
  }, [emitEvent, onImportComplete, onError]);

  /**
   * Import multiple files
   */
  const importFiles = useCallback(async (
    files: File[],
    options: ImportOptions = {}
  ): Promise<ImportResult[]> => {
    const results: ImportResult[] = [];

    for (let i = 0; i < files.length; i++) {
      setCurrentOperation(`Importing ${i + 1} of ${files.length}...`);
      const result = await importFile(files[i], options);
      results.push(result);

      // Update overall progress
      setProgress(Math.round(((i + 1) / files.length) * 100));
    }

    return results;
  }, [importFile]);

  /**
   * Export file
   */
  const exportFile = useCallback(async (
    format: FileFormat,
    options?: ExportOptions
  ): Promise<ExportResult> => {
    setIsExporting(true);
    setProgress(0);
    setCurrentOperation(`Exporting to ${format}...`);

    const startTime = Date.now();
    const exportOptions = {
      ...DEFAULT_EXPORT_OPTIONS[format],
      ...options,
      format,
    } as ExportOptions;

    try {
      emitEvent(IOEventType.EXPORT_START, { format });

      // Get current document from application state
      const documentData = await getCurrentDocument();

      setProgress(25);

      // Validate if requested
      if (exportOptions.validateOutput) {
        setCurrentOperation('Validating document...');
        const validationResult = await validateDocument();

        if (!validationResult.valid) {
          throw new Error(`Document validation failed: ${validationResult.issues.length} issues found`);
        }
      }

      setProgress(50);
      setCurrentOperation('Converting format...');

      // Call Rust backend for export
      const exportData = await invokeExport(documentData, format, exportOptions);

      setProgress(75);
      setCurrentOperation('Saving file...');

      // Trigger download
      const blob = new Blob([exportData.buffer], { type: exportData.mimeType });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = exportData.fileName;
      link.click();
      URL.revokeObjectURL(url);

      setProgress(100);

      const exportResult: ExportResult = {
        success: true,
        fileName: exportData.fileName,
        format,
        fileSize: exportData.buffer.byteLength,
        duration: Date.now() - startTime,
        warnings: exportData.warnings || [],
        errors: [],
      };

      setLastExportResult(exportResult);
      emitEvent(IOEventType.EXPORT_COMPLETE, exportResult);

      if (onExportComplete) {
        onExportComplete(exportResult);
      }

      return exportResult;
    } catch (error) {
      const exportResult: ExportResult = {
        success: false,
        fileName: `export.${format}`,
        format,
        fileSize: 0,
        duration: Date.now() - startTime,
        warnings: [],
        errors: [error instanceof Error ? error.message : 'Unknown error'],
      };

      emitEvent(IOEventType.EXPORT_ERROR, { error: error instanceof Error ? error.message : error });

      if (onError && error instanceof Error) {
        onError(error);
      }

      return exportResult;
    } finally {
      setIsExporting(false);
      setProgress(0);
      setCurrentOperation(null);
    }
  }, [emitEvent, onExportComplete, onError]);

  /**
   * Create batch conversion job
   */
  const createBatchJob = useCallback((job: BatchConversionJob): string => {
    const jobId = job.id || `batch-${Date.now()}`;
    batchJobs.current.set(jobId, { ...job, id: jobId });
    return jobId;
  }, []);

  /**
   * Execute batch conversion job
   */
  const executeBatchJob = useCallback(async (
    jobId: string
  ): Promise<BatchConversionResult> => {
    const job = batchJobs.current.get(jobId);

    if (!job) {
      throw new Error(`Batch job ${jobId} not found`);
    }

    emitEvent(IOEventType.BATCH_START, { jobId });

    // Import all files and export them
    const results = await Promise.all(
      job.inputFiles.map(async (file, index) => {
        try {
          const importResult = await importFile(file);

          if (!importResult.success) {
            return {
              inputFile: file.name,
              success: false,
              error: importResult.errors.join(', '),
              warnings: importResult.warnings,
              duration: importResult.duration,
            };
          }

          const exportResult = await exportFile(job.outputFormat, job.options);

          emitEvent(IOEventType.BATCH_PROGRESS, {
            jobId,
            current: index + 1,
            total: job.inputFiles.length,
          });

          return {
            inputFile: file.name,
            outputFile: exportResult.fileName,
            success: exportResult.success,
            error: exportResult.errors.join(', '),
            warnings: [...importResult.warnings, ...exportResult.warnings],
            duration: importResult.duration + exportResult.duration,
          };
        } catch (error) {
          return {
            inputFile: file.name,
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error',
            warnings: [],
            duration: 0,
          };
        }
      })
    );

    const batchResult: BatchConversionResult = {
      jobId,
      totalFiles: job.inputFiles.length,
      successful: results.filter(r => r.success).length,
      failed: results.filter(r => !r.success).length,
      results,
      totalDuration: results.reduce((sum, r) => sum + r.duration, 0),
      averageDuration: results.reduce((sum, r) => sum + r.duration, 0) / results.length,
    };

    emitEvent(IOEventType.BATCH_COMPLETE, batchResult);

    return batchResult;
  }, [importFile, exportFile, emitEvent]);

  /**
   * Cancel batch job
   */
  const cancelBatchJob = useCallback((jobId: string) => {
    batchJobs.current.delete(jobId);
  }, []);

  /**
   * Validate current document
   */
  const validateDocument = useCallback(async (): Promise<ValidationResult> => {
    emitEvent(IOEventType.VALIDATION_START, {});

    const document = await getCurrentDocument();
    const result = await invokeValidation(document);

    emitEvent(IOEventType.VALIDATION_COMPLETE, result);

    return result;
  }, [emitEvent]);

  /**
   * Repair document based on validation issues
   */
  const repairDocument = useCallback(async (issues: ValidationResult): Promise<boolean> => {
    const document = await getCurrentDocument();
    return await invokeRepair(document, issues);
  }, []);

  // Context value
  const value: ImportExportContextType = {
    importFile,
    importFiles,
    exportFile,
    createBatchJob,
    executeBatchJob,
    cancelBatchJob,
    validateDocument,
    repairDocument,
    isImporting,
    isExporting,
    progress,
    currentOperation,
    lastImportResult,
    lastExportResult,
    addEventListener,
    removeEventListener,
  };

  return (
    <ImportExportContext.Provider value={value}>
      {children}
    </ImportExportContext.Provider>
  );
};

/**
 * Hook to use Import/Export context
 */
export const useImportExport = (): ImportExportContextType => {
  const context = useContext(ImportExportContext);

  if (!context) {
    throw new Error('useImportExport must be used within ImportExportProvider');
  }

  return context;
};

// ===== Helper Functions =====

/**
 * Detect file format from filename
 */
function detectFormat(fileName: string): FileFormat {
  const ext = fileName.split('.').pop()?.toLowerCase();

  const formatMap: Record<string, FileFormat> = {
    'dxf': FileFormat.DXF,
    'dwg': FileFormat.DWG,
    'step': FileFormat.STEP,
    'stp': FileFormat.STEP,
    'iges': FileFormat.IGES,
    'igs': FileFormat.IGES,
    'stl': FileFormat.STL,
    'obj': FileFormat.OBJ,
    'gltf': FileFormat.GLTF,
    'glb': FileFormat.GLTF,
    'cdy': FileFormat.CADDY_BINARY,
    'cdyj': FileFormat.CADDY_JSON,
    'svg': FileFormat.SVG,
  };

  return ext ? formatMap[ext] || FileFormat.AUTO_DETECT : FileFormat.AUTO_DETECT;
}

/**
 * Read file as ArrayBuffer
 */
function readFileAsArrayBuffer(file: File): Promise<ArrayBuffer> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as ArrayBuffer);
    reader.onerror = () => reject(reader.error);
    reader.readAsArrayBuffer(file);
  });
}

// ===== Stub functions for Rust backend integration =====
// These would be replaced with actual WebAssembly or API calls

async function invokeImport(data: ArrayBuffer, format: FileFormat, options: ImportOptions): Promise<any> {
  // TODO: Call Rust backend via WebAssembly
  console.log('Invoking import:', { format, options });
  return { entityCount: 0, layerCount: 0, warnings: [] };
}

async function invokeExport(document: any, format: FileFormat, options: ExportOptions): Promise<any> {
  // TODO: Call Rust backend via WebAssembly
  console.log('Invoking export:', { format, options });
  return {
    buffer: new ArrayBuffer(0),
    fileName: `export.${format}`,
    mimeType: 'application/octet-stream',
    warnings: [],
  };
}

async function getCurrentDocument(): Promise<any> {
  // TODO: Get current document from application state
  return {};
}

async function validateImportedData(data: any): Promise<ValidationResult> {
  // TODO: Validate imported data
  return { valid: true, issues: [], repairableCount: 0 };
}

async function repairImportedData(data: any): Promise<void> {
  // TODO: Repair imported data
}

async function invokeValidation(document: any): Promise<ValidationResult> {
  // TODO: Call Rust validation
  return { valid: true, issues: [], repairableCount: 0 };
}

async function invokeRepair(document: any, issues: ValidationResult): Promise<boolean> {
  // TODO: Call Rust repair
  return true;
}
