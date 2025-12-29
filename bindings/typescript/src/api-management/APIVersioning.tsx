/**
 * CADDY API Versioning
 *
 * API version management with changelog and migration guides.
 */

import React, { useState, useEffect } from 'react';
import { APIVersion, VersionStatus, ChangelogEntry, VersionMigration } from './types';

interface APIVersioningProps {
  onVersionCreate?: (version: Partial<APIVersion>) => Promise<void>;
  onVersionUpdate?: (version: string, update: Partial<APIVersion>) => Promise<void>;
}

export const APIVersioning: React.FC<APIVersioningProps> = ({
  onVersionCreate,
  onVersionUpdate,
}) => {
  const [versions, setVersions] = useState<APIVersion[]>([]);
  const [selectedVersion, setSelectedVersion] = useState<APIVersion | null>(null);
  const [migrations, setMigrations] = useState<VersionMigration[]>([]);
  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    loadVersions();
    loadMigrations();
  }, []);

  const loadVersions = async () => {
    try {
      const mockVersions: APIVersion[] = [
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
    } catch (error) {
      console.error('Failed to load versions:', error);
    }
  };

  const loadMigrations = async () => {
    try {
      const mockMigrations: VersionMigration[] = [
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
    } catch (error) {
      console.error('Failed to load migrations:', error);
    }
  };

  const handleCreateVersion = async (version: Partial<APIVersion>) => {
    try {
      if (onVersionCreate) {
        await onVersionCreate(version);
      }

      const newVersion: APIVersion = {
        ...version,
        releaseDate: Date.now(),
        endpoints: 0,
        breaking: false,
        changelog: [],
        metadata: {},
      } as APIVersion;

      setVersions([newVersion, ...versions]);
      setIsCreating(false);
    } catch (error) {
      console.error('Failed to create version:', error);
    }
  };

  const handleUpdateStatus = async (version: string, status: VersionStatus) => {
    try {
      if (onVersionUpdate) {
        await onVersionUpdate(version, { status });
      }

      setVersions(
        versions.map((v) => (v.version === version ? { ...v, status } : v))
      );
    } catch (error) {
      console.error('Failed to update version status:', error);
    }
  };

  const getStatusColor = (status: VersionStatus): string => {
    const colors: Record<VersionStatus, string> = {
      draft: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200',
      beta: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
      stable: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
      deprecated: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
      retired: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
    };
    return colors[status];
  };

  const getChangeTypeIcon = (type: ChangelogEntry['type']): string => {
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

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            API Versioning
          </h1>
          <button
            onClick={() => setIsCreating(true)}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            + Create Version
          </button>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Versions List */}
          <div className="lg:col-span-2 space-y-4">
            {versions.map((version) => (
              <div
                key={version.version}
                onClick={() => setSelectedVersion(version)}
                className={`bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 cursor-pointer transition-all ${
                  selectedVersion?.version === version.version
                    ? 'ring-2 ring-blue-500'
                    : 'hover:border-blue-300'
                }`}
              >
                <div className="flex items-start justify-between mb-4">
                  <div className="flex-1">
                    <div className="flex items-center space-x-3 mb-2">
                      <h3 className="text-xl font-bold text-gray-900 dark:text-white">
                        {version.version}
                      </h3>
                      <span className={`px-2 py-1 rounded text-xs font-semibold ${getStatusColor(version.status)}`}>
                        {version.status}
                      </span>
                      {version.breaking && (
                        <span className="px-2 py-1 rounded text-xs font-semibold bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200">
                          BREAKING
                        </span>
                      )}
                    </div>

                    <div className="grid grid-cols-2 gap-4 text-sm text-gray-600 dark:text-gray-400">
                      <div>
                        <span className="font-medium">Released:</span>{' '}
                        {new Date(version.releaseDate).toLocaleDateString()}
                      </div>
                      <div>
                        <span className="font-medium">Endpoints:</span> {version.endpoints}
                      </div>
                      {version.deprecationDate && (
                        <div>
                          <span className="font-medium">Deprecated:</span>{' '}
                          {new Date(version.deprecationDate).toLocaleDateString()}
                        </div>
                      )}
                      {version.sunsetDate && (
                        <div className="text-red-600 dark:text-red-400">
                          <span className="font-medium">Sunset:</span>{' '}
                          {new Date(version.sunsetDate).toLocaleDateString()}
                        </div>
                      )}
                    </div>
                  </div>

                  <div className="flex flex-col space-y-2">
                    {version.status !== 'stable' && version.status !== 'retired' && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleUpdateStatus(version.version, 'stable');
                        }}
                        className="text-sm text-green-600 hover:text-green-800 dark:text-green-400"
                      >
                        Mark Stable
                      </button>
                    )}
                    {version.status === 'stable' && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleUpdateStatus(version.version, 'deprecated');
                        }}
                        className="text-sm text-orange-600 hover:text-orange-800 dark:text-orange-400"
                      >
                        Deprecate
                      </button>
                    )}
                  </div>
                </div>

                {/* Changelog Preview */}
                <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                  <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    Recent Changes
                  </h4>
                  <div className="space-y-2">
                    {version.changelog.slice(0, 3).map((change, index) => (
                      <div key={index} className="flex items-start space-x-2 text-sm">
                        <span className="text-lg">{getChangeTypeIcon(change.type)}</span>
                        <div className="flex-1">
                          <span className="text-gray-900 dark:text-white">
                            {change.description}
                          </span>
                          {change.endpoint && (
                            <code className="ml-2 text-xs text-gray-600 dark:text-gray-400">
                              {change.endpoint}
                            </code>
                          )}
                        </div>
                      </div>
                    ))}
                    {version.changelog.length > 3 && (
                      <div className="text-xs text-gray-500 dark:text-gray-400">
                        +{version.changelog.length - 3} more changes
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}

            {versions.length === 0 && (
              <div className="text-center py-12 text-gray-500 dark:text-gray-400">
                No versions found
              </div>
            )}
          </div>

          {/* Version Details / Migration Guides */}
          <div className="lg:col-span-1">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 sticky top-4">
              <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                  Migration Guides
                </h2>
              </div>
              <div className="p-6 space-y-4 max-h-[600px] overflow-y-auto">
                {migrations.map((migration, index) => (
                  <div
                    key={index}
                    className="border border-gray-200 dark:border-gray-700 rounded-lg p-4"
                  >
                    <div className="flex items-center justify-between mb-3">
                      <div className="text-sm font-semibold text-gray-900 dark:text-white">
                        {migration.from} → {migration.to}
                      </div>
                      <span
                        className={`px-2 py-1 rounded text-xs font-semibold ${
                          migration.estimatedEffort === 'low'
                            ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                            : migration.estimatedEffort === 'medium'
                            ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
                            : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                        }`}
                      >
                        {migration.estimatedEffort} effort
                      </span>
                    </div>

                    <div className="bg-gray-50 dark:bg-gray-900 rounded p-3 mb-3">
                      <pre className="text-xs text-gray-900 dark:text-white whitespace-pre-wrap overflow-x-auto">
                        {migration.guide}
                      </pre>
                    </div>

                    {migration.automated && (
                      <div className="flex items-center space-x-2 text-xs text-green-600 dark:text-green-400">
                        <span>✅</span>
                        <span>Automated migration available</span>
                      </div>
                    )}
                  </div>
                ))}

                {migrations.length === 0 && (
                  <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                    No migration guides available
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Create Version Modal */}
        {isCreating && (
          <CreateVersionModal
            onClose={() => setIsCreating(false)}
            onCreate={handleCreateVersion}
          />
        )}
      </div>
    </div>
  );
};

// Helper Components

interface CreateVersionModalProps {
  onClose: () => void;
  onCreate: (version: Partial<APIVersion>) => void;
}

const CreateVersionModal: React.FC<CreateVersionModalProps> = ({ onClose, onCreate }) => {
  const [version, setVersion] = useState('');
  const [status, setStatus] = useState<VersionStatus>('draft');
  const [breaking, setBreaking] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onCreate({ version, status, breaking });
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Create New Version
        </h2>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Version Number
            </label>
            <input
              type="text"
              value={version}
              onChange={(e) => setVersion(e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
              placeholder="v2.1.0"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Status
            </label>
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value as VersionStatus)}
              className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white"
            >
              <option value="draft">Draft</option>
              <option value="beta">Beta</option>
              <option value="stable">Stable</option>
              <option value="deprecated">Deprecated</option>
              <option value="retired">Retired</option>
            </select>
          </div>

          <div className="flex items-center">
            <input
              type="checkbox"
              id="breaking"
              checked={breaking}
              onChange={(e) => setBreaking(e.target.checked)}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            />
            <label htmlFor="breaking" className="ml-2 block text-sm text-gray-700 dark:text-gray-300">
              Contains breaking changes
            </label>
          </div>

          <div className="flex justify-end space-x-4 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
            >
              Create
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default APIVersioning;
