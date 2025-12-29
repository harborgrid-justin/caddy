/**
 * CADDY v0.4.0 Enterprise Settings Search
 * Search across all settings with keyboard navigation
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import { SearchResult, SearchMatch } from './types';

interface SettingsSearchProps {
  query: string;
  onQueryChange: (query: string) => void;
  onClose: () => void;
  onNavigate: (path: string) => void;
}

const SEARCHABLE_SETTINGS = [
  // General Settings
  { section: 'general', title: 'Application Name', description: 'Configure your application name and branding', path: 'general', keywords: ['app', 'name', 'title', 'branding'] },
  { section: 'general', title: 'Timezone & Locale', description: 'Set timezone, locale, and regional settings', path: 'general', keywords: ['timezone', 'locale', 'region', 'language', 'time', 'date'] },
  { section: 'general', title: 'Branding Colors', description: 'Customize primary, secondary, and accent colors', path: 'general', keywords: ['color', 'theme', 'branding', 'logo', 'favicon'] },
  { section: 'general', title: 'Maintenance Mode', description: 'Enable maintenance mode with custom message', path: 'general', keywords: ['maintenance', 'downtime', 'offline'] },

  // Security Settings
  { section: 'security', title: 'Password Policy', description: 'Configure password requirements and complexity', path: 'security', keywords: ['password', 'security', 'requirements', 'complexity', 'length'] },
  { section: 'security', title: 'Two-Factor Authentication', description: 'Enable and configure 2FA methods', path: 'security', keywords: ['2fa', 'mfa', 'totp', 'sms', 'authentication'] },
  { section: 'security', title: 'Single Sign-On (SSO)', description: 'Configure SSO providers and settings', path: 'security', keywords: ['sso', 'saml', 'oauth', 'oidc', 'ldap', 'login'] },
  { section: 'security', title: 'Session Management', description: 'Configure session timeouts and limits', path: 'security', keywords: ['session', 'timeout', 'expire', 'concurrent'] },
  { section: 'security', title: 'Audit Logging', description: 'Enable and configure audit logs', path: 'security', keywords: ['audit', 'log', 'logging', 'tracking', 'history'] },

  // Notification Settings
  { section: 'notifications', title: 'Email Notifications', description: 'Configure email notification settings', path: 'notifications', keywords: ['email', 'smtp', 'mail', 'notification'] },
  { section: 'notifications', title: 'SMS Notifications', description: 'Configure SMS notification settings', path: 'notifications', keywords: ['sms', 'text', 'message', 'twilio', 'notification'] },
  { section: 'notifications', title: 'Push Notifications', description: 'Configure push notification settings', path: 'notifications', keywords: ['push', 'fcm', 'apns', 'notification'] },
  { section: 'notifications', title: 'In-App Notifications', description: 'Configure in-app notification preferences', path: 'notifications', keywords: ['in-app', 'notification', 'sound', 'desktop'] },

  // Integration Settings
  { section: 'integrations', title: 'Third-Party Integrations', description: 'Connect and manage third-party services', path: 'integrations', keywords: ['integration', 'api', 'third-party', 'slack', 'github', 'jira'] },
  { section: 'integrations', title: 'Webhooks', description: 'Configure and manage webhooks', path: 'integrations', keywords: ['webhook', 'callback', 'http', 'endpoint'] },
  { section: 'integrations', title: 'API Rate Limits', description: 'Configure API rate limiting', path: 'integrations', keywords: ['api', 'rate', 'limit', 'throttle'] },

  // Billing Settings
  { section: 'billing', title: 'Subscription Plan', description: 'View and manage your subscription', path: 'billing', keywords: ['plan', 'subscription', 'upgrade', 'downgrade', 'pricing'] },
  { section: 'billing', title: 'Payment Methods', description: 'Manage payment methods and billing', path: 'billing', keywords: ['payment', 'credit card', 'billing', 'invoice'] },
  { section: 'billing', title: 'Usage Statistics', description: 'View current usage and limits', path: 'billing', keywords: ['usage', 'statistics', 'quota', 'limit', 'storage', 'bandwidth'] },
  { section: 'billing', title: 'Invoices', description: 'View and download invoices', path: 'billing', keywords: ['invoice', 'receipt', 'billing', 'payment history'] },

  // Team Settings
  { section: 'team', title: 'Team Members', description: 'Invite and manage team members', path: 'team', keywords: ['member', 'user', 'team', 'invite', 'people'] },
  { section: 'team', title: 'Roles & Permissions', description: 'Configure roles and permissions', path: 'team', keywords: ['role', 'permission', 'access', 'authorization', 'rbac'] },
  { section: 'team', title: 'Groups', description: 'Manage team groups', path: 'team', keywords: ['group', 'team', 'organization'] },

  // Advanced Settings
  { section: 'advanced', title: 'Developer Mode', description: 'Enable developer mode and debugging', path: 'advanced', keywords: ['developer', 'debug', 'development'] },
  { section: 'advanced', title: 'API Keys', description: 'Generate and manage API keys', path: 'advanced', keywords: ['api', 'key', 'token', 'authentication'] },
  { section: 'advanced', title: 'CORS Configuration', description: 'Configure CORS settings', path: 'advanced', keywords: ['cors', 'cross-origin', 'security'] },
  { section: 'advanced', title: 'Logging', description: 'Configure logging and retention', path: 'advanced', keywords: ['log', 'logging', 'retention', 'debug'] },
  { section: 'advanced', title: 'Performance', description: 'Configure caching and performance settings', path: 'advanced', keywords: ['performance', 'cache', 'cdn', 'compression'] },
];

const SettingsSearch: React.FC<SettingsSearchProps> = ({
  query,
  onQueryChange,
  onClose,
  onNavigate,
}) => {
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const resultsContainerRef = useRef<HTMLDivElement>(null);

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Search logic
  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      setSelectedIndex(0);
      return;
    }

    const searchQuery = query.toLowerCase();
    const searchResults: SearchResult[] = [];

    SEARCHABLE_SETTINGS.forEach((setting) => {
      const matches: SearchMatch[] = [];
      let score = 0;

      // Check title match
      if (setting.title.toLowerCase().includes(searchQuery)) {
        matches.push({
          field: 'title',
          value: setting.title,
          highlight: highlightMatch(setting.title, searchQuery),
        });
        score += 10;
      }

      // Check description match
      if (setting.description.toLowerCase().includes(searchQuery)) {
        matches.push({
          field: 'description',
          value: setting.description,
          highlight: highlightMatch(setting.description, searchQuery),
        });
        score += 5;
      }

      // Check keywords match
      setting.keywords.forEach((keyword) => {
        if (keyword.toLowerCase().includes(searchQuery)) {
          matches.push({
            field: 'keyword',
            value: keyword,
            highlight: highlightMatch(keyword, searchQuery),
          });
          score += 3;
        }
      });

      if (matches.length > 0) {
        searchResults.push({
          section: setting.section,
          title: setting.title,
          description: setting.description,
          path: setting.path,
          matches,
        });
      }
    });

    // Sort by relevance (score)
    searchResults.sort((a, b) => b.matches.length - a.matches.length);

    setResults(searchResults);
    setSelectedIndex(0);
  }, [query]);

  // Highlight matching text
  const highlightMatch = (text: string, query: string): string => {
    const index = text.toLowerCase().indexOf(query.toLowerCase());
    if (index === -1) return text;

    const before = text.substring(0, index);
    const match = text.substring(index, index + query.length);
    const after = text.substring(index + query.length);

    return `${before}<mark style="background-color: #ffeb3b; padding: 0 2px; border-radius: 2px;">${match}</mark>${after}`;
  };

  // Keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (results[selectedIndex]) {
          onNavigate(results[selectedIndex].path);
        }
      } else if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    },
    [results, selectedIndex, onNavigate, onClose]
  );

  // Scroll selected item into view
  useEffect(() => {
    if (resultsContainerRef.current) {
      const selectedElement = resultsContainerRef.current.querySelector(
        `[data-index="${selectedIndex}"]`
      );
      if (selectedElement) {
        selectedElement.scrollIntoView({
          block: 'nearest',
          behavior: 'smooth',
        });
      }
    }
  }, [selectedIndex]);

  // Get section badge color
  const getSectionColor = (section: string): string => {
    const colors: Record<string, string> = {
      general: '#2196f3',
      security: '#f44336',
      notifications: '#ff9800',
      integrations: '#9c27b0',
      billing: '#4caf50',
      team: '#00bcd4',
      advanced: '#607d8b',
    };
    return colors[section] || '#666';
  };

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'center',
        padding: '10vh 1rem',
        zIndex: 2000,
      }}
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-labelledby="search-title"
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          width: '100%',
          maxWidth: '600px',
          maxHeight: '70vh',
          display: 'flex',
          flexDirection: 'column',
          boxShadow: '0 4px 24px rgba(0, 0, 0, 0.2)',
        }}
      >
        {/* Search Input */}
        <div
          style={{
            padding: '1rem',
            borderBottom: '1px solid #e0e0e0',
          }}
        >
          <div style={{ position: 'relative' }}>
            <span
              style={{
                position: 'absolute',
                left: '0.75rem',
                top: '50%',
                transform: 'translateY(-50%)',
                fontSize: '1.25rem',
              }}
              aria-hidden="true"
            >
              🔍
            </span>
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => onQueryChange(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Search settings..."
              aria-label="Search settings"
              aria-describedby="search-help"
              style={{
                width: '100%',
                padding: '0.75rem 0.75rem 0.75rem 2.5rem',
                border: 'none',
                fontSize: '1rem',
                outline: 'none',
              }}
            />
          </div>
          <div
            id="search-help"
            style={{
              marginTop: '0.5rem',
              fontSize: '0.75rem',
              color: '#666',
              display: 'flex',
              gap: '1rem',
            }}
          >
            <span>↑↓ Navigate</span>
            <span>↵ Select</span>
            <span>Esc Close</span>
          </div>
        </div>

        {/* Results */}
        <div
          ref={resultsContainerRef}
          style={{
            flex: 1,
            overflowY: 'auto',
            padding: '0.5rem',
          }}
          role="listbox"
          aria-label="Search results"
        >
          {results.length === 0 && query.trim() !== '' && (
            <div
              style={{
                padding: '2rem',
                textAlign: 'center',
                color: '#666',
              }}
              role="status"
            >
              No settings found for "{query}"
            </div>
          )}

          {results.length === 0 && query.trim() === '' && (
            <div
              style={{
                padding: '2rem',
                textAlign: 'center',
                color: '#666',
              }}
            >
              <p style={{ margin: '0 0 1rem 0' }}>Type to search settings...</p>
              <div style={{ fontSize: '0.875rem' }}>
                <p style={{ margin: '0.5rem 0' }}>Try searching for:</p>
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem', justifyContent: 'center' }}>
                  {['password', 'email', 'api', 'team', 'billing', 'security'].map((term) => (
                    <button
                      key={term}
                      onClick={() => onQueryChange(term)}
                      style={{
                        padding: '0.25rem 0.75rem',
                        backgroundColor: '#f5f5f5',
                        border: '1px solid #e0e0e0',
                        borderRadius: '12px',
                        cursor: 'pointer',
                        fontSize: '0.875rem',
                      }}
                    >
                      {term}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}

          {results.map((result, index) => (
            <div
              key={`${result.section}-${result.title}`}
              data-index={index}
              onClick={() => onNavigate(result.path)}
              role="option"
              aria-selected={index === selectedIndex}
              style={{
                padding: '0.75rem',
                borderRadius: '4px',
                marginBottom: '0.25rem',
                cursor: 'pointer',
                backgroundColor: index === selectedIndex ? '#e3f2fd' : 'transparent',
                border: index === selectedIndex ? '1px solid #1976d2' : '1px solid transparent',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.25rem' }}>
                <span
                  style={{
                    padding: '0.125rem 0.5rem',
                    backgroundColor: getSectionColor(result.section),
                    color: '#fff',
                    fontSize: '0.75rem',
                    borderRadius: '4px',
                    textTransform: 'capitalize',
                  }}
                >
                  {result.section}
                </span>
                <span
                  style={{
                    fontWeight: 600,
                    fontSize: '0.9375rem',
                  }}
                  dangerouslySetInnerHTML={{
                    __html: result.matches.find((m) => m.field === 'title')?.highlight || result.title,
                  }}
                />
              </div>
              <div
                style={{
                  fontSize: '0.875rem',
                  color: '#666',
                }}
                dangerouslySetInnerHTML={{
                  __html:
                    result.matches.find((m) => m.field === 'description')?.highlight ||
                    result.description,
                }}
              />
              {result.matches.some((m) => m.field === 'keyword') && (
                <div style={{ marginTop: '0.25rem', fontSize: '0.75rem', color: '#999' }}>
                  Keywords:{' '}
                  {result.matches
                    .filter((m) => m.field === 'keyword')
                    .map((m) => m.value)
                    .join(', ')}
                </div>
              )}
            </div>
          ))}
        </div>

        {/* Footer */}
        {results.length > 0 && (
          <div
            style={{
              padding: '0.75rem 1rem',
              borderTop: '1px solid #e0e0e0',
              fontSize: '0.75rem',
              color: '#666',
              textAlign: 'center',
            }}
          >
            {results.length} result{results.length !== 1 ? 's' : ''} found
          </div>
        )}
      </div>
    </div>
  );
};

export default SettingsSearch;
