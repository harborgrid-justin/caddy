import React from 'react';
import { FileItem, FileShare } from './types';
interface FileShareProps {
    file: FileItem;
    tenantId: string;
    userId: string;
    onClose: () => void;
    onShareUpdate?: (share: FileShare) => void;
    className?: string;
}
export declare const FileShareComponent: React.FC<FileShareProps>;
export default FileShareComponent;
//# sourceMappingURL=FileShare.d.ts.map