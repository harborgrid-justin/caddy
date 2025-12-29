import React, { useState, useEffect, useCallback } from 'react';
export const FileTrash = ({ tenantId, onFileRestore, className = '', }) => {
    const [files, setFiles] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [viewMode, setViewMode] = useState('list');
    const [sortField, setSortField] = useState('modified');
    const [sortDirection, setSortDirection] = useState('desc');
    const [selectedFiles, setSelectedFiles] = useState([]);
    const [storageUsed, setStorageUsed] = useState(0);
    const [retentionDays, setRetentionDays] = useState(30);
    useEffect(() => {
        loadTrash();
    }, [tenantId, sortField, sortDirection]);
    const loadTrash = async () => {
        setLoading(true);
        setError(null);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/trash?sort=${sortField}&direction=${sortDirection}`, {
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to load trash');
            }
            const data = await response.json();
            setFiles(data.files);
            setStorageUsed(data.storageUsed || 0);
            setRetentionDays(data.retentionDays || 30);
        }
        catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load trash');
        }
        finally {
            setLoading(false);
        }
    };
    const restoreFiles = async (fileIds) => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/trash/restore`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ fileIds }),
            });
            if (!response.ok) {
                throw new Error('Failed to restore files');
            }
            const result = await response.json();
            await loadTrash();
            setSelectedFiles([]);
            onFileRestore?.(result.restored);
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to restore files');
        }
    };
    const permanentlyDelete = async (fileIds) => {
        if (!confirm(`Permanently delete ${fileIds.length} item${fileIds.length !== 1 ? 's' : ''}? This action cannot be undone.`)) {
            return;
        }
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/trash/permanent`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ fileIds }),
            });
            if (!response.ok) {
                throw new Error('Failed to delete files');
            }
            await loadTrash();
            setSelectedFiles([]);
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to delete files');
        }
    };
    const emptyTrash = async () => {
        if (!confirm('Empty trash? This will permanently delete all items in trash. This action cannot be undone.')) {
            return;
        }
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/trash/empty`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to empty trash');
            }
            await loadTrash();
            setSelectedFiles([]);
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to empty trash');
        }
    };
    const handleFileClick = useCallback((file, multi = false) => {
        if (multi) {
            setSelectedFiles(prev => prev.includes(file.id)
                ? prev.filter(id => id !== file.id)
                : [...prev, file.id]);
        }
        else {
            setSelectedFiles([file.id]);
        }
    }, []);
    const selectAll = useCallback(() => {
        setSelectedFiles(files.map(f => f.id));
    }, [files]);
    const clearSelection = useCallback(() => {
        setSelectedFiles([]);
    }, []);
    const handleSort = useCallback((field) => {
        setSortDirection(prev => sortField === field && prev === 'asc' ? 'desc' : 'asc');
        setSortField(field);
    }, [sortField]);
    const formatBytes = (bytes) => {
        if (bytes === 0)
            return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
    };
    const formatDate = (date) => {
        const d = new Date(date);
        return d.toLocaleString();
    };
    const getDaysUntilDeletion = (trashedAt) => {
        const trashed = new Date(trashedAt);
        const deleteDate = new Date(trashed.getTime() + retentionDays * 86400000);
        const now = new Date();
        const daysRemaining = Math.ceil((deleteDate.getTime() - now.getTime()) / 86400000);
        return Math.max(0, daysRemaining);
    };
    const getFileIcon = (file) => {
        if (file.type === 'folder')
            return '📁';
        const ext = file.name.split('.').pop()?.toLowerCase() || '';
        const iconMap = {
            jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️',
            pdf: '📕', doc: '📘', docx: '📘', xls: '📗', xlsx: '📗',
            txt: '📄', md: '📝',
            zip: '📦', rar: '📦',
            mp4: '🎬', mov: '🎬',
            mp3: '🎵', wav: '🎵',
        };
        return iconMap[ext] || '📄';
    };
    return (React.createElement("div", { className: `file-trash ${className}` },
        React.createElement("div", { className: "trash-header" },
            React.createElement("div", { className: "header-left" },
                React.createElement("h2", null, "Trash"),
                React.createElement("div", { className: "trash-info" },
                    files.length,
                    " item",
                    files.length !== 1 ? 's' : '',
                    " \u2022 ",
                    formatBytes(storageUsed),
                    " used")),
            React.createElement("div", { className: "header-right" }, files.length > 0 && (React.createElement("button", { onClick: emptyTrash, className: "btn btn-danger" }, "Empty Trash")))),
        files.length > 0 && (React.createElement("div", { className: "trash-toolbar" },
            React.createElement("div", { className: "toolbar-left" }, selectedFiles.length > 0 ? (React.createElement(React.Fragment, null,
                React.createElement("span", { className: "selection-count" },
                    selectedFiles.length,
                    " selected"),
                React.createElement("button", { onClick: () => restoreFiles(selectedFiles), className: "btn btn-primary" }, "Restore"),
                React.createElement("button", { onClick: () => permanentlyDelete(selectedFiles), className: "btn btn-danger" }, "Delete Forever"),
                React.createElement("button", { onClick: clearSelection, className: "btn" }, "Clear Selection"))) : (React.createElement("button", { onClick: selectAll, className: "btn" }, "Select All"))),
            React.createElement("div", { className: "toolbar-right" },
                React.createElement("button", { onClick: () => setViewMode('grid'), className: `btn btn-sm ${viewMode === 'grid' ? 'active' : ''}` }, "Grid"),
                React.createElement("button", { onClick: () => setViewMode('list'), className: `btn btn-sm ${viewMode === 'list' ? 'active' : ''}` }, "List")))),
        files.length > 0 && (React.createElement("div", { className: "trash-banner" },
            React.createElement("div", { className: "banner-icon" }, "\u2139\uFE0F"),
            React.createElement("div", { className: "banner-text" },
                "Items in trash will be permanently deleted after ",
                retentionDays,
                " days"))),
        React.createElement("div", { className: "trash-content" }, loading ? (React.createElement("div", { className: "loading-state" }, "Loading trash...")) : error ? (React.createElement("div", { className: "error-state" }, error)) : files.length === 0 ? (React.createElement("div", { className: "empty-state" },
            React.createElement("div", { className: "empty-icon" }, "\uD83D\uDDD1\uFE0F"),
            React.createElement("h3", null, "Trash is empty"),
            React.createElement("p", null, "Deleted files will appear here"))) : viewMode === 'grid' ? (React.createElement("div", { className: "trash-grid" }, files.map(file => {
            const daysLeft = file.trashedAt ? getDaysUntilDeletion(file.trashedAt) : 0;
            return (React.createElement("div", { key: file.id, className: `file-grid-item ${selectedFiles.includes(file.id) ? 'selected' : ''} ${daysLeft <= 7 ? 'expiring-soon' : ''}`, onClick: (e) => handleFileClick(file, e.ctrlKey || e.metaKey) },
                React.createElement("div", { className: "file-grid-thumbnail" },
                    file.thumbnail ? (React.createElement("img", { src: file.thumbnail, alt: file.name })) : (React.createElement("span", { className: "file-grid-icon" }, getFileIcon(file))),
                    daysLeft <= 7 && (React.createElement("div", { className: "expiry-badge", title: `${daysLeft} days remaining` },
                        daysLeft,
                        "d"))),
                React.createElement("div", { className: "file-grid-name", title: file.name }, file.name),
                React.createElement("div", { className: "file-grid-meta" },
                    "Deleted ",
                    file.trashedAt && formatDate(file.trashedAt).split(',')[0]),
                React.createElement("div", { className: "file-grid-actions" },
                    React.createElement("button", { onClick: (e) => {
                            e.stopPropagation();
                            restoreFiles([file.id]);
                        }, className: "btn btn-sm btn-primary" }, "Restore"),
                    React.createElement("button", { onClick: (e) => {
                            e.stopPropagation();
                            permanentlyDelete([file.id]);
                        }, className: "btn btn-sm btn-danger" }, "Delete"))));
        }))) : (React.createElement("div", { className: "trash-list" },
            React.createElement("div", { className: "list-header" },
                React.createElement("div", { className: "list-column column-checkbox" },
                    React.createElement("input", { type: "checkbox", checked: selectedFiles.length === files.length, onChange: (e) => e.target.checked ? selectAll() : clearSelection() })),
                React.createElement("div", { className: "list-column column-name", onClick: () => handleSort('name') },
                    "Name ",
                    sortField === 'name' && (sortDirection === 'asc' ? '▲' : '▼')),
                React.createElement("div", { className: "list-column column-path" }, "Original Location"),
                React.createElement("div", { className: "list-column column-size", onClick: () => handleSort('size') },
                    "Size ",
                    sortField === 'size' && (sortDirection === 'asc' ? '▲' : '▼')),
                React.createElement("div", { className: "list-column column-deleted" }, "Deleted"),
                React.createElement("div", { className: "list-column column-expires" }, "Expires In"),
                React.createElement("div", { className: "list-column column-actions" }, "Actions")),
            React.createElement("div", { className: "list-body" }, files.map(file => {
                const daysLeft = file.trashedAt ? getDaysUntilDeletion(file.trashedAt) : 0;
                return (React.createElement("div", { key: file.id, className: `list-row ${selectedFiles.includes(file.id) ? 'selected' : ''} ${daysLeft <= 7 ? 'expiring-soon' : ''}`, onClick: (e) => handleFileClick(file, e.ctrlKey || e.metaKey) },
                    React.createElement("div", { className: "list-column column-checkbox" },
                        React.createElement("input", { type: "checkbox", checked: selectedFiles.includes(file.id), onChange: (e) => {
                                e.stopPropagation();
                                handleFileClick(file, true);
                            } })),
                    React.createElement("div", { className: "list-column column-name" },
                        React.createElement("span", { className: "file-icon" }, getFileIcon(file)),
                        React.createElement("span", { className: "file-name" }, file.name)),
                    React.createElement("div", { className: "list-column column-path", title: file.path }, file.path),
                    React.createElement("div", { className: "list-column column-size" }, file.type === 'file' ? formatBytes(file.size) : '—'),
                    React.createElement("div", { className: "list-column column-deleted" }, file.trashedAt && formatDate(file.trashedAt)),
                    React.createElement("div", { className: "list-column column-expires" },
                        React.createElement("span", { className: daysLeft <= 7 ? 'text-danger' : '' },
                            daysLeft,
                            " day",
                            daysLeft !== 1 ? 's' : '')),
                    React.createElement("div", { className: "list-column column-actions" },
                        React.createElement("button", { onClick: (e) => {
                                e.stopPropagation();
                                restoreFiles([file.id]);
                            }, className: "btn btn-sm btn-primary" }, "Restore"),
                        React.createElement("button", { onClick: (e) => {
                                e.stopPropagation();
                                permanentlyDelete([file.id]);
                            }, className: "btn btn-sm btn-danger" }, "Delete"))));
            })))))));
};
export default FileTrash;
//# sourceMappingURL=FileTrash.js.map