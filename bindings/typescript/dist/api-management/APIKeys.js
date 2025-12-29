import React, { useState, useEffect } from 'react';
export const APIKeys = ({ userId = 'current', onKeyCreate, onKeyRevoke, onKeyRotate, }) => {
    const [keys, setKeys] = useState([]);
    const [selectedKey, setSelectedKey] = useState(null);
    const [keyUsage, setKeyUsage] = useState({});
    const [isCreating, setIsCreating] = useState(false);
    const [newKeySecret, setNewKeySecret] = useState(null);
    const [searchQuery, setSearchQuery] = useState('');
    const [filterEnvironment, setFilterEnvironment] = useState('all');
    useEffect(() => {
        loadKeys();
    }, [userId]);
    useEffect(() => {
        if (selectedKey) {
            loadKeyUsage(selectedKey.id);
        }
    }, [selectedKey]);
    const loadKeys = async () => {
        try {
            const mockKeys = [
                {
                    id: '1',
                    name: 'Production API Key',
                    key: 'pk_live_**********************',
                    userId: userId,
                    scopes: ['read:users', 'write:users', 'read:products'],
                    rateLimits: [],
                    environment: 'production',
                    createdAt: Date.now() - 86400000 * 30,
                    lastUsedAt: Date.now() - 3600000,
                    active: true,
                    metadata: {},
                },
                {
                    id: '2',
                    name: 'Development API Key',
                    key: 'pk_test_**********************',
                    userId: userId,
                    scopes: ['read:*', 'write:*'],
                    rateLimits: [],
                    environment: 'development',
                    createdAt: Date.now() - 86400000 * 15,
                    lastUsedAt: Date.now() - 7200000,
                    active: true,
                    metadata: {},
                },
            ];
            setKeys(mockKeys);
        }
        catch (error) {
            console.error('Failed to load API keys:', error);
        }
    };
    const loadKeyUsage = async (keyId) => {
        try {
            const mockUsage = {
                apiKeyId: keyId,
                totalRequests: 15847,
                successfulRequests: 15721,
                failedRequests: 126,
                averageResponseTime: 142,
                lastUsed: Date.now() - 3600000,
                topEndpoints: [
                    { endpoint: '/api/v1/users', count: 5234 },
                    { endpoint: '/api/v1/products', count: 3421 },
                    { endpoint: '/api/v1/orders', count: 2987 },
                ],
            };
            setKeyUsage({ ...keyUsage, [keyId]: mockUsage });
        }
        catch (error) {
            console.error('Failed to load key usage:', error);
        }
    };
    const handleCreate = async (name, scopes, environment) => {
        try {
            let secret;
            if (onKeyCreate) {
                const result = await onKeyCreate(name, scopes);
                secret = result.secret;
                setKeys([...keys, result.key]);
            }
            else {
                secret = `pk_${environment === 'production' ? 'live' : 'test'}_${generateRandomString(32)}`;
                const newKey = {
                    id: Date.now().toString(),
                    name,
                    key: secret.substring(0, 8) + '*'.repeat(22),
                    userId,
                    scopes,
                    rateLimits: [],
                    environment,
                    createdAt: Date.now(),
                    active: true,
                    metadata: {},
                };
                setKeys([...keys, newKey]);
            }
            setNewKeySecret(secret);
            setIsCreating(false);
        }
        catch (error) {
            console.error('Failed to create API key:', error);
        }
    };
    const handleRevoke = async (keyId) => {
        if (!confirm('Are you sure you want to revoke this API key? This action cannot be undone.')) {
            return;
        }
        try {
            if (onKeyRevoke) {
                await onKeyRevoke(keyId);
            }
            setKeys(keys.map((k) => (k.id === keyId ? { ...k, active: false } : k)));
            if (selectedKey?.id === keyId) {
                setSelectedKey(null);
            }
        }
        catch (error) {
            console.error('Failed to revoke API key:', error);
        }
    };
    const handleRotate = async (keyId) => {
        if (!confirm('This will generate a new API key. The old key will stop working immediately.')) {
            return;
        }
        try {
            let secret;
            if (onKeyRotate) {
                const result = await onKeyRotate(keyId);
                secret = result.secret;
                setKeys(keys.map((k) => (k.id === keyId ? result.key : k)));
            }
            else {
                const key = keys.find((k) => k.id === keyId);
                if (!key)
                    return;
                secret = `pk_${key.environment === 'production' ? 'live' : 'test'}_${generateRandomString(32)}`;
                setKeys(keys.map((k) => k.id === keyId
                    ? { ...k, key: secret.substring(0, 8) + '*'.repeat(22) }
                    : k));
            }
            setNewKeySecret(secret);
        }
        catch (error) {
            console.error('Failed to rotate API key:', error);
        }
    };
    const filteredKeys = keys.filter((key) => {
        const matchesSearch = searchQuery === '' ||
            key.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            key.key.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesEnvironment = filterEnvironment === 'all' || key.environment === filterEnvironment;
        return matchesSearch && matchesEnvironment;
    });
    const copyToClipboard = (text) => {
        navigator.clipboard.writeText(text);
    };
    return (React.createElement("div", { className: "min-h-screen bg-gray-50 dark:bg-gray-900" },
        React.createElement("div", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" },
            React.createElement("div", { className: "flex items-center justify-between mb-6" },
                React.createElement("h1", { className: "text-2xl font-bold text-gray-900 dark:text-white" }, "API Keys"),
                React.createElement("button", { onClick: () => setIsCreating(true), className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, "+ Generate API Key")),
            React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 mb-6" },
                React.createElement("div", { className: "grid grid-cols-1 md:grid-cols-2 gap-4" },
                    React.createElement("input", { type: "text", placeholder: "Search API keys...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value), className: "px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" }),
                    React.createElement("select", { value: filterEnvironment, onChange: (e) => setFilterEnvironment(e.target.value), className: "px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                        React.createElement("option", { value: "all" }, "All Environments"),
                        React.createElement("option", { value: "development" }, "Development"),
                        React.createElement("option", { value: "staging" }, "Staging"),
                        React.createElement("option", { value: "production" }, "Production")))),
            React.createElement("div", { className: "grid grid-cols-1 lg:grid-cols-3 gap-6" },
                React.createElement("div", { className: "lg:col-span-2 space-y-4" },
                    filteredKeys.map((key) => (React.createElement("div", { key: key.id, onClick: () => setSelectedKey(key), className: `bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 cursor-pointer transition-all ${selectedKey?.id === key.id ? 'ring-2 ring-blue-500' : 'hover:border-blue-300'}` },
                        React.createElement("div", { className: "flex items-start justify-between mb-4" },
                            React.createElement("div", { className: "flex-1" },
                                React.createElement("div", { className: "flex items-center space-x-3 mb-2" },
                                    React.createElement("h3", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, key.name),
                                    React.createElement("span", { className: `px-2 py-1 rounded text-xs font-semibold ${key.environment === 'production'
                                            ? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                                            : key.environment === 'staging'
                                                ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
                                                : 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'}` }, key.environment),
                                    !key.active && (React.createElement("span", { className: "px-2 py-1 rounded text-xs font-semibold bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200" }, "Revoked"))),
                                React.createElement("div", { className: "flex items-center space-x-2 mb-3" },
                                    React.createElement("code", { className: "text-sm font-mono text-gray-600 dark:text-gray-400" }, key.key),
                                    React.createElement("button", { onClick: (e) => {
                                            e.stopPropagation();
                                            copyToClipboard(key.key);
                                        }, className: "text-blue-600 hover:text-blue-800 dark:text-blue-400", title: "Copy to clipboard" }, "\uD83D\uDCCB")),
                                React.createElement("div", { className: "flex flex-wrap gap-2" },
                                    key.scopes.slice(0, 3).map((scope) => (React.createElement("span", { key: scope, className: "px-2 py-1 rounded text-xs bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300" }, scope))),
                                    key.scopes.length > 3 && (React.createElement("span", { className: "px-2 py-1 rounded text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400" },
                                        "+",
                                        key.scopes.length - 3,
                                        " more")))),
                            key.active && (React.createElement("div", { className: "flex flex-col space-y-2" },
                                React.createElement("button", { onClick: (e) => {
                                        e.stopPropagation();
                                        handleRotate(key.id);
                                    }, className: "text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400" }, "Rotate"),
                                React.createElement("button", { onClick: (e) => {
                                        e.stopPropagation();
                                        handleRevoke(key.id);
                                    }, className: "text-sm text-red-600 hover:text-red-800 dark:text-red-400" }, "Revoke")))),
                        React.createElement("div", { className: "flex items-center justify-between text-sm text-gray-500 dark:text-gray-400" },
                            React.createElement("div", null,
                                "Created ",
                                formatDate(key.createdAt)),
                            key.lastUsedAt && React.createElement("div", null,
                                "Last used ",
                                formatTimeAgo(key.lastUsedAt)))))),
                    filteredKeys.length === 0 && (React.createElement("div", { className: "text-center py-12 text-gray-500 dark:text-gray-400" }, "No API keys found"))),
                React.createElement("div", { className: "lg:col-span-1" }, selectedKey ? (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 sticky top-4" },
                    React.createElement("h3", { className: "text-lg font-semibold text-gray-900 dark:text-white mb-4" }, "Usage Statistics"),
                    keyUsage[selectedKey.id] ? (React.createElement("div", { className: "space-y-4" },
                        React.createElement("div", null,
                            React.createElement("div", { className: "text-sm text-gray-500 dark:text-gray-400 mb-1" }, "Total Requests"),
                            React.createElement("div", { className: "text-2xl font-bold text-gray-900 dark:text-white" }, keyUsage[selectedKey.id].totalRequests.toLocaleString())),
                        React.createElement("div", null,
                            React.createElement("div", { className: "text-sm text-gray-500 dark:text-gray-400 mb-1" }, "Success Rate"),
                            React.createElement("div", { className: "text-2xl font-bold text-green-600 dark:text-green-400" },
                                ((keyUsage[selectedKey.id].successfulRequests /
                                    keyUsage[selectedKey.id].totalRequests) *
                                    100).toFixed(1),
                                "%")),
                        React.createElement("div", null,
                            React.createElement("div", { className: "text-sm text-gray-500 dark:text-gray-400 mb-1" }, "Avg Response Time"),
                            React.createElement("div", { className: "text-2xl font-bold text-gray-900 dark:text-white" },
                                keyUsage[selectedKey.id].averageResponseTime,
                                "ms")),
                        React.createElement("div", null,
                            React.createElement("div", { className: "text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2" }, "Top Endpoints"),
                            React.createElement("div", { className: "space-y-2" }, keyUsage[selectedKey.id].topEndpoints.map((endpoint) => (React.createElement("div", { key: endpoint.endpoint, className: "flex items-center justify-between text-sm" },
                                React.createElement("code", { className: "text-xs text-gray-600 dark:text-gray-400" }, endpoint.endpoint),
                                React.createElement("span", { className: "text-gray-900 dark:text-white font-medium" }, endpoint.count.toLocaleString())))))),
                        React.createElement("div", null,
                            React.createElement("div", { className: "text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2" }, "Scopes"),
                            React.createElement("div", { className: "flex flex-wrap gap-2" }, selectedKey.scopes.map((scope) => (React.createElement("span", { key: scope, className: "px-2 py-1 rounded text-xs bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300" }, scope))))))) : (React.createElement("div", { className: "text-center py-8 text-gray-500 dark:text-gray-400" }, "Loading usage data...")))) : (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 text-center text-gray-500 dark:text-gray-400" }, "Select an API key to view details"))))),
        isCreating && (React.createElement(CreateKeyModal, { onClose: () => setIsCreating(false), onCreate: handleCreate })),
        newKeySecret && (React.createElement(KeySecretModal, { secret: newKeySecret, onClose: () => setNewKeySecret(null) }))));
};
const CreateKeyModal = ({ onClose, onCreate }) => {
    const [name, setName] = useState('');
    const [environment, setEnvironment] = useState('development');
    const [scopes, setScopes] = useState([]);
    const [newScope, setNewScope] = useState('');
    const availableScopes = [
        'read:users',
        'write:users',
        'read:products',
        'write:products',
        'read:orders',
        'write:orders',
        'read:analytics',
        'admin:*',
    ];
    const handleSubmit = (e) => {
        e.preventDefault();
        onCreate(name, scopes, environment);
    };
    return (React.createElement("div", { className: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" },
        React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6" },
            React.createElement("h2", { className: "text-xl font-semibold text-gray-900 dark:text-white mb-4" }, "Generate API Key"),
            React.createElement("form", { onSubmit: handleSubmit, className: "space-y-4" },
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Key Name"),
                    React.createElement("input", { type: "text", value: name, onChange: (e) => setName(e.target.value), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", placeholder: "Production API Key", required: true })),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Environment"),
                    React.createElement("select", { value: environment, onChange: (e) => setEnvironment(e.target.value), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                        React.createElement("option", { value: "development" }, "Development"),
                        React.createElement("option", { value: "staging" }, "Staging"),
                        React.createElement("option", { value: "production" }, "Production"))),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Scopes"),
                    React.createElement("div", { className: "space-y-2 mb-2" }, availableScopes.map((scope) => (React.createElement("label", { key: scope, className: "flex items-center" },
                        React.createElement("input", { type: "checkbox", checked: scopes.includes(scope), onChange: (e) => {
                                if (e.target.checked) {
                                    setScopes([...scopes, scope]);
                                }
                                else {
                                    setScopes(scopes.filter((s) => s !== scope));
                                }
                            }, className: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded" }),
                        React.createElement("span", { className: "ml-2 text-sm text-gray-700 dark:text-gray-300" }, scope)))))),
                React.createElement("div", { className: "flex justify-end space-x-4 pt-4" },
                    React.createElement("button", { type: "button", onClick: onClose, className: "px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors" }, "Cancel"),
                    React.createElement("button", { type: "submit", className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors", disabled: scopes.length === 0 }, "Generate Key"))))));
};
const KeySecretModal = ({ secret, onClose }) => {
    const [copied, setCopied] = useState(false);
    const handleCopy = () => {
        navigator.clipboard.writeText(secret);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };
    return (React.createElement("div", { className: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" },
        React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6" },
            React.createElement("h2", { className: "text-xl font-semibold text-gray-900 dark:text-white mb-4" }, "Your API Key"),
            React.createElement("div", { className: "bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 text-yellow-800 dark:text-yellow-200 px-4 py-3 rounded mb-4" },
                React.createElement("p", { className: "text-sm" }, "Make sure to copy your API key now. You won't be able to see it again!")),
            React.createElement("div", { className: "bg-gray-900 text-gray-100 p-4 rounded-lg mb-4 break-all font-mono text-sm" }, secret),
            React.createElement("div", { className: "flex justify-end space-x-4" },
                React.createElement("button", { onClick: handleCopy, className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, copied ? 'Copied!' : 'Copy to Clipboard'),
                React.createElement("button", { onClick: onClose, className: "px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors" }, "Done")))));
};
function formatDate(timestamp) {
    return new Date(timestamp).toLocaleDateString();
}
function formatTimeAgo(timestamp) {
    const seconds = Math.floor((Date.now() - timestamp) / 1000);
    if (seconds < 60)
        return 'just now';
    if (seconds < 3600)
        return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400)
        return `${Math.floor(seconds / 3600)}h ago`;
    return `${Math.floor(seconds / 86400)}d ago`;
}
function generateRandomString(length) {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
}
export default APIKeys;
//# sourceMappingURL=APIKeys.js.map