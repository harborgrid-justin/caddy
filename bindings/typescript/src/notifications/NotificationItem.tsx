/**
 * CADDY v0.4.0 - Notification Item Component
 * Individual notification display with actions
 */

import React, { useState, useCallback } from 'react';
import {
  Notification,
  NotificationPriority,
  NotificationType,
  NotificationStatus,
  NotificationAction
} from './types';
import { useNotifications } from './useNotifications';

interface NotificationItemProps {
  notification: Notification;
  compact?: boolean;
  showActions?: boolean;
  onSelect?: (id: string) => void;
  selected?: boolean;
}

export const NotificationItem: React.FC<NotificationItemProps> = ({
  notification,
  compact = false,
  showActions = true,
  onSelect,
  selected = false
}) => {
  const { markAsRead, markAsUnread, archiveNotification, deleteNotification, executeAction } = useNotifications();
  const [isExecuting, setIsExecuting] = useState<string | null>(null);
  const [showConfirmation, setShowConfirmation] = useState<{ actionId: string; message: string } | null>(null);

  const isUnread = notification.status !== NotificationStatus.READ && notification.status !== NotificationStatus.ARCHIVED;

  const handleMarkAsRead = useCallback(async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (isUnread) {
      await markAsRead(notification.id);
    } else {
      await markAsUnread(notification.id);
    }
  }, [notification.id, isUnread, markAsRead, markAsUnread]);

  const handleArchive = useCallback(async (e: React.MouseEvent) => {
    e.stopPropagation();
    await archiveNotification(notification.id);
  }, [notification.id, archiveNotification]);

  const handleDelete = useCallback(async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (window.confirm('Are you sure you want to delete this notification?')) {
      await deleteNotification(notification.id);
    }
  }, [notification.id, deleteNotification]);

  const handleActionClick = useCallback(async (action: NotificationAction, e: React.MouseEvent) => {
    e.stopPropagation();

    if (action.requiresConfirmation) {
      setShowConfirmation({
        actionId: action.id,
        message: action.confirmationMessage || `Are you sure you want to ${action.label}?`
      });
      return;
    }

    await executeActionInternal(action.id);
  }, []);

  const executeActionInternal = useCallback(async (actionId: string) => {
    setIsExecuting(actionId);
    try {
      await executeAction(notification.id, actionId);
      await markAsRead(notification.id);
    } catch (err) {
      console.error('Error executing action:', err);
      alert('Failed to execute action. Please try again.');
    } finally {
      setIsExecuting(null);
      setShowConfirmation(null);
    }
  }, [notification.id, executeAction, markAsRead]);

  const handleConfirm = useCallback(() => {
    if (showConfirmation) {
      executeActionInternal(showConfirmation.actionId);
    }
  }, [showConfirmation, executeActionInternal]);

  const handleCancel = useCallback(() => {
    setShowConfirmation(null);
  }, []);

  const handleClick = useCallback(() => {
    if (!isUnread) return;
    markAsRead(notification.id);
    if (notification.metadata?.url) {
      window.location.href = notification.metadata.url;
    }
  }, [notification, isUnread, markAsRead]);

  const getPriorityColor = (priority: NotificationPriority): string => {
    switch (priority) {
      case NotificationPriority.CRITICAL:
        return '#dc2626';
      case NotificationPriority.URGENT:
        return '#ea580c';
      case NotificationPriority.HIGH:
        return '#f59e0b';
      case NotificationPriority.MEDIUM:
        return '#3b82f6';
      case NotificationPriority.LOW:
        return '#6b7280';
      default:
        return '#6b7280';
    }
  };

  const getTypeIcon = (type: NotificationType): string => {
    switch (type) {
      case NotificationType.SUCCESS:
        return '✓';
      case NotificationType.ERROR:
        return '✕';
      case NotificationType.WARNING:
        return '⚠';
      case NotificationType.INFO:
        return 'ℹ';
      case NotificationType.TASK:
        return '📋';
      case NotificationType.MENTION:
        return '@';
      case NotificationType.COMMENT:
        return '💬';
      case NotificationType.APPROVAL:
        return '✔';
      case NotificationType.REMINDER:
        return '🔔';
      case NotificationType.ALERT:
        return '🚨';
      case NotificationType.SYSTEM:
        return '⚙';
      default:
        return '•';
    }
  };

  const formatTimestamp = (date: Date): string => {
    const now = new Date();
    const diff = now.getTime() - new Date(date).getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return new Date(date).toLocaleDateString();
  };

  return (
    <>
      <div
        onClick={handleClick}
        className={`notification-item ${isUnread ? 'unread' : ''} ${selected ? 'selected' : ''} ${compact ? 'compact' : ''}`}
        style={{
          position: 'relative',
          display: 'flex',
          gap: '12px',
          padding: compact ? '8px 12px' : '12px 16px',
          backgroundColor: isUnread ? '#f0f9ff' : '#ffffff',
          borderLeft: `4px solid ${getPriorityColor(notification.priority)}`,
          borderBottom: '1px solid #e5e7eb',
          cursor: notification.metadata?.url ? 'pointer' : 'default',
          transition: 'all 0.2s ease',
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = isUnread ? '#e0f2fe' : '#f9fafb';
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = isUnread ? '#f0f9ff' : '#ffffff';
        }}
      >
        {/* Selection checkbox */}
        {onSelect && (
          <input
            type="checkbox"
            checked={selected}
            onChange={() => onSelect(notification.id)}
            onClick={(e) => e.stopPropagation()}
            style={{
              width: '16px',
              height: '16px',
              marginTop: '4px',
              cursor: 'pointer'
            }}
          />
        )}

        {/* Icon/Avatar */}
        <div
          style={{
            flexShrink: 0,
            width: compact ? '32px' : '40px',
            height: compact ? '32px' : '40px',
            borderRadius: '50%',
            backgroundColor: notification.metadata?.avatarUrl ? 'transparent' : '#e5e7eb',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: compact ? '16px' : '20px',
            fontWeight: 'bold',
            color: getPriorityColor(notification.priority),
            backgroundImage: notification.metadata?.avatarUrl ? `url(${notification.metadata.avatarUrl})` : undefined,
            backgroundSize: 'cover',
            backgroundPosition: 'center'
          }}
        >
          {!notification.metadata?.avatarUrl && getTypeIcon(notification.type)}
        </div>

        {/* Content */}
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: '8px' }}>
            <div style={{ flex: 1, minWidth: 0 }}>
              <h4
                style={{
                  margin: 0,
                  fontSize: compact ? '13px' : '14px',
                  fontWeight: isUnread ? '600' : '500',
                  color: '#111827',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: compact ? 'nowrap' : 'normal'
                }}
              >
                {notification.title}
              </h4>
              <p
                style={{
                  margin: '4px 0 0 0',
                  fontSize: compact ? '12px' : '13px',
                  color: '#6b7280',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  display: '-webkit-box',
                  WebkitLineClamp: compact ? 1 : 2,
                  WebkitBoxOrient: 'vertical'
                }}
              >
                {compact && notification.shortMessage ? notification.shortMessage : notification.message}
              </p>
            </div>

            {/* Timestamp */}
            <span
              style={{
                fontSize: '11px',
                color: '#9ca3af',
                whiteSpace: 'nowrap',
                flexShrink: 0
              }}
            >
              {formatTimestamp(notification.createdAt)}
            </span>
          </div>

          {/* Metadata tags */}
          {!compact && notification.metadata?.tags && notification.metadata.tags.length > 0 && (
            <div style={{ display: 'flex', gap: '4px', marginTop: '8px', flexWrap: 'wrap' }}>
              {notification.metadata.tags.map((tag, index) => (
                <span
                  key={index}
                  style={{
                    padding: '2px 8px',
                    fontSize: '11px',
                    backgroundColor: '#e5e7eb',
                    color: '#4b5563',
                    borderRadius: '12px',
                    fontWeight: '500'
                  }}
                >
                  {tag}
                </span>
              ))}
            </div>
          )}

          {/* Actions */}
          {showActions && notification.actions && notification.actions.length > 0 && (
            <div style={{ display: 'flex', gap: '8px', marginTop: '12px', flexWrap: 'wrap' }}>
              {notification.actions.map((action) => (
                <button
                  key={action.id}
                  onClick={(e) => handleActionClick(action, e)}
                  disabled={isExecuting === action.id}
                  style={{
                    padding: '6px 12px',
                    fontSize: '12px',
                    fontWeight: '500',
                    border: action.type === 'primary' ? 'none' : '1px solid #d1d5db',
                    borderRadius: '6px',
                    backgroundColor: action.type === 'primary' ? '#3b82f6' : action.type === 'danger' ? '#dc2626' : '#ffffff',
                    color: action.type === 'primary' || action.type === 'danger' ? '#ffffff' : '#374151',
                    cursor: isExecuting === action.id ? 'not-allowed' : 'pointer',
                    opacity: isExecuting === action.id ? 0.6 : 1,
                    transition: 'all 0.2s ease'
                  }}
                >
                  {isExecuting === action.id ? 'Processing...' : action.label}
                </button>
              ))}
            </div>
          )}

          {/* Quick actions */}
          {showActions && !compact && (
            <div style={{ display: 'flex', gap: '12px', marginTop: '8px' }}>
              <button
                onClick={handleMarkAsRead}
                style={{
                  padding: '4px',
                  fontSize: '11px',
                  color: '#6b7280',
                  background: 'none',
                  border: 'none',
                  cursor: 'pointer',
                  textDecoration: 'underline'
                }}
              >
                {isUnread ? 'Mark as read' : 'Mark as unread'}
              </button>
              <button
                onClick={handleArchive}
                style={{
                  padding: '4px',
                  fontSize: '11px',
                  color: '#6b7280',
                  background: 'none',
                  border: 'none',
                  cursor: 'pointer',
                  textDecoration: 'underline'
                }}
              >
                Archive
              </button>
              <button
                onClick={handleDelete}
                style={{
                  padding: '4px',
                  fontSize: '11px',
                  color: '#dc2626',
                  background: 'none',
                  border: 'none',
                  cursor: 'pointer',
                  textDecoration: 'underline'
                }}
              >
                Delete
              </button>
            </div>
          )}
        </div>

        {/* Unread indicator */}
        {isUnread && (
          <div
            style={{
              position: 'absolute',
              top: '16px',
              right: '16px',
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              backgroundColor: '#3b82f6'
            }}
          />
        )}
      </div>

      {/* Confirmation Modal */}
      {showConfirmation && (
        <div
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
            zIndex: 10000
          }}
          onClick={handleCancel}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              backgroundColor: '#ffffff',
              borderRadius: '8px',
              padding: '24px',
              maxWidth: '400px',
              width: '90%',
              boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
            }}
          >
            <h3 style={{ margin: '0 0 16px 0', fontSize: '18px', fontWeight: '600', color: '#111827' }}>
              Confirm Action
            </h3>
            <p style={{ margin: '0 0 24px 0', fontSize: '14px', color: '#6b7280' }}>
              {showConfirmation.message}
            </p>
            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
              <button
                onClick={handleCancel}
                style={{
                  padding: '8px 16px',
                  fontSize: '14px',
                  fontWeight: '500',
                  border: '1px solid #d1d5db',
                  borderRadius: '6px',
                  backgroundColor: '#ffffff',
                  color: '#374151',
                  cursor: 'pointer'
                }}
              >
                Cancel
              </button>
              <button
                onClick={handleConfirm}
                disabled={isExecuting !== null}
                style={{
                  padding: '8px 16px',
                  fontSize: '14px',
                  fontWeight: '500',
                  border: 'none',
                  borderRadius: '6px',
                  backgroundColor: '#3b82f6',
                  color: '#ffffff',
                  cursor: isExecuting ? 'not-allowed' : 'pointer',
                  opacity: isExecuting ? 0.6 : 1
                }}
              >
                {isExecuting ? 'Processing...' : 'Confirm'}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
};
