/**
 * CADDY v0.4.0 Enterprise Settings Layout
 * Main tabbed interface for settings management
 */

import React, { useState, useCallback, useMemo, useEffect } from 'react';
import {
  SettingsTab,
  ToastNotification,
  ConfirmationDialog,
  SettingsHistory,
  AutoSaveState,
} from './types';
import GeneralSettings from './GeneralSettings';
import SecuritySettings from './SecuritySettings';
import NotificationSettings from './NotificationSettings';
import IntegrationSettings from './IntegrationSettings';
import BillingSettings from './BillingSettings';
import TeamSettings from './TeamSettings';
import AdvancedSettings from './AdvancedSettings';
import SettingsSearch from './SettingsSearch';

interface SettingsLayoutProps {
  defaultTab?: string;
  onSettingsChange?: (section: string, data: unknown) => void;
  userRole?: string;
}

const SettingsLayout: React.FC<SettingsLayoutProps> = ({
  defaultTab = 'general',
  onSettingsChange,
  userRole = 'admin',
}) => {
  const [activeTab, setActiveTab] = useState<string>(defaultTab);
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [toasts, setToasts] = useState<ToastNotification[]>([]);
  const [confirmDialog, setConfirmDialog] = useState<ConfirmationDialog>({
    open: false,
    title: '',
    message: '',
    severity: 'info',
    onConfirm: () => {},
    onCancel: () => {},
  });
  const [autoSaveState, setAutoSaveState] = useState<AutoSaveState>({
    saving: false,
  });
  const [history, setHistory] = useState<SettingsHistory[]>([]);
  const [showHistory, setShowHistory] = useState(false);

  // Define available tabs based on user role
  const tabs = useMemo<SettingsTab[]>(() => {
    const allTabs: SettingsTab[] = [
      {
        id: 'general',
        label: 'General',
        icon: 'settings',
        component: GeneralSettings,
      },
      {
        id: 'security',
        label: 'Security',
        icon: 'security',
        component: SecuritySettings,
      },
      {
        id: 'notifications',
        label: 'Notifications',
        icon: 'notifications',
        component: NotificationSettings,
      },
      {
        id: 'integrations',
        label: 'Integrations',
        icon: 'extension',
        component: IntegrationSettings,
      },
      {
        id: 'billing',
        label: 'Billing',
        icon: 'payment',
        component: BillingSettings,
        disabled: userRole !== 'admin' && userRole !== 'owner',
      },
      {
        id: 'team',
        label: 'Team',
        icon: 'people',
        component: TeamSettings,
      },
      {
        id: 'advanced',
        label: 'Advanced',
        icon: 'code',
        component: AdvancedSettings,
        disabled: userRole !== 'admin' && userRole !== 'developer',
      },
    ];

    return allTabs.filter((tab) => !tab.disabled);
  }, [userRole]);

  // Toast notification handlers
  const addToast = useCallback((toast: Omit<ToastNotification, 'id'>) => {
    const id = `toast-${Date.now()}-${Math.random()}`;
    setToasts((prev) => [...prev, { ...toast, id }]);

    const duration = toast.duration || 5000;
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  // Confirmation dialog handlers
  const showConfirmation = useCallback((config: Omit<ConfirmationDialog, 'open'>) => {
    setConfirmDialog({
      ...config,
      open: true,
    });
  }, []);

  const hideConfirmation = useCallback(() => {
    setConfirmDialog((prev) => ({ ...prev, open: false }));
  }, []);

  // Auto-save handler
  const handleAutoSave = useCallback(
    async (section: string, data: unknown) => {
      setAutoSaveState({ saving: true });

      try {
        // Simulate API call
        await new Promise((resolve) => setTimeout(resolve, 500));

        if (onSettingsChange) {
          onSettingsChange(section, data);
        }

        setAutoSaveState({
          saving: false,
          lastSaved: new Date(),
        });

        addToast({
          type: 'success',
          message: 'Settings saved successfully',
          duration: 3000,
        });
      } catch (error) {
        setAutoSaveState({
          saving: false,
          error: error instanceof Error ? error.message : 'Save failed',
        });

        addToast({
          type: 'error',
          message: 'Failed to save settings',
          duration: 5000,
        });
      }
    },
    [onSettingsChange, addToast]
  );

  // Add to history
  const addToHistory = useCallback((entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => {
    const historyEntry: SettingsHistory = {
      ...entry,
      id: `history-${Date.now()}`,
      timestamp: new Date(),
    };
    setHistory((prev) => [historyEntry, ...prev].slice(0, 100)); // Keep last 100 entries
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      // Ctrl/Cmd + K for search
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        setSearchOpen(true);
      }

      // Ctrl/Cmd + H for history
      if ((e.ctrlKey || e.metaKey) && e.key === 'h') {
        e.preventDefault();
        setShowHistory((prev) => !prev);
      }

      // Escape to close dialogs
      if (e.key === 'Escape') {
        setSearchOpen(false);
        setShowHistory(false);
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, []);

  // Get active component
  const ActiveComponent = useMemo(() => {
    const tab = tabs.find((t) => t.id === activeTab);
    return tab?.component || GeneralSettings;
  }, [activeTab, tabs]);

  return (
    <div
      className="settings-layout"
      role="main"
      aria-label="Settings"
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        backgroundColor: '#f5f5f5',
      }}
    >
      {/* Header */}
      <header
        style={{
          backgroundColor: '#fff',
          borderBottom: '1px solid #e0e0e0',
          padding: '1rem 2rem',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <h1
          style={{
            margin: 0,
            fontSize: '1.5rem',
            fontWeight: 600,
            color: '#1a1a1a',
          }}
        >
          Settings
        </h1>

        <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
          {/* Search Button */}
          <button
            onClick={() => setSearchOpen(true)}
            aria-label="Search settings (Ctrl+K)"
            title="Search settings (Ctrl+K)"
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#f5f5f5',
              border: '1px solid #e0e0e0',
              borderRadius: '4px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
            }}
          >
            <span aria-hidden="true">🔍</span>
            <span>Search</span>
            <kbd
              style={{
                backgroundColor: '#fff',
                border: '1px solid #d0d0d0',
                borderRadius: '2px',
                padding: '0.125rem 0.25rem',
                fontSize: '0.75rem',
              }}
            >
              ⌘K
            </kbd>
          </button>

          {/* History Button */}
          <button
            onClick={() => setShowHistory(!showHistory)}
            aria-label="View settings history (Ctrl+H)"
            title="View settings history (Ctrl+H)"
            style={{
              padding: '0.5rem',
              backgroundColor: showHistory ? '#1976d2' : '#f5f5f5',
              color: showHistory ? '#fff' : '#1a1a1a',
              border: '1px solid #e0e0e0',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            <span aria-hidden="true">📜</span>
          </button>

          {/* Auto-save indicator */}
          {autoSaveState.saving && (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '0.5rem',
                color: '#666',
                fontSize: '0.875rem',
              }}
              role="status"
              aria-live="polite"
            >
              <span aria-hidden="true">💾</span>
              <span>Saving...</span>
            </div>
          )}
          {autoSaveState.lastSaved && !autoSaveState.saving && (
            <div
              style={{
                color: '#666',
                fontSize: '0.875rem',
              }}
              role="status"
              aria-live="polite"
            >
              Last saved: {autoSaveState.lastSaved.toLocaleTimeString()}
            </div>
          )}
        </div>
      </header>

      {/* Main Content */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Sidebar Navigation */}
        <nav
          style={{
            width: '240px',
            backgroundColor: '#fff',
            borderRight: '1px solid #e0e0e0',
            overflowY: 'auto',
          }}
          aria-label="Settings navigation"
        >
          <ul
            style={{
              listStyle: 'none',
              margin: 0,
              padding: '1rem 0',
            }}
          >
            {tabs.map((tab) => (
              <li key={tab.id}>
                <button
                  onClick={() => setActiveTab(tab.id)}
                  disabled={tab.disabled}
                  aria-current={activeTab === tab.id ? 'page' : undefined}
                  style={{
                    width: '100%',
                    padding: '0.75rem 1.5rem',
                    backgroundColor: activeTab === tab.id ? '#e3f2fd' : 'transparent',
                    color: activeTab === tab.id ? '#1976d2' : '#333',
                    border: 'none',
                    borderLeft: activeTab === tab.id ? '3px solid #1976d2' : '3px solid transparent',
                    textAlign: 'left',
                    cursor: tab.disabled ? 'not-allowed' : 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '0.75rem',
                    fontSize: '0.9375rem',
                    fontWeight: activeTab === tab.id ? 600 : 400,
                    opacity: tab.disabled ? 0.5 : 1,
                  }}
                >
                  <span aria-hidden="true">{getIconForTab(tab.icon)}</span>
                  <span>{tab.label}</span>
                  {tab.badge !== undefined && (
                    <span
                      style={{
                        marginLeft: 'auto',
                        backgroundColor: '#f44336',
                        color: '#fff',
                        borderRadius: '10px',
                        padding: '0.125rem 0.5rem',
                        fontSize: '0.75rem',
                      }}
                      aria-label={`${tab.badge} notifications`}
                    >
                      {tab.badge}
                    </span>
                  )}
                </button>
              </li>
            ))}
          </ul>
        </nav>

        {/* Content Area */}
        <main
          style={{
            flex: 1,
            overflowY: 'auto',
            padding: '2rem',
          }}
        >
          <ActiveComponent
            onSave={handleAutoSave}
            onConfirm={showConfirmation}
            addToast={addToast}
            addToHistory={addToHistory}
          />
        </main>

        {/* History Sidebar */}
        {showHistory && (
          <aside
            style={{
              width: '320px',
              backgroundColor: '#fff',
              borderLeft: '1px solid #e0e0e0',
              overflowY: 'auto',
              padding: '1rem',
            }}
            aria-label="Settings history"
          >
            <h2 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>
              Change History
            </h2>
            {history.length === 0 ? (
              <p style={{ color: '#666', fontSize: '0.875rem' }}>
                No changes recorded yet
              </p>
            ) : (
              <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
                {history.map((entry) => (
                  <li
                    key={entry.id}
                    style={{
                      padding: '0.75rem',
                      borderBottom: '1px solid #f0f0f0',
                      fontSize: '0.875rem',
                    }}
                  >
                    <div style={{ fontWeight: 600, marginBottom: '0.25rem' }}>
                      {entry.section}
                    </div>
                    <div style={{ color: '#666', marginBottom: '0.25rem' }}>
                      {entry.userName} • {entry.action}
                    </div>
                    <div style={{ color: '#999', fontSize: '0.75rem' }}>
                      {entry.timestamp.toLocaleString()}
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </aside>
        )}
      </div>

      {/* Search Modal */}
      {searchOpen && (
        <SettingsSearch
          query={searchQuery}
          onQueryChange={setSearchQuery}
          onClose={() => setSearchOpen(false)}
          onNavigate={(path) => {
            setActiveTab(path);
            setSearchOpen(false);
          }}
        />
      )}

      {/* Confirmation Dialog */}
      {confirmDialog.open && (
        <div
          role="alertdialog"
          aria-labelledby="confirm-dialog-title"
          aria-describedby="confirm-dialog-description"
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
        >
          <div
            style={{
              backgroundColor: '#fff',
              borderRadius: '8px',
              padding: '2rem',
              maxWidth: '500px',
              width: '90%',
            }}
          >
            <h2
              id="confirm-dialog-title"
              style={{
                margin: '0 0 1rem 0',
                fontSize: '1.25rem',
                color: confirmDialog.severity === 'error' ? '#d32f2f' : '#1a1a1a',
              }}
            >
              {confirmDialog.title}
            </h2>
            <p
              id="confirm-dialog-description"
              style={{ margin: '0 0 1.5rem 0', color: '#666' }}
            >
              {confirmDialog.message}
            </p>
            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
              <button
                onClick={() => {
                  confirmDialog.onCancel();
                  hideConfirmation();
                }}
                style={{
                  padding: '0.5rem 1.5rem',
                  backgroundColor: '#f5f5f5',
                  border: '1px solid #e0e0e0',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                {confirmDialog.cancelText || 'Cancel'}
              </button>
              <button
                onClick={() => {
                  confirmDialog.onConfirm();
                  hideConfirmation();
                }}
                style={{
                  padding: '0.5rem 1.5rem',
                  backgroundColor:
                    confirmDialog.severity === 'error' ? '#d32f2f' : '#1976d2',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                {confirmDialog.confirmText || 'Confirm'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Toast Notifications */}
      <div
        style={{
          position: 'fixed',
          bottom: '1rem',
          right: '1rem',
          zIndex: 1100,
          display: 'flex',
          flexDirection: 'column',
          gap: '0.5rem',
        }}
        role="region"
        aria-label="Notifications"
        aria-live="polite"
      >
        {toasts.map((toast) => (
          <div
            key={toast.id}
            style={{
              backgroundColor: getToastColor(toast.type),
              color: '#fff',
              padding: '1rem',
              borderRadius: '4px',
              boxShadow: '0 2px 8px rgba(0, 0, 0, 0.2)',
              display: 'flex',
              alignItems: 'center',
              gap: '0.75rem',
              minWidth: '300px',
            }}
          >
            <span aria-hidden="true">{getToastIcon(toast.type)}</span>
            <span style={{ flex: 1 }}>{toast.message}</span>
            {toast.action && (
              <button
                onClick={toast.action.onClick}
                style={{
                  backgroundColor: 'rgba(255, 255, 255, 0.2)',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '2px',
                  padding: '0.25rem 0.5rem',
                  cursor: 'pointer',
                }}
              >
                {toast.action.label}
              </button>
            )}
            <button
              onClick={() => removeToast(toast.id)}
              aria-label="Dismiss notification"
              style={{
                backgroundColor: 'transparent',
                color: '#fff',
                border: 'none',
                cursor: 'pointer',
                fontSize: '1.25rem',
              }}
            >
              ×
            </button>
          </div>
        ))}
      </div>
    </div>
  );
};

// Helper functions
function getIconForTab(icon: string): string {
  const icons: Record<string, string> = {
    settings: '⚙️',
    security: '🔒',
    notifications: '🔔',
    extension: '🧩',
    payment: '💳',
    people: '👥',
    code: '💻',
  };
  return icons[icon] || '📄';
}

function getToastColor(type: ToastNotification['type']): string {
  const colors: Record<string, string> = {
    success: '#4caf50',
    error: '#f44336',
    warning: '#ff9800',
    info: '#2196f3',
  };
  return colors[type] || colors.info;
}

function getToastIcon(type: ToastNotification['type']): string {
  const icons: Record<string, string> = {
    success: '✓',
    error: '✗',
    warning: '⚠',
    info: 'ℹ',
  };
  return icons[type] || icons.info;
}

export default SettingsLayout;
