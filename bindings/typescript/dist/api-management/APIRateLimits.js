import React, { useState, useEffect } from 'react';
export const APIRateLimits = ({ onRateLimitCreate, onRateLimitUpdate, onRateLimitDelete, }) => {
    const [rateLimits, setRateLimits] = useState([]);
    const [selectedLimit, setSelectedLimit] = useState(null);
    const [isCreating, setIsCreating] = useState(false);
    const [isEditing, setIsEditing] = useState(false);
    const [currentStatus, setCurrentStatus] = useState({});
    useEffect(() => {
        loadRateLimits();
        const interval = setInterval(loadCurrentStatus, 5000);
        return () => clearInterval(interval);
    }, []);
    const loadRateLimits = async () => {
        try {
            const mockLimits = [
                {
                    id: '1',
                    name: 'Global API Rate Limit',
                    type: 'sliding_window',
                    limit: 1000,
                    window: 60,
                    scope: 'global',
                    priority: 1,
                    active: true,
                    actions: [{ type: 'throttle', config: {} }],
                    metadata: {},
                },
                {
                    id: '2',
                    name: 'Per User Rate Limit',
                    type: 'token_bucket',
                    limit: 100,
                    window: 60,
                    scope: 'user',
                    priority: 2,
                    active: true,
                    actions: [{ type: 'block', config: {} }],
                    metadata: {},
                },
                {
                    id: '3',
                    name: 'Search Endpoint Limit',
                    type: 'fixed_window',
                    limit: 20,
                    window: 60,
                    scope: 'endpoint',
                    endpoints: ['/api/v1/search'],
                    priority: 3,
                    active: true,
                    actions: [{ type: 'block', config: {} }],
                    metadata: {},
                },
            ];
            setRateLimits(mockLimits);
        }
        catch (error) {
            console.error('Failed to load rate limits:', error);
        }
    };
    const loadCurrentStatus = async () => {
        try {
            const mockStatus = {
                '1': {
                    scope: 'global',
                    limit: 1000,
                    remaining: 847,
                    resetAt: Date.now() + 35000,
                },
                '2': {
                    scope: 'user:current',
                    limit: 100,
                    remaining: 73,
                    resetAt: Date.now() + 42000,
                },
                '3': {
                    scope: 'endpoint:/api/v1/search',
                    limit: 20,
                    remaining: 8,
                    resetAt: Date.now() + 28000,
                },
            };
            setCurrentStatus(mockStatus);
        }
        catch (error) {
            console.error('Failed to load rate limit status:', error);
        }
    };
    const handleCreate = async (rateLimit) => {
        try {
            if (onRateLimitCreate) {
                await onRateLimitCreate(rateLimit);
            }
            const newLimit = {
                id: Date.now().toString(),
                ...rateLimit,
            };
            setRateLimits([...rateLimits, newLimit]);
            setIsCreating(false);
        }
        catch (error) {
            console.error('Failed to create rate limit:', error);
        }
    };
    const handleUpdate = async (id, rateLimit) => {
        try {
            if (onRateLimitUpdate) {
                await onRateLimitUpdate(id, rateLimit);
            }
            setRateLimits(rateLimits.map((r) => (r.id === id ? { ...r, ...rateLimit } : r)));
            setIsEditing(false);
            setSelectedLimit(null);
        }
        catch (error) {
            console.error('Failed to update rate limit:', error);
        }
    };
    const handleDelete = async (id) => {
        if (!confirm('Are you sure you want to delete this rate limit?'))
            return;
        try {
            if (onRateLimitDelete) {
                await onRateLimitDelete(id);
            }
            setRateLimits(rateLimits.filter((r) => r.id !== id));
            if (selectedLimit?.id === id) {
                setSelectedLimit(null);
            }
        }
        catch (error) {
            console.error('Failed to delete rate limit:', error);
        }
    };
    const toggleActive = async (id) => {
        const limit = rateLimits.find((r) => r.id === id);
        if (!limit)
            return;
        await handleUpdate(id, { active: !limit.active });
    };
    return (React.createElement("div", { className: "min-h-screen bg-gray-50 dark:bg-gray-900" },
        React.createElement("div", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" },
            React.createElement("div", { className: "flex items-center justify-between mb-6" },
                React.createElement("h1", { className: "text-2xl font-bold text-gray-900 dark:text-white" }, "Rate Limits"),
                React.createElement("button", { onClick: () => setIsCreating(true), className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, "+ Create Rate Limit")),
            React.createElement("div", { className: "grid grid-cols-1 md:grid-cols-3 gap-6 mb-6" }, Object.entries(currentStatus).map(([id, status]) => {
                const limit = rateLimits.find((r) => r.id === id);
                if (!limit)
                    return null;
                const percentage = (status.remaining / status.limit) * 100;
                const timeLeft = Math.max(0, status.resetAt - Date.now()) / 1000;
                return (React.createElement("div", { key: id, className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6" },
                    React.createElement("h3", { className: "text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2" }, limit.name),
                    React.createElement("div", { className: "mb-4" },
                        React.createElement("div", { className: "flex items-baseline space-x-2" },
                            React.createElement("span", { className: "text-3xl font-bold text-gray-900 dark:text-white" }, status.remaining),
                            React.createElement("span", { className: "text-gray-500 dark:text-gray-400" },
                                "/ ",
                                status.limit)),
                        React.createElement("div", { className: "text-sm text-gray-500 dark:text-gray-400" },
                            "Resets in ",
                            Math.ceil(timeLeft),
                            "s")),
                    React.createElement("div", { className: "w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2" },
                        React.createElement("div", { className: `h-2 rounded-full transition-all ${percentage > 50
                                ? 'bg-green-600'
                                : percentage > 25
                                    ? 'bg-yellow-600'
                                    : 'bg-red-600'}`, style: { width: `${percentage}%` } }))));
            })),
            !isCreating && !isEditing ? (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden" },
                React.createElement("table", { className: "min-w-full divide-y divide-gray-200 dark:divide-gray-700" },
                    React.createElement("thead", { className: "bg-gray-50 dark:bg-gray-700" },
                        React.createElement("tr", null,
                            React.createElement("th", { className: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "Name"),
                            React.createElement("th", { className: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "Type"),
                            React.createElement("th", { className: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "Limit"),
                            React.createElement("th", { className: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "Scope"),
                            React.createElement("th", { className: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "Status"),
                            React.createElement("th", { className: "px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "Actions"))),
                    React.createElement("tbody", { className: "bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700" }, rateLimits.map((limit) => (React.createElement("tr", { key: limit.id, className: "hover:bg-gray-50 dark:hover:bg-gray-700/50" },
                        React.createElement("td", { className: "px-6 py-4" },
                            React.createElement("div", { className: "text-sm font-medium text-gray-900 dark:text-white" }, limit.name)),
                        React.createElement("td", { className: "px-6 py-4" },
                            React.createElement("span", { className: "px-2 py-1 rounded text-xs bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200" }, limit.type.replace('_', ' '))),
                        React.createElement("td", { className: "px-6 py-4" },
                            React.createElement("div", { className: "text-sm text-gray-900 dark:text-white" },
                                limit.limit,
                                " / ",
                                limit.window,
                                "s")),
                        React.createElement("td", { className: "px-6 py-4" },
                            React.createElement("span", { className: "px-2 py-1 rounded text-xs bg-purple-100 dark:bg-purple-900 text-purple-800 dark:text-purple-200" }, limit.scope)),
                        React.createElement("td", { className: "px-6 py-4" },
                            React.createElement("button", { onClick: () => toggleActive(limit.id), className: `px-2 py-1 rounded text-xs font-semibold ${limit.active
                                    ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                                    : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'}` }, limit.active ? 'Active' : 'Inactive')),
                        React.createElement("td", { className: "px-6 py-4 text-right text-sm font-medium space-x-2" },
                            React.createElement("button", { onClick: () => {
                                    setSelectedLimit(limit);
                                    setIsEditing(true);
                                }, className: "text-blue-600 hover:text-blue-900 dark:text-blue-400" }, "Edit"),
                            React.createElement("button", { onClick: () => handleDelete(limit.id), className: "text-red-600 hover:text-red-900 dark:text-red-400" }, "Delete"))))))),
                rateLimits.length === 0 && (React.createElement("div", { className: "text-center py-12 text-gray-500 dark:text-gray-400" }, "No rate limits configured")))) : (React.createElement(RateLimitEditor, { rateLimit: selectedLimit, onSave: (limit) => {
                    if (isCreating) {
                        handleCreate(limit);
                    }
                    else if (selectedLimit) {
                        handleUpdate(selectedLimit.id, limit);
                    }
                }, onCancel: () => {
                    setIsCreating(false);
                    setIsEditing(false);
                    setSelectedLimit(null);
                } })))));
};
const RateLimitEditor = ({ rateLimit, onSave, onCancel }) => {
    const [formData, setFormData] = useState(rateLimit || {
        name: '',
        type: 'fixed_window',
        limit: 100,
        window: 60,
        scope: 'global',
        priority: 10,
        active: true,
        actions: [{ type: 'throttle', config: {} }],
        metadata: {},
    });
    const handleSubmit = (e) => {
        e.preventDefault();
        onSave(formData);
    };
    return (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6" },
        React.createElement("h2", { className: "text-xl font-semibold text-gray-900 dark:text-white mb-6" }, rateLimit ? 'Edit Rate Limit' : 'Create Rate Limit'),
        React.createElement("form", { onSubmit: handleSubmit, className: "space-y-6" },
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Name"),
                React.createElement("input", { type: "text", value: formData.name, onChange: (e) => setFormData({ ...formData, name: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", required: true })),
            React.createElement("div", { className: "grid grid-cols-1 md:grid-cols-2 gap-6" },
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Type"),
                    React.createElement("select", { value: formData.type, onChange: (e) => setFormData({ ...formData, type: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                        React.createElement("option", { value: "fixed_window" }, "Fixed Window"),
                        React.createElement("option", { value: "sliding_window" }, "Sliding Window"),
                        React.createElement("option", { value: "token_bucket" }, "Token Bucket"),
                        React.createElement("option", { value: "leaky_bucket" }, "Leaky Bucket"),
                        React.createElement("option", { value: "concurrent" }, "Concurrent"))),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Scope"),
                    React.createElement("select", { value: formData.scope, onChange: (e) => setFormData({ ...formData, scope: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                        React.createElement("option", { value: "global" }, "Global"),
                        React.createElement("option", { value: "user" }, "Per User"),
                        React.createElement("option", { value: "api_key" }, "Per API Key"),
                        React.createElement("option", { value: "ip" }, "Per IP"),
                        React.createElement("option", { value: "endpoint" }, "Per Endpoint")))),
            React.createElement("div", { className: "grid grid-cols-1 md:grid-cols-2 gap-6" },
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Limit (requests)"),
                    React.createElement("input", { type: "number", value: formData.limit, onChange: (e) => setFormData({ ...formData, limit: parseInt(e.target.value) }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", min: "1", required: true })),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Window (seconds)"),
                    React.createElement("input", { type: "number", value: formData.window, onChange: (e) => setFormData({ ...formData, window: parseInt(e.target.value) }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", min: "1", required: true }))),
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Priority (lower = higher priority)"),
                React.createElement("input", { type: "number", value: formData.priority, onChange: (e) => setFormData({ ...formData, priority: parseInt(e.target.value) }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", min: "1" })),
            React.createElement("div", { className: "flex items-center" },
                React.createElement("input", { type: "checkbox", id: "active", checked: formData.active, onChange: (e) => setFormData({ ...formData, active: e.target.checked }), className: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded" }),
                React.createElement("label", { htmlFor: "active", className: "ml-2 block text-sm text-gray-700 dark:text-gray-300" }, "Active")),
            React.createElement("div", { className: "flex justify-end space-x-4" },
                React.createElement("button", { type: "button", onClick: onCancel, className: "px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700" }, "Cancel"),
                React.createElement("button", { type: "submit", className: "px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700" }, "Save")))));
};
export default APIRateLimits;
//# sourceMappingURL=APIRateLimits.js.map