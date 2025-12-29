/**
 * CADDY v0.4.0 - Notification Provider
 * React context provider for notification management with WebSocket support
 */

import React, { createContext, useState, useEffect, useCallback, useRef, ReactNode } from 'react';
import {
  Notification,
  NotificationContextValue,
  NotificationFilter,
  NotificationGroup,
  NotificationPreference,
  NotificationStats,
  NotificationStatus,
  NotificationType,
  NotificationPriority,
  NotificationChannel,
  WebSocketNotificationEvent,
  NotificationGroupBy
} from './types';

export const NotificationContext = createContext<NotificationContextValue | null>(null);

interface NotificationProviderProps {
  children: ReactNode;
  apiUrl?: string;
  wsUrl?: string;
  tenantId: string;
  userId: string;
  autoConnect?: boolean;
  pollInterval?: number;
}

export const NotificationProvider: React.FC<NotificationProviderProps> = ({
  children,
  apiUrl = '/api/notifications',
  wsUrl = '/ws/notifications',
  tenantId,
  userId,
  autoConnect = true,
  pollInterval = 30000
}) => {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [groups, setGroups] = useState<NotificationGroup[]>([]);
  const [stats, setStats] = useState<NotificationStats>({
    total: 0,
    unread: 0,
    byType: {} as Record<NotificationType, number>,
    byPriority: {} as Record<NotificationPriority, number>,
    byStatus: {} as Record<NotificationStatus, number>,
    byChannel: {} as Record<NotificationChannel, number>,
    todayCount: 0,
    weekCount: 0,
    monthCount: 0
  });
  const [preferences, setPreferences] = useState<NotificationPreference | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<Error | null>(null);
  const [filter, setFilter] = useState<NotificationFilter>({
    groupBy: NotificationGroupBy.NONE
  });

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const pollIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef<number>(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 3000;

  /**
   * Calculate statistics from notifications
   */
  const calculateStats = useCallback((notifs: Notification[]): NotificationStats => {
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
    const monthAgo = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000);

    const stats: NotificationStats = {
      total: notifs.length,
      unread: 0,
      byType: {} as Record<NotificationType, number>,
      byPriority: {} as Record<NotificationPriority, number>,
      byStatus: {} as Record<NotificationStatus, number>,
      byChannel: {} as Record<NotificationChannel, number>,
      todayCount: 0,
      weekCount: 0,
      monthCount: 0
    };

    // Initialize counters
    Object.values(NotificationType).forEach(type => {
      stats.byType[type] = 0;
    });
    Object.values(NotificationPriority).forEach(priority => {
      stats.byPriority[priority] = 0;
    });
    Object.values(NotificationStatus).forEach(status => {
      stats.byStatus[status] = 0;
    });
    Object.values(NotificationChannel).forEach(channel => {
      stats.byChannel[channel] = 0;
    });

    notifs.forEach(notif => {
      // Unread count
      if (notif.status !== NotificationStatus.READ && notif.status !== NotificationStatus.ARCHIVED) {
        stats.unread++;
      }

      // By type
      stats.byType[notif.type]++;

      // By priority
      stats.byPriority[notif.priority]++;

      // By status
      stats.byStatus[notif.status]++;

      // By channel
      notif.channels.forEach(channel => {
        stats.byChannel[channel] = (stats.byChannel[channel] || 0) + 1;
      });

      // Time-based counts
      const createdAt = new Date(notif.createdAt);
      if (createdAt >= today) {
        stats.todayCount++;
      }
      if (createdAt >= weekAgo) {
        stats.weekCount++;
      }
      if (createdAt >= monthAgo) {
        stats.monthCount++;
      }
    });

    return stats;
  }, []);

  /**
   * Fetch notifications from API
   */
  const fetchNotifications = useCallback(async (customFilter?: NotificationFilter) => {
    setLoading(true);
    setError(null);

    try {
      const filterToUse = customFilter || filter;
      const params = new URLSearchParams({
        tenantId,
        userId,
        ...(filterToUse.status && { status: filterToUse.status.join(',') }),
        ...(filterToUse.type && { type: filterToUse.type.join(',') }),
        ...(filterToUse.priority && { priority: filterToUse.priority.join(',') }),
        ...(filterToUse.dateFrom && { dateFrom: filterToUse.dateFrom.toISOString() }),
        ...(filterToUse.dateTo && { dateTo: filterToUse.dateTo.toISOString() }),
        ...(filterToUse.search && { search: filterToUse.search }),
        ...(filterToUse.unreadOnly && { unreadOnly: 'true' })
      });

      const response = await fetch(`${apiUrl}?${params}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include'
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch notifications: ${response.statusText}`);
      }

      const data = await response.json();
      const notifs = data.notifications || [];

      setNotifications(notifs);
      setStats(calculateStats(notifs));
    } catch (err) {
      setError(err as Error);
      console.error('Error fetching notifications:', err);
    } finally {
      setLoading(false);
    }
  }, [apiUrl, tenantId, userId, filter, calculateStats]);

  /**
   * Fetch user preferences
   */
  const fetchPreferences = useCallback(async () => {
    try {
      const response = await fetch(`${apiUrl}/preferences?tenantId=${tenantId}&userId=${userId}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include'
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch preferences: ${response.statusText}`);
      }

      const data = await response.json();
      setPreferences(data);
    } catch (err) {
      console.error('Error fetching preferences:', err);
    }
  }, [apiUrl, tenantId, userId]);

  /**
   * Mark notifications as read
   */
  const markAsRead = useCallback(async (id: string | string[]) => {
    const ids = Array.isArray(id) ? id : [id];

    try {
      const response = await fetch(`${apiUrl}/read`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify({ ids, tenantId, userId })
      });

      if (!response.ok) {
        throw new Error(`Failed to mark as read: ${response.statusText}`);
      }

      // Optimistically update local state
      setNotifications(prev =>
        prev.map(notif =>
          ids.includes(notif.id)
            ? { ...notif, status: NotificationStatus.READ, readAt: new Date() }
            : notif
        )
      );
    } catch (err) {
      console.error('Error marking as read:', err);
      throw err;
    }
  }, [apiUrl, tenantId, userId]);

  /**
   * Mark notifications as unread
   */
  const markAsUnread = useCallback(async (id: string | string[]) => {
    const ids = Array.isArray(id) ? id : [id];

    try {
      const response = await fetch(`${apiUrl}/unread`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify({ ids, tenantId, userId })
      });

      if (!response.ok) {
        throw new Error(`Failed to mark as unread: ${response.statusText}`);
      }

      // Optimistically update local state
      setNotifications(prev =>
        prev.map(notif =>
          ids.includes(notif.id)
            ? { ...notif, status: NotificationStatus.SENT, readAt: undefined }
            : notif
        )
      );
    } catch (err) {
      console.error('Error marking as unread:', err);
      throw err;
    }
  }, [apiUrl, tenantId, userId]);

  /**
   * Mark all notifications as read
   */
  const markAllAsRead = useCallback(async () => {
    try {
      const response = await fetch(`${apiUrl}/read-all`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify({ tenantId, userId })
      });

      if (!response.ok) {
        throw new Error(`Failed to mark all as read: ${response.statusText}`);
      }

      // Update local state
      setNotifications(prev =>
        prev.map(notif => ({
          ...notif,
          status: NotificationStatus.READ,
          readAt: new Date()
        }))
      );
    } catch (err) {
      console.error('Error marking all as read:', err);
      throw err;
    }
  }, [apiUrl, tenantId, userId]);

  /**
   * Archive notifications
   */
  const archiveNotification = useCallback(async (id: string | string[]) => {
    const ids = Array.isArray(id) ? id : [id];

    try {
      const response = await fetch(`${apiUrl}/archive`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify({ ids, tenantId, userId })
      });

      if (!response.ok) {
        throw new Error(`Failed to archive: ${response.statusText}`);
      }

      // Remove from local state
      setNotifications(prev => prev.filter(notif => !ids.includes(notif.id)));
    } catch (err) {
      console.error('Error archiving:', err);
      throw err;
    }
  }, [apiUrl, tenantId, userId]);

  /**
   * Delete notifications
   */
  const deleteNotification = useCallback(async (id: string | string[]) => {
    const ids = Array.isArray(id) ? id : [id];

    try {
      const response = await fetch(`${apiUrl}`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify({ ids, tenantId, userId })
      });

      if (!response.ok) {
        throw new Error(`Failed to delete: ${response.statusText}`);
      }

      // Remove from local state
      setNotifications(prev => prev.filter(notif => !ids.includes(notif.id)));
    } catch (err) {
      console.error('Error deleting:', err);
      throw err;
    }
  }, [apiUrl, tenantId, userId]);

  /**
   * Execute notification action
   */
  const executeAction = useCallback(async (notificationId: string, actionId: string) => {
    try {
      const response = await fetch(`${apiUrl}/${notificationId}/actions/${actionId}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify({ tenantId, userId })
      });

      if (!response.ok) {
        throw new Error(`Failed to execute action: ${response.statusText}`);
      }

      const result = await response.json();
      return result;
    } catch (err) {
      console.error('Error executing action:', err);
      throw err;
    }
  }, [apiUrl, tenantId, userId]);

  /**
   * Update user preferences
   */
  const updatePreferences = useCallback(async (updates: Partial<NotificationPreference>) => {
    try {
      const response = await fetch(`${apiUrl}/preferences`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify({ ...updates, tenantId, userId })
      });

      if (!response.ok) {
        throw new Error(`Failed to update preferences: ${response.statusText}`);
      }

      const updated = await response.json();
      setPreferences(updated);
    } catch (err) {
      console.error('Error updating preferences:', err);
      throw err;
    }
  }, [apiUrl, tenantId, userId]);

  /**
   * Handle WebSocket messages
   */
  const handleWebSocketMessage = useCallback((event: MessageEvent) => {
    try {
      const wsEvent: WebSocketNotificationEvent = JSON.parse(event.data);

      switch (wsEvent.type) {
        case 'notification.created':
          setNotifications(prev => {
            const notification = wsEvent.data as Notification;
            // Check if notification already exists
            if (prev.some(n => n.id === notification.id)) {
              return prev;
            }
            return [notification, ...prev];
          });
          break;

        case 'notification.updated':
          setNotifications(prev =>
            prev.map(notif =>
              notif.id === (wsEvent.data as Notification).id ? (wsEvent.data as Notification) : notif
            )
          );
          break;

        case 'notification.deleted':
        case 'notification.archived':
          setNotifications(prev =>
            prev.filter(notif => notif.id !== (wsEvent.data as { id: string }).id)
          );
          break;

        case 'notification.read':
          setNotifications(prev =>
            prev.map(notif =>
              notif.id === (wsEvent.data as { id: string }).id
                ? { ...notif, status: NotificationStatus.READ, readAt: new Date() }
                : notif
            )
          );
          break;
      }
    } catch (err) {
      console.error('Error handling WebSocket message:', err);
    }
  }, []);

  /**
   * Connect to WebSocket
   */
  const connectWebSocket = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const wsUrlFull = wsUrl.startsWith('ws') ? wsUrl : `${protocol}//${window.location.host}${wsUrl}`;
      const ws = new WebSocket(`${wsUrlFull}?tenantId=${tenantId}&userId=${userId}`);

      ws.onopen = () => {
        console.log('WebSocket connected');
        reconnectAttempts.current = 0;
      };

      ws.onmessage = handleWebSocketMessage;

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        wsRef.current = null;

        // Attempt to reconnect
        if (reconnectAttempts.current < maxReconnectAttempts) {
          reconnectAttempts.current++;
          reconnectTimeoutRef.current = setTimeout(() => {
            console.log(`Reconnecting... (attempt ${reconnectAttempts.current})`);
            connectWebSocket();
          }, reconnectDelay * reconnectAttempts.current);
        }
      };

      wsRef.current = ws;
    } catch (err) {
      console.error('Error connecting to WebSocket:', err);
    }
  }, [wsUrl, tenantId, userId, handleWebSocketMessage]);

  /**
   * Disconnect from WebSocket
   */
  const disconnectWebSocket = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
  }, []);

  /**
   * Subscribe to real-time updates
   */
  const subscribe = useCallback(() => {
    connectWebSocket();
  }, [connectWebSocket]);

  /**
   * Unsubscribe from real-time updates
   */
  const unsubscribe = useCallback(() => {
    disconnectWebSocket();
  }, [disconnectWebSocket]);

  /**
   * Update stats when notifications change
   */
  useEffect(() => {
    setStats(calculateStats(notifications));
  }, [notifications, calculateStats]);

  /**
   * Initial data fetch
   */
  useEffect(() => {
    fetchNotifications();
    fetchPreferences();
  }, []);

  /**
   * Setup WebSocket and polling
   */
  useEffect(() => {
    if (autoConnect) {
      connectWebSocket();
    }

    // Fallback polling
    if (pollInterval > 0) {
      pollIntervalRef.current = setInterval(() => {
        if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) {
          fetchNotifications();
        }
      }, pollInterval);
    }

    return () => {
      disconnectWebSocket();
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
      }
    };
  }, [autoConnect, pollInterval, connectWebSocket, disconnectWebSocket, fetchNotifications]);

  const value: NotificationContextValue = {
    notifications,
    groups,
    stats,
    preferences,
    loading,
    error,
    filter,
    fetchNotifications,
    markAsRead,
    markAsUnread,
    markAllAsRead,
    archiveNotification,
    deleteNotification,
    executeAction,
    updatePreferences,
    setFilter,
    subscribe,
    unsubscribe
  };

  return (
    <NotificationContext.Provider value={value}>
      {children}
    </NotificationContext.Provider>
  );
};
