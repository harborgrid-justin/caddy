import React, { useState, useEffect } from 'react';
export const FileStorage = ({ tenantId, onUpgrade, className = '', }) => {
    const [quota, setQuota] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [viewMode, setViewMode] = useState('overview');
    useEffect(() => {
        loadQuota();
    }, [tenantId]);
    const loadQuota = async () => {
        setLoading(true);
        setError(null);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/storage/quota`, {
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to load storage quota');
            }
            const data = await response.json();
            setQuota(data);
        }
        catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load storage quota');
        }
        finally {
            setLoading(false);
        }
    };
    const formatBytes = (bytes) => {
        if (bytes === 0)
            return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
    };
    const getStorageColor = (percentage) => {
        if (percentage >= 90)
            return '#ef4444';
        if (percentage >= 75)
            return '#f59e0b';
        if (percentage >= 50)
            return '#3b82f6';
        return '#10b981';
    };
    const calculatePercentage = (value, total) => {
        return total > 0 ? (value / total) * 100 : 0;
    };
    if (loading) {
        return (React.createElement("div", { className: `file-storage ${className}` },
            React.createElement("div", { className: "loading-state" }, "Loading storage information...")));
    }
    if (error) {
        return (React.createElement("div", { className: `file-storage ${className}` },
            React.createElement("div", { className: "error-state" }, error)));
    }
    if (!quota) {
        return null;
    }
    return (React.createElement("div", { className: `file-storage ${className}` },
        React.createElement("div", { className: "storage-header" },
            React.createElement("h2", null, "Storage"),
            React.createElement("div", { className: "storage-tabs" },
                React.createElement("button", { className: `tab ${viewMode === 'overview' ? 'active' : ''}`, onClick: () => setViewMode('overview') }, "Overview"),
                React.createElement("button", { className: `tab ${viewMode === 'breakdown' ? 'active' : ''}`, onClick: () => setViewMode('breakdown') }, "Breakdown"),
                React.createElement("button", { className: `tab ${viewMode === 'limits' ? 'active' : ''}`, onClick: () => setViewMode('limits') }, "Limits"))),
        viewMode === 'overview' && (React.createElement("div", { className: "storage-overview" },
            React.createElement("div", { className: "usage-summary" },
                React.createElement("div", { className: "usage-circle" },
                    React.createElement("svg", { viewBox: "0 0 200 200", className: "circle-chart" },
                        React.createElement("circle", { cx: "100", cy: "100", r: "80", fill: "none", stroke: "#e5e7eb", strokeWidth: "20" }),
                        React.createElement("circle", { cx: "100", cy: "100", r: "80", fill: "none", stroke: getStorageColor(quota.percentage), strokeWidth: "20", strokeDasharray: `${quota.percentage * 5.03} 503`, strokeLinecap: "round", transform: "rotate(-90 100 100)" }),
                        React.createElement("text", { x: "100", y: "100", textAnchor: "middle", dy: "0.3em", className: "percentage-text" },
                            Math.round(quota.percentage),
                            "%"))),
                React.createElement("div", { className: "usage-details" },
                    React.createElement("div", { className: "usage-label" }, "Storage Used"),
                    React.createElement("div", { className: "usage-value" },
                        formatBytes(quota.used),
                        " of ",
                        formatBytes(quota.total)),
                    React.createElement("div", { className: "usage-available" },
                        formatBytes(quota.available),
                        " available"))),
            React.createElement("div", { className: "quick-stats" },
                React.createElement("div", { className: "stat-card" },
                    React.createElement("div", { className: "stat-icon" }, "\uD83D\uDCC4"),
                    React.createElement("div", { className: "stat-label" }, "Documents"),
                    React.createElement("div", { className: "stat-value" }, formatBytes(quota.breakdown.documents))),
                React.createElement("div", { className: "stat-card" },
                    React.createElement("div", { className: "stat-icon" }, "\uD83D\uDDBC\uFE0F"),
                    React.createElement("div", { className: "stat-label" }, "Images"),
                    React.createElement("div", { className: "stat-value" }, formatBytes(quota.breakdown.images))),
                React.createElement("div", { className: "stat-card" },
                    React.createElement("div", { className: "stat-icon" }, "\uD83C\uDFAC"),
                    React.createElement("div", { className: "stat-label" }, "Videos"),
                    React.createElement("div", { className: "stat-value" }, formatBytes(quota.breakdown.videos))),
                React.createElement("div", { className: "stat-card" },
                    React.createElement("div", { className: "stat-icon" }, "\uD83D\uDDD1\uFE0F"),
                    React.createElement("div", { className: "stat-label" }, "Trash"),
                    React.createElement("div", { className: "stat-value" }, formatBytes(quota.breakdown.trash)))),
            quota.percentage >= 90 && (React.createElement("div", { className: "storage-warning storage-critical" },
                React.createElement("strong", null, "Storage Almost Full!"),
                React.createElement("p", null,
                    "You're using ",
                    quota.percentage.toFixed(1),
                    "% of your storage.",
                    onUpgrade && ' Consider upgrading your plan for more storage.'),
                onUpgrade && (React.createElement("button", { onClick: onUpgrade, className: "btn btn-primary" }, "Upgrade Plan")))),
            quota.percentage >= 75 && quota.percentage < 90 && (React.createElement("div", { className: "storage-warning storage-warning-level" },
                React.createElement("strong", null, "Storage Running Low"),
                React.createElement("p", null,
                    "You're using ",
                    quota.percentage.toFixed(1),
                    "% of your storage. Consider cleaning up old files or upgrading your plan."))),
            React.createElement("div", { className: "plan-info" },
                React.createElement("div", { className: "plan-badge" }, quota.plan),
                React.createElement("div", { className: "plan-details" },
                    "You're on the ",
                    React.createElement("strong", null, quota.plan),
                    " plan"),
                onUpgrade && (React.createElement("button", { onClick: onUpgrade, className: "btn btn-sm" }, "Change Plan"))))),
        viewMode === 'breakdown' && (React.createElement("div", { className: "storage-breakdown" },
            React.createElement("div", { className: "breakdown-chart" },
                React.createElement("div", { className: "breakdown-bar" }, Object.entries(quota.breakdown).map(([type, size]) => {
                    const percentage = calculatePercentage(size, quota.used);
                    if (percentage < 0.1)
                        return null;
                    const colors = {
                        documents: '#3b82f6',
                        images: '#8b5cf6',
                        videos: '#ec4899',
                        audio: '#10b981',
                        archives: '#f59e0b',
                        other: '#6b7280',
                        trash: '#ef4444',
                    };
                    return (React.createElement("div", { key: type, className: "breakdown-segment", style: {
                            width: `${percentage}%`,
                            backgroundColor: colors[type] || '#6b7280',
                        }, title: `${type}: ${formatBytes(size)} (${percentage.toFixed(1)}%)` }));
                }))),
            React.createElement("div", { className: "breakdown-list" }, Object.entries(quota.breakdown)
                .sort(([, a], [, b]) => b - a)
                .map(([type, size]) => {
                const percentage = calculatePercentage(size, quota.used);
                const icons = {
                    documents: '📄',
                    images: '🖼️',
                    videos: '🎬',
                    audio: '🎵',
                    archives: '📦',
                    other: '📁',
                    trash: '🗑️',
                };
                return (React.createElement("div", { key: type, className: "breakdown-item" },
                    React.createElement("div", { className: "breakdown-type" },
                        React.createElement("span", { className: "type-icon" }, icons[type] || '📁'),
                        React.createElement("span", { className: "type-name" }, type.charAt(0).toUpperCase() + type.slice(1))),
                    React.createElement("div", { className: "breakdown-size" }, formatBytes(size)),
                    React.createElement("div", { className: "breakdown-percentage" },
                        percentage.toFixed(1),
                        "%")));
            })))),
        viewMode === 'limits' && (React.createElement("div", { className: "storage-limits" },
            React.createElement("div", { className: "limits-list" },
                React.createElement("div", { className: "limit-item" },
                    React.createElement("div", { className: "limit-label" },
                        React.createElement("strong", null, "Maximum File Size"),
                        React.createElement("p", null, "The largest file you can upload")),
                    React.createElement("div", { className: "limit-value" }, formatBytes(quota.limits.maxFileSize))),
                React.createElement("div", { className: "limit-item" },
                    React.createElement("div", { className: "limit-label" },
                        React.createElement("strong", null, "Total Storage"),
                        React.createElement("p", null, "Total storage space available")),
                    React.createElement("div", { className: "limit-value" }, formatBytes(quota.limits.maxTotalStorage))),
                React.createElement("div", { className: "limit-item" },
                    React.createElement("div", { className: "limit-label" },
                        React.createElement("strong", null, "Files Per Folder"),
                        React.createElement("p", null, "Maximum number of items in a single folder")),
                    React.createElement("div", { className: "limit-value" }, quota.limits.maxFilesPerFolder.toLocaleString())),
                React.createElement("div", { className: "limit-item" },
                    React.createElement("div", { className: "limit-label" },
                        React.createElement("strong", null, "Version History"),
                        React.createElement("p", null, "Maximum versions kept per file")),
                    React.createElement("div", { className: "limit-value" }, quota.limits.maxVersionsPerFile)),
                React.createElement("div", { className: "limit-item" },
                    React.createElement("div", { className: "limit-label" },
                        React.createElement("strong", null, "Trash Retention"),
                        React.createElement("p", null, "Files in trash are kept for")),
                    React.createElement("div", { className: "limit-value" },
                        quota.limits.retentionDays,
                        " days")),
                quota.limits.allowedFileTypes.length > 0 && (React.createElement("div", { className: "limit-item" },
                    React.createElement("div", { className: "limit-label" },
                        React.createElement("strong", null, "Allowed File Types"),
                        React.createElement("p", null, "File types you can upload")),
                    React.createElement("div", { className: "limit-value" },
                        React.createElement("div", { className: "file-types" },
                            quota.limits.allowedFileTypes.slice(0, 10).map(type => (React.createElement("span", { key: type, className: "file-type-badge" }, type))),
                            quota.limits.allowedFileTypes.length > 10 && (React.createElement("span", { className: "file-type-badge" },
                                "+",
                                quota.limits.allowedFileTypes.length - 10,
                                " more"))))))),
            onUpgrade && (React.createElement("div", { className: "limits-upgrade" },
                React.createElement("h3", null, "Need Higher Limits?"),
                React.createElement("p", null, "Upgrade your plan to get higher storage limits, larger file sizes, and more features."),
                React.createElement("button", { onClick: onUpgrade, className: "btn btn-primary" }, "View Plans")))))));
};
export default FileStorage;
//# sourceMappingURL=FileStorage.js.map