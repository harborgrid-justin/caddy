/**
 * CADDY API Explorer
 *
 * Interactive API explorer and playground for testing endpoints
 * with request builder, response visualization, and code generation.
 */

import React, { useState, useEffect } from 'react';
import {
  APIEndpoint,
  HTTPMethod,
  APIParameter,
  APITestRequest,
  APITestResponse,
  AuthConfig,
  CodeLanguage,
} from './types';

interface APIExplorerProps {
  endpoints?: APIEndpoint[];
  onExecuteRequest?: (request: APITestRequest) => Promise<APITestResponse>;
  enableCodeGen?: boolean;
  defaultAuth?: AuthConfig;
}

export const APIExplorer: React.FC<APIExplorerProps> = ({
  endpoints = [],
  onExecuteRequest,
  enableCodeGen = true,
  defaultAuth,
}) => {
  const [selectedEndpoint, setSelectedEndpoint] = useState<APIEndpoint | null>(null);
  const [parameters, setParameters] = useState<Record<string, unknown>>({});
  const [headers, setHeaders] = useState<Record<string, string>>({});
  const [body, setBody] = useState<string>('');
  const [authConfig, setAuthConfig] = useState<AuthConfig>(
    defaultAuth || { type: 'none', credentials: {} }
  );
  const [response, setResponse] = useState<APITestResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'params' | 'headers' | 'body' | 'auth'>('params');
  const [responseTab, setResponseTab] = useState<'body' | 'headers' | 'code'>('body');
  const [codeLanguage, setCodeLanguage] = useState<CodeLanguage>('typescript');
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    if (selectedEndpoint) {
      initializeParameters();
    }
  }, [selectedEndpoint]);

  const initializeParameters = () => {
    const params: Record<string, unknown> = {};
    selectedEndpoint?.parameters.forEach((param) => {
      if (param.example !== undefined) {
        params[param.name] = param.example;
      }
    });
    setParameters(params);
  };

  const handleExecute = async () => {
    if (!selectedEndpoint) return;

    setIsLoading(true);
    setError(null);

    try {
      const request: APITestRequest = {
        endpoint: selectedEndpoint,
        parameters,
        headers,
        body: body ? JSON.parse(body) : undefined,
        auth: authConfig,
      };

      let result: APITestResponse;
      if (onExecuteRequest) {
        result = await onExecuteRequest(request);
      } else {
        // Simulate request
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
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Request failed');
    } finally {
      setIsLoading(false);
    }
  };

  const generateCode = (): string => {
    if (!selectedEndpoint) return '';

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

  const buildURL = (): string => {
    if (!selectedEndpoint) return '';
    let url = selectedEndpoint.path;

    // Replace path parameters
    selectedEndpoint.parameters
      .filter((p) => p.in === 'path')
      .forEach((param) => {
        const value = parameters[param.name];
        url = url.replace(`{${param.name}}`, String(value || ''));
      });

    // Add query parameters
    const queryParams = selectedEndpoint.parameters
      .filter((p) => p.in === 'query' && parameters[p.name] !== undefined)
      .map((param) => `${param.name}=${encodeURIComponent(String(parameters[param.name]))}`)
      .join('&');

    return queryParams ? `${url}?${queryParams}` : url;
  };

  const generateTypeScriptCode = (method: HTTPMethod, url: string): string => {
    return `// TypeScript with fetch
const response = await fetch('${url}', {
  method: '${method}',
  headers: ${JSON.stringify(headers, null, 2)},
  ${body ? `body: JSON.stringify(${body}),` : ''}
});

const data = await response.json();
console.log(data);`;
  };

  const generateJavaScriptCode = (method: HTTPMethod, url: string): string => {
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

  const generatePythonCode = (method: HTTPMethod, url: string): string => {
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

  const generateCurlCode = (method: HTTPMethod, url: string): string => {
    let cmd = `curl -X ${method} '${url}'`;
    Object.entries(headers).forEach(([key, value]) => {
      cmd += ` \\\n  -H '${key}: ${value}'`;
    });
    if (body) {
      cmd += ` \\\n  -d '${body}'`;
    }
    return cmd;
  };

  const filteredEndpoints = endpoints.filter(
    (endpoint) =>
      endpoint.path.toLowerCase().includes(searchQuery.toLowerCase()) ||
      endpoint.summary.toLowerCase().includes(searchQuery.toLowerCase()) ||
      endpoint.tags.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  const getMethodColor = (method: HTTPMethod): string => {
    const colors: Record<HTTPMethod, string> = {
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

  const getStatusColor = (status: number): string => {
    if (status >= 200 && status < 300) return 'text-green-600 dark:text-green-400';
    if (status >= 300 && status < 400) return 'text-blue-600 dark:text-blue-400';
    if (status >= 400 && status < 500) return 'text-orange-600 dark:text-orange-400';
    return 'text-red-600 dark:text-red-400';
  };

  return (
    <div className="flex h-screen bg-gray-50 dark:bg-gray-900">
      {/* Sidebar - Endpoint List */}
      <div className="w-80 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col">
        <div className="p-4 border-b border-gray-200 dark:border-gray-700">
          <input
            type="text"
            placeholder="Search endpoints..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
          />
        </div>
        <div className="flex-1 overflow-y-auto">
          {filteredEndpoints.map((endpoint) => (
            <button
              key={endpoint.id}
              onClick={() => setSelectedEndpoint(endpoint)}
              className={`w-full text-left px-4 py-3 border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors ${
                selectedEndpoint?.id === endpoint.id ? 'bg-blue-50 dark:bg-blue-900/20' : ''
              }`}
            >
              <div className="flex items-center space-x-2 mb-1">
                <span className={`px-2 py-0.5 rounded text-xs font-semibold ${getMethodColor(endpoint.method)}`}>
                  {endpoint.method}
                </span>
                {endpoint.deprecated && (
                  <span className="px-2 py-0.5 rounded text-xs font-semibold bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200">
                    DEPRECATED
                  </span>
                )}
              </div>
              <div className="text-sm font-mono text-gray-900 dark:text-white mb-1">
                {endpoint.path}
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400 truncate">
                {endpoint.summary}
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {selectedEndpoint ? (
          <>
            {/* Endpoint Header */}
            <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-6">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-3 mb-2">
                    <span className={`px-3 py-1 rounded font-semibold ${getMethodColor(selectedEndpoint.method)}`}>
                      {selectedEndpoint.method}
                    </span>
                    <code className="text-lg font-mono text-gray-900 dark:text-white">
                      {selectedEndpoint.path}
                    </code>
                  </div>
                  <p className="text-gray-600 dark:text-gray-400">{selectedEndpoint.description}</p>
                  {selectedEndpoint.tags.length > 0 && (
                    <div className="flex items-center space-x-2 mt-2">
                      {selectedEndpoint.tags.map((tag) => (
                        <span
                          key={tag}
                          className="px-2 py-1 rounded text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
                <button
                  onClick={handleExecute}
                  disabled={isLoading}
                  className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  {isLoading ? 'Sending...' : 'Send Request'}
                </button>
              </div>
            </div>

            <div className="flex-1 flex overflow-hidden">
              {/* Request Panel */}
              <div className="flex-1 flex flex-col overflow-hidden">
                <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
                  <div className="flex space-x-1 px-6 pt-4">
                    {[
                      { id: 'params', label: 'Parameters' },
                      { id: 'headers', label: 'Headers' },
                      { id: 'body', label: 'Body' },
                      { id: 'auth', label: 'Auth' },
                    ].map((tab) => (
                      <button
                        key={tab.id}
                        onClick={() => setActiveTab(tab.id as any)}
                        className={`px-4 py-2 border-b-2 transition-colors ${
                          activeTab === tab.id
                            ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                            : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                        }`}
                      >
                        {tab.label}
                      </button>
                    ))}
                  </div>
                </div>

                <div className="flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-900 p-6">
                  {activeTab === 'params' && (
                    <ParametersPanel
                      parameters={selectedEndpoint.parameters}
                      values={parameters}
                      onChange={setParameters}
                    />
                  )}
                  {activeTab === 'headers' && (
                    <HeadersPanel headers={headers} onChange={setHeaders} />
                  )}
                  {activeTab === 'body' && (
                    <BodyPanel body={body} onChange={setBody} />
                  )}
                  {activeTab === 'auth' && (
                    <AuthPanel config={authConfig} onChange={setAuthConfig} />
                  )}
                </div>
              </div>

              {/* Response Panel */}
              <div className="w-1/2 border-l border-gray-200 dark:border-gray-700 flex flex-col bg-white dark:bg-gray-800">
                <div className="border-b border-gray-200 dark:border-gray-700">
                  <div className="flex items-center justify-between px-6 py-3">
                    <h3 className="font-semibold text-gray-900 dark:text-white">Response</h3>
                    {response && (
                      <div className="flex items-center space-x-4 text-sm">
                        <span className={`font-semibold ${getStatusColor(response.status)}`}>
                          {response.status} {response.statusText}
                        </span>
                        <span className="text-gray-500 dark:text-gray-400">
                          {response.duration}ms
                        </span>
                        <span className="text-gray-500 dark:text-gray-400">
                          {(response.size / 1024).toFixed(2)} KB
                        </span>
                      </div>
                    )}
                  </div>
                  <div className="flex space-x-1 px-6">
                    {[
                      { id: 'body', label: 'Body' },
                      { id: 'headers', label: 'Headers' },
                      { id: 'code', label: 'Code' },
                    ].map((tab) => (
                      <button
                        key={tab.id}
                        onClick={() => setResponseTab(tab.id as any)}
                        className={`px-4 py-2 border-b-2 transition-colors ${
                          responseTab === tab.id
                            ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                            : 'border-transparent text-gray-600 dark:text-gray-400'
                        }`}
                      >
                        {tab.label}
                      </button>
                    ))}
                  </div>
                </div>

                <div className="flex-1 overflow-y-auto p-6">
                  {error && (
                    <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 px-4 py-3 rounded">
                      {error}
                    </div>
                  )}

                  {!response && !error && (
                    <div className="text-center text-gray-500 dark:text-gray-400 py-12">
                      Send a request to see the response
                    </div>
                  )}

                  {response && responseTab === 'body' && (
                    <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                      <code>{JSON.stringify(response.body, null, 2)}</code>
                    </pre>
                  )}

                  {response && responseTab === 'headers' && (
                    <div className="space-y-2">
                      {Object.entries(response.headers).map(([key, value]) => (
                        <div key={key} className="flex items-center space-x-2 text-sm">
                          <span className="font-semibold text-gray-700 dark:text-gray-300">
                            {key}:
                          </span>
                          <span className="text-gray-600 dark:text-gray-400">{value}</span>
                        </div>
                      ))}
                    </div>
                  )}

                  {responseTab === 'code' && (
                    <div>
                      <div className="mb-4">
                        <select
                          value={codeLanguage}
                          onChange={(e) => setCodeLanguage(e.target.value as CodeLanguage)}
                          className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
                        >
                          <option value="typescript">TypeScript</option>
                          <option value="javascript">JavaScript</option>
                          <option value="python">Python</option>
                          <option value="curl">cURL</option>
                        </select>
                      </div>
                      <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                        <code>{generateCode()}</code>
                      </pre>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </>
        ) : (
          <div className="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
            Select an endpoint to get started
          </div>
        )}
      </div>
    </div>
  );
};

// Helper Components

interface ParametersPanelProps {
  parameters: APIParameter[];
  values: Record<string, unknown>;
  onChange: (values: Record<string, unknown>) => void;
}

const ParametersPanel: React.FC<ParametersPanelProps> = ({ parameters, values, onChange }) => {
  if (parameters.length === 0) {
    return <div className="text-gray-500 dark:text-gray-400">No parameters</div>;
  }

  return (
    <div className="space-y-4">
      {parameters.map((param) => (
        <div key={param.name}>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            {param.name}
            {param.required && <span className="text-red-500 ml-1">*</span>}
            <span className="ml-2 text-xs text-gray-500">({param.in})</span>
          </label>
          <input
            type="text"
            value={String(values[param.name] || '')}
            onChange={(e) => onChange({ ...values, [param.name]: e.target.value })}
            placeholder={param.description}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
          />
          {param.description && (
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">{param.description}</p>
          )}
        </div>
      ))}
    </div>
  );
};

interface HeadersPanelProps {
  headers: Record<string, string>;
  onChange: (headers: Record<string, string>) => void;
}

const HeadersPanel: React.FC<HeadersPanelProps> = ({ headers, onChange }) => {
  const addHeader = () => {
    onChange({ ...headers, '': '' });
  };

  return (
    <div className="space-y-2">
      {Object.entries(headers).map(([key, value], index) => (
        <div key={index} className="flex space-x-2">
          <input
            type="text"
            value={key}
            onChange={(e) => {
              const newHeaders = { ...headers };
              delete newHeaders[key];
              newHeaders[e.target.value] = value;
              onChange(newHeaders);
            }}
            placeholder="Header name"
            className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
          />
          <input
            type="text"
            value={value}
            onChange={(e) => onChange({ ...headers, [key]: e.target.value })}
            placeholder="Header value"
            className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
          />
          <button
            onClick={() => {
              const newHeaders = { ...headers };
              delete newHeaders[key];
              onChange(newHeaders);
            }}
            className="px-3 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg"
          >
            Remove
          </button>
        </div>
      ))}
      <button
        onClick={addHeader}
        className="px-4 py-2 text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-lg"
      >
        + Add Header
      </button>
    </div>
  );
};

interface BodyPanelProps {
  body: string;
  onChange: (body: string) => void;
}

const BodyPanel: React.FC<BodyPanelProps> = ({ body, onChange }) => {
  return (
    <textarea
      value={body}
      onChange={(e) => onChange(e.target.value)}
      placeholder='{"key": "value"}'
      className="w-full h-64 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg font-mono dark:bg-gray-700 dark:text-white"
    />
  );
};

interface AuthPanelProps {
  config: AuthConfig;
  onChange: (config: AuthConfig) => void;
}

const AuthPanel: React.FC<AuthPanelProps> = ({ config, onChange }) => {
  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Auth Type
        </label>
        <select
          value={config.type}
          onChange={(e) => onChange({ ...config, type: e.target.value as any })}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
        >
          <option value="none">No Auth</option>
          <option value="api_key">API Key</option>
          <option value="bearer">Bearer Token</option>
          <option value="basic">Basic Auth</option>
          <option value="oauth2">OAuth 2.0</option>
        </select>
      </div>

      {config.type === 'api_key' && (
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            API Key
          </label>
          <input
            type="text"
            value={config.credentials.apiKey || ''}
            onChange={(e) =>
              onChange({ ...config, credentials: { ...config.credentials, apiKey: e.target.value } })
            }
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
          />
        </div>
      )}

      {config.type === 'bearer' && (
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Token
          </label>
          <input
            type="text"
            value={config.credentials.token || ''}
            onChange={(e) =>
              onChange({ ...config, credentials: { ...config.credentials, token: e.target.value } })
            }
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
          />
        </div>
      )}

      {config.type === 'basic' && (
        <>
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Username
            </label>
            <input
              type="text"
              value={config.credentials.username || ''}
              onChange={(e) =>
                onChange({
                  ...config,
                  credentials: { ...config.credentials, username: e.target.value },
                })
              }
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Password
            </label>
            <input
              type="password"
              value={config.credentials.password || ''}
              onChange={(e) =>
                onChange({
                  ...config,
                  credentials: { ...config.credentials, password: e.target.value },
                })
              }
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
            />
          </div>
        </>
      )}
    </div>
  );
};

export default APIExplorer;
