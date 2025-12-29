import React, { useState, useEffect } from 'react';
export const APIEndpoints = ({ projectId = 'default', onEndpointCreate, onEndpointUpdate, onEndpointDelete, }) => {
    const [endpoints, setEndpoints] = useState([]);
    const [selectedEndpoint, setSelectedEndpoint] = useState(null);
    const [isCreating, setIsCreating] = useState(false);
    const [isEditing, setIsEditing] = useState(false);
    const [searchQuery, setSearchQuery] = useState('');
    const [filterMethod, setFilterMethod] = useState('all');
    const [filterVersion, setFilterVersion] = useState('all');
    const [isLoading, setIsLoading] = useState(false);
    useEffect(() => {
        loadEndpoints();
    }, [projectId]);
    const loadEndpoints = async () => {
        setIsLoading(true);
        try {
            await new Promise((resolve) => setTimeout(resolve, 500));
            const mockEndpoints = [
                {
                    id: '1',
                    path: '/api/v1/users',
                    method: 'GET',
                    version: 'v1',
                    summary: 'List all users',
                    description: 'Returns a paginated list of all users in the system',
                    tags: ['Users'],
                    deprecated: false,
                    security: [{ bearer: [] }],
                    parameters: [
                        {
                            name: 'page',
                            in: 'query',
                            description: 'Page number',
                            required: false,
                            deprecated: false,
                            schema: { type: 'integer', default: 1 },
                        },
                        {
                            name: 'limit',
                            in: 'query',
                            description: 'Items per page',
                            required: false,
                            deprecated: false,
                            schema: { type: 'integer', default: 20 },
                        },
                    ],
                    responses: {
                        '200': {
                            description: 'Success',
                            content: {
                                'application/json': {
                                    schema: { type: 'object' },
                                },
                            },
                        },
                    },
                    operationId: 'listUsers',
                    metadata: {},
                    createdAt: Date.now() - 86400000,
                    updatedAt: Date.now() - 3600000,
                },
                {
                    id: '2',
                    path: '/api/v1/users/{id}',
                    method: 'GET',
                    version: 'v1',
                    summary: 'Get user by ID',
                    description: 'Returns details for a specific user',
                    tags: ['Users'],
                    deprecated: false,
                    security: [{ bearer: [] }],
                    parameters: [
                        {
                            name: 'id',
                            in: 'path',
                            description: 'User ID',
                            required: true,
                            deprecated: false,
                            schema: { type: 'string' },
                        },
                    ],
                    responses: {
                        '200': {
                            description: 'Success',
                        },
                        '404': {
                            description: 'User not found',
                        },
                    },
                    operationId: 'getUserById',
                    metadata: {},
                    createdAt: Date.now() - 172800000,
                    updatedAt: Date.now() - 7200000,
                },
            ];
            setEndpoints(mockEndpoints);
        }
        catch (error) {
            console.error('Failed to load endpoints:', error);
        }
        finally {
            setIsLoading(false);
        }
    };
    const handleCreate = () => {
        setIsCreating(true);
        setIsEditing(false);
        setSelectedEndpoint(null);
    };
    const handleEdit = (endpoint) => {
        setSelectedEndpoint(endpoint);
        setIsEditing(true);
        setIsCreating(false);
    };
    const handleDelete = async (id) => {
        if (!confirm('Are you sure you want to delete this endpoint?'))
            return;
        try {
            if (onEndpointDelete) {
                await onEndpointDelete(id);
            }
            setEndpoints(endpoints.filter((e) => e.id !== id));
        }
        catch (error) {
            console.error('Failed to delete endpoint:', error);
        }
    };
    const handleSave = async (endpoint) => {
        try {
            if (isCreating) {
                if (onEndpointCreate) {
                    await onEndpointCreate(endpoint);
                }
                const newEndpoint = {
                    id: Date.now().toString(),
                    ...endpoint,
                    createdAt: Date.now(),
                    updatedAt: Date.now(),
                };
                setEndpoints([...endpoints, newEndpoint]);
            }
            else if (selectedEndpoint) {
                if (onEndpointUpdate) {
                    await onEndpointUpdate(selectedEndpoint.id, endpoint);
                }
                setEndpoints(endpoints.map((e) => e.id === selectedEndpoint.id
                    ? { ...e, ...endpoint, updatedAt: Date.now() }
                    : e));
            }
            setIsCreating(false);
            setIsEditing(false);
            setSelectedEndpoint(null);
        }
        catch (error) {
            console.error('Failed to save endpoint:', error);
        }
    };
    const filteredEndpoints = endpoints.filter((endpoint) => {
        const matchesSearch = searchQuery === '' ||
            endpoint.path.toLowerCase().includes(searchQuery.toLowerCase()) ||
            endpoint.summary.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesMethod = filterMethod === 'all' || endpoint.method === filterMethod;
        const matchesVersion = filterVersion === 'all' || endpoint.version === filterVersion;
        return matchesSearch && matchesMethod && matchesVersion;
    });
    const versions = Array.from(new Set(endpoints.map((e) => e.version)));
    const getMethodColor = (method) => {
        const colors = {
            GET: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
            POST: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
            PUT: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
            PATCH: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
            DELETE: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
            HEAD: 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200',
            OPTIONS: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
        };
        return colors[method];
    };
    return (React.createElement("div", { className: "min-h-screen bg-gray-50 dark:bg-gray-900" },
        React.createElement("div", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" },
            React.createElement("div", { className: "flex items-center justify-between mb-6" },
                React.createElement("h1", { className: "text-2xl font-bold text-gray-900 dark:text-white" }, "API Endpoints"),
                React.createElement("button", { onClick: handleCreate, className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, "+ Create Endpoint")),
            React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 mb-6" },
                React.createElement("div", { className: "grid grid-cols-1 md:grid-cols-3 gap-4" },
                    React.createElement("input", { type: "text", placeholder: "Search endpoints...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value), className: "px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" }),
                    React.createElement("select", { value: filterMethod, onChange: (e) => setFilterMethod(e.target.value), className: "px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                        React.createElement("option", { value: "all" }, "All Methods"),
                        React.createElement("option", { value: "GET" }, "GET"),
                        React.createElement("option", { value: "POST" }, "POST"),
                        React.createElement("option", { value: "PUT" }, "PUT"),
                        React.createElement("option", { value: "PATCH" }, "PATCH"),
                        React.createElement("option", { value: "DELETE" }, "DELETE")),
                    React.createElement("select", { value: filterVersion, onChange: (e) => setFilterVersion(e.target.value), className: "px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                        React.createElement("option", { value: "all" }, "All Versions"),
                        versions.map((version) => (React.createElement("option", { key: version, value: version }, version)))))),
            !isCreating && !isEditing ? (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden" },
                React.createElement("table", { className: "min-w-full divide-y divide-gray-200 dark:divide-gray-700" },
                    React.createElement("thead", { className: "bg-gray-50 dark:bg-gray-700" },
                        React.createElement("tr", null,
                            React.createElement("th", { className: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider" }, "Method"),
                            React.createElement("th", { className: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider" }, "Path"),
                            React.createElement("th", { className: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider" }, "Summary"),
                            React.createElement("th", { className: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider" }, "Version"),
                            React.createElement("th", { className: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider" }, "Status"),
                            React.createElement("th", { className: "px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider" }, "Actions"))),
                    React.createElement("tbody", { className: "bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700" }, filteredEndpoints.map((endpoint) => (React.createElement("tr", { key: endpoint.id, className: "hover:bg-gray-50 dark:hover:bg-gray-700/50" },
                        React.createElement("td", { className: "px-6 py-4 whitespace-nowrap" },
                            React.createElement("span", { className: `px-2 py-1 rounded text-xs font-semibold ${getMethodColor(endpoint.method)}` }, endpoint.method)),
                        React.createElement("td", { className: "px-6 py-4 whitespace-nowrap" },
                            React.createElement("code", { className: "text-sm font-mono text-gray-900 dark:text-white" }, endpoint.path)),
                        React.createElement("td", { className: "px-6 py-4" },
                            React.createElement("div", { className: "text-sm text-gray-900 dark:text-white" }, endpoint.summary)),
                        React.createElement("td", { className: "px-6 py-4 whitespace-nowrap" },
                            React.createElement("span", { className: "px-2 py-1 rounded text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300" }, endpoint.version)),
                        React.createElement("td", { className: "px-6 py-4 whitespace-nowrap" }, endpoint.deprecated ? (React.createElement("span", { className: "px-2 py-1 rounded text-xs font-semibold bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200" }, "Deprecated")) : (React.createElement("span", { className: "px-2 py-1 rounded text-xs font-semibold bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200" }, "Active"))),
                        React.createElement("td", { className: "px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2" },
                            React.createElement("button", { onClick: () => handleEdit(endpoint), className: "text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300" }, "Edit"),
                            React.createElement("button", { onClick: () => handleDelete(endpoint.id), className: "text-red-600 hover:text-red-900 dark:text-red-400 dark:hover:text-red-300" }, "Delete"))))))),
                filteredEndpoints.length === 0 && (React.createElement("div", { className: "text-center py-12 text-gray-500 dark:text-gray-400" }, "No endpoints found")))) : (React.createElement(EndpointEditor, { endpoint: selectedEndpoint, onSave: handleSave, onCancel: () => {
                    setIsCreating(false);
                    setIsEditing(false);
                    setSelectedEndpoint(null);
                } })))));
};
const EndpointEditor = ({ endpoint, onSave, onCancel }) => {
    const [formData, setFormData] = useState(endpoint || {
        path: '',
        method: 'GET',
        version: 'v1',
        summary: '',
        description: '',
        tags: [],
        deprecated: false,
        security: [],
        parameters: [],
        responses: {},
        operationId: '',
        metadata: {},
    });
    const handleSubmit = (e) => {
        e.preventDefault();
        onSave(formData);
    };
    return (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6" },
        React.createElement("h2", { className: "text-xl font-semibold text-gray-900 dark:text-white mb-6" }, endpoint ? 'Edit Endpoint' : 'Create Endpoint'),
        React.createElement("form", { onSubmit: handleSubmit, className: "space-y-6" },
            React.createElement("div", { className: "grid grid-cols-1 md:grid-cols-2 gap-6" },
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "HTTP Method"),
                    React.createElement("select", { value: formData.method, onChange: (e) => setFormData({ ...formData, method: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", required: true },
                        React.createElement("option", { value: "GET" }, "GET"),
                        React.createElement("option", { value: "POST" }, "POST"),
                        React.createElement("option", { value: "PUT" }, "PUT"),
                        React.createElement("option", { value: "PATCH" }, "PATCH"),
                        React.createElement("option", { value: "DELETE" }, "DELETE"),
                        React.createElement("option", { value: "HEAD" }, "HEAD"),
                        React.createElement("option", { value: "OPTIONS" }, "OPTIONS"))),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Version"),
                    React.createElement("input", { type: "text", value: formData.version, onChange: (e) => setFormData({ ...formData, version: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", placeholder: "v1", required: true }))),
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Path"),
                React.createElement("input", { type: "text", value: formData.path, onChange: (e) => setFormData({ ...formData, path: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg font-mono dark:bg-gray-700 dark:text-white", placeholder: "/api/v1/resource", required: true })),
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Summary"),
                React.createElement("input", { type: "text", value: formData.summary, onChange: (e) => setFormData({ ...formData, summary: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", placeholder: "Brief description", required: true })),
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Description"),
                React.createElement("textarea", { value: formData.description, onChange: (e) => setFormData({ ...formData, description: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", rows: 4, placeholder: "Detailed description of the endpoint" })),
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Operation ID"),
                React.createElement("input", { type: "text", value: formData.operationId, onChange: (e) => setFormData({ ...formData, operationId: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg font-mono dark:bg-gray-700 dark:text-white", placeholder: "getUsers" })),
            React.createElement("div", { className: "flex items-center" },
                React.createElement("input", { type: "checkbox", id: "deprecated", checked: formData.deprecated, onChange: (e) => setFormData({ ...formData, deprecated: e.target.checked }), className: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded" }),
                React.createElement("label", { htmlFor: "deprecated", className: "ml-2 block text-sm text-gray-700 dark:text-gray-300" }, "Mark as deprecated")),
            React.createElement("div", { className: "flex justify-end space-x-4" },
                React.createElement("button", { type: "button", onClick: onCancel, className: "px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors" }, "Cancel"),
                React.createElement("button", { type: "submit", className: "px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, "Save Endpoint")))));
};
export default APIEndpoints;
//# sourceMappingURL=APIEndpoints.js.map