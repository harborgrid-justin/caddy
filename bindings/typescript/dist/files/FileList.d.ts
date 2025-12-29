import React from 'react';
import { FileItem, ViewMode, SortField, SortDirection } from './types';
interface FileListProps {
    files: FileItem[];
    viewMode: ViewMode;
    sortField: SortField;
    sortDirection: SortDirection;
    selectedFiles: string[];
    onFileSelect: (fileId: string, multi: boolean) => void;
    onFileOpen: (file: FileItem) => void;
    onFileContextMenu: (e: React.MouseEvent, file: FileItem) => void;
    onSort: (field: SortField) => void;
    onDrop?: (fileIds: string[], targetId: string | null) => void;
    showThumbnails?: boolean;
    enableDragDrop?: boolean;
    className?: string;
}
export declare const FileList: React.FC<FileListProps>;
export default FileList;
//# sourceMappingURL=FileList.d.ts.map