import React, { useState, useEffect, useCallback, useRef } from 'react';
const DEFAULT_CONFIG = {
    chunkSize: 5 * 1024 * 1024,
    maxConcurrentUploads: 3,
    maxRetries: 3,
    allowedFileTypes: [],
    maxFileSize: 5 * 1024 * 1024 * 1024,
    thumbnailSize: 200,
    previewFormats: {
        images: ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'],
        videos: ['mp4', 'webm', 'ogg', 'mov', 'avi'],
        documents: ['pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'txt', 'md'],
        audio: ['mp3', 'wav', 'ogg', 'flac', 'm4a'],
    },
    enableVersioning: true,
    enableCloudSync: true,
    enableFullTextSearch: true,
    trashRetentionDays: 30,
};
export const FileManager = ({ rootFolderId = null, config: userConfig = {}, callbacks = {}, tenantId, userId, className = '', }) => {
    const config = { ...DEFAULT_CONFIG, ...userConfig };
    const [state, setState] = useState({
        currentPath: '/',
        currentFolderId: rootFolderId,
        files: [],
        selectedFiles: [],
        viewMode: 'grid',
        sortField: 'name',
        sortDirection: 'asc',
        loading: false,
        error: null,
        breadcrumbs: [{ id: 'root', name: 'My Files', path: '/' }],
        clipboard: {
            items: [],
            operation: null,
        },
    });
    const [showUploadPanel, setShowUploadPanel] = useState(false);
    const [showPreview, setShowPreview] = useState(false);
    const [previewFile, setPreviewFile] = useState(null);
    const [showShareDialog, setShowShareDialog] = useState(false);
    const [shareFile, setShareFile] = useState(null);
    const [showVersions, setShowVersions] = useState(false);
    const [versionsFile, setVersionsFile] = useState(null);
    const [contextMenu, setContextMenu] = useState(null);
    const [operations, setOperations] = useState([]);
    const fileListRef = useRef(null);
    const loadFiles = useCallback(async () => {
        setState(prev => ({ ...prev, loading: true, error: null }));
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files?folderId=${state.currentFolderId || 'root'}&sort=${state.sortField}&direction=${state.sortDirection}`, {
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error(`Failed to load files: ${response.statusText}`);
            }
            const data = await response.json();
            setState(prev => ({ ...prev, files: data.files, loading: false }));
        }
        catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Failed to load files';
            setState(prev => ({ ...prev, error: errorMessage, loading: false }));
            callbacks.onError?.(error instanceof Error ? error : new Error(errorMessage));
        }
    }, [tenantId, state.currentFolderId, state.sortField, state.sortDirection, callbacks]);
    useEffect(() => {
        loadFiles();
    }, [loadFiles]);
    const navigateToFolder = useCallback(async (folderId, folderName) => {
        let newBreadcrumbs = [];
        let newPath = '/';
        if (folderId) {
            try {
                const response = await fetch(`/api/v1/tenants/${tenantId}/files/${folderId}/path`, {
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    },
                });
                if (response.ok) {
                    const pathData = await response.json();
                    newBreadcrumbs = pathData.breadcrumbs;
                    newPath = pathData.path;
                }
            }
            catch (error) {
                console.error('Failed to fetch folder path:', error);
            }
        }
        else {
            newBreadcrumbs = [{ id: 'root', name: 'My Files', path: '/' }];
        }
        setState(prev => ({
            ...prev,
            currentFolderId: folderId,
            currentPath: newPath,
            breadcrumbs: newBreadcrumbs,
            selectedFiles: [],
        }));
        callbacks.onFolderChange?.(folderId);
    }, [tenantId, callbacks]);
    const toggleFileSelection = useCallback((fileId, multi = false) => {
        setState(prev => {
            if (multi) {
                const isSelected = prev.selectedFiles.includes(fileId);
                return {
                    ...prev,
                    selectedFiles: isSelected
                        ? prev.selectedFiles.filter(id => id !== fileId)
                        : [...prev.selectedFiles, fileId],
                };
            }
            else {
                return {
                    ...prev,
                    selectedFiles: [fileId],
                };
            }
        });
    }, []);
    const selectAll = useCallback(() => {
        setState(prev => ({
            ...prev,
            selectedFiles: prev.files.map(f => f.id),
        }));
    }, []);
    const clearSelection = useCallback(() => {
        setState(prev => ({
            ...prev,
            selectedFiles: [],
        }));
    }, []);
    const createFolder = useCallback(async (name) => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/folders`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    name,
                    parentId: state.currentFolderId,
                }),
            });
            if (!response.ok) {
                throw new Error('Failed to create folder');
            }
            await loadFiles();
        }
        catch (error) {
            callbacks.onError?.(error instanceof Error ? error : new Error('Failed to create folder'));
        }
    }, [tenantId, state.currentFolderId, loadFiles, callbacks]);
    const deleteFiles = useCallback(async (fileIds) => {
        const operationId = `delete-${Date.now()}`;
        const operation = {
            id: operationId,
            type: 'delete',
            fileIds,
            status: 'processing',
            progress: 0,
            total: fileIds.length,
            completed: 0,
            failed: 0,
            createdAt: new Date(),
        };
        setOperations(prev => [...prev, operation]);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/bulk-delete`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ fileIds }),
            });
            if (!response.ok) {
                throw new Error('Failed to delete files');
            }
            const result = await response.json();
            setOperations(prev => prev.map(op => op.id === operationId
                ? { ...op, status: 'complete', progress: 100, completed: result.successCount, failed: result.failedCount, completedAt: new Date() }
                : op));
            callbacks.onFileDelete?.(result.success);
            await loadFiles();
            clearSelection();
        }
        catch (error) {
            setOperations(prev => prev.map(op => op.id === operationId
                ? { ...op, status: 'error', error: error instanceof Error ? error.message : 'Delete failed', completedAt: new Date() }
                : op));
            callbacks.onError?.(error instanceof Error ? error : new Error('Failed to delete files'));
        }
    }, [tenantId, loadFiles, clearSelection, callbacks]);
    const renameFile = useCallback(async (fileId, newName) => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${fileId}/rename`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ name: newName }),
            });
            if (!response.ok) {
                throw new Error('Failed to rename file');
            }
            await loadFiles();
        }
        catch (error) {
            callbacks.onError?.(error instanceof Error ? error : new Error('Failed to rename file'));
        }
    }, [tenantId, loadFiles, callbacks]);
    const moveFiles = useCallback(async (fileIds, destinationId) => {
        const operationId = `move-${Date.now()}`;
        const operation = {
            id: operationId,
            type: 'move',
            fileIds,
            status: 'processing',
            progress: 0,
            total: fileIds.length,
            completed: 0,
            failed: 0,
            destination: destinationId || 'root',
            createdAt: new Date(),
        };
        setOperations(prev => [...prev, operation]);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/bulk-move`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ fileIds, destinationId }),
            });
            if (!response.ok) {
                throw new Error('Failed to move files');
            }
            const result = await response.json();
            setOperations(prev => prev.map(op => op.id === operationId
                ? { ...op, status: 'complete', progress: 100, completed: result.successCount, failed: result.failedCount, completedAt: new Date() }
                : op));
            await loadFiles();
            clearSelection();
        }
        catch (error) {
            setOperations(prev => prev.map(op => op.id === operationId
                ? { ...op, status: 'error', error: error instanceof Error ? error.message : 'Move failed', completedAt: new Date() }
                : op));
            callbacks.onError?.(error instanceof Error ? error : new Error('Failed to move files'));
        }
    }, [tenantId, loadFiles, clearSelection, callbacks]);
    const copyFiles = useCallback(async (fileIds, destinationId) => {
        const operationId = `copy-${Date.now()}`;
        const operation = {
            id: operationId,
            type: 'copy',
            fileIds,
            status: 'processing',
            progress: 0,
            total: fileIds.length,
            completed: 0,
            failed: 0,
            destination: destinationId || 'root',
            createdAt: new Date(),
        };
        setOperations(prev => [...prev, operation]);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/bulk-copy`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ fileIds, destinationId }),
            });
            if (!response.ok) {
                throw new Error('Failed to copy files');
            }
            const result = await response.json();
            setOperations(prev => prev.map(op => op.id === operationId
                ? { ...op, status: 'complete', progress: 100, completed: result.successCount, failed: result.failedCount, completedAt: new Date() }
                : op));
            await loadFiles();
        }
        catch (error) {
            setOperations(prev => prev.map(op => op.id === operationId
                ? { ...op, status: 'error', error: error instanceof Error ? error.message : 'Copy failed', completedAt: new Date() }
                : op));
            callbacks.onError?.(error instanceof Error ? error : new Error('Failed to copy files'));
        }
    }, [tenantId, loadFiles, callbacks]);
    const toggleStar = useCallback(async (fileId) => {
        try {
            const file = state.files.find(f => f.id === fileId);
            if (!file)
                return;
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${fileId}/star`, {
                method: file.isStarred ? 'DELETE' : 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to toggle star');
            }
            await loadFiles();
        }
        catch (error) {
            callbacks.onError?.(error instanceof Error ? error : new Error('Failed to toggle star'));
        }
    }, [tenantId, state.files, loadFiles, callbacks]);
    const downloadFiles = useCallback(async (fileIds) => {
        try {
            if (fileIds.length === 1) {
                window.open(`/api/v1/tenants/${tenantId}/files/${fileIds[0]}/download?token=${localStorage.getItem('token')}`, '_blank');
            }
            else {
                const response = await fetch(`/api/v1/tenants/${tenantId}/files/bulk-download`, {
                    method: 'POST',
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('token')}`,
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ fileIds }),
                });
                if (!response.ok) {
                    throw new Error('Failed to prepare download');
                }
                const { downloadUrl } = await response.json();
                window.open(downloadUrl, '_blank');
            }
        }
        catch (error) {
            callbacks.onError?.(error instanceof Error ? error : new Error('Failed to download files'));
        }
    }, [tenantId, callbacks]);
    const setViewMode = useCallback((mode) => {
        setState(prev => ({ ...prev, viewMode: mode }));
    }, []);
    const setSorting = useCallback((field, direction) => {
        setState(prev => ({
            ...prev,
            sortField: field,
            sortDirection: direction || (prev.sortField === field && prev.sortDirection === 'asc' ? 'desc' : 'asc'),
        }));
    }, []);
    useEffect(() => {
        const handleKeyDown = (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
                e.preventDefault();
                selectAll();
            }
            if (e.key === 'Delete' && state.selectedFiles.length > 0) {
                e.preventDefault();
                deleteFiles(state.selectedFiles);
            }
            if (e.key === 'Escape') {
                clearSelection();
                setContextMenu(null);
            }
            if ((e.ctrlKey || e.metaKey) && e.key === 'c' && state.selectedFiles.length > 0) {
                e.preventDefault();
                setState(prev => ({
                    ...prev,
                    clipboard: { items: prev.selectedFiles, operation: 'copy' },
                }));
            }
            if ((e.ctrlKey || e.metaKey) && e.key === 'x' && state.selectedFiles.length > 0) {
                e.preventDefault();
                setState(prev => ({
                    ...prev,
                    clipboard: { items: prev.selectedFiles, operation: 'cut' },
                }));
            }
            if ((e.ctrlKey || e.metaKey) && e.key === 'v' && state.clipboard.items.length > 0) {
                e.preventDefault();
                if (state.clipboard.operation === 'copy') {
                    copyFiles(state.clipboard.items, state.currentFolderId);
                }
                else if (state.clipboard.operation === 'cut') {
                    moveFiles(state.clipboard.items, state.currentFolderId);
                    setState(prev => ({ ...prev, clipboard: { items: [], operation: null } }));
                }
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [state.selectedFiles, state.clipboard, state.currentFolderId, selectAll, clearSelection, deleteFiles, copyFiles, moveFiles]);
    const handleContextMenu = useCallback((e, file) => {
        e.preventDefault();
        setContextMenu({ x: e.clientX, y: e.clientY, file });
    }, []);
    useEffect(() => {
        const handleClick = () => setContextMenu(null);
        if (contextMenu) {
            window.addEventListener('click', handleClick);
            return () => window.removeEventListener('click', handleClick);
        }
        return undefined;
    }, [contextMenu]);
    const handleFileOpen = useCallback((file) => {
        if (file.type === 'folder') {
            navigateToFolder(file.id, file.name);
        }
        else {
            setPreviewFile(file);
            setShowPreview(true);
            callbacks.onFileOpen?.(file);
        }
    }, [navigateToFolder, callbacks]);
    return (React.createElement("div", { className: `caddy-file-manager ${className}` },
        React.createElement("div", { className: "file-manager-toolbar" },
            React.createElement("div", { className: "toolbar-left" },
                React.createElement("button", { onClick: () => setShowUploadPanel(true), className: "btn btn-primary" }, "Upload Files"),
                React.createElement("button", { onClick: () => createFolder(prompt('Folder name:') || 'New Folder'), className: "btn" }, "New Folder"),
                state.selectedFiles.length > 0 && (React.createElement(React.Fragment, null,
                    React.createElement("button", { onClick: () => downloadFiles(state.selectedFiles), className: "btn" },
                        "Download (",
                        state.selectedFiles.length,
                        ")"),
                    React.createElement("button", { onClick: () => deleteFiles(state.selectedFiles), className: "btn btn-danger" },
                        "Delete (",
                        state.selectedFiles.length,
                        ")")))),
            React.createElement("div", { className: "toolbar-right" },
                React.createElement("button", { onClick: () => setViewMode('grid'), className: `btn ${state.viewMode === 'grid' ? 'active' : ''}` }, "Grid"),
                React.createElement("button", { onClick: () => setViewMode('list'), className: `btn ${state.viewMode === 'list' ? 'active' : ''}` }, "List"))),
        React.createElement("div", { className: "file-manager-breadcrumbs" }, state.breadcrumbs.map((crumb, index) => (React.createElement(React.Fragment, { key: crumb.id },
            React.createElement("button", { onClick: () => navigateToFolder(crumb.id === 'root' ? null : crumb.id), className: "breadcrumb-item" }, crumb.name),
            index < state.breadcrumbs.length - 1 && React.createElement("span", { className: "breadcrumb-separator" }, "/"))))),
        React.createElement("div", { ref: fileListRef, className: "file-manager-content" }, state.loading ? (React.createElement("div", { className: "loading-state" }, "Loading files...")) : state.error ? (React.createElement("div", { className: "error-state" }, state.error)) : state.files.length === 0 ? (React.createElement("div", { className: "empty-state" }, "This folder is empty")) : (React.createElement("div", { className: `file-list file-list-${state.viewMode}` }, state.files.map(file => (React.createElement("div", { key: file.id, className: `file-item ${state.selectedFiles.includes(file.id) ? 'selected' : ''}`, onClick: (e) => toggleFileSelection(file.id, e.ctrlKey || e.metaKey), onDoubleClick: () => handleFileOpen(file), onContextMenu: (e) => handleContextMenu(e, file) },
            React.createElement("div", { className: "file-icon" }, file.type === 'folder' ? '📁' : '📄'),
            React.createElement("div", { className: "file-name" }, file.name),
            state.viewMode === 'list' && (React.createElement(React.Fragment, null,
                React.createElement("div", { className: "file-size" }, formatBytes(file.size)),
                React.createElement("div", { className: "file-modified" }, formatDate(file.modifiedAt)))))))))),
        contextMenu && (React.createElement("div", { className: "context-menu", style: { left: contextMenu.x, top: contextMenu.y } },
            React.createElement("button", { onClick: () => handleFileOpen(contextMenu.file) }, "Open"),
            React.createElement("button", { onClick: () => downloadFiles([contextMenu.file.id]) }, "Download"),
            React.createElement("button", { onClick: () => { setShareFile(contextMenu.file); setShowShareDialog(true); } }, "Share"),
            React.createElement("button", { onClick: () => toggleStar(contextMenu.file.id) }, contextMenu.file.isStarred ? 'Unstar' : 'Star'),
            React.createElement("button", { onClick: () => { setVersionsFile(contextMenu.file); setShowVersions(true); } }, "Version History"),
            React.createElement("hr", null),
            React.createElement("button", { onClick: () => renameFile(contextMenu.file.id, prompt('New name:') || contextMenu.file.name) }, "Rename"),
            React.createElement("button", { onClick: () => deleteFiles([contextMenu.file.id]), className: "danger" }, "Delete"))),
        operations.filter(op => op.status === 'processing').length > 0 && (React.createElement("div", { className: "operations-panel" }, operations.filter(op => op.status === 'processing').map(op => (React.createElement("div", { key: op.id, className: "operation-item" },
            React.createElement("div", { className: "operation-title" },
                op.type,
                " (",
                op.completed,
                "/",
                op.total,
                ")"),
            React.createElement("div", { className: "operation-progress" },
                React.createElement("div", { className: "progress-bar", style: { width: `${op.progress}%` } })))))))));
};
function formatBytes(bytes) {
    if (bytes === 0)
        return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}
function formatDate(date) {
    const d = new Date(date);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    if (diffDays === 0)
        return 'Today';
    if (diffDays === 1)
        return 'Yesterday';
    if (diffDays < 7)
        return `${diffDays} days ago`;
    return d.toLocaleDateString();
}
export default FileManager;
//# sourceMappingURL=FileManager.js.map