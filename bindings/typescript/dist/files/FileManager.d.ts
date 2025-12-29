import React from 'react';
import { FileManagerConfig, FileManagerCallbacks } from './types';
interface FileManagerProps {
    rootFolderId?: string | null;
    config?: Partial<FileManagerConfig>;
    callbacks?: FileManagerCallbacks;
    tenantId: string;
    userId: string;
    className?: string;
}
export declare const FileManager: React.FC<FileManagerProps>;
export default FileManager;
//# sourceMappingURL=FileManager.d.ts.map