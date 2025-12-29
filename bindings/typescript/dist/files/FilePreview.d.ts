import React from 'react';
import { FileItem, PreviewConfig } from './types';
interface FilePreviewProps {
    file: FileItem;
    tenantId: string;
    config?: Partial<PreviewConfig>;
    onClose: () => void;
    onDownload?: () => void;
    onShare?: () => void;
    onDelete?: () => void;
    onPrevious?: () => void;
    onNext?: () => void;
    className?: string;
}
export declare const FilePreview: React.FC<FilePreviewProps>;
export default FilePreview;
//# sourceMappingURL=FilePreview.d.ts.map