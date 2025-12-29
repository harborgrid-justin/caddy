/**
 * SSO Configuration Component
 * Enterprise SSO setup wizard with support for multiple providers
 */

import React, { useState, useEffect } from 'react';
import { Button, Input, Select, Modal, Tabs } from '../enterprise';
import type { SsoConfig, SsoProvider, SSOConfigurationProps, SsoTestResult } from './types';

const SSO_PROVIDERS: Array<{ value: SsoProvider; label: string; description: string }> = [
  {
    value: 'google_workspace',
    label: 'Google Workspace',
    description: 'Google Workspace SSO with OAuth 2.0',
  },
  {
    value: 'azure_ad',
    label: 'Microsoft Azure AD',
    description: 'Azure Active Directory with OIDC',
  },
  {
    value: 'okta',
    label: 'Okta',
    description: 'Okta SSO with OIDC',
  },
  {
    value: 'saml2',
    label: 'SAML 2.0',
    description: 'Generic SAML 2.0 provider',
  },
  {
    value: 'oidc',
    label: 'OpenID Connect',
    description: 'Generic OIDC provider',
  },
  {
    value: 'oauth2',
    label: 'OAuth 2.0',
    description: 'Generic OAuth 2.0 provider',
  },
  {
    value: 'active_directory',
    label: 'Active Directory',
    description: 'Microsoft Active Directory / LDAP',
  },
  {
    value: 'ldap',
    label: 'LDAP',
    description: 'Generic LDAP server',
  },
];

const DEFAULT_SCOPES: Record<SsoProvider, string[]> = {
  google_workspace: ['openid', 'profile', 'email'],
  azure_ad: ['openid', 'profile', 'email'],
  okta: ['openid', 'profile', 'email'],
  oidc: ['openid', 'profile', 'email'],
  oauth2: ['profile', 'email'],
  saml2: [],
  active_directory: [],
  ldap: [],
};

export const SSOConfiguration: React.FC<SSOConfigurationProps> = ({
  onSave,
  onCancel,
  initialConfig,
  mode = 'create',
}) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [config, setConfig] = useState<Partial<SsoConfig>>(
    initialConfig || {
      enabled: true,
      auto_provision: true,
      attribute_mapping: {},
      scopes: [],
    }
  );
  const [testResult, setTestResult] = useState<SsoTestResult | null>(null);
  const [isTesting, setIsTesting] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    // Update default scopes when provider changes
    if (config.provider && !config.scopes?.length) {
      setConfig((prev) => ({
        ...prev,
        scopes: DEFAULT_SCOPES[config.provider!] || [],
      }));
    }
  }, [config.provider]);

  const updateConfig = (field: keyof SsoConfig, value: any) => {
    setConfig((prev) => ({ ...prev, [field]: value }));
    // Clear error for this field
    if (errors[field]) {
      setErrors((prev) => {
        const newErrors = { ...prev };
        delete newErrors[field];
        return newErrors;
      });
    }
  };

  const validateStep = (step: number): boolean => {
    const newErrors: Record<string, string> = {};

    switch (step) {
      case 0: // Provider selection
        if (!config.provider) {
          newErrors.provider = 'Please select an SSO provider';
        }
        if (!config.provider_name?.trim()) {
          newErrors.provider_name = 'Provider name is required';
        }
        break;

      case 1: // Provider configuration
        if (config.provider === 'saml2') {
          if (!config.saml_entity_id?.trim()) {
            newErrors.saml_entity_id = 'SAML Entity ID is required';
          }
          if (!config.saml_sso_url?.trim()) {
            newErrors.saml_sso_url = 'SAML SSO URL is required';
          }
        } else if (config.provider === 'active_directory' || config.provider === 'ldap') {
          if (!config.ldap_url?.trim()) {
            newErrors.ldap_url = 'LDAP URL is required';
          }
          if (!config.ldap_base_dn?.trim()) {
            newErrors.ldap_base_dn = 'Base DN is required';
          }
        } else {
          // OAuth/OIDC providers
          if (!config.client_id?.trim()) {
            newErrors.client_id = 'Client ID is required';
          }
          if (!config.client_secret?.trim()) {
            newErrors.client_secret = 'Client Secret is required';
          }
          if (!config.authorization_endpoint?.trim()) {
            newErrors.authorization_endpoint = 'Authorization endpoint is required';
          }
          if (!config.token_endpoint?.trim()) {
            newErrors.token_endpoint = 'Token endpoint is required';
          }
        }
        if (!config.redirect_uri?.trim()) {
          newErrors.redirect_uri = 'Redirect URI is required';
        }
        break;
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleNext = () => {
    if (validateStep(currentStep)) {
      setCurrentStep((prev) => prev + 1);
    }
  };

  const handleBack = () => {
    setCurrentStep((prev) => prev - 1);
  };

  const handleTest = async () => {
    setIsTesting(true);
    setTestResult(null);

    try {
      // In production, call actual API to test SSO configuration
      const response = await fetch('/api/auth/sso/test', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });

      const result: SsoTestResult = await response.json();
      setTestResult(result);
    } catch (error) {
      setTestResult({
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setIsTesting(false);
    }
  };

  const handleSave = () => {
    if (validateStep(currentStep) && config.provider) {
      onSave?.(config as SsoConfig);
    }
  };

  const renderProviderSelection = () => (
    <div className="space-y-6">
      <div>
        <label className="block text-sm font-medium mb-2">SSO Provider</label>
        <Select
          value={config.provider}
          onChange={(e) => updateConfig('provider', e.target.value as SsoProvider)}
          options={SSO_PROVIDERS.map((p) => ({
            value: p.value,
            label: `${p.label} - ${p.description}`,
          }))}
          placeholder="Select SSO Provider"
        />
        {errors.provider && <p className="text-red-500 text-sm mt-1">{errors.provider}</p>}
      </div>

      <Input
        label="Provider Name"
        value={config.provider_name || ''}
        onChange={(e) => updateConfig('provider_name', e.target.value)}
        placeholder="e.g., Company SSO"
        error={errors.provider_name}
        required
      />

      <Input
        label="Redirect URI"
        value={config.redirect_uri || ''}
        onChange={(e) => updateConfig('redirect_uri', e.target.value)}
        placeholder="https://your-domain.com/auth/callback"
        helperText="The URL where users will be redirected after authentication"
        error={errors.redirect_uri}
        required
      />
    </div>
  );

  const renderOAuthConfig = () => (
    <div className="space-y-6">
      <Input
        label="Client ID"
        value={config.client_id || ''}
        onChange={(e) => updateConfig('client_id', e.target.value)}
        placeholder="Enter client ID from your SSO provider"
        error={errors.client_id}
        required
      />

      <Input
        label="Client Secret"
        type="password"
        value={config.client_secret || ''}
        onChange={(e) => updateConfig('client_secret', e.target.value)}
        placeholder="Enter client secret"
        error={errors.client_secret}
        required
      />

      <Input
        label="Authorization Endpoint"
        value={config.authorization_endpoint || ''}
        onChange={(e) => updateConfig('authorization_endpoint', e.target.value)}
        placeholder="https://provider.com/oauth2/authorize"
        error={errors.authorization_endpoint}
        required
      />

      <Input
        label="Token Endpoint"
        value={config.token_endpoint || ''}
        onChange={(e) => updateConfig('token_endpoint', e.target.value)}
        placeholder="https://provider.com/oauth2/token"
        error={errors.token_endpoint}
        required
      />

      {(config.provider === 'oidc' || config.provider === 'google_workspace' ||
        config.provider === 'azure_ad' || config.provider === 'okta') && (
        <>
          <Input
            label="User Info Endpoint"
            value={config.userinfo_endpoint || ''}
            onChange={(e) => updateConfig('userinfo_endpoint', e.target.value)}
            placeholder="https://provider.com/oauth2/userinfo"
          />

          <Input
            label="JWKS URI"
            value={config.jwks_uri || ''}
            onChange={(e) => updateConfig('jwks_uri', e.target.value)}
            placeholder="https://provider.com/.well-known/jwks.json"
          />

          <Input
            label="Issuer"
            value={config.issuer || ''}
            onChange={(e) => updateConfig('issuer', e.target.value)}
            placeholder="https://provider.com"
          />
        </>
      )}

      <div>
        <label className="block text-sm font-medium mb-2">Scopes</label>
        <Input
          value={config.scopes?.join(' ') || ''}
          onChange={(e) => updateConfig('scopes', e.target.value.split(' ').filter(Boolean))}
          placeholder="openid profile email"
          helperText="Space-separated list of OAuth scopes"
        />
      </div>
    </div>
  );

  const renderSamlConfig = () => (
    <div className="space-y-6">
      <Input
        label="SAML Entity ID"
        value={config.saml_entity_id || ''}
        onChange={(e) => updateConfig('saml_entity_id', e.target.value)}
        placeholder="https://your-domain.com/saml/metadata"
        error={errors.saml_entity_id}
        required
      />

      <Input
        label="SAML SSO URL"
        value={config.saml_sso_url || ''}
        onChange={(e) => updateConfig('saml_sso_url', e.target.value)}
        placeholder="https://provider.com/saml/sso"
        error={errors.saml_sso_url}
        required
      />

      <div>
        <label className="block text-sm font-medium mb-2">SAML Certificate</label>
        <textarea
          className="w-full h-32 p-2 border rounded font-mono text-sm"
          value={config.saml_certificate || ''}
          onChange={(e) => updateConfig('saml_certificate', e.target.value)}
          placeholder="-----BEGIN CERTIFICATE-----&#10;...&#10;-----END CERTIFICATE-----"
        />
        {errors.saml_certificate && (
          <p className="text-red-500 text-sm mt-1">{errors.saml_certificate}</p>
        )}
      </div>
    </div>
  );

  const renderLdapConfig = () => (
    <div className="space-y-6">
      <Input
        label="LDAP Server URL"
        value={config.ldap_url || ''}
        onChange={(e) => updateConfig('ldap_url', e.target.value)}
        placeholder="ldap://ldap.company.com:389"
        error={errors.ldap_url}
        required
      />

      <Input
        label="Base DN"
        value={config.ldap_base_dn || ''}
        onChange={(e) => updateConfig('ldap_base_dn', e.target.value)}
        placeholder="dc=company,dc=com"
        error={errors.ldap_base_dn}
        required
      />

      <Input
        label="Bind DN"
        value={config.ldap_bind_dn || ''}
        onChange={(e) => updateConfig('ldap_bind_dn', e.target.value)}
        placeholder="cn=admin,dc=company,dc=com"
        helperText="Service account for LDAP queries"
      />

      <Input
        label="Bind Password"
        type="password"
        value={config.ldap_bind_password || ''}
        onChange={(e) => updateConfig('ldap_bind_password', e.target.value)}
        placeholder="Enter bind password"
      />

      <Input
        label="User Filter"
        value={config.ldap_user_filter || ''}
        onChange={(e) => updateConfig('ldap_user_filter', e.target.value)}
        placeholder="(&(objectClass=user)(sAMAccountName=%s))"
        helperText="%s will be replaced with the username"
      />
    </div>
  );

  const renderProviderConfig = () => {
    if (config.provider === 'saml2') {
      return renderSamlConfig();
    } else if (config.provider === 'active_directory' || config.provider === 'ldap') {
      return renderLdapConfig();
    } else {
      return renderOAuthConfig();
    }
  };

  const renderAttributeMapping = () => (
    <div className="space-y-6">
      <div className="bg-blue-50 p-4 rounded">
        <h4 className="font-medium mb-2">Attribute Mapping</h4>
        <p className="text-sm text-gray-600">
          Map SSO provider attributes to CADDY user attributes
        </p>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <Input
          label="Username Attribute"
          value={config.attribute_mapping?.username || ''}
          onChange={(e) =>
            updateConfig('attribute_mapping', {
              ...config.attribute_mapping,
              username: e.target.value,
            })
          }
          placeholder="preferred_username"
        />

        <Input
          label="Email Attribute"
          value={config.attribute_mapping?.email || ''}
          onChange={(e) =>
            updateConfig('attribute_mapping', {
              ...config.attribute_mapping,
              email: e.target.value,
            })
          }
          placeholder="email"
        />

        <Input
          label="First Name Attribute"
          value={config.attribute_mapping?.first_name || ''}
          onChange={(e) =>
            updateConfig('attribute_mapping', {
              ...config.attribute_mapping,
              first_name: e.target.value,
            })
          }
          placeholder="given_name"
        />

        <Input
          label="Last Name Attribute"
          value={config.attribute_mapping?.last_name || ''}
          onChange={(e) =>
            updateConfig('attribute_mapping', {
              ...config.attribute_mapping,
              last_name: e.target.value,
            })
          }
          placeholder="family_name"
        />
      </div>

      <div className="border-t pt-4">
        <label className="flex items-center space-x-2">
          <input
            type="checkbox"
            checked={config.auto_provision || false}
            onChange={(e) => updateConfig('auto_provision', e.target.checked)}
            className="rounded"
          />
          <span>Automatically provision new users</span>
        </label>
      </div>

      {config.auto_provision && (
        <Input
          label="Default Role for New Users"
          value={config.default_role || ''}
          onChange={(e) => updateConfig('default_role', e.target.value)}
          placeholder="viewer"
          helperText="Role assigned to auto-provisioned users"
        />
      )}
    </div>
  );

  const renderReview = () => (
    <div className="space-y-6">
      <div className="bg-gray-50 p-4 rounded">
        <h4 className="font-medium mb-4">Configuration Summary</h4>
        <dl className="space-y-2 text-sm">
          <div className="flex justify-between">
            <dt className="text-gray-600">Provider:</dt>
            <dd className="font-medium">{config.provider_name}</dd>
          </div>
          <div className="flex justify-between">
            <dt className="text-gray-600">Type:</dt>
            <dd className="font-medium">{config.provider}</dd>
          </div>
          <div className="flex justify-between">
            <dt className="text-gray-600">Redirect URI:</dt>
            <dd className="font-mono text-xs">{config.redirect_uri}</dd>
          </div>
          {config.auto_provision && (
            <div className="flex justify-between">
              <dt className="text-gray-600">Auto Provision:</dt>
              <dd className="font-medium text-green-600">Enabled</dd>
            </div>
          )}
        </dl>
      </div>

      <div>
        <Button onClick={handleTest} loading={isTesting} variant="secondary" fullWidth>
          Test Configuration
        </Button>
      </div>

      {testResult && (
        <div
          className={`p-4 rounded ${
            testResult.success ? 'bg-green-50 text-green-800' : 'bg-red-50 text-red-800'
          }`}
        >
          <h5 className="font-medium mb-2">
            {testResult.success ? 'Test Successful' : 'Test Failed'}
          </h5>
          {testResult.error && <p className="text-sm">{testResult.error}</p>}
          {testResult.user_info && (
            <pre className="text-xs mt-2 overflow-auto">
              {JSON.stringify(testResult.user_info, null, 2)}
            </pre>
          )}
        </div>
      )}
    </div>
  );

  const steps = [
    { title: 'Provider', component: renderProviderSelection },
    { title: 'Configuration', component: renderProviderConfig },
    { title: 'Attributes', component: renderAttributeMapping },
    { title: 'Review', component: renderReview },
  ];

  return (
    <div className="max-w-4xl mx-auto p-6">
      <h2 className="text-2xl font-bold mb-6">
        {mode === 'create' ? 'Configure SSO Provider' : 'Edit SSO Configuration'}
      </h2>

      {/* Progress Steps */}
      <div className="flex items-center justify-between mb-8">
        {steps.map((step, index) => (
          <div key={index} className="flex items-center flex-1">
            <div
              className={`flex items-center justify-center w-8 h-8 rounded-full ${
                index <= currentStep
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-200 text-gray-600'
              }`}
            >
              {index + 1}
            </div>
            <span
              className={`ml-2 text-sm font-medium ${
                index <= currentStep ? 'text-gray-900' : 'text-gray-500'
              }`}
            >
              {step.title}
            </span>
            {index < steps.length - 1 && (
              <div
                className={`flex-1 h-1 mx-4 ${
                  index < currentStep ? 'bg-blue-600' : 'bg-gray-200'
                }`}
              />
            )}
          </div>
        ))}
      </div>

      {/* Step Content */}
      <div className="bg-white rounded-lg border p-6 min-h-[400px]">
        {steps[currentStep].component()}
      </div>

      {/* Navigation */}
      <div className="flex justify-between mt-6">
        <div>
          {onCancel && (
            <Button variant="ghost" onClick={onCancel}>
              Cancel
            </Button>
          )}
        </div>
        <div className="flex space-x-2">
          {currentStep > 0 && (
            <Button variant="secondary" onClick={handleBack}>
              Back
            </Button>
          )}
          {currentStep < steps.length - 1 ? (
            <Button onClick={handleNext}>Next</Button>
          ) : (
            <Button onClick={handleSave} variant="primary">
              {mode === 'create' ? 'Create Configuration' : 'Save Changes'}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
};
