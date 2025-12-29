import React from 'react';
import { FileFormat, ImportOptions, ExportOptions, ImportResult, ExportResult, BatchConversionJob, BatchConversionResult, ValidationResult, IOEvent, IOEventType } from './types';
interface ImportExportContextType {
    importFile: (file: File, options?: ImportOptions) => Promise<ImportResult>;
    importFiles: (files: File[], options?: ImportOptions) => Promise<ImportResult[]>;
    exportFile: (format: FileFormat, options?: ExportOptions) => Promise<ExportResult>;
    createBatchJob: (job: BatchConversionJob) => string;
    executeBatchJob: (jobId: string) => Promise<BatchConversionResult>;
    cancelBatchJob: (jobId: string) => void;
    validateDocument: () => Promise<ValidationResult>;
    repairDocument: (issues: ValidationResult) => Promise<boolean>;
    isImporting: boolean;
    isExporting: boolean;
    progress: number;
    currentOperation: string | null;
    lastImportResult: ImportResult | null;
    lastExportResult: ExportResult | null;
    addEventListener: (type: IOEventType, callback: (event: IOEvent) => void) => void;
    removeEventListener: (type: IOEventType, callback: (event: IOEvent) => void) => void;
}
interface ImportExportProviderProps {
    children: React.ReactNode;
    onImportComplete?: (result: ImportResult) => void;
    onExportComplete?: (result: ExportResult) => void;
    onError?: (error: Error) => void;
}
export declare const ImportExportProvider: React.FC<ImportExportProviderProps>;
export declare const useImportExport: () => ImportExportContextType;
export {};
//# sourceMappingURL=ImportExportProvider.d.ts.map