import React from 'react';
import { FileItem } from './types';
interface FileRecentProps {
    tenantId: string;
    onFileSelect?: (file: FileItem) => void;
    onFileOpen?: (file: FileItem) => void;
    limit?: number;
    className?: string;
}
export declare const FileRecent: React.FC<FileRecentProps>;
export default FileRecent;
//# sourceMappingURL=FileRecent.d.ts.map