import React, { useState, useCallback, useMemo, useEffect } from 'react';
import GeneralSettings from './GeneralSettings';
import SecuritySettings from './SecuritySettings';
import NotificationSettings from './NotificationSettings';
import IntegrationSettings from './IntegrationSettings';
import BillingSettings from './BillingSettings';
import TeamSettings from './TeamSettings';
import AdvancedSettings from './AdvancedSettings';
import SettingsSearch from './SettingsSearch';
const SettingsLayout = ({ defaultTab = 'general', onSettingsChange, userRole = 'admin', }) => {
    const [activeTab, setActiveTab] = useState(defaultTab);
    const [searchOpen, setSearchOpen] = useState(false);
    const [searchQuery, setSearchQuery] = useState('');
    const [toasts, setToasts] = useState([]);
    const [confirmDialog, setConfirmDialog] = useState({
        open: false,
        title: '',
        message: '',
        severity: 'info',
        onConfirm: () => { },
        onCancel: () => { },
    });
    const [autoSaveState, setAutoSaveState] = useState({
        saving: false,
    });
    const [history, setHistory] = useState([]);
    const [showHistory, setShowHistory] = useState(false);
    const tabs = useMemo(() => {
        const allTabs = [
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
    const addToast = useCallback((toast) => {
        const id = `toast-${Date.now()}-${Math.random()}`;
        setToasts((prev) => [...prev, { ...toast, id }]);
        const duration = toast.duration || 5000;
        setTimeout(() => {
            removeToast(id);
        }, duration);
    }, []);
    const removeToast = useCallback((id) => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
    }, []);
    const showConfirmation = useCallback((config) => {
        setConfirmDialog({
            ...config,
            open: true,
        });
    }, []);
    const hideConfirmation = useCallback(() => {
        setConfirmDialog((prev) => ({ ...prev, open: false }));
    }, []);
    const handleAutoSave = useCallback(async (section, data) => {
        setAutoSaveState({ saving: true });
        try {
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
        }
        catch (error) {
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
    }, [onSettingsChange, addToast]);
    const addToHistory = useCallback((entry) => {
        const historyEntry = {
            ...entry,
            id: `history-${Date.now()}`,
            timestamp: new Date(),
        };
        setHistory((prev) => [historyEntry, ...prev].slice(0, 100));
    }, []);
    useEffect(() => {
        const handleKeyPress = (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                setSearchOpen(true);
            }
            if ((e.ctrlKey || e.metaKey) && e.key === 'h') {
                e.preventDefault();
                setShowHistory((prev) => !prev);
            }
            if (e.key === 'Escape') {
                setSearchOpen(false);
                setShowHistory(false);
            }
        };
        window.addEventListener('keydown', handleKeyPress);
        return () => window.removeEventListener('keydown', handleKeyPress);
    }, []);
    const ActiveComponent = useMemo(() => {
        const tab = tabs.find((t) => t.id === activeTab);
        return tab?.component || GeneralSettings;
    }, [activeTab, tabs]);
    return (React.createElement("div", { className: "settings-layout", role: "main", "aria-label": "Settings", style: {
            display: 'flex',
            flexDirection: 'column',
            height: '100vh',
            backgroundColor: '#f5f5f5',
        } },
        React.createElement("header", { style: {
                backgroundColor: '#fff',
                borderBottom: '1px solid #e0e0e0',
                padding: '1rem 2rem',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
            } },
            React.createElement("h1", { style: {
                    margin: 0,
                    fontSize: '1.5rem',
                    fontWeight: 600,
                    color: '#1a1a1a',
                } }, "Settings"),
            React.createElement("div", { style: { display: 'flex', gap: '1rem', alignItems: 'center' } },
                React.createElement("button", { onClick: () => setSearchOpen(true), "aria-label": "Search settings (Ctrl+K)", title: "Search settings (Ctrl+K)", style: {
                        padding: '0.5rem 1rem',
                        backgroundColor: '#f5f5f5',
                        border: '1px solid #e0e0e0',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '0.5rem',
                    } },
                    React.createElement("span", { "aria-hidden": "true" }, "\uD83D\uDD0D"),
                    React.createElement("span", null, "Search"),
                    React.createElement("kbd", { style: {
                            backgroundColor: '#fff',
                            border: '1px solid #d0d0d0',
                            borderRadius: '2px',
                            padding: '0.125rem 0.25rem',
                            fontSize: '0.75rem',
                        } }, "\u2318K")),
                React.createElement("button", { onClick: () => setShowHistory(!showHistory), "aria-label": "View settings history (Ctrl+H)", title: "View settings history (Ctrl+H)", style: {
                        padding: '0.5rem',
                        backgroundColor: showHistory ? '#1976d2' : '#f5f5f5',
                        color: showHistory ? '#fff' : '#1a1a1a',
                        border: '1px solid #e0e0e0',
                        borderRadius: '4px',
                        cursor: 'pointer',
                    } },
                    React.createElement("span", { "aria-hidden": "true" }, "\uD83D\uDCDC")),
                autoSaveState.saving && (React.createElement("div", { style: {
                        display: 'flex',
                        alignItems: 'center',
                        gap: '0.5rem',
                        color: '#666',
                        fontSize: '0.875rem',
                    }, role: "status", "aria-live": "polite" },
                    React.createElement("span", { "aria-hidden": "true" }, "\uD83D\uDCBE"),
                    React.createElement("span", null, "Saving..."))),
                autoSaveState.lastSaved && !autoSaveState.saving && (React.createElement("div", { style: {
                        color: '#666',
                        fontSize: '0.875rem',
                    }, role: "status", "aria-live": "polite" },
                    "Last saved: ",
                    autoSaveState.lastSaved.toLocaleTimeString())))),
        React.createElement("div", { style: { display: 'flex', flex: 1, overflow: 'hidden' } },
            React.createElement("nav", { style: {
                    width: '240px',
                    backgroundColor: '#fff',
                    borderRight: '1px solid #e0e0e0',
                    overflowY: 'auto',
                }, "aria-label": "Settings navigation" },
                React.createElement("ul", { style: {
                        listStyle: 'none',
                        margin: 0,
                        padding: '1rem 0',
                    } }, tabs.map((tab) => (React.createElement("li", { key: tab.id },
                    React.createElement("button", { onClick: () => setActiveTab(tab.id), disabled: tab.disabled, "aria-current": activeTab === tab.id ? 'page' : undefined, style: {
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
                        } },
                        React.createElement("span", { "aria-hidden": "true" }, getIconForTab(tab.icon)),
                        React.createElement("span", null, tab.label),
                        tab.badge !== undefined && (React.createElement("span", { style: {
                                marginLeft: 'auto',
                                backgroundColor: '#f44336',
                                color: '#fff',
                                borderRadius: '10px',
                                padding: '0.125rem 0.5rem',
                                fontSize: '0.75rem',
                            }, "aria-label": `${tab.badge} notifications` }, tab.badge)))))))),
            React.createElement("main", { style: {
                    flex: 1,
                    overflowY: 'auto',
                    padding: '2rem',
                } },
                React.createElement(ActiveComponent, { onSave: handleAutoSave, onConfirm: showConfirmation, addToast: addToast, addToHistory: addToHistory })),
            showHistory && (React.createElement("aside", { style: {
                    width: '320px',
                    backgroundColor: '#fff',
                    borderLeft: '1px solid #e0e0e0',
                    overflowY: 'auto',
                    padding: '1rem',
                }, "aria-label": "Settings history" },
                React.createElement("h2", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Change History"),
                history.length === 0 ? (React.createElement("p", { style: { color: '#666', fontSize: '0.875rem' } }, "No changes recorded yet")) : (React.createElement("ul", { style: { listStyle: 'none', padding: 0, margin: 0 } }, history.map((entry) => (React.createElement("li", { key: entry.id, style: {
                        padding: '0.75rem',
                        borderBottom: '1px solid #f0f0f0',
                        fontSize: '0.875rem',
                    } },
                    React.createElement("div", { style: { fontWeight: 600, marginBottom: '0.25rem' } }, entry.section),
                    React.createElement("div", { style: { color: '#666', marginBottom: '0.25rem' } },
                        entry.userName,
                        " \u2022 ",
                        entry.action),
                    React.createElement("div", { style: { color: '#999', fontSize: '0.75rem' } }, entry.timestamp.toLocaleString()))))))))),
        searchOpen && (React.createElement(SettingsSearch, { query: searchQuery, onQueryChange: setSearchQuery, onClose: () => setSearchOpen(false), onNavigate: (path) => {
                setActiveTab(path);
                setSearchOpen(false);
            } })),
        confirmDialog.open && (React.createElement("div", { role: "alertdialog", "aria-labelledby": "confirm-dialog-title", "aria-describedby": "confirm-dialog-description", style: {
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
            } },
            React.createElement("div", { style: {
                    backgroundColor: '#fff',
                    borderRadius: '8px',
                    padding: '2rem',
                    maxWidth: '500px',
                    width: '90%',
                } },
                React.createElement("h2", { id: "confirm-dialog-title", style: {
                        margin: '0 0 1rem 0',
                        fontSize: '1.25rem',
                        color: confirmDialog.severity === 'error' ? '#d32f2f' : '#1a1a1a',
                    } }, confirmDialog.title),
                React.createElement("p", { id: "confirm-dialog-description", style: { margin: '0 0 1.5rem 0', color: '#666' } }, confirmDialog.message),
                React.createElement("div", { style: { display: 'flex', gap: '1rem', justifyContent: 'flex-end' } },
                    React.createElement("button", { onClick: () => {
                            confirmDialog.onCancel();
                            hideConfirmation();
                        }, style: {
                            padding: '0.5rem 1.5rem',
                            backgroundColor: '#f5f5f5',
                            border: '1px solid #e0e0e0',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, confirmDialog.cancelText || 'Cancel'),
                    React.createElement("button", { onClick: () => {
                            confirmDialog.onConfirm();
                            hideConfirmation();
                        }, style: {
                            padding: '0.5rem 1.5rem',
                            backgroundColor: confirmDialog.severity === 'error' ? '#d32f2f' : '#1976d2',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, confirmDialog.confirmText || 'Confirm'))))),
        React.createElement("div", { style: {
                position: 'fixed',
                bottom: '1rem',
                right: '1rem',
                zIndex: 1100,
                display: 'flex',
                flexDirection: 'column',
                gap: '0.5rem',
            }, role: "region", "aria-label": "Notifications", "aria-live": "polite" }, toasts.map((toast) => (React.createElement("div", { key: toast.id, style: {
                backgroundColor: getToastColor(toast.type),
                color: '#fff',
                padding: '1rem',
                borderRadius: '4px',
                boxShadow: '0 2px 8px rgba(0, 0, 0, 0.2)',
                display: 'flex',
                alignItems: 'center',
                gap: '0.75rem',
                minWidth: '300px',
            } },
            React.createElement("span", { "aria-hidden": "true" }, getToastIcon(toast.type)),
            React.createElement("span", { style: { flex: 1 } }, toast.message),
            toast.action && (React.createElement("button", { onClick: toast.action.onClick, style: {
                    backgroundColor: 'rgba(255, 255, 255, 0.2)',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '2px',
                    padding: '0.25rem 0.5rem',
                    cursor: 'pointer',
                } }, toast.action.label)),
            React.createElement("button", { onClick: () => removeToast(toast.id), "aria-label": "Dismiss notification", style: {
                    backgroundColor: 'transparent',
                    color: '#fff',
                    border: 'none',
                    cursor: 'pointer',
                    fontSize: '1.25rem',
                } }, "\u00D7")))))));
};
function getIconForTab(icon) {
    const icons = {
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
function getToastColor(type) {
    const colors = {
        success: '#4caf50',
        error: '#f44336',
        warning: '#ff9800',
        info: '#2196f3',
    };
    return colors[type] || colors.info;
}
function getToastIcon(type) {
    const icons = {
        success: '✓',
        error: '✗',
        warning: '⚠',
        info: 'ℹ',
    };
    return icons[type] || icons.info;
}
export default SettingsLayout;
//# sourceMappingURL=SettingsLayout.js.map