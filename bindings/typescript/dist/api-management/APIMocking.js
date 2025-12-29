import React, { useState, useEffect } from 'react';
export const APIMocking = ({ onServerCreate, onServerUpdate, onServerDelete, }) => {
    const [servers, setServers] = useState([]);
    const [selectedServer, setSelectedServer] = useState(null);
    const [selectedEndpoint, setSelectedEndpoint] = useState(null);
    const [isCreatingServer, setIsCreatingServer] = useState(false);
    const [isCreatingEndpoint, setIsCreatingEndpoint] = useState(false);
    useEffect(() => {
        loadServers();
    }, []);
    const loadServers = async () => {
        try {
            const mockServers = [
                {
                    id: '1',
                    name: 'Development Mock Server',
                    baseUrl: 'http://localhost:3000/mock',
                    endpoints: [
                        {
                            id: 'e1',
                            path: '/api/v1/users',
                            method: 'GET',
                            responses: [
                                {
                                    id: 'r1',
                                    name: 'Success Response',
                                    statusCode: 200,
                                    headers: { 'Content-Type': 'application/json' },
                                    body: {
                                        users: [
                                            { id: 1, name: 'John Doe', email: 'john@example.com' },
                                            { id: 2, name: 'Jane Smith', email: 'jane@example.com' },
                                        ],
                                    },
                                    weight: 90,
                                },
                                {
                                    id: 'r2',
                                    name: 'Error Response',
                                    statusCode: 500,
                                    headers: { 'Content-Type': 'application/json' },
                                    body: { error: 'Internal Server Error' },
                                    weight: 10,
                                },
                            ],
                            delay: { min: 100, max: 500 },
                            active: true,
                            createdAt: Date.now() - 86400000,
                        },
                        {
                            id: 'e2',
                            path: '/api/v1/products/{id}',
                            method: 'GET',
                            responses: [
                                {
                                    id: 'r3',
                                    name: 'Product Found',
                                    statusCode: 200,
                                    headers: { 'Content-Type': 'application/json' },
                                    body: {
                                        id: 1,
                                        name: 'Sample Product',
                                        price: 99.99,
                                    },
                                    weight: 100,
                                },
                            ],
                            active: true,
                            createdAt: Date.now() - 172800000,
                        },
                    ],
                    active: true,
                    createdAt: Date.now() - 86400000 * 15,
                },
            ];
            setServers(mockServers);
            if (mockServers.length > 0) {
                setSelectedServer(mockServers[0]);
            }
        }
        catch (error) {
            console.error('Failed to load mock servers:', error);
        }
    };
    const handleCreateServer = async (server) => {
        try {
            if (onServerCreate) {
                await onServerCreate(server);
            }
            const newServer = {
                id: Date.now().toString(),
                ...server,
                endpoints: [],
                createdAt: Date.now(),
            };
            setServers([...servers, newServer]);
            setIsCreatingServer(false);
        }
        catch (error) {
            console.error('Failed to create mock server:', error);
        }
    };
    const handleDeleteServer = async (id) => {
        if (!confirm('Are you sure you want to delete this mock server?'))
            return;
        try {
            if (onServerDelete) {
                await onServerDelete(id);
            }
            setServers(servers.filter((s) => s.id !== id));
            if (selectedServer?.id === id) {
                setSelectedServer(null);
            }
        }
        catch (error) {
            console.error('Failed to delete mock server:', error);
        }
    };
    const handleCreateEndpoint = (endpoint) => {
        if (!selectedServer)
            return;
        const updatedServer = {
            ...selectedServer,
            endpoints: [...selectedServer.endpoints, endpoint],
        };
        setServers(servers.map((s) => (s.id === selectedServer.id ? updatedServer : s)));
        setSelectedServer(updatedServer);
        setIsCreatingEndpoint(false);
    };
    const handleDeleteEndpoint = (endpointId) => {
        if (!selectedServer)
            return;
        if (!confirm('Are you sure you want to delete this endpoint?'))
            return;
        const updatedServer = {
            ...selectedServer,
            endpoints: selectedServer.endpoints.filter((e) => e.id !== endpointId),
        };
        setServers(servers.map((s) => (s.id === selectedServer.id ? updatedServer : s)));
        setSelectedServer(updatedServer);
        if (selectedEndpoint?.id === endpointId) {
            setSelectedEndpoint(null);
        }
    };
    const toggleServerActive = (id) => {
        const server = servers.find((s) => s.id === id);
        if (!server)
            return;
        const updatedServer = { ...server, active: !server.active };
        setServers(servers.map((s) => (s.id === id ? updatedServer : s)));
        if (selectedServer?.id === id) {
            setSelectedServer(updatedServer);
        }
    };
    const copyMockUrl = (path) => {
        if (selectedServer) {
            const fullUrl = `${selectedServer.baseUrl}${path}`;
            navigator.clipboard.writeText(fullUrl);
        }
    };
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
                React.createElement("h1", { className: "text-2xl font-bold text-gray-900 dark:text-white" }, "Mock Servers"),
                React.createElement("button", { onClick: () => setIsCreatingServer(true), className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, "+ Create Mock Server")),
            servers.length > 0 && (React.createElement("div", { className: "mb-6 flex items-center space-x-4" },
                React.createElement("select", { value: selectedServer?.id || '', onChange: (e) => setSelectedServer(servers.find((s) => s.id === e.target.value) || null), className: "px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-800 dark:text-white" }, servers.map((server) => (React.createElement("option", { key: server.id, value: server.id }, server.name)))),
                selectedServer && (React.createElement(React.Fragment, null,
                    React.createElement("div", { className: "flex items-center space-x-2" },
                        React.createElement("code", { className: "px-3 py-2 bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white rounded text-sm font-mono" }, selectedServer.baseUrl),
                        React.createElement("button", { onClick: () => navigator.clipboard.writeText(selectedServer.baseUrl), className: "text-blue-600 hover:text-blue-800 dark:text-blue-400", title: "Copy URL" }, "\uD83D\uDCCB")),
                    React.createElement("button", { onClick: () => toggleServerActive(selectedServer.id), className: `px-3 py-2 rounded text-sm font-semibold ${selectedServer.active
                            ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                            : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'}` }, selectedServer.active ? 'Active' : 'Inactive'),
                    React.createElement("button", { onClick: () => handleDeleteServer(selectedServer.id), className: "text-red-600 hover:text-red-800 dark:text-red-400" }, "Delete Server"))))),
            selectedServer && (React.createElement("div", { className: "grid grid-cols-1 lg:grid-cols-3 gap-6" },
                React.createElement("div", { className: "lg:col-span-2" },
                    React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700" },
                        React.createElement("div", { className: "px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between" },
                            React.createElement("h2", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, "Mock Endpoints"),
                            React.createElement("button", { onClick: () => setIsCreatingEndpoint(true), className: "px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700" }, "+ Add Endpoint")),
                        React.createElement("div", { className: "divide-y divide-gray-200 dark:divide-gray-700" },
                            selectedServer.endpoints.map((endpoint) => (React.createElement("div", { key: endpoint.id, onClick: () => setSelectedEndpoint(endpoint), className: `px-6 py-4 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700/50 ${selectedEndpoint?.id === endpoint.id ? 'bg-blue-50 dark:bg-blue-900/20' : ''}` },
                                React.createElement("div", { className: "flex items-center justify-between mb-2" },
                                    React.createElement("div", { className: "flex items-center space-x-3" },
                                        React.createElement("span", { className: `px-2 py-1 rounded text-xs font-semibold ${getMethodColor(endpoint.method)}` }, endpoint.method),
                                        React.createElement("code", { className: "text-sm font-mono text-gray-900 dark:text-white" }, endpoint.path),
                                        React.createElement("button", { onClick: (e) => {
                                                e.stopPropagation();
                                                copyMockUrl(endpoint.path);
                                            }, className: "text-blue-600 hover:text-blue-800 dark:text-blue-400", title: "Copy mock URL" }, "\uD83D\uDCCB")),
                                    React.createElement("button", { onClick: (e) => {
                                            e.stopPropagation();
                                            handleDeleteEndpoint(endpoint.id);
                                        }, className: "text-red-600 hover:text-red-800 dark:text-red-400 text-sm" }, "Delete")),
                                React.createElement("div", { className: "flex items-center space-x-4 text-sm text-gray-600 dark:text-gray-400" },
                                    React.createElement("span", null,
                                        endpoint.responses.length,
                                        " response(s)"),
                                    endpoint.delay && (React.createElement("span", null,
                                        "Delay: ",
                                        endpoint.delay.min,
                                        "-",
                                        endpoint.delay.max,
                                        "ms")),
                                    React.createElement("span", { className: endpoint.active ? 'text-green-600' : 'text-gray-400' }, endpoint.active ? 'Active' : 'Inactive'))))),
                            selectedServer.endpoints.length === 0 && (React.createElement("div", { className: "px-6 py-12 text-center text-gray-500 dark:text-gray-400" }, "No endpoints configured. Add your first endpoint to get started."))))),
                React.createElement("div", { className: "lg:col-span-1" }, selectedEndpoint ? (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 sticky top-4" },
                    React.createElement("div", { className: "px-6 py-4 border-b border-gray-200 dark:border-gray-700" },
                        React.createElement("h3", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, "Mock Responses")),
                    React.createElement("div", { className: "p-6 space-y-4 max-h-[600px] overflow-y-auto" }, selectedEndpoint.responses.map((response) => (React.createElement("div", { key: response.id, className: "border border-gray-200 dark:border-gray-700 rounded-lg p-4" },
                        React.createElement("div", { className: "flex items-center justify-between mb-2" },
                            React.createElement("span", { className: "text-sm font-medium text-gray-900 dark:text-white" }, response.name),
                            React.createElement("span", { className: `px-2 py-1 rounded text-xs font-semibold ${getStatusColor(response.statusCode)}` }, response.statusCode)),
                        React.createElement("div", { className: "text-xs text-gray-600 dark:text-gray-400 mb-2" },
                            "Weight: ",
                            response.weight,
                            "%"),
                        React.createElement("div", { className: "bg-gray-900 text-gray-100 p-3 rounded text-xs overflow-x-auto" },
                            React.createElement("pre", null, JSON.stringify(response.body, null, 2))))))))) : (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 text-center text-gray-500 dark:text-gray-400" }, "Select an endpoint to view responses"))))),
            isCreatingServer && (React.createElement(CreateServerModal, { onClose: () => setIsCreatingServer(false), onCreate: handleCreateServer })),
            isCreatingEndpoint && (React.createElement(CreateEndpointModal, { onClose: () => setIsCreatingEndpoint(false), onCreate: handleCreateEndpoint })))));
};
const CreateServerModal = ({ onClose, onCreate }) => {
    const [name, setName] = useState('');
    const [baseUrl, setBaseUrl] = useState('http://localhost:3000/mock');
    const handleSubmit = (e) => {
        e.preventDefault();
        onCreate({ name, baseUrl, active: true });
    };
    return (React.createElement("div", { className: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" },
        React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6" },
            React.createElement("h2", { className: "text-xl font-semibold text-gray-900 dark:text-white mb-4" }, "Create Mock Server"),
            React.createElement("form", { onSubmit: handleSubmit, className: "space-y-4" },
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Server Name"),
                    React.createElement("input", { type: "text", value: name, onChange: (e) => setName(e.target.value), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", required: true })),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Base URL"),
                    React.createElement("input", { type: "url", value: baseUrl, onChange: (e) => setBaseUrl(e.target.value), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", required: true })),
                React.createElement("div", { className: "flex justify-end space-x-4 pt-4" },
                    React.createElement("button", { type: "button", onClick: onClose, className: "px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700" }, "Cancel"),
                    React.createElement("button", { type: "submit", className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700" }, "Create"))))));
};
const CreateEndpointModal = ({ onClose, onCreate }) => {
    const [path, setPath] = useState('');
    const [method, setMethod] = useState('GET');
    const [responseBody, setResponseBody] = useState('{"message": "Success"}');
    const handleSubmit = (e) => {
        e.preventDefault();
        try {
            const body = JSON.parse(responseBody);
            const endpoint = {
                id: Date.now().toString(),
                path,
                method,
                responses: [
                    {
                        id: '1',
                        name: 'Default Response',
                        statusCode: 200,
                        headers: { 'Content-Type': 'application/json' },
                        body,
                        weight: 100,
                    },
                ],
                active: true,
                createdAt: Date.now(),
            };
            onCreate(endpoint);
        }
        catch (error) {
            alert('Invalid JSON in response body');
        }
    };
    return (React.createElement("div", { className: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" },
        React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6" },
            React.createElement("h2", { className: "text-xl font-semibold text-gray-900 dark:text-white mb-4" }, "Create Mock Endpoint"),
            React.createElement("form", { onSubmit: handleSubmit, className: "space-y-4" },
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Path"),
                    React.createElement("input", { type: "text", value: path, onChange: (e) => setPath(e.target.value), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg font-mono dark:bg-gray-700 dark:text-white", placeholder: "/api/v1/resource", required: true })),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Method"),
                    React.createElement("select", { value: method, onChange: (e) => setMethod(e.target.value), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                        React.createElement("option", { value: "GET" }, "GET"),
                        React.createElement("option", { value: "POST" }, "POST"),
                        React.createElement("option", { value: "PUT" }, "PUT"),
                        React.createElement("option", { value: "PATCH" }, "PATCH"),
                        React.createElement("option", { value: "DELETE" }, "DELETE"))),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Response Body (JSON)"),
                    React.createElement("textarea", { value: responseBody, onChange: (e) => setResponseBody(e.target.value), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg font-mono dark:bg-gray-700 dark:text-white", rows: 6 })),
                React.createElement("div", { className: "flex justify-end space-x-4 pt-4" },
                    React.createElement("button", { type: "button", onClick: onClose, className: "px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700" }, "Cancel"),
                    React.createElement("button", { type: "submit", className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700" }, "Create"))))));
};
function getStatusColor(status) {
    if (status >= 200 && status < 300)
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
    if (status >= 300 && status < 400)
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
    if (status >= 400 && status < 500)
        return 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200';
    return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
}
export default APIMocking;
//# sourceMappingURL=APIMocking.js.map