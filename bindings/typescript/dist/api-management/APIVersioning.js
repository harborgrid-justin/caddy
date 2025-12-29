import React, { useState, useEffect } from 'react';
export const APIVersioning = ({ onVersionCreate, onVersionUpdate, }) => {
    const [versions, setVersions] = useState([]);
    const [selectedVersion, setSelectedVersion] = useState(null);
    const [migrations, setMigrations] = useState([]);
    const [isCreating, setIsCreating] = useState(false);
    useEffect(() => {
        loadVersions();
        loadMigrations();
    }, []);
    const loadVersions = async () => {
        try {
            const mockVersions = [
                {
                    version: 'v2.0.0',
                    status: 'stable',
                    releaseDate: Date.now() - 86400000 * 30,
                    endpoints: 47,
                    breaking: true,
                    changelog: [
                        {
                            type: 'added',
                            description: 'New webhooks API for event notifications',
                            breaking: false,
                            timestamp: Date.now() - 86400000 * 30,
                        },
                        {
                            type: 'changed',
                            description: 'Authentication now requires API v2 keys',
                            breaking: true,
                            endpoint: '/api/v2/auth',
                            timestamp: Date.now() - 86400000 * 30,
                        },
                        {
                            type: 'removed',
                            description: 'Removed deprecated /users endpoint',
                            breaking: true,
                            endpoint: '/api/v2/users',
                            timestamp: Date.now() - 86400000 * 30,
                        },
                    ],
                    metadata: {},
                },
                {
                    version: 'v1.5.2',
                    status: 'stable',
                    releaseDate: Date.now() - 86400000 * 90,
                    deprecationDate: Date.now() + 86400000 * 90,
                    endpoints: 43,
                    breaking: false,
                    changelog: [
                        {
                            type: 'fixed',
                            description: 'Fixed pagination bug in user listing',
                            breaking: false,
                            endpoint: '/api/v1/users',
                            timestamp: Date.now() - 86400000 * 90,
                        },
                        {
                            type: 'security',
                            description: 'Enhanced rate limiting for authentication endpoints',
                            breaking: false,
                            timestamp: Date.now() - 86400000 * 90,
                        },
                    ],
                    metadata: {},
                },
                {
                    version: 'v1.0.0',
                    status: 'deprecated',
                    releaseDate: Date.now() - 86400000 * 365,
                    deprecationDate: Date.now() - 86400000 * 90,
                    sunsetDate: Date.now() + 86400000 * 30,
                    endpoints: 35,
                    breaking: false,
                    changelog: [
                        {
                            type: 'added',
                            description: 'Initial API release',
                            breaking: false,
                            timestamp: Date.now() - 86400000 * 365,
                        },
                    ],
                    metadata: {},
                },
            ];
            setVersions(mockVersions);
            if (mockVersions.length > 0) {
                setSelectedVersion(mockVersions[0]);
            }
        }
        catch (error) {
            console.error('Failed to load versions:', error);
        }
    };
    const loadMigrations = async () => {
        try {
            const mockMigrations = [
                {
                    from: 'v1.5.2',
                    to: 'v2.0.0',
                    guide: `# Migration Guide: v1.5.2 to v2.0.0

## Breaking Changes

1. **Authentication Changes**
   - API v1 keys are no longer supported
   - Generate new API v2 keys from the dashboard
   - Update all requests to use new authentication headers

2. **Endpoint Removals**
   - \`/users\` endpoint removed, use \`/users/list\` instead
   - \`/auth/token\` replaced with \`/auth/v2/token\`

3. **Response Format Changes**
   - All timestamps now in ISO 8601 format
   - Pagination now uses cursor-based approach

## Migration Steps

1. Generate new API v2 keys
2. Update authentication in your application
3. Test endpoints in staging environment
4. Deploy to production`,
                    automated: false,
                    estimatedEffort: 'medium',
                },
            ];
            setMigrations(mockMigrations);
        }
        catch (error) {
            console.error('Failed to load migrations:', error);
        }
    };
    const handleCreateVersion = async (version) => {
        try {
            if (onVersionCreate) {
                await onVersionCreate(version);
            }
            const newVersion = {
                ...version,
                releaseDate: Date.now(),
                endpoints: 0,
                breaking: false,
                changelog: [],
                metadata: {},
            };
            setVersions([newVersion, ...versions]);
            setIsCreating(false);
        }
        catch (error) {
            console.error('Failed to create version:', error);
        }
    };
    const handleUpdateStatus = async (version, status) => {
        try {
            if (onVersionUpdate) {
                await onVersionUpdate(version, { status });
            }
            setVersions(versions.map((v) => (v.version === version ? { ...v, status } : v)));
        }
        catch (error) {
            console.error('Failed to update version status:', error);
        }
    };
    const getStatusColor = (status) => {
        const colors = {
            draft: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200',
            beta: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
            stable: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
            deprecated: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
            retired: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
        };
        return colors[status];
    };
    const getChangeTypeIcon = (type) => {
        const icons = {
            added: '➕',
            changed: '🔄',
            deprecated: '⚠️',
            removed: '❌',
            fixed: '🔧',
            security: '🔒',
        };
        return icons[type];
    };
    return (React.createElement("div", { className: "min-h-screen bg-gray-50 dark:bg-gray-900" },
        React.createElement("div", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" },
            React.createElement("div", { className: "flex items-center justify-between mb-6" },
                React.createElement("h1", { className: "text-2xl font-bold text-gray-900 dark:text-white" }, "API Versioning"),
                React.createElement("button", { onClick: () => setIsCreating(true), className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, "+ Create Version")),
            React.createElement("div", { className: "grid grid-cols-1 lg:grid-cols-3 gap-6" },
                React.createElement("div", { className: "lg:col-span-2 space-y-4" },
                    versions.map((version) => (React.createElement("div", { key: version.version, onClick: () => setSelectedVersion(version), className: `bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 cursor-pointer transition-all ${selectedVersion?.version === version.version
                            ? 'ring-2 ring-blue-500'
                            : 'hover:border-blue-300'}` },
                        React.createElement("div", { className: "flex items-start justify-between mb-4" },
                            React.createElement("div", { className: "flex-1" },
                                React.createElement("div", { className: "flex items-center space-x-3 mb-2" },
                                    React.createElement("h3", { className: "text-xl font-bold text-gray-900 dark:text-white" }, version.version),
                                    React.createElement("span", { className: `px-2 py-1 rounded text-xs font-semibold ${getStatusColor(version.status)}` }, version.status),
                                    version.breaking && (React.createElement("span", { className: "px-2 py-1 rounded text-xs font-semibold bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200" }, "BREAKING"))),
                                React.createElement("div", { className: "grid grid-cols-2 gap-4 text-sm text-gray-600 dark:text-gray-400" },
                                    React.createElement("div", null,
                                        React.createElement("span", { className: "font-medium" }, "Released:"),
                                        ' ',
                                        new Date(version.releaseDate).toLocaleDateString()),
                                    React.createElement("div", null,
                                        React.createElement("span", { className: "font-medium" }, "Endpoints:"),
                                        " ",
                                        version.endpoints),
                                    version.deprecationDate && (React.createElement("div", null,
                                        React.createElement("span", { className: "font-medium" }, "Deprecated:"),
                                        ' ',
                                        new Date(version.deprecationDate).toLocaleDateString())),
                                    version.sunsetDate && (React.createElement("div", { className: "text-red-600 dark:text-red-400" },
                                        React.createElement("span", { className: "font-medium" }, "Sunset:"),
                                        ' ',
                                        new Date(version.sunsetDate).toLocaleDateString())))),
                            React.createElement("div", { className: "flex flex-col space-y-2" },
                                version.status !== 'stable' && version.status !== 'retired' && (React.createElement("button", { onClick: (e) => {
                                        e.stopPropagation();
                                        handleUpdateStatus(version.version, 'stable');
                                    }, className: "text-sm text-green-600 hover:text-green-800 dark:text-green-400" }, "Mark Stable")),
                                version.status === 'stable' && (React.createElement("button", { onClick: (e) => {
                                        e.stopPropagation();
                                        handleUpdateStatus(version.version, 'deprecated');
                                    }, className: "text-sm text-orange-600 hover:text-orange-800 dark:text-orange-400" }, "Deprecate")))),
                        React.createElement("div", { className: "mt-4 pt-4 border-t border-gray-200 dark:border-gray-700" },
                            React.createElement("h4", { className: "text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2" }, "Recent Changes"),
                            React.createElement("div", { className: "space-y-2" },
                                version.changelog.slice(0, 3).map((change, index) => (React.createElement("div", { key: index, className: "flex items-start space-x-2 text-sm" },
                                    React.createElement("span", { className: "text-lg" }, getChangeTypeIcon(change.type)),
                                    React.createElement("div", { className: "flex-1" },
                                        React.createElement("span", { className: "text-gray-900 dark:text-white" }, change.description),
                                        change.endpoint && (React.createElement("code", { className: "ml-2 text-xs text-gray-600 dark:text-gray-400" }, change.endpoint)))))),
                                version.changelog.length > 3 && (React.createElement("div", { className: "text-xs text-gray-500 dark:text-gray-400" },
                                    "+",
                                    version.changelog.length - 3,
                                    " more changes"))))))),
                    versions.length === 0 && (React.createElement("div", { className: "text-center py-12 text-gray-500 dark:text-gray-400" }, "No versions found"))),
                React.createElement("div", { className: "lg:col-span-1" },
                    React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 sticky top-4" },
                        React.createElement("div", { className: "px-6 py-4 border-b border-gray-200 dark:border-gray-700" },
                            React.createElement("h2", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, "Migration Guides")),
                        React.createElement("div", { className: "p-6 space-y-4 max-h-[600px] overflow-y-auto" },
                            migrations.map((migration, index) => (React.createElement("div", { key: index, className: "border border-gray-200 dark:border-gray-700 rounded-lg p-4" },
                                React.createElement("div", { className: "flex items-center justify-between mb-3" },
                                    React.createElement("div", { className: "text-sm font-semibold text-gray-900 dark:text-white" },
                                        migration.from,
                                        " \u2192 ",
                                        migration.to),
                                    React.createElement("span", { className: `px-2 py-1 rounded text-xs font-semibold ${migration.estimatedEffort === 'low'
                                            ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                                            : migration.estimatedEffort === 'medium'
                                                ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
                                                : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'}` },
                                        migration.estimatedEffort,
                                        " effort")),
                                React.createElement("div", { className: "bg-gray-50 dark:bg-gray-900 rounded p-3 mb-3" },
                                    React.createElement("pre", { className: "text-xs text-gray-900 dark:text-white whitespace-pre-wrap overflow-x-auto" }, migration.guide)),
                                migration.automated && (React.createElement("div", { className: "flex items-center space-x-2 text-xs text-green-600 dark:text-green-400" },
                                    React.createElement("span", null, "\u2705"),
                                    React.createElement("span", null, "Automated migration available")))))),
                            migrations.length === 0 && (React.createElement("div", { className: "text-center py-8 text-gray-500 dark:text-gray-400" }, "No migration guides available")))))),
            isCreating && (React.createElement(CreateVersionModal, { onClose: () => setIsCreating(false), onCreate: handleCreateVersion })))));
};
const CreateVersionModal = ({ onClose, onCreate }) => {
    const [version, setVersion] = useState('');
    const [status, setStatus] = useState('draft');
    const [breaking, setBreaking] = useState(false);
    const handleSubmit = (e) => {
        e.preventDefault();
        onCreate({ version, status, breaking });
    };
    return (React.createElement("div", { className: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" },
        React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6" },
            React.createElement("h2", { className: "text-xl font-semibold text-gray-900 dark:text-white mb-4" }, "Create New Version"),
            React.createElement("form", { onSubmit: handleSubmit, className: "space-y-4" },
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Version Number"),
                    React.createElement("input", { type: "text", value: version, onChange: (e) => setVersion(e.target.value), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", placeholder: "v2.1.0", required: true })),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Status"),
                    React.createElement("select", { value: status, onChange: (e) => setStatus(e.target.value), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                        React.createElement("option", { value: "draft" }, "Draft"),
                        React.createElement("option", { value: "beta" }, "Beta"),
                        React.createElement("option", { value: "stable" }, "Stable"),
                        React.createElement("option", { value: "deprecated" }, "Deprecated"),
                        React.createElement("option", { value: "retired" }, "Retired"))),
                React.createElement("div", { className: "flex items-center" },
                    React.createElement("input", { type: "checkbox", id: "breaking", checked: breaking, onChange: (e) => setBreaking(e.target.checked), className: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded" }),
                    React.createElement("label", { htmlFor: "breaking", className: "ml-2 block text-sm text-gray-700 dark:text-gray-300" }, "Contains breaking changes")),
                React.createElement("div", { className: "flex justify-end space-x-4 pt-4" },
                    React.createElement("button", { type: "button", onClick: onClose, className: "px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700" }, "Cancel"),
                    React.createElement("button", { type: "submit", className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700" }, "Create"))))));
};
export default APIVersioning;
//# sourceMappingURL=APIVersioning.js.map