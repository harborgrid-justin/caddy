import React, { useState } from 'react';
import { NotificationList } from './NotificationList';
import { NotificationPreferences } from './NotificationPreferences';
import { NotificationHistory } from './NotificationHistory';
import { useNotificationStats, useDoNotDisturb } from './useNotifications';
export const NotificationCenter = ({ defaultTab = 'notifications', showTabs = true }) => {
    const [activeTab, setActiveTab] = useState(defaultTab);
    const stats = useNotificationStats();
    const { enabled: dndEnabled, isActive: dndActive, toggle: toggleDND } = useDoNotDisturb();
    return (React.createElement("div", { style: {
            display: 'flex',
            flexDirection: 'column',
            height: '100vh',
            backgroundColor: '#ffffff'
        } },
        React.createElement("div", { style: {
                padding: '16px 24px',
                borderBottom: '1px solid #e5e7eb',
                backgroundColor: '#ffffff'
            } },
            React.createElement("div", { style: { display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' } },
                React.createElement("div", null,
                    React.createElement("h1", { style: { margin: '0 0 4px 0', fontSize: '24px', fontWeight: '700', color: '#111827' } }, "Notification Center"),
                    React.createElement("p", { style: { margin: 0, fontSize: '14px', color: '#6b7280' } }, "Manage all your notifications and preferences")),
                React.createElement("button", { onClick: toggleDND, style: {
                        display: 'flex',
                        alignItems: 'center',
                        gap: '8px',
                        padding: '8px 16px',
                        fontSize: '14px',
                        fontWeight: '500',
                        border: '1px solid #d1d5db',
                        borderRadius: '8px',
                        backgroundColor: dndActive ? '#fef2f2' : '#ffffff',
                        color: dndActive ? '#dc2626' : '#374151',
                        cursor: 'pointer',
                        transition: 'all 0.2s'
                    } },
                    React.createElement("span", null, dndActive ? '🔕' : '🔔'),
                    React.createElement("span", null, dndActive ? 'Do Not Disturb' : 'Notifications On'))),
            React.createElement("div", { style: { display: 'flex', gap: '16px', flexWrap: 'wrap' } },
                React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                    React.createElement("div", { style: {
                            width: '8px',
                            height: '8px',
                            borderRadius: '50%',
                            backgroundColor: '#3b82f6'
                        } }),
                    React.createElement("span", { style: { fontSize: '13px', color: '#6b7280' } },
                        React.createElement("strong", { style: { color: '#111827', fontWeight: '600' } }, stats.unread),
                        " unread")),
                React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                    React.createElement("div", { style: {
                            width: '8px',
                            height: '8px',
                            borderRadius: '50%',
                            backgroundColor: '#10b981'
                        } }),
                    React.createElement("span", { style: { fontSize: '13px', color: '#6b7280' } },
                        React.createElement("strong", { style: { color: '#111827', fontWeight: '600' } }, stats.todayCount),
                        " today")),
                React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                    React.createElement("div", { style: {
                            width: '8px',
                            height: '8px',
                            borderRadius: '50%',
                            backgroundColor: '#f59e0b'
                        } }),
                    React.createElement("span", { style: { fontSize: '13px', color: '#6b7280' } },
                        React.createElement("strong", { style: { color: '#111827', fontWeight: '600' } }, stats.weekCount),
                        " this week")),
                React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                    React.createElement("div", { style: {
                            width: '8px',
                            height: '8px',
                            borderRadius: '50%',
                            backgroundColor: '#6b7280'
                        } }),
                    React.createElement("span", { style: { fontSize: '13px', color: '#6b7280' } },
                        React.createElement("strong", { style: { color: '#111827', fontWeight: '600' } }, stats.total),
                        " total"))),
            showTabs && (React.createElement("div", { style: { display: 'flex', gap: '4px', marginTop: '16px' } },
                React.createElement("button", { onClick: () => setActiveTab('notifications'), style: {
                        padding: '8px 16px',
                        fontSize: '14px',
                        fontWeight: '500',
                        border: 'none',
                        borderBottom: activeTab === 'notifications' ? '2px solid #3b82f6' : '2px solid transparent',
                        backgroundColor: 'transparent',
                        color: activeTab === 'notifications' ? '#3b82f6' : '#6b7280',
                        cursor: 'pointer',
                        transition: 'all 0.2s'
                    } }, "Notifications"),
                React.createElement("button", { onClick: () => setActiveTab('preferences'), style: {
                        padding: '8px 16px',
                        fontSize: '14px',
                        fontWeight: '500',
                        border: 'none',
                        borderBottom: activeTab === 'preferences' ? '2px solid #3b82f6' : '2px solid transparent',
                        backgroundColor: 'transparent',
                        color: activeTab === 'preferences' ? '#3b82f6' : '#6b7280',
                        cursor: 'pointer',
                        transition: 'all 0.2s'
                    } }, "Preferences"),
                React.createElement("button", { onClick: () => setActiveTab('history'), style: {
                        padding: '8px 16px',
                        fontSize: '14px',
                        fontWeight: '500',
                        border: 'none',
                        borderBottom: activeTab === 'history' ? '2px solid #3b82f6' : '2px solid transparent',
                        backgroundColor: 'transparent',
                        color: activeTab === 'history' ? '#3b82f6' : '#6b7280',
                        cursor: 'pointer',
                        transition: 'all 0.2s'
                    } }, "History")))),
        React.createElement("div", { style: { flex: 1, overflow: 'hidden' } },
            activeTab === 'notifications' && (React.createElement(NotificationList, { showFilters: true, showGrouping: true, showBulkActions: true, maxHeight: "100%" })),
            activeTab === 'preferences' && (React.createElement("div", { style: { height: '100%', overflowY: 'auto' } },
                React.createElement(NotificationPreferences, null))),
            activeTab === 'history' && (React.createElement("div", { style: { height: '100%', overflowY: 'auto' } },
                React.createElement(NotificationHistory, null))))));
};
//# sourceMappingURL=NotificationCenter.js.map