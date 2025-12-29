import React from 'react';
import { FileItem } from './types';
interface FileUploadProps {
    tenantId: string;
    parentId?: string | null;
    chunkSize?: number;
    maxConcurrentUploads?: number;
    maxRetries?: number;
    allowedFileTypes?: string[];
    maxFileSize?: number;
    onUploadComplete?: (files: FileItem[]) => void;
    onUploadError?: (error: Error) => void;
    onClose?: () => void;
    className?: string;
}
export declare const FileUpload: React.FC<FileUploadProps>;
export default FileUpload;
//# sourceMappingURL=FileUpload.d.ts.map