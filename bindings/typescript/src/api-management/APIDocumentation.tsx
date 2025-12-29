/**
 * CADDY API Documentation
 *
 * Auto-generated API documentation with OpenAPI/Swagger support,
 * interactive examples, and schema visualization.
 */

import React, { useState, useEffect } from 'react';
import {
  OpenAPISpec,
  APIEndpoint,
  HTTPMethod,
  JSONSchema,
  PathItem,
  Operation,
} from './types';

interface APIDocumentationProps {
  spec?: OpenAPISpec;
  endpoints?: APIEndpoint[];
  onTryEndpoint?: (endpoint: APIEndpoint) => void;
  showTryItOut?: boolean;
}

export const APIDocumentation: React.FC<APIDocumentationProps> = ({
  spec,
  endpoints = [],
  onTryEndpoint,
  showTryItOut = true,
}) => {
  const [selectedTag, setSelectedTag] = useState<string>('all');
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set());
  const [searchQuery, setSearchQuery] = useState('');
  const [viewMode, setViewMode] = useState<'grouped' | 'list'>('grouped');

  const tags = Array.from(
    new Set(endpoints.flatMap((e) => e.tags))
  );

  const filteredEndpoints = endpoints.filter((endpoint) => {
    const matchesSearch =
      searchQuery === '' ||
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
      }, {} as Record<string, APIEndpoint[]>)
    : { all: filteredEndpoints };

  const toggleSection = (id: string) => {
    const newExpanded = new Set(expandedSections);
    if (newExpanded.has(id)) {
      newExpanded.delete(id);
    } else {
      newExpanded.add(id);
    }
    setExpandedSections(newExpanded);
  };

  const getMethodColor = (method: HTTPMethod): string => {
    const colors: Record<HTTPMethod, string> = {
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

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            {spec?.info.title || 'API Documentation'}
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mb-4">
            {spec?.info.description || 'Comprehensive API documentation'}
          </p>
          {spec?.info && (
            <div className="flex items-center space-x-6 text-sm text-gray-600 dark:text-gray-400">
              <div>Version: {spec.info.version}</div>
              {spec.info.contact?.email && (
                <div>Contact: {spec.info.contact.email}</div>
              )}
              {spec.info.license && (
                <div>License: {spec.info.license.name}</div>
              )}
            </div>
          )}
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Filters and Search */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 mb-6">
          <div className="flex items-center justify-between space-x-4">
            <div className="flex items-center space-x-4 flex-1">
              <input
                type="text"
                placeholder="Search endpoints..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="flex-1 max-w-md px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
              />

              <select
                value={selectedTag}
                onChange={(e) => setSelectedTag(e.target.value)}
                className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
              >
                <option value="all">All Tags</option>
                {tags.map((tag) => (
                  <option key={tag} value={tag}>
                    {tag}
                  </option>
                ))}
              </select>
            </div>

            <div className="flex items-center space-x-2 bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
              <button
                onClick={() => setViewMode('grouped')}
                className={`px-3 py-1 rounded ${
                  viewMode === 'grouped'
                    ? 'bg-white dark:bg-gray-600 shadow'
                    : 'text-gray-600 dark:text-gray-400'
                }`}
              >
                Grouped
              </button>
              <button
                onClick={() => setViewMode('list')}
                className={`px-3 py-1 rounded ${
                  viewMode === 'list'
                    ? 'bg-white dark:bg-gray-600 shadow'
                    : 'text-gray-600 dark:text-gray-400'
                }`}
              >
                List
              </button>
            </div>
          </div>
        </div>

        {/* Endpoints Documentation */}
        <div className="space-y-6">
          {Object.entries(groupedEndpoints).map(([tag, tagEndpoints]) => (
            <div key={tag} className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
              {viewMode === 'grouped' && (
                <div className="bg-gray-50 dark:bg-gray-700 px-6 py-3 border-b border-gray-200 dark:border-gray-600">
                  <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                    {tag.charAt(0).toUpperCase() + tag.slice(1)}
                  </h2>
                </div>
              )}

              <div className="divide-y divide-gray-200 dark:divide-gray-700">
                {tagEndpoints.map((endpoint) => (
                  <EndpointDocumentation
                    key={endpoint.id}
                    endpoint={endpoint}
                    isExpanded={expandedSections.has(endpoint.id)}
                    onToggle={() => toggleSection(endpoint.id)}
                    onTryIt={onTryEndpoint ? () => onTryEndpoint(endpoint) : undefined}
                    showTryItOut={showTryItOut}
                    getMethodColor={getMethodColor}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>

        {filteredEndpoints.length === 0 && (
          <div className="text-center py-12 text-gray-500 dark:text-gray-400">
            No endpoints found matching your criteria
          </div>
        )}

        {/* Schemas Section */}
        {spec?.components?.schemas && (
          <div className="mt-8 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
            <div className="bg-gray-50 dark:bg-gray-700 px-6 py-3 border-b border-gray-200 dark:border-gray-600">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                Schemas
              </h2>
            </div>
            <div className="p-6 space-y-4">
              {Object.entries(spec.components.schemas).map(([name, schema]) => (
                <SchemaViewer key={name} name={name} schema={schema} />
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

// Helper Components

interface EndpointDocumentationProps {
  endpoint: APIEndpoint;
  isExpanded: boolean;
  onToggle: () => void;
  onTryIt?: () => void;
  showTryItOut: boolean;
  getMethodColor: (method: HTTPMethod) => string;
}

const EndpointDocumentation: React.FC<EndpointDocumentationProps> = ({
  endpoint,
  isExpanded,
  onToggle,
  onTryIt,
  showTryItOut,
  getMethodColor,
}) => {
  return (
    <div>
      {/* Endpoint Header */}
      <button
        onClick={onToggle}
        className="w-full px-6 py-4 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
      >
        <div className="flex items-center space-x-4">
          <span className={`px-3 py-1 rounded font-semibold text-sm ${getMethodColor(endpoint.method)}`}>
            {endpoint.method}
          </span>
          <code className="text-base font-mono text-gray-900 dark:text-white">
            {endpoint.path}
          </code>
          {endpoint.deprecated && (
            <span className="px-2 py-1 rounded text-xs font-semibold bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200">
              DEPRECATED
            </span>
          )}
        </div>
        <div className="flex items-center space-x-3">
          <span className="text-gray-600 dark:text-gray-400">{endpoint.summary}</span>
          <span className="text-gray-400">{isExpanded ? '▲' : '▼'}</span>
        </div>
      </button>

      {/* Endpoint Details */}
      {isExpanded && (
        <div className="px-6 py-4 bg-gray-50 dark:bg-gray-900/50 border-t border-gray-200 dark:border-gray-700">
          <div className="mb-6">
            <p className="text-gray-700 dark:text-gray-300">{endpoint.description}</p>
          </div>

          {/* Try It Out Button */}
          {showTryItOut && onTryIt && (
            <div className="mb-6">
              <button
                onClick={onTryIt}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Try it out
              </button>
            </div>
          )}

          {/* Parameters */}
          {endpoint.parameters.length > 0 && (
            <div className="mb-6">
              <h3 className="text-sm font-semibold text-gray-900 dark:text-white mb-3">
                Parameters
              </h3>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                  <thead className="bg-gray-100 dark:bg-gray-800">
                    <tr>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                        Name
                      </th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                        In
                      </th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                        Type
                      </th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                        Required
                      </th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                        Description
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                    {endpoint.parameters.map((param) => (
                      <tr key={param.name}>
                        <td className="px-4 py-2 text-sm font-mono text-gray-900 dark:text-white">
                          {param.name}
                        </td>
                        <td className="px-4 py-2 text-sm text-gray-600 dark:text-gray-400">
                          {param.in}
                        </td>
                        <td className="px-4 py-2 text-sm text-gray-600 dark:text-gray-400">
                          {param.schema.type || 'any'}
                        </td>
                        <td className="px-4 py-2 text-sm">
                          {param.required ? (
                            <span className="text-red-600 dark:text-red-400">Yes</span>
                          ) : (
                            <span className="text-gray-400">No</span>
                          )}
                        </td>
                        <td className="px-4 py-2 text-sm text-gray-600 dark:text-gray-400">
                          {param.description}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Request Body */}
          {endpoint.requestBody && (
            <div className="mb-6">
              <h3 className="text-sm font-semibold text-gray-900 dark:text-white mb-3">
                Request Body
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                {endpoint.requestBody.description}
              </p>
              {Object.entries(endpoint.requestBody.content).map(([contentType, media]) => (
                <div key={contentType} className="mb-4">
                  <div className="text-xs text-gray-500 dark:text-gray-400 mb-2">
                    Content-Type: {contentType}
                  </div>
                  <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code>{JSON.stringify(media.example || media.schema, null, 2)}</code>
                  </pre>
                </div>
              ))}
            </div>
          )}

          {/* Responses */}
          <div className="mb-6">
            <h3 className="text-sm font-semibold text-gray-900 dark:text-white mb-3">
              Responses
            </h3>
            <div className="space-y-3">
              {Object.entries(endpoint.responses).map(([statusCode, response]) => (
                <div key={statusCode} className="border border-gray-200 dark:border-gray-700 rounded-lg">
                  <div className="px-4 py-2 bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
                    <div className="flex items-center space-x-3">
                      <span className={`font-mono font-semibold ${getStatusColor(parseInt(statusCode))}`}>
                        {statusCode}
                      </span>
                      <span className="text-sm text-gray-600 dark:text-gray-400">
                        {response.description}
                      </span>
                    </div>
                  </div>
                  {response.content && (
                    <div className="p-4">
                      {Object.entries(response.content).map(([contentType, media]) => (
                        <div key={contentType}>
                          <div className="text-xs text-gray-500 dark:text-gray-400 mb-2">
                            Content-Type: {contentType}
                          </div>
                          {media.example !== undefined && (
                            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                              <code>{JSON.stringify(media.example, null, 2)}</code>
                            </pre>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Security */}
          {endpoint.security.length > 0 && (
            <div>
              <h3 className="text-sm font-semibold text-gray-900 dark:text-white mb-3">
                Security
              </h3>
              <div className="space-y-2">
                {endpoint.security.map((sec, index) => (
                  <div key={index} className="text-sm">
                    {Object.entries(sec).map(([name, scopes]) => (
                      <div key={name} className="flex items-center space-x-2">
                        <span className="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded text-xs font-medium">
                          {name}
                        </span>
                        {scopes.length > 0 && (
                          <span className="text-gray-600 dark:text-gray-400">
                            Scopes: {scopes.join(', ')}
                          </span>
                        )}
                      </div>
                    ))}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

interface SchemaViewerProps {
  name: string;
  schema: JSONSchema;
}

const SchemaViewer: React.FC<SchemaViewerProps> = ({ name, schema }) => {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full px-4 py-3 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
      >
        <div className="flex items-center space-x-3">
          <span className="font-mono font-semibold text-gray-900 dark:text-white">
            {name}
          </span>
          {schema.description && (
            <span className="text-sm text-gray-600 dark:text-gray-400">
              {schema.description}
            </span>
          )}
        </div>
        <span className="text-gray-400">{isExpanded ? '▲' : '▼'}</span>
      </button>

      {isExpanded && (
        <div className="px-4 py-3 bg-gray-50 dark:bg-gray-900/50 border-t border-gray-200 dark:border-gray-700">
          <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
            <code>{JSON.stringify(schema, null, 2)}</code>
          </pre>
        </div>
      )}
    </div>
  );
};

function getStatusColor(status: number): string {
  if (status >= 200 && status < 300) return 'text-green-600 dark:text-green-400';
  if (status >= 300 && status < 400) return 'text-blue-600 dark:text-blue-400';
  if (status >= 400 && status < 500) return 'text-orange-600 dark:text-orange-400';
  return 'text-red-600 dark:text-red-400';
}

export default APIDocumentation;
