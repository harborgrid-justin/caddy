import React from 'react';
import { FileItem, FileVersion } from './types';
interface FileVersionsProps {
    file: FileItem;
    tenantId: string;
    onClose: () => void;
    onVersionRestore?: (version: FileVersion) => void;
    className?: string;
}
export declare const FileVersions: React.FC<FileVersionsProps>;
export default FileVersions;
//# sourceMappingURL=FileVersions.d.ts.map