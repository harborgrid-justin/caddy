import React, { useState, useCallback, useMemo, useRef } from 'react';
export const FileList = ({ files, viewMode, sortField, sortDirection, selectedFiles, onFileSelect, onFileOpen, onFileContextMenu, onSort, onDrop, showThumbnails = true, enableDragDrop = true, className = '', }) => {
    const [dragOverId, setDragOverId] = useState(null);
    const [draggedFiles, setDraggedFiles] = useState([]);
    const [rangeStartId, setRangeStartId] = useState(null);
    const listRef = useRef(null);
    const sortedFiles = useMemo(() => {
        const sorted = [...files].sort((a, b) => {
            let comparison = 0;
            if (a.type === 'folder' && b.type !== 'folder')
                return -1;
            if (a.type !== 'folder' && b.type === 'folder')
                return 1;
            switch (sortField) {
                case 'name':
                    comparison = a.name.localeCompare(b.name);
                    break;
                case 'size':
                    comparison = a.size - b.size;
                    break;
                case 'modified':
                    comparison = new Date(a.modifiedAt).getTime() - new Date(b.modifiedAt).getTime();
                    break;
                case 'created':
                    comparison = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
                    break;
                case 'type':
                    comparison = a.mimeType.localeCompare(b.mimeType);
                    break;
            }
            return sortDirection === 'asc' ? comparison : -comparison;
        });
        return sorted;
    }, [files, sortField, sortDirection]);
    const handleFileClick = useCallback((e, file) => {
        e.stopPropagation();
        if (e.shiftKey && rangeStartId) {
            const startIndex = sortedFiles.findIndex(f => f.id === rangeStartId);
            const endIndex = sortedFiles.findIndex(f => f.id === file.id);
            const [start, end] = startIndex < endIndex ? [startIndex, endIndex] : [endIndex, startIndex];
            const rangeIds = sortedFiles.slice(start, end + 1).map(f => f.id);
            rangeIds.forEach(id => {
                if (!selectedFiles.includes(id)) {
                    onFileSelect(id, true);
                }
            });
        }
        else {
            const isMulti = e.ctrlKey || e.metaKey;
            onFileSelect(file.id, isMulti);
            if (!isMulti) {
                setRangeStartId(file.id);
            }
        }
    }, [sortedFiles, selectedFiles, rangeStartId, onFileSelect]);
    const handleFileDoubleClick = useCallback((e, file) => {
        e.stopPropagation();
        onFileOpen(file);
    }, [onFileOpen]);
    const handleDragStart = useCallback((e, file) => {
        if (!enableDragDrop)
            return;
        const filesToDrag = selectedFiles.includes(file.id)
            ? selectedFiles
            : [file.id];
        setDraggedFiles(filesToDrag);
        e.dataTransfer.effectAllowed = 'move';
        e.dataTransfer.setData('application/caddy-files', JSON.stringify(filesToDrag));
        if (filesToDrag.length > 1) {
            const dragImage = document.createElement('div');
            dragImage.className = 'drag-image';
            dragImage.textContent = `${filesToDrag.length} items`;
            dragImage.style.position = 'absolute';
            dragImage.style.top = '-1000px';
            document.body.appendChild(dragImage);
            e.dataTransfer.setDragImage(dragImage, 0, 0);
            setTimeout(() => document.body.removeChild(dragImage), 0);
        }
    }, [selectedFiles, enableDragDrop]);
    const handleDragOver = useCallback((e, file) => {
        if (!enableDragDrop)
            return;
        e.preventDefault();
        e.stopPropagation();
        if (file.type === 'folder' && !draggedFiles.includes(file.id)) {
            e.dataTransfer.dropEffect = 'move';
            setDragOverId(file.id);
        }
        else {
            e.dataTransfer.dropEffect = 'none';
        }
    }, [draggedFiles, enableDragDrop]);
    const handleDragLeave = useCallback((e) => {
        e.stopPropagation();
        setDragOverId(null);
    }, []);
    const handleDrop = useCallback((e, targetFile) => {
        if (!enableDragDrop)
            return;
        e.preventDefault();
        e.stopPropagation();
        setDragOverId(null);
        const fileIds = JSON.parse(e.dataTransfer.getData('application/caddy-files') || '[]');
        const targetId = targetFile?.type === 'folder' ? targetFile.id : null;
        if (fileIds.length > 0 && onDrop) {
            onDrop(fileIds, targetId);
        }
        setDraggedFiles([]);
    }, [onDrop, enableDragDrop]);
    const handleDragEnd = useCallback(() => {
        setDragOverId(null);
        setDraggedFiles([]);
    }, []);
    const getFileIcon = useCallback((file) => {
        if (file.type === 'folder') {
            return '📁';
        }
        const ext = file.name.split('.').pop()?.toLowerCase() || '';
        const iconMap = {
            jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', svg: '🖼️', webp: '🖼️',
            pdf: '📕', doc: '📘', docx: '📘', xls: '📗', xlsx: '📗', ppt: '📙', pptx: '📙',
            txt: '📄', md: '📝',
            zip: '📦', rar: '📦', '7z': '📦', tar: '📦', gz: '📦',
            js: '📜', ts: '📜', jsx: '📜', tsx: '📜', html: '📜', css: '📜', json: '📜',
            py: '🐍', java: '☕', cpp: '⚙️', c: '⚙️',
            mp4: '🎬', mov: '🎬', avi: '🎬', mkv: '🎬',
            mp3: '🎵', wav: '🎵', flac: '🎵',
        };
        return iconMap[ext] || '📄';
    }, []);
    const formatSize = useCallback((bytes) => {
        if (bytes === 0)
            return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
    }, []);
    const formatDate = useCallback((date) => {
        const d = new Date(date);
        const now = new Date();
        const diffMs = now.getTime() - d.getTime();
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMs / 3600000);
        const diffDays = Math.floor(diffMs / 86400000);
        if (diffMins < 1)
            return 'Just now';
        if (diffMins < 60)
            return `${diffMins}m ago`;
        if (diffHours < 24)
            return `${diffHours}h ago`;
        if (diffDays < 7)
            return `${diffDays}d ago`;
        return d.toLocaleDateString();
    }, []);
    const renderSortIndicator = (field) => {
        if (sortField !== field)
            return null;
        return sortDirection === 'asc' ? ' ▲' : ' ▼';
    };
    const renderGridView = () => (React.createElement("div", { className: `file-grid ${className}`, ref: listRef }, sortedFiles.map(file => (React.createElement("div", { key: file.id, className: `file-grid-item ${selectedFiles.includes(file.id) ? 'selected' : ''} ${dragOverId === file.id ? 'drag-over' : ''} ${draggedFiles.includes(file.id) ? 'dragging' : ''}`, onClick: (e) => handleFileClick(e, file), onDoubleClick: (e) => handleFileDoubleClick(e, file), onContextMenu: (e) => onFileContextMenu(e, file), draggable: enableDragDrop, onDragStart: (e) => handleDragStart(e, file), onDragOver: (e) => handleDragOver(e, file), onDragLeave: handleDragLeave, onDrop: (e) => handleDrop(e, file), onDragEnd: handleDragEnd },
        React.createElement("div", { className: "file-grid-thumbnail" },
            showThumbnails && file.thumbnail ? (React.createElement("img", { src: file.thumbnail, alt: file.name })) : (React.createElement("span", { className: "file-grid-icon" }, getFileIcon(file))),
            file.isStarred && React.createElement("span", { className: "file-star" }, "\u2B50")),
        React.createElement("div", { className: "file-grid-name", title: file.name }, file.name),
        React.createElement("div", { className: "file-grid-meta" }, file.type === 'file' && formatSize(file.size)))))));
    const renderListView = () => (React.createElement("div", { className: `file-list-container ${className}` },
        React.createElement("div", { className: "file-list-header" },
            React.createElement("div", { className: "file-list-column file-list-name", onClick: () => onSort('name') },
                "Name",
                renderSortIndicator('name')),
            React.createElement("div", { className: "file-list-column file-list-size", onClick: () => onSort('size') },
                "Size",
                renderSortIndicator('size')),
            React.createElement("div", { className: "file-list-column file-list-type", onClick: () => onSort('type') },
                "Type",
                renderSortIndicator('type')),
            React.createElement("div", { className: "file-list-column file-list-modified", onClick: () => onSort('modified') },
                "Modified",
                renderSortIndicator('modified')),
            React.createElement("div", { className: "file-list-column file-list-owner" }, "Owner")),
        React.createElement("div", { className: "file-list-body", ref: listRef }, sortedFiles.map(file => (React.createElement("div", { key: file.id, className: `file-list-row ${selectedFiles.includes(file.id) ? 'selected' : ''} ${dragOverId === file.id ? 'drag-over' : ''} ${draggedFiles.includes(file.id) ? 'dragging' : ''}`, onClick: (e) => handleFileClick(e, file), onDoubleClick: (e) => handleFileDoubleClick(e, file), onContextMenu: (e) => onFileContextMenu(e, file), draggable: enableDragDrop, onDragStart: (e) => handleDragStart(e, file), onDragOver: (e) => handleDragOver(e, file), onDragLeave: handleDragLeave, onDrop: (e) => handleDrop(e, file), onDragEnd: handleDragEnd },
            React.createElement("div", { className: "file-list-column file-list-name" },
                React.createElement("span", { className: "file-icon" }, getFileIcon(file)),
                React.createElement("span", { className: "file-name-text", title: file.name }, file.name),
                file.isStarred && React.createElement("span", { className: "file-star" }, "\u2B50")),
            React.createElement("div", { className: "file-list-column file-list-size" }, file.type === 'file' ? formatSize(file.size) : '—'),
            React.createElement("div", { className: "file-list-column file-list-type" }, file.type === 'folder' ? 'Folder' : file.mimeType.split('/')[0]),
            React.createElement("div", { className: "file-list-column file-list-modified" }, formatDate(file.modifiedAt)),
            React.createElement("div", { className: "file-list-column file-list-owner" }, file.modifiedBy)))))));
    return viewMode === 'grid' ? renderGridView() : renderListView();
};
export default FileList;
//# sourceMappingURL=FileList.js.map