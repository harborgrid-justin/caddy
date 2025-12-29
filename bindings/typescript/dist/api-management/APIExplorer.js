import React, { useState, useEffect } from 'react';
export const APIExplorer = ({ endpoints = [], onExecuteRequest, enableCodeGen = true, defaultAuth, }) => {
    const [selectedEndpoint, setSelectedEndpoint] = useState(null);
    const [parameters, setParameters] = useState({});
    const [headers, setHeaders] = useState({});
    const [body, setBody] = useState('');
    const [authConfig, setAuthConfig] = useState(defaultAuth || { type: 'none', credentials: {} });
    const [response, setResponse] = useState(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);
    const [activeTab, setActiveTab] = useState('params');
    const [responseTab, setResponseTab] = useState('body');
    const [codeLanguage, setCodeLanguage] = useState('typescript');
    const [searchQuery, setSearchQuery] = useState('');
    useEffect(() => {
        if (selectedEndpoint) {
            initializeParameters();
        }
    }, [selectedEndpoint]);
    const initializeParameters = () => {
        const params = {};
        selectedEndpoint?.parameters.forEach((param) => {
            if (param.example !== undefined) {
                params[param.name] = param.example;
            }
        });
        setParameters(params);
    };
    const handleExecute = async () => {
        if (!selectedEndpoint)
            return;
        setIsLoading(true);
        setError(null);
        try {
            const request = {
                endpoint: selectedEndpoint,
                parameters,
                headers,
                body: body ? JSON.parse(body) : undefined,
                auth: authConfig,
            };
            let result;
            if (onExecuteRequest) {
                result = await onExecuteRequest(request);
            }
            else {
                await new Promise((resolve) => setTimeout(resolve, 500));
                result = {
                    status: 200,
                    statusText: 'OK',
                    headers: {
                        'content-type': 'application/json',
                        'x-response-time': '142ms',
                    },
                    body: { success: true, data: { message: 'Sample response' } },
                    duration: 142,
                    size: 156,
                    timestamp: Date.now(),
                };
            }
            setResponse(result);
            setResponseTab('body');
        }
        catch (err) {
            setError(err instanceof Error ? err.message : 'Request failed');
        }
        finally {
            setIsLoading(false);
        }
    };
    const generateCode = () => {
        if (!selectedEndpoint)
            return '';
        const url = buildURL();
        const method = selectedEndpoint.method;
        switch (codeLanguage) {
            case 'typescript':
                return generateTypeScriptCode(method, url);
            case 'javascript':
                return generateJavaScriptCode(method, url);
            case 'python':
                return generatePythonCode(method, url);
            case 'curl':
                return generateCurlCode(method, url);
            default:
                return generateTypeScriptCode(method, url);
        }
    };
    const buildURL = () => {
        if (!selectedEndpoint)
            return '';
        let url = selectedEndpoint.path;
        selectedEndpoint.parameters
            .filter((p) => p.in === 'path')
            .forEach((param) => {
            const value = parameters[param.name];
            url = url.replace(`{${param.name}}`, String(value || ''));
        });
        const queryParams = selectedEndpoint.parameters
            .filter((p) => p.in === 'query' && parameters[p.name] !== undefined)
            .map((param) => `${param.name}=${encodeURIComponent(String(parameters[param.name]))}`)
            .join('&');
        return queryParams ? `${url}?${queryParams}` : url;
    };
    const generateTypeScriptCode = (method, url) => {
        return `// TypeScript with fetch
const response = await fetch('${url}', {
  method: '${method}',
  headers: ${JSON.stringify(headers, null, 2)},
  ${body ? `body: JSON.stringify(${body}),` : ''}
});

const data = await response.json();
console.log(data);`;
    };
    const generateJavaScriptCode = (method, url) => {
        return `// JavaScript with fetch
fetch('${url}', {
  method: '${method}',
  headers: ${JSON.stringify(headers, null, 2)},
  ${body ? `body: JSON.stringify(${body}),` : ''}
})
  .then(response => response.json())
  .then(data => console.log(data))
  .catch(error => console.error('Error:', error));`;
    };
    const generatePythonCode = (method, url) => {
        return `# Python with requests
import requests

response = requests.${method.toLowerCase()}(
    '${url}',
    headers=${JSON.stringify(headers, null, 4).replace(/"/g, "'")},
    ${body ? `json=${body},` : ''}
)

data = response.json()
print(data)`;
    };
    const generateCurlCode = (method, url) => {
        let cmd = `curl -X ${method} '${url}'`;
        Object.entries(headers).forEach(([key, value]) => {
            cmd += ` \\\n  -H '${key}: ${value}'`;
        });
        if (body) {
            cmd += ` \\\n  -d '${body}'`;
        }
        return cmd;
    };
    const filteredEndpoints = endpoints.filter((endpoint) => endpoint.path.toLowerCase().includes(searchQuery.toLowerCase()) ||
        endpoint.summary.toLowerCase().includes(searchQuery.toLowerCase()) ||
        endpoint.tags.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase())));
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
    const getStatusColor = (status) => {
        if (status >= 200 && status < 300)
            return 'text-green-600 dark:text-green-400';
        if (status >= 300 && status < 400)
            return 'text-blue-600 dark:text-blue-400';
        if (status >= 400 && status < 500)
            return 'text-orange-600 dark:text-orange-400';
        return 'text-red-600 dark:text-red-400';
    };
    return (React.createElement("div", { className: "flex h-screen bg-gray-50 dark:bg-gray-900" },
        React.createElement("div", { className: "w-80 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col" },
            React.createElement("div", { className: "p-4 border-b border-gray-200 dark:border-gray-700" },
                React.createElement("input", { type: "text", placeholder: "Search endpoints...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value), className: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" })),
            React.createElement("div", { className: "flex-1 overflow-y-auto" }, filteredEndpoints.map((endpoint) => (React.createElement("button", { key: endpoint.id, onClick: () => setSelectedEndpoint(endpoint), className: `w-full text-left px-4 py-3 border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors ${selectedEndpoint?.id === endpoint.id ? 'bg-blue-50 dark:bg-blue-900/20' : ''}` },
                React.createElement("div", { className: "flex items-center space-x-2 mb-1" },
                    React.createElement("span", { className: `px-2 py-0.5 rounded text-xs font-semibold ${getMethodColor(endpoint.method)}` }, endpoint.method),
                    endpoint.deprecated && (React.createElement("span", { className: "px-2 py-0.5 rounded text-xs font-semibold bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200" }, "DEPRECATED"))),
                React.createElement("div", { className: "text-sm font-mono text-gray-900 dark:text-white mb-1" }, endpoint.path),
                React.createElement("div", { className: "text-xs text-gray-500 dark:text-gray-400 truncate" }, endpoint.summary)))))),
        React.createElement("div", { className: "flex-1 flex flex-col overflow-hidden" }, selectedEndpoint ? (React.createElement(React.Fragment, null,
            React.createElement("div", { className: "bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-6" },
                React.createElement("div", { className: "flex items-start justify-between" },
                    React.createElement("div", { className: "flex-1" },
                        React.createElement("div", { className: "flex items-center space-x-3 mb-2" },
                            React.createElement("span", { className: `px-3 py-1 rounded font-semibold ${getMethodColor(selectedEndpoint.method)}` }, selectedEndpoint.method),
                            React.createElement("code", { className: "text-lg font-mono text-gray-900 dark:text-white" }, selectedEndpoint.path)),
                        React.createElement("p", { className: "text-gray-600 dark:text-gray-400" }, selectedEndpoint.description),
                        selectedEndpoint.tags.length > 0 && (React.createElement("div", { className: "flex items-center space-x-2 mt-2" }, selectedEndpoint.tags.map((tag) => (React.createElement("span", { key: tag, className: "px-2 py-1 rounded text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300" }, tag)))))),
                    React.createElement("button", { onClick: handleExecute, disabled: isLoading, className: "px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors" }, isLoading ? 'Sending...' : 'Send Request'))),
            React.createElement("div", { className: "flex-1 flex overflow-hidden" },
                React.createElement("div", { className: "flex-1 flex flex-col overflow-hidden" },
                    React.createElement("div", { className: "bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700" },
                        React.createElement("div", { className: "flex space-x-1 px-6 pt-4" }, [
                            { id: 'params', label: 'Parameters' },
                            { id: 'headers', label: 'Headers' },
                            { id: 'body', label: 'Body' },
                            { id: 'auth', label: 'Auth' },
                        ].map((tab) => (React.createElement("button", { key: tab.id, onClick: () => setActiveTab(tab.id), className: `px-4 py-2 border-b-2 transition-colors ${activeTab === tab.id
                                ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                                : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}` }, tab.label))))),
                    React.createElement("div", { className: "flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-900 p-6" },
                        activeTab === 'params' && (React.createElement(ParametersPanel, { parameters: selectedEndpoint.parameters, values: parameters, onChange: setParameters })),
                        activeTab === 'headers' && (React.createElement(HeadersPanel, { headers: headers, onChange: setHeaders })),
                        activeTab === 'body' && (React.createElement(BodyPanel, { body: body, onChange: setBody })),
                        activeTab === 'auth' && (React.createElement(AuthPanel, { config: authConfig, onChange: setAuthConfig })))),
                React.createElement("div", { className: "w-1/2 border-l border-gray-200 dark:border-gray-700 flex flex-col bg-white dark:bg-gray-800" },
                    React.createElement("div", { className: "border-b border-gray-200 dark:border-gray-700" },
                        React.createElement("div", { className: "flex items-center justify-between px-6 py-3" },
                            React.createElement("h3", { className: "font-semibold text-gray-900 dark:text-white" }, "Response"),
                            response && (React.createElement("div", { className: "flex items-center space-x-4 text-sm" },
                                React.createElement("span", { className: `font-semibold ${getStatusColor(response.status)}` },
                                    response.status,
                                    " ",
                                    response.statusText),
                                React.createElement("span", { className: "text-gray-500 dark:text-gray-400" },
                                    response.duration,
                                    "ms"),
                                React.createElement("span", { className: "text-gray-500 dark:text-gray-400" },
                                    (response.size / 1024).toFixed(2),
                                    " KB")))),
                        React.createElement("div", { className: "flex space-x-1 px-6" }, [
                            { id: 'body', label: 'Body' },
                            { id: 'headers', label: 'Headers' },
                            { id: 'code', label: 'Code' },
                        ].map((tab) => (React.createElement("button", { key: tab.id, onClick: () => setResponseTab(tab.id), className: `px-4 py-2 border-b-2 transition-colors ${responseTab === tab.id
                                ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                                : 'border-transparent text-gray-600 dark:text-gray-400'}` }, tab.label))))),
                    React.createElement("div", { className: "flex-1 overflow-y-auto p-6" },
                        error && (React.createElement("div", { className: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 px-4 py-3 rounded" }, error)),
                        !response && !error && (React.createElement("div", { className: "text-center text-gray-500 dark:text-gray-400 py-12" }, "Send a request to see the response")),
                        response && responseTab === 'body' && (React.createElement("pre", { className: "bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto" },
                            React.createElement("code", null, JSON.stringify(response.body, null, 2)))),
                        response && responseTab === 'headers' && (React.createElement("div", { className: "space-y-2" }, Object.entries(response.headers).map(([key, value]) => (React.createElement("div", { key: key, className: "flex items-center space-x-2 text-sm" },
                            React.createElement("span", { className: "font-semibold text-gray-700 dark:text-gray-300" },
                                key,
                                ":"),
                            React.createElement("span", { className: "text-gray-600 dark:text-gray-400" }, value)))))),
                        responseTab === 'code' && (React.createElement("div", null,
                            React.createElement("div", { className: "mb-4" },
                                React.createElement("select", { value: codeLanguage, onChange: (e) => setCodeLanguage(e.target.value), className: "px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                                    React.createElement("option", { value: "typescript" }, "TypeScript"),
                                    React.createElement("option", { value: "javascript" }, "JavaScript"),
                                    React.createElement("option", { value: "python" }, "Python"),
                                    React.createElement("option", { value: "curl" }, "cURL"))),
                            React.createElement("pre", { className: "bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto" },
                                React.createElement("code", null, generateCode()))))))))) : (React.createElement("div", { className: "flex items-center justify-center h-full text-gray-500 dark:text-gray-400" }, "Select an endpoint to get started")))));
};
const ParametersPanel = ({ parameters, values, onChange }) => {
    if (parameters.length === 0) {
        return React.createElement("div", { className: "text-gray-500 dark:text-gray-400" }, "No parameters");
    }
    return (React.createElement("div", { className: "space-y-4" }, parameters.map((param) => (React.createElement("div", { key: param.name },
        React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1" },
            param.name,
            param.required && React.createElement("span", { className: "text-red-500 ml-1" }, "*"),
            React.createElement("span", { className: "ml-2 text-xs text-gray-500" },
                "(",
                param.in,
                ")")),
        React.createElement("input", { type: "text", value: String(values[param.name] || ''), onChange: (e) => onChange({ ...values, [param.name]: e.target.value }), placeholder: param.description, className: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" }),
        param.description && (React.createElement("p", { className: "mt-1 text-xs text-gray-500 dark:text-gray-400" }, param.description)))))));
};
const HeadersPanel = ({ headers, onChange }) => {
    const addHeader = () => {
        onChange({ ...headers, '': '' });
    };
    return (React.createElement("div", { className: "space-y-2" },
        Object.entries(headers).map(([key, value], index) => (React.createElement("div", { key: index, className: "flex space-x-2" },
            React.createElement("input", { type: "text", value: key, onChange: (e) => {
                    const newHeaders = { ...headers };
                    delete newHeaders[key];
                    newHeaders[e.target.value] = value;
                    onChange(newHeaders);
                }, placeholder: "Header name", className: "flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" }),
            React.createElement("input", { type: "text", value: value, onChange: (e) => onChange({ ...headers, [key]: e.target.value }), placeholder: "Header value", className: "flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" }),
            React.createElement("button", { onClick: () => {
                    const newHeaders = { ...headers };
                    delete newHeaders[key];
                    onChange(newHeaders);
                }, className: "px-3 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg" }, "Remove")))),
        React.createElement("button", { onClick: addHeader, className: "px-4 py-2 text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-lg" }, "+ Add Header")));
};
const BodyPanel = ({ body, onChange }) => {
    return (React.createElement("textarea", { value: body, onChange: (e) => onChange(e.target.value), placeholder: '{"key": "value"}', className: "w-full h-64 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg font-mono dark:bg-gray-700 dark:text-white" }));
};
const AuthPanel = ({ config, onChange }) => {
    return (React.createElement("div", { className: "space-y-4" },
        React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1" }, "Auth Type"),
            React.createElement("select", { value: config.type, onChange: (e) => onChange({ ...config, type: e.target.value }), className: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                React.createElement("option", { value: "none" }, "No Auth"),
                React.createElement("option", { value: "api_key" }, "API Key"),
                React.createElement("option", { value: "bearer" }, "Bearer Token"),
                React.createElement("option", { value: "basic" }, "Basic Auth"),
                React.createElement("option", { value: "oauth2" }, "OAuth 2.0"))),
        config.type === 'api_key' && (React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1" }, "API Key"),
            React.createElement("input", { type: "text", value: config.credentials.apiKey || '', onChange: (e) => onChange({ ...config, credentials: { ...config.credentials, apiKey: e.target.value } }), className: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" }))),
        config.type === 'bearer' && (React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1" }, "Token"),
            React.createElement("input", { type: "text", value: config.credentials.token || '', onChange: (e) => onChange({ ...config, credentials: { ...config.credentials, token: e.target.value } }), className: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" }))),
        config.type === 'basic' && (React.createElement(React.Fragment, null,
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1" }, "Username"),
                React.createElement("input", { type: "text", value: config.credentials.username || '', onChange: (e) => onChange({
                        ...config,
                        credentials: { ...config.credentials, username: e.target.value },
                    }), className: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" })),
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1" }, "Password"),
                React.createElement("input", { type: "password", value: config.credentials.password || '', onChange: (e) => onChange({
                        ...config,
                        credentials: { ...config.credentials, password: e.target.value },
                    }), className: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" }))))));
};
export default APIExplorer;
//# sourceMappingURL=APIExplorer.js.map