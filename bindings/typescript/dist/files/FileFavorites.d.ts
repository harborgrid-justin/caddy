import React from 'react';
import { FileItem } from './types';
interface FileFavoritesProps {
    tenantId: string;
    onFileSelect?: (file: FileItem) => void;
    onFileOpen?: (file: FileItem) => void;
    className?: string;
}
export declare const FileFavorites: React.FC<FileFavoritesProps>;
export default FileFavorites;
//# sourceMappingURL=FileFavorites.d.ts.map