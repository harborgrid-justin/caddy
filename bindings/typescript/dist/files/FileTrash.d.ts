import React from 'react';
import { FileItem } from './types';
interface FileTrashProps {
    tenantId: string;
    onFileRestore?: (files: FileItem[]) => void;
    className?: string;
}
export declare const FileTrash: React.FC<FileTrashProps>;
export default FileTrash;
//# sourceMappingURL=FileTrash.d.ts.map