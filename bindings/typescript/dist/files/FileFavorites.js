import React, { useState, useEffect, useCallback } from 'react';
export const FileFavorites = ({ tenantId, onFileSelect, onFileOpen, className = '', }) => {
    const [files, setFiles] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [viewMode, setViewMode] = useState('list');
    const [sortField, setSortField] = useState('name');
    const [sortDirection, setSortDirection] = useState('asc');
    const [selectedFiles, setSelectedFiles] = useState([]);
    useEffect(() => {
        loadFavorites();
    }, [tenantId, sortField, sortDirection]);
    const loadFavorites = async () => {
        setLoading(true);
        setError(null);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/favorites?sort=${sortField}&direction=${sortDirection}`, {
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to load favorites');
            }
            const data = await response.json();
            setFiles(data.files);
        }
        catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load favorites');
        }
        finally {
            setLoading(false);
        }
    };
    const toggleStar = async (fileId) => {
        try {
            const file = files.find(f => f.id === fileId);
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
            await loadFavorites();
        }
        catch (err) {
            console.error('Failed to toggle star:', err);
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
        onFileSelect?.(file);
    }, [onFileSelect]);
    const handleFileDoubleClick = useCallback((file) => {
        onFileOpen?.(file);
    }, [onFileOpen]);
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
    return (React.createElement("div", { className: `file-favorites ${className}` },
        React.createElement("div", { className: "favorites-header" },
            React.createElement("h2", null, "Starred Files"),
            React.createElement("div", { className: "header-actions" },
                React.createElement("button", { onClick: () => setViewMode('grid'), className: `btn btn-sm ${viewMode === 'grid' ? 'active' : ''}` }, "Grid"),
                React.createElement("button", { onClick: () => setViewMode('list'), className: `btn btn-sm ${viewMode === 'list' ? 'active' : ''}` }, "List"))),
        React.createElement("div", { className: "favorites-content" }, loading ? (React.createElement("div", { className: "loading-state" }, "Loading favorites...")) : error ? (React.createElement("div", { className: "error-state" }, error)) : files.length === 0 ? (React.createElement("div", { className: "empty-state" },
            React.createElement("div", { className: "empty-icon" }, "\u2B50"),
            React.createElement("h3", null, "No starred files yet"),
            React.createElement("p", null, "Star files to quickly access them here"))) : viewMode === 'grid' ? (React.createElement("div", { className: "favorites-grid" }, files.map(file => (React.createElement("div", { key: file.id, className: `file-grid-item ${selectedFiles.includes(file.id) ? 'selected' : ''}`, onClick: (e) => handleFileClick(file, e.ctrlKey || e.metaKey), onDoubleClick: () => handleFileDoubleClick(file) },
            React.createElement("div", { className: "file-grid-thumbnail" },
                file.thumbnail ? (React.createElement("img", { src: file.thumbnail, alt: file.name })) : (React.createElement("span", { className: "file-grid-icon" }, getFileIcon(file))),
                React.createElement("button", { className: "star-button active", onClick: (e) => {
                        e.stopPropagation();
                        toggleStar(file.id);
                    } }, "\u2B50")),
            React.createElement("div", { className: "file-grid-name", title: file.name }, file.name),
            React.createElement("div", { className: "file-grid-meta" }, file.type === 'file' && formatBytes(file.size))))))) : (React.createElement("div", { className: "favorites-list" },
            React.createElement("div", { className: "list-header" },
                React.createElement("div", { className: "list-column column-name", onClick: () => handleSort('name') },
                    "Name ",
                    sortField === 'name' && (sortDirection === 'asc' ? '▲' : '▼')),
                React.createElement("div", { className: "list-column column-size", onClick: () => handleSort('size') },
                    "Size ",
                    sortField === 'size' && (sortDirection === 'asc' ? '▲' : '▼')),
                React.createElement("div", { className: "list-column column-modified", onClick: () => handleSort('modified') },
                    "Modified ",
                    sortField === 'modified' && (sortDirection === 'asc' ? '▲' : '▼')),
                React.createElement("div", { className: "list-column column-path" }, "Path"),
                React.createElement("div", { className: "list-column column-actions" }, "Actions")),
            React.createElement("div", { className: "list-body" }, files.map(file => (React.createElement("div", { key: file.id, className: `list-row ${selectedFiles.includes(file.id) ? 'selected' : ''}`, onClick: (e) => handleFileClick(file, e.ctrlKey || e.metaKey), onDoubleClick: () => handleFileDoubleClick(file) },
                React.createElement("div", { className: "list-column column-name" },
                    React.createElement("span", { className: "file-icon" }, getFileIcon(file)),
                    React.createElement("span", { className: "file-name" }, file.name)),
                React.createElement("div", { className: "list-column column-size" }, file.type === 'file' ? formatBytes(file.size) : '—'),
                React.createElement("div", { className: "list-column column-modified" }, formatDate(file.modifiedAt)),
                React.createElement("div", { className: "list-column column-path", title: file.path }, file.path),
                React.createElement("div", { className: "list-column column-actions" },
                    React.createElement("button", { onClick: (e) => {
                            e.stopPropagation();
                            toggleStar(file.id);
                        }, className: "btn btn-sm", title: "Remove from favorites" }, "Unstar")))))))))));
};
export default FileFavorites;
//# sourceMappingURL=FileFavorites.js.map