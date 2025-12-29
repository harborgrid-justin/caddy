/**
 * CADDY v0.4.0 - Notification Center
 * Central notification hub with advanced features
 */

import React, { useState } from 'react';
import { NotificationList } from './NotificationList';
import { NotificationPreferences } from './NotificationPreferences';
import { NotificationHistory } from './NotificationHistory';
import { useNotifications, useNotificationStats, useDoNotDisturb } from './useNotifications';

type Tab = 'notifications' | 'preferences' | 'history';

interface NotificationCenterProps {
  defaultTab?: Tab;
  showTabs?: boolean;
}

export const NotificationCenter: React.FC<NotificationCenterProps> = ({
  defaultTab = 'notifications',
  showTabs = true
}) => {
  const [activeTab, setActiveTab] = useState<Tab>(defaultTab);
  const stats = useNotificationStats();
  const { enabled: dndEnabled, isActive: dndActive, toggle: toggleDND } = useDoNotDisturb();

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        backgroundColor: '#ffffff'
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: '16px 24px',
          borderBottom: '1px solid #e5e7eb',
          backgroundColor: '#ffffff'
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' }}>
          <div>
            <h1 style={{ margin: '0 0 4px 0', fontSize: '24px', fontWeight: '700', color: '#111827' }}>
              Notification Center
            </h1>
            <p style={{ margin: 0, fontSize: '14px', color: '#6b7280' }}>
              Manage all your notifications and preferences
            </p>
          </div>

          {/* DND Toggle */}
          <button
            onClick={toggleDND}
            style={{
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
            }}
          >
            <span>{dndActive ? '🔕' : '🔔'}</span>
            <span>{dndActive ? 'Do Not Disturb' : 'Notifications On'}</span>
          </button>
        </div>

        {/* Stats */}
        <div style={{ display: 'flex', gap: '16px', flexWrap: 'wrap' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div
              style={{
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                backgroundColor: '#3b82f6'
              }}
            />
            <span style={{ fontSize: '13px', color: '#6b7280' }}>
              <strong style={{ color: '#111827', fontWeight: '600' }}>{stats.unread}</strong> unread
            </span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div
              style={{
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                backgroundColor: '#10b981'
              }}
            />
            <span style={{ fontSize: '13px', color: '#6b7280' }}>
              <strong style={{ color: '#111827', fontWeight: '600' }}>{stats.todayCount}</strong> today
            </span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div
              style={{
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                backgroundColor: '#f59e0b'
              }}
            />
            <span style={{ fontSize: '13px', color: '#6b7280' }}>
              <strong style={{ color: '#111827', fontWeight: '600' }}>{stats.weekCount}</strong> this week
            </span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div
              style={{
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                backgroundColor: '#6b7280'
              }}
            />
            <span style={{ fontSize: '13px', color: '#6b7280' }}>
              <strong style={{ color: '#111827', fontWeight: '600' }}>{stats.total}</strong> total
            </span>
          </div>
        </div>

        {/* Tabs */}
        {showTabs && (
          <div style={{ display: 'flex', gap: '4px', marginTop: '16px' }}>
            <button
              onClick={() => setActiveTab('notifications')}
              style={{
                padding: '8px 16px',
                fontSize: '14px',
                fontWeight: '500',
                border: 'none',
                borderBottom: activeTab === 'notifications' ? '2px solid #3b82f6' : '2px solid transparent',
                backgroundColor: 'transparent',
                color: activeTab === 'notifications' ? '#3b82f6' : '#6b7280',
                cursor: 'pointer',
                transition: 'all 0.2s'
              }}
            >
              Notifications
            </button>
            <button
              onClick={() => setActiveTab('preferences')}
              style={{
                padding: '8px 16px',
                fontSize: '14px',
                fontWeight: '500',
                border: 'none',
                borderBottom: activeTab === 'preferences' ? '2px solid #3b82f6' : '2px solid transparent',
                backgroundColor: 'transparent',
                color: activeTab === 'preferences' ? '#3b82f6' : '#6b7280',
                cursor: 'pointer',
                transition: 'all 0.2s'
              }}
            >
              Preferences
            </button>
            <button
              onClick={() => setActiveTab('history')}
              style={{
                padding: '8px 16px',
                fontSize: '14px',
                fontWeight: '500',
                border: 'none',
                borderBottom: activeTab === 'history' ? '2px solid #3b82f6' : '2px solid transparent',
                backgroundColor: 'transparent',
                color: activeTab === 'history' ? '#3b82f6' : '#6b7280',
                cursor: 'pointer',
                transition: 'all 0.2s'
              }}
            >
              History
            </button>
          </div>
        )}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: 'hidden' }}>
        {activeTab === 'notifications' && (
          <NotificationList
            showFilters={true}
            showGrouping={true}
            showBulkActions={true}
            maxHeight="100%"
          />
        )}

        {activeTab === 'preferences' && (
          <div style={{ height: '100%', overflowY: 'auto' }}>
            <NotificationPreferences />
          </div>
        )}

        {activeTab === 'history' && (
          <div style={{ height: '100%', overflowY: 'auto' }}>
            <NotificationHistory />
          </div>
        )}
      </div>
    </div>
  );
};
