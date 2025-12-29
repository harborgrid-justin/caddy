import React, { useState, useEffect } from 'react';
export const FileVersions = ({ file, tenantId, onClose, onVersionRestore, className = '', }) => {
    const [versions, setVersions] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [selectedVersions, setSelectedVersions] = useState(null);
    const [comparing, setComparing] = useState(false);
    const [comparisonData, setComparisonData] = useState(null);
    useEffect(() => {
        loadVersions();
    }, [file.id]);
    const loadVersions = async () => {
        setLoading(true);
        setError(null);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/versions`, {
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to load versions');
            }
            const data = await response.json();
            setVersions(data.versions);
        }
        catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load versions');
        }
        finally {
            setLoading(false);
        }
    };
    const restoreVersion = async (version) => {
        if (!confirm(`Restore version ${version.version}? This will create a new version with the content from version ${version.version}.`)) {
            return;
        }
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/versions/${version.id}/restore`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to restore version');
            }
            await loadVersions();
            onVersionRestore?.(version);
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to restore version');
        }
    };
    const downloadVersion = (version) => {
        const token = localStorage.getItem('token');
        window.open(`${version.url}?token=${token}`, '_blank');
    };
    const deleteVersion = async (version) => {
        if (version.isCurrent) {
            alert('Cannot delete the current version');
            return;
        }
        if (!confirm(`Delete version ${version.version}? This action cannot be undone.`)) {
            return;
        }
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/versions/${version.id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to delete version');
            }
            await loadVersions();
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to delete version');
        }
    };
    const compareVersions = async () => {
        if (!selectedVersions || selectedVersions.length !== 2)
            return;
        setComparing(true);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/versions/compare`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    versionId1: selectedVersions[0],
                    versionId2: selectedVersions[1],
                }),
            });
            if (!response.ok) {
                throw new Error('Failed to compare versions');
            }
            const data = await response.json();
            setComparisonData(data);
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to compare versions');
        }
        finally {
            setComparing(false);
        }
    };
    const toggleVersionSelection = (versionId) => {
        if (!selectedVersions) {
            setSelectedVersions([versionId, '']);
        }
        else if (selectedVersions[0] === versionId) {
            setSelectedVersions(null);
        }
        else if (selectedVersions[1] === versionId) {
            setSelectedVersions([selectedVersions[0], '']);
        }
        else if (!selectedVersions[1]) {
            setSelectedVersions([selectedVersions[0], versionId]);
        }
        else {
            setSelectedVersions([versionId, '']);
        }
    };
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
    const getSizeChange = (current, previous) => {
        const diff = current - previous;
        if (diff === 0)
            return 'No change';
        const sign = diff > 0 ? '+' : '';
        return `${sign}${formatBytes(Math.abs(diff))}`;
    };
    return (React.createElement("div", { className: `file-versions-modal ${className}` },
        React.createElement("div", { className: "versions-overlay", onClick: onClose }),
        React.createElement("div", { className: "versions-container" },
            React.createElement("div", { className: "versions-header" },
                React.createElement("h2", null,
                    "Version History - ",
                    file.name),
                React.createElement("button", { onClick: onClose, className: "btn-close" }, "\u2715")),
            selectedVersions && selectedVersions[0] && selectedVersions[1] && (React.createElement("div", { className: "versions-toolbar" },
                React.createElement("button", { onClick: compareVersions, disabled: comparing, className: "btn btn-primary" }, comparing ? 'Comparing...' : 'Compare Selected Versions'),
                React.createElement("button", { onClick: () => setSelectedVersions(null), className: "btn" }, "Clear Selection"))),
            React.createElement("div", { className: "versions-content" },
                loading ? (React.createElement("div", { className: "loading-state" }, "Loading versions...")) : error ? (React.createElement("div", { className: "error-state" }, error)) : versions.length === 0 ? (React.createElement("div", { className: "empty-state" }, "No version history available")) : (React.createElement("div", { className: "versions-list" }, versions.map((version, index) => {
                    const previousVersion = versions[index + 1];
                    const isSelected = selectedVersions?.includes(version.id) || false;
                    return (React.createElement("div", { key: version.id, className: `version-item ${version.isCurrent ? 'current' : ''} ${isSelected ? 'selected' : ''}` },
                        React.createElement("div", { className: "version-selector" },
                            React.createElement("input", { type: "checkbox", checked: isSelected, onChange: () => toggleVersionSelection(version.id) })),
                        React.createElement("div", { className: "version-info" },
                            React.createElement("div", { className: "version-header" },
                                React.createElement("div", { className: "version-number" },
                                    "Version ",
                                    version.version,
                                    version.isCurrent && (React.createElement("span", { className: "current-badge" }, "Current"))),
                                React.createElement("div", { className: "version-date" }, formatDate(version.createdAt))),
                            React.createElement("div", { className: "version-meta" },
                                React.createElement("div", { className: "meta-item" },
                                    React.createElement("strong", null, "Size:"),
                                    " ",
                                    formatBytes(version.size),
                                    previousVersion && (React.createElement("span", { className: "size-change" },
                                        ' ',
                                        "(",
                                        getSizeChange(version.size, previousVersion.size),
                                        ")"))),
                                React.createElement("div", { className: "meta-item" },
                                    React.createElement("strong", null, "Created by:"),
                                    " ",
                                    version.createdBy),
                                version.checksum && (React.createElement("div", { className: "meta-item" },
                                    React.createElement("strong", null, "Checksum:"),
                                    ' ',
                                    React.createElement("code", { className: "checksum" },
                                        version.checksum.slice(0, 16),
                                        "...")))),
                            version.comment && (React.createElement("div", { className: "version-comment" },
                                React.createElement("strong", null, "Comment:"),
                                " ",
                                version.comment))),
                        React.createElement("div", { className: "version-actions" },
                            React.createElement("button", { onClick: () => downloadVersion(version), className: "btn btn-sm", title: "Download this version" }, "Download"),
                            !version.isCurrent && (React.createElement(React.Fragment, null,
                                React.createElement("button", { onClick: () => restoreVersion(version), className: "btn btn-sm btn-primary", title: "Restore this version" }, "Restore"),
                                React.createElement("button", { onClick: () => deleteVersion(version), className: "btn btn-sm btn-danger", title: "Delete this version" }, "Delete"))))));
                }))),
                comparisonData && (React.createElement("div", { className: "comparison-panel" },
                    React.createElement("div", { className: "comparison-header" },
                        React.createElement("h3", null, "Version Comparison"),
                        React.createElement("button", { onClick: () => setComparisonData(null), className: "btn-close" }, "\u2715")),
                    React.createElement("div", { className: "comparison-content" },
                        React.createElement("div", { className: "comparison-stats" },
                            React.createElement("div", { className: "stat" },
                                React.createElement("strong", null, "Size Difference:"),
                                ' ',
                                formatBytes(Math.abs(comparisonData.sizeDiff))),
                            React.createElement("div", { className: "stat" },
                                React.createElement("strong", null, "Changes:"),
                                " ",
                                comparisonData.changes || 'N/A')),
                        comparisonData.diff && (React.createElement("div", { className: "comparison-diff" },
                            React.createElement("pre", null, comparisonData.diff))))))),
            React.createElement("div", { className: "versions-footer" },
                React.createElement("div", { className: "versions-summary" },
                    "Total versions: ",
                    versions.length,
                    " \u2022 Current version: ",
                    file.version)))));
};
export default FileVersions;
//# sourceMappingURL=FileVersions.js.map