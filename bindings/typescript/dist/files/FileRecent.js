import React, { useState, useEffect, useCallback } from 'react';
export const FileRecent = ({ tenantId, onFileSelect, onFileOpen, limit = 50, className = '', }) => {
    const [recentFiles, setRecentFiles] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [viewMode, setViewMode] = useState('list');
    const [filterType, setFilterType] = useState('all');
    const [selectedFiles, setSelectedFiles] = useState([]);
    useEffect(() => {
        loadRecentFiles();
    }, [tenantId, limit, filterType]);
    const loadRecentFiles = async () => {
        setLoading(true);
        setError(null);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/recent?limit=${limit}${filterType !== 'all' ? `&type=${filterType}` : ''}`, {
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to load recent files');
            }
            const data = await response.json();
            setRecentFiles(data.recent);
        }
        catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load recent files');
        }
        finally {
            setLoading(false);
        }
    };
    const clearHistory = async () => {
        if (!confirm('Clear all recent files history?')) {
            return;
        }
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/recent`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to clear history');
            }
            await loadRecentFiles();
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to clear history');
        }
    };
    const removeFromRecent = async (fileId) => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${fileId}/recent`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to remove from recent');
            }
            await loadRecentFiles();
        }
        catch (err) {
            console.error('Failed to remove from recent:', err);
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
    const getAccessTypeIcon = (type) => {
        const icons = {
            view: '👁️',
            edit: '✏️',
            download: '⬇️',
            share: '🔗',
        };
        return icons[type];
    };
    const getAccessTypeLabel = (type) => {
        const labels = {
            view: 'Viewed',
            edit: 'Edited',
            download: 'Downloaded',
            share: 'Shared',
        };
        return labels[type];
    };
    const groupedFiles = recentFiles.reduce((groups, recent) => {
        const date = new Date(recent.accessedAt);
        const today = new Date();
        const yesterday = new Date(today);
        yesterday.setDate(yesterday.getDate() - 1);
        let groupKey;
        if (date.toDateString() === today.toDateString()) {
            groupKey = 'Today';
        }
        else if (date.toDateString() === yesterday.toDateString()) {
            groupKey = 'Yesterday';
        }
        else if (date > new Date(today.getTime() - 7 * 86400000)) {
            groupKey = 'This Week';
        }
        else if (date > new Date(today.getTime() - 30 * 86400000)) {
            groupKey = 'This Month';
        }
        else {
            groupKey = 'Older';
        }
        if (!groups[groupKey]) {
            groups[groupKey] = [];
        }
        groups[groupKey].push(recent);
        return groups;
    }, {});
    const groupOrder = ['Today', 'Yesterday', 'This Week', 'This Month', 'Older'];
    return (React.createElement("div", { className: `file-recent ${className}` },
        React.createElement("div", { className: "recent-header" },
            React.createElement("h2", null, "Recent Files"),
            React.createElement("div", { className: "header-actions" },
                React.createElement("select", { value: filterType, onChange: (e) => setFilterType(e.target.value), className: "form-select form-select-sm" },
                    React.createElement("option", { value: "all" }, "All Activity"),
                    React.createElement("option", { value: "view" }, "Viewed"),
                    React.createElement("option", { value: "edit" }, "Edited"),
                    React.createElement("option", { value: "download" }, "Downloaded")),
                React.createElement("button", { onClick: () => setViewMode('grid'), className: `btn btn-sm ${viewMode === 'grid' ? 'active' : ''}` }, "Grid"),
                React.createElement("button", { onClick: () => setViewMode('list'), className: `btn btn-sm ${viewMode === 'list' ? 'active' : ''}` }, "List"),
                recentFiles.length > 0 && (React.createElement("button", { onClick: clearHistory, className: "btn btn-sm btn-danger" }, "Clear History")))),
        React.createElement("div", { className: "recent-content" }, loading ? (React.createElement("div", { className: "loading-state" }, "Loading recent files...")) : error ? (React.createElement("div", { className: "error-state" }, error)) : recentFiles.length === 0 ? (React.createElement("div", { className: "empty-state" },
            React.createElement("div", { className: "empty-icon" }, "\uD83D\uDD52"),
            React.createElement("h3", null, "No recent files"),
            React.createElement("p", null, "Files you access will appear here"))) : viewMode === 'grid' ? (React.createElement("div", { className: "recent-groups" }, groupOrder.map(groupKey => {
            const group = groupedFiles[groupKey];
            if (!group || group.length === 0)
                return null;
            return (React.createElement("div", { key: groupKey, className: "recent-group" },
                React.createElement("h3", { className: "group-title" }, groupKey),
                React.createElement("div", { className: "recent-grid" }, group.map(recent => (React.createElement("div", { key: recent.file.id, className: `file-grid-item ${selectedFiles.includes(recent.file.id) ? 'selected' : ''}`, onClick: (e) => handleFileClick(recent.file, e.ctrlKey || e.metaKey), onDoubleClick: () => handleFileDoubleClick(recent.file) },
                    React.createElement("div", { className: "file-grid-thumbnail" },
                        recent.file.thumbnail ? (React.createElement("img", { src: recent.file.thumbnail, alt: recent.file.name })) : (React.createElement("span", { className: "file-grid-icon" }, getFileIcon(recent.file))),
                        React.createElement("div", { className: "access-badge", title: getAccessTypeLabel(recent.accessType) }, getAccessTypeIcon(recent.accessType))),
                    React.createElement("div", { className: "file-grid-name", title: recent.file.name }, recent.file.name),
                    React.createElement("div", { className: "file-grid-meta" }, formatDate(recent.accessedAt)),
                    React.createElement("button", { className: "remove-button", onClick: (e) => {
                            e.stopPropagation();
                            removeFromRecent(recent.file.id);
                        }, title: "Remove from recent" }, "\u2715")))))));
        }))) : (React.createElement("div", { className: "recent-groups" }, groupOrder.map(groupKey => {
            const group = groupedFiles[groupKey];
            if (!group || group.length === 0)
                return null;
            return (React.createElement("div", { key: groupKey, className: "recent-group" },
                React.createElement("h3", { className: "group-title" }, groupKey),
                React.createElement("div", { className: "recent-list" },
                    React.createElement("div", { className: "list-header" },
                        React.createElement("div", { className: "list-column column-name" }, "Name"),
                        React.createElement("div", { className: "list-column column-activity" }, "Activity"),
                        React.createElement("div", { className: "list-column column-size" }, "Size"),
                        React.createElement("div", { className: "list-column column-accessed" }, "Accessed"),
                        React.createElement("div", { className: "list-column column-count" }, "Times"),
                        React.createElement("div", { className: "list-column column-actions" }, "Actions")),
                    React.createElement("div", { className: "list-body" }, group.map(recent => (React.createElement("div", { key: recent.file.id, className: `list-row ${selectedFiles.includes(recent.file.id) ? 'selected' : ''}`, onClick: (e) => handleFileClick(recent.file, e.ctrlKey || e.metaKey), onDoubleClick: () => handleFileDoubleClick(recent.file) },
                        React.createElement("div", { className: "list-column column-name" },
                            React.createElement("span", { className: "file-icon" }, getFileIcon(recent.file)),
                            React.createElement("span", { className: "file-name" }, recent.file.name),
                            recent.file.isStarred && React.createElement("span", { className: "star-icon" }, "\u2B50")),
                        React.createElement("div", { className: "list-column column-activity" },
                            React.createElement("span", { className: "access-badge", title: getAccessTypeLabel(recent.accessType) },
                                getAccessTypeIcon(recent.accessType),
                                ' ',
                                getAccessTypeLabel(recent.accessType))),
                        React.createElement("div", { className: "list-column column-size" }, recent.file.type === 'file'
                            ? formatBytes(recent.file.size)
                            : '—'),
                        React.createElement("div", { className: "list-column column-accessed" }, formatDate(recent.accessedAt)),
                        React.createElement("div", { className: "list-column column-count" },
                            recent.accessCount,
                            "x"),
                        React.createElement("div", { className: "list-column column-actions" },
                            React.createElement("button", { onClick: (e) => {
                                    e.stopPropagation();
                                    removeFromRecent(recent.file.id);
                                }, className: "btn btn-sm" }, "Remove")))))))));
        }))))));
};
export default FileRecent;
//# sourceMappingURL=FileRecent.js.map