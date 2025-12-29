import React, { useState, useCallback } from 'react';
import { useSSOProviders } from './UserHooks';
export const SSOConfiguration = ({ onProviderCreate, onProviderUpdate, onProviderDelete, className = '', }) => {
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [editingProvider, setEditingProvider] = useState(null);
    const [selectedType, setSelectedType] = useState('saml');
    const [testing, setTesting] = useState(null);
    const { providers, loading, createProvider, updateProvider, deleteProvider, testProvider, } = useSSOProviders();
    const handleCreateProvider = useCallback(async () => {
        if (!editingProvider)
            return;
        try {
            const provider = await createProvider(editingProvider);
            setShowCreateModal(false);
            setEditingProvider(null);
            onProviderCreate?.(provider);
        }
        catch (err) {
            console.error('Failed to create SSO provider:', err);
            alert('Failed to create SSO provider. Please check the configuration.');
        }
    }, [editingProvider, createProvider, onProviderCreate]);
    const handleUpdateProvider = useCallback(async (providerId, updates) => {
        try {
            const provider = await updateProvider(providerId, updates);
            setEditingProvider(null);
            onProviderUpdate?.(provider);
        }
        catch (err) {
            console.error('Failed to update SSO provider:', err);
            alert('Failed to update SSO provider.');
        }
    }, [updateProvider, onProviderUpdate]);
    const handleDeleteProvider = useCallback(async (providerId) => {
        if (window.confirm('Are you sure you want to delete this SSO provider? Users will no longer be able to sign in using this method.')) {
            try {
                await deleteProvider(providerId);
                onProviderDelete?.(providerId);
            }
            catch (err) {
                console.error('Failed to delete SSO provider:', err);
            }
        }
    }, [deleteProvider, onProviderDelete]);
    const handleTestProvider = useCallback(async (providerId) => {
        setTesting(providerId);
        try {
            const result = await testProvider(providerId);
            if (result.success) {
                alert('Connection test successful!');
            }
            else {
                alert(`Connection test failed: ${result.message}`);
            }
        }
        catch (err) {
            console.error('Failed to test SSO provider:', err);
            alert('Connection test failed. Please check the configuration.');
        }
        finally {
            setTesting(null);
        }
    }, [testProvider]);
    const renderSAMLConfig = () => (React.createElement("div", { className: "space-y-4" },
        React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Entity ID"),
            React.createElement("input", { type: "text", value: editingProvider?.config?.entityId || '', onChange: (e) => setEditingProvider({
                    ...editingProvider,
                    config: { ...editingProvider?.config, entityId: e.target.value },
                }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
        React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "SSO URL"),
            React.createElement("input", { type: "url", value: editingProvider?.config?.ssoUrl || '', onChange: (e) => setEditingProvider({
                    ...editingProvider,
                    config: { ...editingProvider?.config, ssoUrl: e.target.value },
                }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
        React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Certificate"),
            React.createElement("textarea", { value: editingProvider?.config?.certificate || '', onChange: (e) => setEditingProvider({
                    ...editingProvider,
                    config: { ...editingProvider?.config, certificate: e.target.value },
                }), rows: 6, className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm font-mono text-xs", placeholder: "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----" })),
        React.createElement("div", { className: "flex items-center space-x-6" },
            React.createElement("div", { className: "flex items-center" },
                React.createElement("input", { type: "checkbox", checked: editingProvider?.config?.signRequests || false, onChange: (e) => setEditingProvider({
                        ...editingProvider,
                        config: {
                            ...editingProvider?.config,
                            signRequests: e.target.checked,
                        },
                    }), className: "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded" }),
                React.createElement("label", { className: "ml-2 block text-sm text-gray-700" }, "Sign Requests")),
            React.createElement("div", { className: "flex items-center" },
                React.createElement("input", { type: "checkbox", checked: editingProvider?.config?.encryptAssertions || false, onChange: (e) => setEditingProvider({
                        ...editingProvider,
                        config: {
                            ...editingProvider?.config,
                            encryptAssertions: e.target.checked,
                        },
                    }), className: "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded" }),
                React.createElement("label", { className: "ml-2 block text-sm text-gray-700" }, "Encrypt Assertions")))));
    const renderOAuthConfig = () => (React.createElement("div", { className: "space-y-4" },
        React.createElement("div", { className: "grid grid-cols-2 gap-4" },
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Client ID"),
                React.createElement("input", { type: "text", value: editingProvider?.config?.clientId || '', onChange: (e) => setEditingProvider({
                        ...editingProvider,
                        config: { ...editingProvider?.config, clientId: e.target.value },
                    }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Client Secret"),
                React.createElement("input", { type: "password", value: editingProvider?.config?.clientSecret || '', onChange: (e) => setEditingProvider({
                        ...editingProvider,
                        config: {
                            ...editingProvider?.config,
                            clientSecret: e.target.value,
                        },
                    }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" }))),
        React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Authorization URL"),
            React.createElement("input", { type: "url", value: editingProvider?.config?.authorizationUrl || '', onChange: (e) => setEditingProvider({
                    ...editingProvider,
                    config: {
                        ...editingProvider?.config,
                        authorizationUrl: e.target.value,
                    },
                }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
        React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Token URL"),
            React.createElement("input", { type: "url", value: editingProvider?.config?.tokenUrl || '', onChange: (e) => setEditingProvider({
                    ...editingProvider,
                    config: { ...editingProvider?.config, tokenUrl: e.target.value },
                }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
        React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Scopes (comma-separated)"),
            React.createElement("input", { type: "text", value: editingProvider?.config?.scopes?.join(', ') || '', onChange: (e) => setEditingProvider({
                    ...editingProvider,
                    config: {
                        ...editingProvider?.config,
                        scopes: e.target.value.split(',').map((s) => s.trim()),
                    },
                }), placeholder: "openid, profile, email", className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" }))));
    const renderLDAPConfig = () => (React.createElement("div", { className: "space-y-4" },
        React.createElement("div", { className: "grid grid-cols-2 gap-4" },
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Host"),
                React.createElement("input", { type: "text", value: editingProvider?.config?.host || '', onChange: (e) => setEditingProvider({
                        ...editingProvider,
                        config: { ...editingProvider?.config, host: e.target.value },
                    }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Port"),
                React.createElement("input", { type: "number", value: editingProvider?.config?.port || 389, onChange: (e) => setEditingProvider({
                        ...editingProvider,
                        config: { ...editingProvider?.config, port: parseInt(e.target.value) },
                    }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" }))),
        React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Base DN"),
            React.createElement("input", { type: "text", value: editingProvider?.config?.baseDN || '', onChange: (e) => setEditingProvider({
                    ...editingProvider,
                    config: { ...editingProvider?.config, baseDN: e.target.value },
                }), placeholder: "dc=example,dc=com", className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
        React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Bind DN"),
            React.createElement("input", { type: "text", value: editingProvider?.config?.bindDN || '', onChange: (e) => setEditingProvider({
                    ...editingProvider,
                    config: { ...editingProvider?.config, bindDN: e.target.value },
                }), placeholder: "cn=admin,dc=example,dc=com", className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
        React.createElement("div", null,
            React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Bind Password"),
            React.createElement("input", { type: "password", value: editingProvider?.config?.bindPassword || '', onChange: (e) => setEditingProvider({
                    ...editingProvider,
                    config: { ...editingProvider?.config, bindPassword: e.target.value },
                }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
        React.createElement("div", { className: "flex items-center" },
            React.createElement("input", { type: "checkbox", checked: editingProvider?.config?.useTLS || false, onChange: (e) => setEditingProvider({
                    ...editingProvider,
                    config: { ...editingProvider?.config, useTLS: e.target.checked },
                }), className: "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded" }),
            React.createElement("label", { className: "ml-2 block text-sm text-gray-700" }, "Use TLS"))));
    if (loading) {
        return (React.createElement("div", { className: "flex justify-center items-center h-64" },
            React.createElement("div", { className: "animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600" })));
    }
    return (React.createElement("div", { className: `bg-white shadow sm:rounded-lg ${className}` },
        React.createElement("div", { className: "px-4 py-5 sm:p-6" },
            React.createElement("div", { className: "flex items-center justify-between mb-6" },
                React.createElement("div", null,
                    React.createElement("h3", { className: "text-lg font-medium text-gray-900" }, "SSO Configuration"),
                    React.createElement("p", { className: "mt-1 text-sm text-gray-500" }, "Configure Single Sign-On providers for your organization")),
                React.createElement("button", { onClick: () => {
                        setShowCreateModal(true);
                        setEditingProvider({
                            name: '',
                            displayName: '',
                            type: 'saml',
                            enabled: true,
                            config: { type: 'saml' },
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
                    }, className: "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700" }, "Add SSO Provider")),
            providers.length === 0 ? (React.createElement("div", { className: "text-center py-12" },
                React.createElement("svg", { className: "mx-auto h-12 w-12 text-gray-400", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                    React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" })),
                React.createElement("h3", { className: "mt-2 text-sm font-medium text-gray-900" }, "No SSO providers"),
                React.createElement("p", { className: "mt-1 text-sm text-gray-500" }, "Get started by adding your first SSO provider."))) : (React.createElement("div", { className: "space-y-4" }, providers.map((provider) => (React.createElement("div", { key: provider.id, className: "border border-gray-200 rounded-lg p-4" },
                React.createElement("div", { className: "flex items-start justify-between" },
                    React.createElement("div", { className: "flex-1" },
                        React.createElement("div", { className: "flex items-center space-x-2 mb-2" },
                            React.createElement("h4", { className: "text-sm font-medium text-gray-900" }, provider.displayName),
                            React.createElement("span", { className: `inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${provider.enabled
                                    ? 'bg-green-100 text-green-800'
                                    : 'bg-gray-100 text-gray-800'}` }, provider.enabled ? 'Enabled' : 'Disabled'),
                            React.createElement("span", { className: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800" }, provider.type.toUpperCase())),
                        React.createElement("div", { className: "grid grid-cols-2 gap-4 mt-3 text-sm" },
                            React.createElement("div", null,
                                React.createElement("dt", { className: "text-gray-500" }, "User Provisioning"),
                                React.createElement("dd", { className: "text-gray-900" }, provider.provisioning.enabled ? 'Enabled' : 'Disabled')),
                            React.createElement("div", null,
                                React.createElement("dt", { className: "text-gray-500" }, "Auto-create Users"),
                                React.createElement("dd", { className: "text-gray-900" }, provider.provisioning.createUsers ? 'Yes' : 'No')))),
                    React.createElement("div", { className: "flex items-center space-x-3 ml-4" },
                        React.createElement("button", { onClick: () => handleTestProvider(provider.id), disabled: testing === provider.id, className: "text-sm text-indigo-600 hover:text-indigo-900 font-medium disabled:opacity-50" }, testing === provider.id ? 'Testing...' : 'Test'),
                        React.createElement("button", { onClick: () => setEditingProvider(provider), className: "text-sm text-indigo-600 hover:text-indigo-900 font-medium" }, "Edit"),
                        React.createElement("button", { onClick: () => handleDeleteProvider(provider.id), className: "text-sm text-red-600 hover:text-red-900 font-medium" }, "Delete"))))))))),
        (showCreateModal || editingProvider) && (React.createElement("div", { className: "fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50" },
            React.createElement("div", { className: "bg-white rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col" },
                React.createElement("div", { className: "px-6 py-4 border-b border-gray-200" },
                    React.createElement("h3", { className: "text-lg font-medium text-gray-900" }, editingProvider?.id ? 'Edit SSO Provider' : 'Add SSO Provider')),
                React.createElement("div", { className: "flex-1 overflow-y-auto px-6 py-4" },
                    React.createElement("div", { className: "space-y-6" },
                        React.createElement("div", { className: "grid grid-cols-2 gap-4" },
                            React.createElement("div", null,
                                React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Name"),
                                React.createElement("input", { type: "text", value: editingProvider?.name || '', onChange: (e) => setEditingProvider({ ...editingProvider, name: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
                            React.createElement("div", null,
                                React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Display Name"),
                                React.createElement("input", { type: "text", value: editingProvider?.displayName || '', onChange: (e) => setEditingProvider({ ...editingProvider, displayName: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" }))),
                        React.createElement("div", null,
                            React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Provider Type"),
                            React.createElement("select", { value: editingProvider?.type || 'saml', onChange: (e) => {
                                    const type = e.target.value;
                                    setSelectedType(type);
                                    setEditingProvider({
                                        ...editingProvider,
                                        type,
                                        config: { type },
                                    });
                                }, className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" },
                                React.createElement("option", { value: "saml" }, "SAML 2.0"),
                                React.createElement("option", { value: "oauth" }, "OAuth 2.0"),
                                React.createElement("option", { value: "oidc" }, "OpenID Connect"),
                                React.createElement("option", { value: "ldap" }, "LDAP"),
                                React.createElement("option", { value: "azure_ad" }, "Azure AD"),
                                React.createElement("option", { value: "google" }, "Google Workspace"),
                                React.createElement("option", { value: "okta" }, "Okta"))),
                        React.createElement("div", null,
                            React.createElement("h4", { className: "text-sm font-medium text-gray-900 mb-3" }, "Provider Configuration"),
                            editingProvider?.type === 'saml' && renderSAMLConfig(),
                            (editingProvider?.type === 'oauth' || editingProvider?.type === 'oidc') &&
                                renderOAuthConfig(),
                            editingProvider?.type === 'ldap' && renderLDAPConfig()),
                        React.createElement("div", { className: "flex items-center" },
                            React.createElement("input", { type: "checkbox", checked: editingProvider?.enabled || false, onChange: (e) => setEditingProvider({ ...editingProvider, enabled: e.target.checked }), className: "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded" }),
                            React.createElement("label", { className: "ml-2 block text-sm text-gray-700" }, "Enable this provider")))),
                React.createElement("div", { className: "px-6 py-4 border-t border-gray-200 flex justify-end space-x-3" },
                    React.createElement("button", { onClick: () => {
                            setShowCreateModal(false);
                            setEditingProvider(null);
                        }, className: "inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Cancel"),
                    React.createElement("button", { onClick: () => {
                            if (editingProvider?.id) {
                                handleUpdateProvider(editingProvider.id, editingProvider);
                            }
                            else {
                                handleCreateProvider();
                            }
                        }, className: "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700" },
                        editingProvider?.id ? 'Update' : 'Create',
                        " Provider")))))));
};
export default SSOConfiguration;
//# sourceMappingURL=SSOConfiguration.js.map