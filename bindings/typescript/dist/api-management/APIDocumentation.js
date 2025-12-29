import React, { useState } from 'react';
export const APIDocumentation = ({ spec, endpoints = [], onTryEndpoint, showTryItOut = true, }) => {
    const [selectedTag, setSelectedTag] = useState('all');
    const [expandedSections, setExpandedSections] = useState(new Set());
    const [searchQuery, setSearchQuery] = useState('');
    const [viewMode, setViewMode] = useState('grouped');
    const tags = Array.from(new Set(endpoints.flatMap((e) => e.tags)));
    const filteredEndpoints = endpoints.filter((endpoint) => {
        const matchesSearch = searchQuery === '' ||
            endpoint.path.toLowerCase().includes(searchQuery.toLowerCase()) ||
            endpoint.summary.toLowerCase().includes(searchQuery.toLowerCase()) ||
            endpoint.description.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesTag = selectedTag === 'all' || endpoint.tags.includes(selectedTag);
        return matchesSearch && matchesTag;
    });
    const groupedEndpoints = viewMode === 'grouped'
        ? tags.reduce((acc, tag) => {
            acc[tag] = filteredEndpoints.filter((e) => e.tags.includes(tag));
            return acc;
        }, {})
        : { all: filteredEndpoints };
    const toggleSection = (id) => {
        const newExpanded = new Set(expandedSections);
        if (newExpanded.has(id)) {
            newExpanded.delete(id);
        }
        else {
            newExpanded.add(id);
        }
        setExpandedSections(newExpanded);
    };
    const getMethodColor = (method) => {
        const colors = {
            GET: 'bg-blue-600 text-white',
            POST: 'bg-green-600 text-white',
            PUT: 'bg-yellow-600 text-white',
            PATCH: 'bg-orange-600 text-white',
            DELETE: 'bg-red-600 text-white',
            HEAD: 'bg-gray-600 text-white',
            OPTIONS: 'bg-purple-600 text-white',
        };
        return colors[method];
    };
    return (React.createElement("div", { className: "min-h-screen bg-gray-50 dark:bg-gray-900" },
        React.createElement("div", { className: "bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700" },
            React.createElement("div", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6" },
                React.createElement("h1", { className: "text-3xl font-bold text-gray-900 dark:text-white mb-2" }, spec?.info.title || 'API Documentation'),
                React.createElement("p", { className: "text-gray-600 dark:text-gray-400 mb-4" }, spec?.info.description || 'Comprehensive API documentation'),
                spec?.info && (React.createElement("div", { className: "flex items-center space-x-6 text-sm text-gray-600 dark:text-gray-400" },
                    React.createElement("div", null,
                        "Version: ",
                        spec.info.version),
                    spec.info.contact?.email && (React.createElement("div", null,
                        "Contact: ",
                        spec.info.contact.email)),
                    spec.info.license && (React.createElement("div", null,
                        "License: ",
                        spec.info.license.name)))))),
        React.createElement("div", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" },
            React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 mb-6" },
                React.createElement("div", { className: "flex items-center justify-between space-x-4" },
                    React.createElement("div", { className: "flex items-center space-x-4 flex-1" },
                        React.createElement("input", { type: "text", placeholder: "Search endpoints...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value), className: "flex-1 max-w-md px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" }),
                        React.createElement("select", { value: selectedTag, onChange: (e) => setSelectedTag(e.target.value), className: "px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white" },
                            React.createElement("option", { value: "all" }, "All Tags"),
                            tags.map((tag) => (React.createElement("option", { key: tag, value: tag }, tag))))),
                    React.createElement("div", { className: "flex items-center space-x-2 bg-gray-100 dark:bg-gray-700 rounded-lg p-1" },
                        React.createElement("button", { onClick: () => setViewMode('grouped'), className: `px-3 py-1 rounded ${viewMode === 'grouped'
                                ? 'bg-white dark:bg-gray-600 shadow'
                                : 'text-gray-600 dark:text-gray-400'}` }, "Grouped"),
                        React.createElement("button", { onClick: () => setViewMode('list'), className: `px-3 py-1 rounded ${viewMode === 'list'
                                ? 'bg-white dark:bg-gray-600 shadow'
                                : 'text-gray-600 dark:text-gray-400'}` }, "List")))),
            React.createElement("div", { className: "space-y-6" }, Object.entries(groupedEndpoints).map(([tag, tagEndpoints]) => (React.createElement("div", { key: tag, className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden" },
                viewMode === 'grouped' && (React.createElement("div", { className: "bg-gray-50 dark:bg-gray-700 px-6 py-3 border-b border-gray-200 dark:border-gray-600" },
                    React.createElement("h2", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, tag.charAt(0).toUpperCase() + tag.slice(1)))),
                React.createElement("div", { className: "divide-y divide-gray-200 dark:divide-gray-700" }, tagEndpoints.map((endpoint) => (React.createElement(EndpointDocumentation, { key: endpoint.id, endpoint: endpoint, isExpanded: expandedSections.has(endpoint.id), onToggle: () => toggleSection(endpoint.id), onTryIt: onTryEndpoint ? () => onTryEndpoint(endpoint) : undefined, showTryItOut: showTryItOut, getMethodColor: getMethodColor })))))))),
            filteredEndpoints.length === 0 && (React.createElement("div", { className: "text-center py-12 text-gray-500 dark:text-gray-400" }, "No endpoints found matching your criteria")),
            spec?.components?.schemas && (React.createElement("div", { className: "mt-8 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700" },
                React.createElement("div", { className: "bg-gray-50 dark:bg-gray-700 px-6 py-3 border-b border-gray-200 dark:border-gray-600" },
                    React.createElement("h2", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, "Schemas")),
                React.createElement("div", { className: "p-6 space-y-4" }, Object.entries(spec.components.schemas).map(([name, schema]) => (React.createElement(SchemaViewer, { key: name, name: name, schema: schema })))))))));
};
const EndpointDocumentation = ({ endpoint, isExpanded, onToggle, onTryIt, showTryItOut, getMethodColor, }) => {
    return (React.createElement("div", null,
        React.createElement("button", { onClick: onToggle, className: "w-full px-6 py-4 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors" },
            React.createElement("div", { className: "flex items-center space-x-4" },
                React.createElement("span", { className: `px-3 py-1 rounded font-semibold text-sm ${getMethodColor(endpoint.method)}` }, endpoint.method),
                React.createElement("code", { className: "text-base font-mono text-gray-900 dark:text-white" }, endpoint.path),
                endpoint.deprecated && (React.createElement("span", { className: "px-2 py-1 rounded text-xs font-semibold bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200" }, "DEPRECATED"))),
            React.createElement("div", { className: "flex items-center space-x-3" },
                React.createElement("span", { className: "text-gray-600 dark:text-gray-400" }, endpoint.summary),
                React.createElement("span", { className: "text-gray-400" }, isExpanded ? '▲' : '▼'))),
        isExpanded && (React.createElement("div", { className: "px-6 py-4 bg-gray-50 dark:bg-gray-900/50 border-t border-gray-200 dark:border-gray-700" },
            React.createElement("div", { className: "mb-6" },
                React.createElement("p", { className: "text-gray-700 dark:text-gray-300" }, endpoint.description)),
            showTryItOut && onTryIt && (React.createElement("div", { className: "mb-6" },
                React.createElement("button", { onClick: onTryIt, className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, "Try it out"))),
            endpoint.parameters.length > 0 && (React.createElement("div", { className: "mb-6" },
                React.createElement("h3", { className: "text-sm font-semibold text-gray-900 dark:text-white mb-3" }, "Parameters"),
                React.createElement("div", { className: "overflow-x-auto" },
                    React.createElement("table", { className: "min-w-full divide-y divide-gray-200 dark:divide-gray-700" },
                        React.createElement("thead", { className: "bg-gray-100 dark:bg-gray-800" },
                            React.createElement("tr", null,
                                React.createElement("th", { className: "px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "Name"),
                                React.createElement("th", { className: "px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "In"),
                                React.createElement("th", { className: "px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "Type"),
                                React.createElement("th", { className: "px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "Required"),
                                React.createElement("th", { className: "px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase" }, "Description"))),
                        React.createElement("tbody", { className: "bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700" }, endpoint.parameters.map((param) => (React.createElement("tr", { key: param.name },
                            React.createElement("td", { className: "px-4 py-2 text-sm font-mono text-gray-900 dark:text-white" }, param.name),
                            React.createElement("td", { className: "px-4 py-2 text-sm text-gray-600 dark:text-gray-400" }, param.in),
                            React.createElement("td", { className: "px-4 py-2 text-sm text-gray-600 dark:text-gray-400" }, param.schema.type || 'any'),
                            React.createElement("td", { className: "px-4 py-2 text-sm" }, param.required ? (React.createElement("span", { className: "text-red-600 dark:text-red-400" }, "Yes")) : (React.createElement("span", { className: "text-gray-400" }, "No"))),
                            React.createElement("td", { className: "px-4 py-2 text-sm text-gray-600 dark:text-gray-400" }, param.description))))))))),
            endpoint.requestBody && (React.createElement("div", { className: "mb-6" },
                React.createElement("h3", { className: "text-sm font-semibold text-gray-900 dark:text-white mb-3" }, "Request Body"),
                React.createElement("p", { className: "text-sm text-gray-600 dark:text-gray-400 mb-2" }, endpoint.requestBody.description),
                Object.entries(endpoint.requestBody.content).map(([contentType, media]) => (React.createElement("div", { key: contentType, className: "mb-4" },
                    React.createElement("div", { className: "text-xs text-gray-500 dark:text-gray-400 mb-2" },
                        "Content-Type: ",
                        contentType),
                    React.createElement("pre", { className: "bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm" },
                        React.createElement("code", null, JSON.stringify(media.example || media.schema, null, 2)))))))),
            React.createElement("div", { className: "mb-6" },
                React.createElement("h3", { className: "text-sm font-semibold text-gray-900 dark:text-white mb-3" }, "Responses"),
                React.createElement("div", { className: "space-y-3" }, Object.entries(endpoint.responses).map(([statusCode, response]) => (React.createElement("div", { key: statusCode, className: "border border-gray-200 dark:border-gray-700 rounded-lg" },
                    React.createElement("div", { className: "px-4 py-2 bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700" },
                        React.createElement("div", { className: "flex items-center space-x-3" },
                            React.createElement("span", { className: `font-mono font-semibold ${getStatusColor(parseInt(statusCode))}` }, statusCode),
                            React.createElement("span", { className: "text-sm text-gray-600 dark:text-gray-400" }, response.description))),
                    response.content && (React.createElement("div", { className: "p-4" }, Object.entries(response.content).map(([contentType, media]) => (React.createElement("div", { key: contentType },
                        React.createElement("div", { className: "text-xs text-gray-500 dark:text-gray-400 mb-2" },
                            "Content-Type: ",
                            contentType),
                        media.example !== undefined && (React.createElement("pre", { className: "bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm" },
                            React.createElement("code", null, JSON.stringify(media.example, null, 2)))))))))))))),
            endpoint.security.length > 0 && (React.createElement("div", null,
                React.createElement("h3", { className: "text-sm font-semibold text-gray-900 dark:text-white mb-3" }, "Security"),
                React.createElement("div", { className: "space-y-2" }, endpoint.security.map((sec, index) => (React.createElement("div", { key: index, className: "text-sm" }, Object.entries(sec).map(([name, scopes]) => (React.createElement("div", { key: name, className: "flex items-center space-x-2" },
                    React.createElement("span", { className: "px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded text-xs font-medium" }, name),
                    scopes.length > 0 && (React.createElement("span", { className: "text-gray-600 dark:text-gray-400" },
                        "Scopes: ",
                        scopes.join(', '))))))))))))))));
};
const SchemaViewer = ({ name, schema }) => {
    const [isExpanded, setIsExpanded] = useState(false);
    return (React.createElement("div", { className: "border border-gray-200 dark:border-gray-700 rounded-lg" },
        React.createElement("button", { onClick: () => setIsExpanded(!isExpanded), className: "w-full px-4 py-3 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors" },
            React.createElement("div", { className: "flex items-center space-x-3" },
                React.createElement("span", { className: "font-mono font-semibold text-gray-900 dark:text-white" }, name),
                schema.description && (React.createElement("span", { className: "text-sm text-gray-600 dark:text-gray-400" }, schema.description))),
            React.createElement("span", { className: "text-gray-400" }, isExpanded ? '▲' : '▼')),
        isExpanded && (React.createElement("div", { className: "px-4 py-3 bg-gray-50 dark:bg-gray-900/50 border-t border-gray-200 dark:border-gray-700" },
            React.createElement("pre", { className: "bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm" },
                React.createElement("code", null, JSON.stringify(schema, null, 2)))))));
};
function getStatusColor(status) {
    if (status >= 200 && status < 300)
        return 'text-green-600 dark:text-green-400';
    if (status >= 300 && status < 400)
        return 'text-blue-600 dark:text-blue-400';
    if (status >= 400 && status < 500)
        return 'text-orange-600 dark:text-orange-400';
    return 'text-red-600 dark:text-red-400';
}
export default APIDocumentation;
//# sourceMappingURL=APIDocumentation.js.map