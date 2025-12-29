import React from 'react';
import { FileItem } from './types';
interface FileSearchProps {
    tenantId: string;
    onFileSelect?: (file: FileItem) => void;
    onFileOpen?: (file: FileItem) => void;
    initialQuery?: string;
    className?: string;
}
export declare const FileSearch: React.FC<FileSearchProps>;
export default FileSearch;
//# sourceMappingURL=FileSearch.d.ts.map