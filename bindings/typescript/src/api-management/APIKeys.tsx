/**
 * CADDY API Keys Management
 *
 * API key generation, management, and usage tracking interface.
 */

import React, { useState, useEffect } from 'react';
import { APIKey, APIKeyUsage } from './types';

interface APIKeysProps {
  userId?: string;
  onKeyCreate?: (name: string, scopes: string[]) => Promise<{ key: APIKey; secret: string }>;
  onKeyRevoke?: (keyId: string) => Promise<void>;
  onKeyRotate?: (keyId: string) => Promise<{ key: APIKey; secret: string }>;
}

export const APIKeys: React.FC<APIKeysProps> = ({
  userId = 'current',
  onKeyCreate,
  onKeyRevoke,
  onKeyRotate,
}) => {
  const [keys, setKeys] = useState<APIKey[]>([]);
  const [selectedKey, setSelectedKey] = useState<APIKey | null>(null);
  const [keyUsage, setKeyUsage] = useState<Record<string, APIKeyUsage>>({});
  const [isCreating, setIsCreating] = useState(false);
  const [newKeySecret, setNewKeySecret] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterEnvironment, setFilterEnvironment] = useState<'all' | 'development' | 'staging' | 'production'>('all');

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
      // Simulate loading keys
      const mockKeys: APIKey[] = [
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
    } catch (error) {
      console.error('Failed to load API keys:', error);
    }
  };

  const loadKeyUsage = async (keyId: string) => {
    try {
      // Simulate loading usage data
      const mockUsage: APIKeyUsage = {
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
    } catch (error) {
      console.error('Failed to load key usage:', error);
    }
  };

  const handleCreate = async (name: string, scopes: string[], environment: APIKey['environment']) => {
    try {
      let secret: string;
      if (onKeyCreate) {
        const result = await onKeyCreate(name, scopes);
        secret = result.secret;
        setKeys([...keys, result.key]);
      } else {
        // Simulate creation
        secret = `pk_${environment === 'production' ? 'live' : 'test'}_${generateRandomString(32)}`;
        const newKey: APIKey = {
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
    } catch (error) {
      console.error('Failed to create API key:', error);
    }
  };

  const handleRevoke = async (keyId: string) => {
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
    } catch (error) {
      console.error('Failed to revoke API key:', error);
    }
  };

  const handleRotate = async (keyId: string) => {
    if (!confirm('This will generate a new API key. The old key will stop working immediately.')) {
      return;
    }

    try {
      let secret: string;
      if (onKeyRotate) {
        const result = await onKeyRotate(keyId);
        secret = result.secret;
        setKeys(keys.map((k) => (k.id === keyId ? result.key : k)));
      } else {
        const key = keys.find((k) => k.id === keyId);
        if (!key) return;

        secret = `pk_${key.environment === 'production' ? 'live' : 'test'}_${generateRandomString(32)}`;
        setKeys(
          keys.map((k) =>
            k.id === keyId
              ? { ...k, key: secret.substring(0, 8) + '*'.repeat(22) }
              : k
          )
        );
      }

      setNewKeySecret(secret);
    } catch (error) {
      console.error('Failed to rotate API key:', error);
    }
  };

  const filteredKeys = keys.filter((key) => {
    const matchesSearch =
      searchQuery === '' ||
      key.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      key.key.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesEnvironment = filterEnvironment === 'all' || key.environment === filterEnvironment;

    return matchesSearch && matchesEnvironment;
  });

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">API Keys</h1>
          <button
            onClick={() => setIsCreating(true)}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            + Generate API Key
          </button>
        </div>

        {/* Filters */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 mb-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <input
              type="text"
              placeholder="Search API keys..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
            />

            <select
              value={filterEnvironment}
              onChange={(e) => setFilterEnvironment(e.target.value as any)}
              className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
            >
              <option value="all">All Environments</option>
              <option value="development">Development</option>
              <option value="staging">Staging</option>
              <option value="production">Production</option>
            </select>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Keys List */}
          <div className="lg:col-span-2 space-y-4">
            {filteredKeys.map((key) => (
              <div
                key={key.id}
                onClick={() => setSelectedKey(key)}
                className={`bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 cursor-pointer transition-all ${
                  selectedKey?.id === key.id ? 'ring-2 ring-blue-500' : 'hover:border-blue-300'
                }`}
              >
                <div className="flex items-start justify-between mb-4">
                  <div className="flex-1">
                    <div className="flex items-center space-x-3 mb-2">
                      <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                        {key.name}
                      </h3>
                      <span
                        className={`px-2 py-1 rounded text-xs font-semibold ${
                          key.environment === 'production'
                            ? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                            : key.environment === 'staging'
                            ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
                            : 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                        }`}
                      >
                        {key.environment}
                      </span>
                      {!key.active && (
                        <span className="px-2 py-1 rounded text-xs font-semibold bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200">
                          Revoked
                        </span>
                      )}
                    </div>
                    <div className="flex items-center space-x-2 mb-3">
                      <code className="text-sm font-mono text-gray-600 dark:text-gray-400">
                        {key.key}
                      </code>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          copyToClipboard(key.key);
                        }}
                        className="text-blue-600 hover:text-blue-800 dark:text-blue-400"
                        title="Copy to clipboard"
                      >
                        📋
                      </button>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {key.scopes.slice(0, 3).map((scope) => (
                        <span
                          key={scope}
                          className="px-2 py-1 rounded text-xs bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300"
                        >
                          {scope}
                        </span>
                      ))}
                      {key.scopes.length > 3 && (
                        <span className="px-2 py-1 rounded text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400">
                          +{key.scopes.length - 3} more
                        </span>
                      )}
                    </div>
                  </div>
                  {key.active && (
                    <div className="flex flex-col space-y-2">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRotate(key.id);
                        }}
                        className="text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400"
                      >
                        Rotate
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRevoke(key.id);
                        }}
                        className="text-sm text-red-600 hover:text-red-800 dark:text-red-400"
                      >
                        Revoke
                      </button>
                    </div>
                  )}
                </div>

                <div className="flex items-center justify-between text-sm text-gray-500 dark:text-gray-400">
                  <div>Created {formatDate(key.createdAt)}</div>
                  {key.lastUsedAt && <div>Last used {formatTimeAgo(key.lastUsedAt)}</div>}
                </div>
              </div>
            ))}

            {filteredKeys.length === 0 && (
              <div className="text-center py-12 text-gray-500 dark:text-gray-400">
                No API keys found
              </div>
            )}
          </div>

          {/* Key Details/Usage */}
          <div className="lg:col-span-1">
            {selectedKey ? (
              <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 sticky top-4">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                  Usage Statistics
                </h3>

                {keyUsage[selectedKey.id] ? (
                  <div className="space-y-4">
                    <div>
                      <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">
                        Total Requests
                      </div>
                      <div className="text-2xl font-bold text-gray-900 dark:text-white">
                        {keyUsage[selectedKey.id].totalRequests.toLocaleString()}
                      </div>
                    </div>

                    <div>
                      <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">
                        Success Rate
                      </div>
                      <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                        {(
                          (keyUsage[selectedKey.id].successfulRequests /
                            keyUsage[selectedKey.id].totalRequests) *
                          100
                        ).toFixed(1)}
                        %
                      </div>
                    </div>

                    <div>
                      <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">
                        Avg Response Time
                      </div>
                      <div className="text-2xl font-bold text-gray-900 dark:text-white">
                        {keyUsage[selectedKey.id].averageResponseTime}ms
                      </div>
                    </div>

                    <div>
                      <div className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                        Top Endpoints
                      </div>
                      <div className="space-y-2">
                        {keyUsage[selectedKey.id].topEndpoints.map((endpoint) => (
                          <div
                            key={endpoint.endpoint}
                            className="flex items-center justify-between text-sm"
                          >
                            <code className="text-xs text-gray-600 dark:text-gray-400">
                              {endpoint.endpoint}
                            </code>
                            <span className="text-gray-900 dark:text-white font-medium">
                              {endpoint.count.toLocaleString()}
                            </span>
                          </div>
                        ))}
                      </div>
                    </div>

                    <div>
                      <div className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                        Scopes
                      </div>
                      <div className="flex flex-wrap gap-2">
                        {selectedKey.scopes.map((scope) => (
                          <span
                            key={scope}
                            className="px-2 py-1 rounded text-xs bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300"
                          >
                            {scope}
                          </span>
                        ))}
                      </div>
                    </div>
                  </div>
                ) : (
                  <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                    Loading usage data...
                  </div>
                )}
              </div>
            ) : (
              <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 text-center text-gray-500 dark:text-gray-400">
                Select an API key to view details
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Create Key Modal */}
      {isCreating && (
        <CreateKeyModal
          onClose={() => setIsCreating(false)}
          onCreate={handleCreate}
        />
      )}

      {/* New Key Secret Modal */}
      {newKeySecret && (
        <KeySecretModal secret={newKeySecret} onClose={() => setNewKeySecret(null)} />
      )}
    </div>
  );
};

// Helper Components

interface CreateKeyModalProps {
  onClose: () => void;
  onCreate: (name: string, scopes: string[], environment: APIKey['environment']) => void;
}

const CreateKeyModal: React.FC<CreateKeyModalProps> = ({ onClose, onCreate }) => {
  const [name, setName] = useState('');
  const [environment, setEnvironment] = useState<APIKey['environment']>('development');
  const [scopes, setScopes] = useState<string[]>([]);
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

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onCreate(name, scopes, environment);
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Generate API Key
        </h2>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Key Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
              placeholder="Production API Key"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Environment
            </label>
            <select
              value={environment}
              onChange={(e) => setEnvironment(e.target.value as APIKey['environment'])}
              className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
            >
              <option value="development">Development</option>
              <option value="staging">Staging</option>
              <option value="production">Production</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Scopes
            </label>
            <div className="space-y-2 mb-2">
              {availableScopes.map((scope) => (
                <label key={scope} className="flex items-center">
                  <input
                    type="checkbox"
                    checked={scopes.includes(scope)}
                    onChange={(e) => {
                      if (e.target.checked) {
                        setScopes([...scopes, scope]);
                      } else {
                        setScopes(scopes.filter((s) => s !== scope));
                      }
                    }}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <span className="ml-2 text-sm text-gray-700 dark:text-gray-300">{scope}</span>
                </label>
              ))}
            </div>
          </div>

          <div className="flex justify-end space-x-4 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              disabled={scopes.length === 0}
            >
              Generate Key
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

interface KeySecretModalProps {
  secret: string;
  onClose: () => void;
}

const KeySecretModal: React.FC<KeySecretModalProps> = ({ secret, onClose }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(secret);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Your API Key
        </h2>

        <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 text-yellow-800 dark:text-yellow-200 px-4 py-3 rounded mb-4">
          <p className="text-sm">
            Make sure to copy your API key now. You won't be able to see it again!
          </p>
        </div>

        <div className="bg-gray-900 text-gray-100 p-4 rounded-lg mb-4 break-all font-mono text-sm">
          {secret}
        </div>

        <div className="flex justify-end space-x-4">
          <button
            onClick={handleCopy}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            {copied ? 'Copied!' : 'Copy to Clipboard'}
          </button>
          <button
            onClick={onClose}
            className="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
};

// Helper Functions

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString();
}

function formatTimeAgo(timestamp: number): string {
  const seconds = Math.floor((Date.now() - timestamp) / 1000);

  if (seconds < 60) return 'just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

function generateRandomString(length: number): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

export default APIKeys;
