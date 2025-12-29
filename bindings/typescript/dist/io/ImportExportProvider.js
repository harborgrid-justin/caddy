import React, { createContext, useContext, useState, useCallback, useRef } from 'react';
import { FileFormat, IOEventType, DEFAULT_EXPORT_OPTIONS, } from './types';
const ImportExportContext = createContext(undefined);
export const ImportExportProvider = ({ children, onImportComplete, onExportComplete, onError, }) => {
    const [isImporting, setIsImporting] = useState(false);
    const [isExporting, setIsExporting] = useState(false);
    const [progress, setProgress] = useState(0);
    const [currentOperation, setCurrentOperation] = useState(null);
    const [lastImportResult, setLastImportResult] = useState(null);
    const [lastExportResult, setLastExportResult] = useState(null);
    const eventListeners = useRef(new Map());
    const batchJobs = useRef(new Map());
    const emitEvent = useCallback((type, payload) => {
        const event = {
            type,
            payload,
            timestamp: Date.now(),
        };
        const listeners = eventListeners.current.get(type);
        if (listeners) {
            listeners.forEach(callback => callback(event));
        }
    }, []);
    const addEventListener = useCallback((type, callback) => {
        if (!eventListeners.current.has(type)) {
            eventListeners.current.set(type, new Set());
        }
        eventListeners.current.get(type).add(callback);
    }, []);
    const removeEventListener = useCallback((type, callback) => {
        const listeners = eventListeners.current.get(type);
        if (listeners) {
            listeners.delete(callback);
        }
    }, []);
    const importFile = useCallback(async (file, options = {}) => {
        setIsImporting(true);
        setProgress(0);
        setCurrentOperation(`Importing ${file.name}...`);
        const startTime = Date.now();
        try {
            emitEvent(IOEventType.IMPORT_START, { fileName: file.name });
            const format = options.format || detectFormat(file.name);
            const fileData = await readFileAsArrayBuffer(file);
            setProgress(25);
            const result = await invokeImport(fileData, format, options);
            setProgress(75);
            if (options.validateInput) {
                setCurrentOperation('Validating import...');
                const validationResult = await validateImportedData(result);
                if (!validationResult.valid && options.repairGeometry) {
                    setCurrentOperation('Repairing geometry...');
                    await repairImportedData(result);
                }
            }
            setProgress(100);
            const importResult = {
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
        }
        catch (error) {
            const importResult = {
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
        }
        finally {
            setIsImporting(false);
            setProgress(0);
            setCurrentOperation(null);
        }
    }, [emitEvent, onImportComplete, onError]);
    const importFiles = useCallback(async (files, options = {}) => {
        const results = [];
        for (let i = 0; i < files.length; i++) {
            setCurrentOperation(`Importing ${i + 1} of ${files.length}...`);
            const result = await importFile(files[i], options);
            results.push(result);
            setProgress(Math.round(((i + 1) / files.length) * 100));
        }
        return results;
    }, [importFile]);
    const exportFile = useCallback(async (format, options) => {
        setIsExporting(true);
        setProgress(0);
        setCurrentOperation(`Exporting to ${format}...`);
        const startTime = Date.now();
        const exportOptions = {
            ...DEFAULT_EXPORT_OPTIONS[format],
            ...options,
            format,
        };
        try {
            emitEvent(IOEventType.EXPORT_START, { format });
            const documentData = await getCurrentDocument();
            setProgress(25);
            if (exportOptions.validateOutput) {
                setCurrentOperation('Validating document...');
                const validationResult = await validateDocument();
                if (!validationResult.valid) {
                    throw new Error(`Document validation failed: ${validationResult.issues.length} issues found`);
                }
            }
            setProgress(50);
            setCurrentOperation('Converting format...');
            const exportData = await invokeExport(documentData, format, exportOptions);
            setProgress(75);
            setCurrentOperation('Saving file...');
            const blob = new Blob([exportData.buffer], { type: exportData.mimeType });
            const url = URL.createObjectURL(blob);
            const link = document.createElement('a');
            link.href = url;
            link.download = exportData.fileName;
            link.click();
            URL.revokeObjectURL(url);
            setProgress(100);
            const exportResult = {
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
        }
        catch (error) {
            const exportResult = {
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
        }
        finally {
            setIsExporting(false);
            setProgress(0);
            setCurrentOperation(null);
        }
    }, [emitEvent, onExportComplete, onError]);
    const createBatchJob = useCallback((job) => {
        const jobId = job.id || `batch-${Date.now()}`;
        batchJobs.current.set(jobId, { ...job, id: jobId });
        return jobId;
    }, []);
    const executeBatchJob = useCallback(async (jobId) => {
        const job = batchJobs.current.get(jobId);
        if (!job) {
            throw new Error(`Batch job ${jobId} not found`);
        }
        emitEvent(IOEventType.BATCH_START, { jobId });
        const results = await Promise.all(job.inputFiles.map(async (file, index) => {
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
            }
            catch (error) {
                return {
                    inputFile: file.name,
                    success: false,
                    error: error instanceof Error ? error.message : 'Unknown error',
                    warnings: [],
                    duration: 0,
                };
            }
        }));
        const batchResult = {
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
    const cancelBatchJob = useCallback((jobId) => {
        batchJobs.current.delete(jobId);
    }, []);
    const validateDocument = useCallback(async () => {
        emitEvent(IOEventType.VALIDATION_START, {});
        const document = await getCurrentDocument();
        const result = await invokeValidation(document);
        emitEvent(IOEventType.VALIDATION_COMPLETE, result);
        return result;
    }, [emitEvent]);
    const repairDocument = useCallback(async (issues) => {
        const document = await getCurrentDocument();
        return await invokeRepair(document, issues);
    }, []);
    const value = {
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
    return (React.createElement(ImportExportContext.Provider, { value: value }, children));
};
export const useImportExport = () => {
    const context = useContext(ImportExportContext);
    if (!context) {
        throw new Error('useImportExport must be used within ImportExportProvider');
    }
    return context;
};
function detectFormat(fileName) {
    const ext = fileName.split('.').pop()?.toLowerCase();
    const formatMap = {
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
function readFileAsArrayBuffer(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result);
        reader.onerror = () => reject(reader.error);
        reader.readAsArrayBuffer(file);
    });
}
async function invokeImport(data, format, options) {
    console.log('Invoking import:', { format, options });
    return { entityCount: 0, layerCount: 0, warnings: [] };
}
async function invokeExport(document, format, options) {
    console.log('Invoking export:', { format, options });
    return {
        buffer: new ArrayBuffer(0),
        fileName: `export.${format}`,
        mimeType: 'application/octet-stream',
        warnings: [],
    };
}
async function getCurrentDocument() {
    return {};
}
async function validateImportedData(data) {
    return { valid: true, issues: [], repairableCount: 0 };
}
async function repairImportedData(data) {
}
async function invokeValidation(document) {
    return { valid: true, issues: [], repairableCount: 0 };
}
async function invokeRepair(document, issues) {
    return true;
}
//# sourceMappingURL=ImportExportProvider.js.map