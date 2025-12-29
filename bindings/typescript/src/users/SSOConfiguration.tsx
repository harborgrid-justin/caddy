/**
 * CADDY v0.4.0 - SSO Configuration Component
 *
 * Single Sign-On management with:
 * - SAML 2.0 configuration
 * - OAuth/OIDC setup
 * - LDAP integration
 * - Attribute mapping
 * - User provisioning rules
 * - Connection testing
 * - Multi-provider support
 */

import React, { useState, useCallback } from 'react';
import {
  SSOProvider,
  SSOProviderType,
  SAMLConfig,
  OAuthConfig,
  LDAPConfig,
  AttributeMapping,
  ProvisioningConfig,
} from './types';
import { useSSOProviders } from './UserHooks';

interface SSOConfigurationProps {
  onProviderCreate?: (provider: SSOProvider) => void;
  onProviderUpdate?: (provider: SSOProvider) => void;
  onProviderDelete?: (providerId: string) => void;
  className?: string;
}

export const SSOConfiguration: React.FC<SSOConfigurationProps> = ({
  onProviderCreate,
  onProviderUpdate,
  onProviderDelete,
  className = '',
}) => {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingProvider, setEditingProvider] = useState<Partial<SSOProvider> | null>(null);
  const [selectedType, setSelectedType] = useState<SSOProviderType>('saml');
  const [testing, setTesting] = useState<string | null>(null);

  const {
    providers,
    loading,
    createProvider,
    updateProvider,
    deleteProvider,
    testProvider,
  } = useSSOProviders();

  const handleCreateProvider = useCallback(async () => {
    if (!editingProvider) return;

    try {
      const provider = await createProvider(editingProvider);
      setShowCreateModal(false);
      setEditingProvider(null);
      onProviderCreate?.(provider);
    } catch (err) {
      console.error('Failed to create SSO provider:', err);
      alert('Failed to create SSO provider. Please check the configuration.');
    }
  }, [editingProvider, createProvider, onProviderCreate]);

  const handleUpdateProvider = useCallback(
    async (providerId: string, updates: Partial<SSOProvider>) => {
      try {
        const provider = await updateProvider(providerId, updates);
        setEditingProvider(null);
        onProviderUpdate?.(provider);
      } catch (err) {
        console.error('Failed to update SSO provider:', err);
        alert('Failed to update SSO provider.');
      }
    },
    [updateProvider, onProviderUpdate]
  );

  const handleDeleteProvider = useCallback(
    async (providerId: string) => {
      if (
        window.confirm(
          'Are you sure you want to delete this SSO provider? Users will no longer be able to sign in using this method.'
        )
      ) {
        try {
          await deleteProvider(providerId);
          onProviderDelete?.(providerId);
        } catch (err) {
          console.error('Failed to delete SSO provider:', err);
        }
      }
    },
    [deleteProvider, onProviderDelete]
  );

  const handleTestProvider = useCallback(
    async (providerId: string) => {
      setTesting(providerId);
      try {
        const result = await testProvider(providerId);
        if (result.success) {
          alert('Connection test successful!');
        } else {
          alert(`Connection test failed: ${result.message}`);
        }
      } catch (err) {
        console.error('Failed to test SSO provider:', err);
        alert('Connection test failed. Please check the configuration.');
      } finally {
        setTesting(null);
      }
    },
    [testProvider]
  );

  const renderSAMLConfig = () => (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium text-gray-700">Entity ID</label>
        <input
          type="text"
          value={(editingProvider?.config as SAMLConfig)?.entityId || ''}
          onChange={(e) =>
            setEditingProvider({
              ...editingProvider,
              config: { ...editingProvider?.config, entityId: e.target.value } as SAMLConfig,
            })
          }
          className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">SSO URL</label>
        <input
          type="url"
          value={(editingProvider?.config as SAMLConfig)?.ssoUrl || ''}
          onChange={(e) =>
            setEditingProvider({
              ...editingProvider,
              config: { ...editingProvider?.config, ssoUrl: e.target.value } as SAMLConfig,
            })
          }
          className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">Certificate</label>
        <textarea
          value={(editingProvider?.config as SAMLConfig)?.certificate || ''}
          onChange={(e) =>
            setEditingProvider({
              ...editingProvider,
              config: { ...editingProvider?.config, certificate: e.target.value } as SAMLConfig,
            })
          }
          rows={6}
          className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm font-mono text-xs"
          placeholder="-----BEGIN CERTIFICATE-----&#10;...&#10;-----END CERTIFICATE-----"
        />
      </div>
      <div className="flex items-center space-x-6">
        <div className="flex items-center">
          <input
            type="checkbox"
            checked={(editingProvider?.config as SAMLConfig)?.signRequests || false}
            onChange={(e) =>
              setEditingProvider({
                ...editingProvider,
                config: {
                  ...editingProvider?.config,
                  signRequests: e.target.checked,
                } as SAMLConfig,
              })
            }
            className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
          />
          <label className="ml-2 block text-sm text-gray-700">Sign Requests</label>
        </div>
        <div className="flex items-center">
          <input
            type="checkbox"
            checked={(editingProvider?.config as SAMLConfig)?.encryptAssertions || false}
            onChange={(e) =>
              setEditingProvider({
                ...editingProvider,
                config: {
                  ...editingProvider?.config,
                  encryptAssertions: e.target.checked,
                } as SAMLConfig,
              })
            }
            className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
          />
          <label className="ml-2 block text-sm text-gray-700">Encrypt Assertions</label>
        </div>
      </div>
    </div>
  );

  const renderOAuthConfig = () => (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700">Client ID</label>
          <input
            type="text"
            value={(editingProvider?.config as OAuthConfig)?.clientId || ''}
            onChange={(e) =>
              setEditingProvider({
                ...editingProvider,
                config: { ...editingProvider?.config, clientId: e.target.value } as OAuthConfig,
              })
            }
            className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700">Client Secret</label>
          <input
            type="password"
            value={(editingProvider?.config as OAuthConfig)?.clientSecret || ''}
            onChange={(e) =>
              setEditingProvider({
                ...editingProvider,
                config: {
                  ...editingProvider?.config,
                  clientSecret: e.target.value,
                } as OAuthConfig,
              })
            }
            className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          />
        </div>
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">Authorization URL</label>
        <input
          type="url"
          value={(editingProvider?.config as OAuthConfig)?.authorizationUrl || ''}
          onChange={(e) =>
            setEditingProvider({
              ...editingProvider,
              config: {
                ...editingProvider?.config,
                authorizationUrl: e.target.value,
              } as OAuthConfig,
            })
          }
          className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">Token URL</label>
        <input
          type="url"
          value={(editingProvider?.config as OAuthConfig)?.tokenUrl || ''}
          onChange={(e) =>
            setEditingProvider({
              ...editingProvider,
              config: { ...editingProvider?.config, tokenUrl: e.target.value } as OAuthConfig,
            })
          }
          className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">
          Scopes (comma-separated)
        </label>
        <input
          type="text"
          value={(editingProvider?.config as OAuthConfig)?.scopes?.join(', ') || ''}
          onChange={(e) =>
            setEditingProvider({
              ...editingProvider,
              config: {
                ...editingProvider?.config,
                scopes: e.target.value.split(',').map((s) => s.trim()),
              } as OAuthConfig,
            })
          }
          placeholder="openid, profile, email"
          className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
        />
      </div>
    </div>
  );

  const renderLDAPConfig = () => (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700">Host</label>
          <input
            type="text"
            value={(editingProvider?.config as LDAPConfig)?.host || ''}
            onChange={(e) =>
              setEditingProvider({
                ...editingProvider,
                config: { ...editingProvider?.config, host: e.target.value } as LDAPConfig,
              })
            }
            className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700">Port</label>
          <input
            type="number"
            value={(editingProvider?.config as LDAPConfig)?.port || 389}
            onChange={(e) =>
              setEditingProvider({
                ...editingProvider,
                config: { ...editingProvider?.config, port: parseInt(e.target.value) } as LDAPConfig,
              })
            }
            className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          />
        </div>
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">Base DN</label>
        <input
          type="text"
          value={(editingProvider?.config as LDAPConfig)?.baseDN || ''}
          onChange={(e) =>
            setEditingProvider({
              ...editingProvider,
              config: { ...editingProvider?.config, baseDN: e.target.value } as LDAPConfig,
            })
          }
          placeholder="dc=example,dc=com"
          className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">Bind DN</label>
        <input
          type="text"
          value={(editingProvider?.config as LDAPConfig)?.bindDN || ''}
          onChange={(e) =>
            setEditingProvider({
              ...editingProvider,
              config: { ...editingProvider?.config, bindDN: e.target.value } as LDAPConfig,
            })
          }
          placeholder="cn=admin,dc=example,dc=com"
          className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">Bind Password</label>
        <input
          type="password"
          value={(editingProvider?.config as LDAPConfig)?.bindPassword || ''}
          onChange={(e) =>
            setEditingProvider({
              ...editingProvider,
              config: { ...editingProvider?.config, bindPassword: e.target.value } as LDAPConfig,
            })
          }
          className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
        />
      </div>
      <div className="flex items-center">
        <input
          type="checkbox"
          checked={(editingProvider?.config as LDAPConfig)?.useTLS || false}
          onChange={(e) =>
            setEditingProvider({
              ...editingProvider,
              config: { ...editingProvider?.config, useTLS: e.target.checked } as LDAPConfig,
            })
          }
          className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
        />
        <label className="ml-2 block text-sm text-gray-700">Use TLS</label>
      </div>
    </div>
  );

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  return (
    <div className={`bg-white shadow sm:rounded-lg ${className}`}>
      <div className="px-4 py-5 sm:p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="text-lg font-medium text-gray-900">SSO Configuration</h3>
            <p className="mt-1 text-sm text-gray-500">
              Configure Single Sign-On providers for your organization
            </p>
          </div>
          <button
            onClick={() => {
              setShowCreateModal(true);
              setEditingProvider({
                name: '',
                displayName: '',
                type: 'saml',
                enabled: true,
                config: { type: 'saml' } as SAMLConfig,
                attributeMapping: {
                  userId: 'id',
                  username: 'username',
                  email: 'email',
                  firstName: 'firstName',
                  lastName: 'lastName',
                  customAttributes: {},
                },
                provisioning: {
                  enabled: true,
                  createUsers: true,
                  updateUsers: true,
                  deactivateUsers: false,
                  syncGroups: false,
                  syncRoles: false,
                  defaultRoles: [],
                  defaultTeams: [],
                },
              });
            }}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
          >
            Add SSO Provider
          </button>
        </div>

        {providers.length === 0 ? (
          <div className="text-center py-12">
            <svg
              className="mx-auto h-12 w-12 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
              />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900">No SSO providers</h3>
            <p className="mt-1 text-sm text-gray-500">
              Get started by adding your first SSO provider.
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            {providers.map((provider) => (
              <div key={provider.id} className="border border-gray-200 rounded-lg p-4">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2 mb-2">
                      <h4 className="text-sm font-medium text-gray-900">
                        {provider.displayName}
                      </h4>
                      <span
                        className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
                          provider.enabled
                            ? 'bg-green-100 text-green-800'
                            : 'bg-gray-100 text-gray-800'
                        }`}
                      >
                        {provider.enabled ? 'Enabled' : 'Disabled'}
                      </span>
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                        {provider.type.toUpperCase()}
                      </span>
                    </div>
                    <div className="grid grid-cols-2 gap-4 mt-3 text-sm">
                      <div>
                        <dt className="text-gray-500">User Provisioning</dt>
                        <dd className="text-gray-900">
                          {provider.provisioning.enabled ? 'Enabled' : 'Disabled'}
                        </dd>
                      </div>
                      <div>
                        <dt className="text-gray-500">Auto-create Users</dt>
                        <dd className="text-gray-900">
                          {provider.provisioning.createUsers ? 'Yes' : 'No'}
                        </dd>
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center space-x-3 ml-4">
                    <button
                      onClick={() => handleTestProvider(provider.id)}
                      disabled={testing === provider.id}
                      className="text-sm text-indigo-600 hover:text-indigo-900 font-medium disabled:opacity-50"
                    >
                      {testing === provider.id ? 'Testing...' : 'Test'}
                    </button>
                    <button
                      onClick={() => setEditingProvider(provider)}
                      className="text-sm text-indigo-600 hover:text-indigo-900 font-medium"
                    >
                      Edit
                    </button>
                    <button
                      onClick={() => handleDeleteProvider(provider.id)}
                      className="text-sm text-red-600 hover:text-red-900 font-medium"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {(showCreateModal || editingProvider) && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col">
            <div className="px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-medium text-gray-900">
                {editingProvider?.id ? 'Edit SSO Provider' : 'Add SSO Provider'}
              </h3>
            </div>
            <div className="flex-1 overflow-y-auto px-6 py-4">
              <div className="space-y-6">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700">Name</label>
                    <input
                      type="text"
                      value={editingProvider?.name || ''}
                      onChange={(e) =>
                        setEditingProvider({ ...editingProvider, name: e.target.value })
                      }
                      className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700">
                      Display Name
                    </label>
                    <input
                      type="text"
                      value={editingProvider?.displayName || ''}
                      onChange={(e) =>
                        setEditingProvider({ ...editingProvider, displayName: e.target.value })
                      }
                      className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                    />
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">Provider Type</label>
                  <select
                    value={editingProvider?.type || 'saml'}
                    onChange={(e) => {
                      const type = e.target.value as SSOProviderType;
                      setSelectedType(type);
                      setEditingProvider({
                        ...editingProvider,
                        type,
                        config: { type } as any,
                      });
                    }}
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  >
                    <option value="saml">SAML 2.0</option>
                    <option value="oauth">OAuth 2.0</option>
                    <option value="oidc">OpenID Connect</option>
                    <option value="ldap">LDAP</option>
                    <option value="azure_ad">Azure AD</option>
                    <option value="google">Google Workspace</option>
                    <option value="okta">Okta</option>
                  </select>
                </div>

                <div>
                  <h4 className="text-sm font-medium text-gray-900 mb-3">Provider Configuration</h4>
                  {editingProvider?.type === 'saml' && renderSAMLConfig()}
                  {(editingProvider?.type === 'oauth' || editingProvider?.type === 'oidc') &&
                    renderOAuthConfig()}
                  {editingProvider?.type === 'ldap' && renderLDAPConfig()}
                </div>

                <div className="flex items-center">
                  <input
                    type="checkbox"
                    checked={editingProvider?.enabled || false}
                    onChange={(e) =>
                      setEditingProvider({ ...editingProvider, enabled: e.target.checked })
                    }
                    className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                  />
                  <label className="ml-2 block text-sm text-gray-700">Enable this provider</label>
                </div>
              </div>
            </div>
            <div className="px-6 py-4 border-t border-gray-200 flex justify-end space-x-3">
              <button
                onClick={() => {
                  setShowCreateModal(false);
                  setEditingProvider(null);
                }}
                className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  if (editingProvider?.id) {
                    handleUpdateProvider(editingProvider.id, editingProvider);
                  } else {
                    handleCreateProvider();
                  }
                }}
                className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
              >
                {editingProvider?.id ? 'Update' : 'Create'} Provider
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default SSOConfiguration;
