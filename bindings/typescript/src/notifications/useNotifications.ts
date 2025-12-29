/**
 * CADDY v0.4.0 - Notification Hooks
 * Custom React hooks for notification management
 */

import { useContext, useState, useCallback, useEffect, useMemo } from 'react';
import { NotificationContext } from './NotificationProvider';
import {
  Notification,
  NotificationFilter,
  NotificationGroup,
  NotificationPriority,
  NotificationStatus,
  NotificationType,
  NotificationGroupBy,
  NotificationContextValue
} from './types';

/**
 * Main hook for accessing notification system
 */
export function useNotifications(): NotificationContextValue {
  const context = useContext(NotificationContext);
  if (!context) {
    throw new Error('useNotifications must be used within NotificationProvider');
  }
  return context;
}

/**
 * Hook for unread notification count with optional filtering
 */
export function useUnreadCount(filter?: { type?: NotificationType; priority?: NotificationPriority }): number {
  const { notifications } = useNotifications();

  return useMemo(() => {
    let filtered = notifications.filter(n => n.status !== NotificationStatus.READ && n.status !== NotificationStatus.ARCHIVED);

    if (filter?.type) {
      filtered = filtered.filter(n => n.type === filter.type);
    }

    if (filter?.priority) {
      filtered = filtered.filter(n => n.priority === filter.priority);
    }

    return filtered.length;
  }, [notifications, filter]);
}

/**
 * Hook for urgent/critical notifications
 */
export function useUrgentNotifications(): Notification[] {
  const { notifications } = useNotifications();

  return useMemo(() => {
    return notifications.filter(
      n => (n.priority === NotificationPriority.URGENT || n.priority === NotificationPriority.CRITICAL) &&
           n.status !== NotificationStatus.READ &&
           n.status !== NotificationStatus.ARCHIVED
    ).sort((a, b) => {
      // Sort by priority first, then by date
      const priorityOrder = {
        [NotificationPriority.CRITICAL]: 5,
        [NotificationPriority.URGENT]: 4,
        [NotificationPriority.HIGH]: 3,
        [NotificationPriority.MEDIUM]: 2,
        [NotificationPriority.LOW]: 1
      };
      if (priorityOrder[a.priority] !== priorityOrder[b.priority]) {
        return priorityOrder[b.priority] - priorityOrder[a.priority];
      }
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
    });
  }, [notifications]);
}

/**
 * Hook for grouped notifications
 */
export function useGroupedNotifications(groupBy: NotificationGroupBy = NotificationGroupBy.TYPE): NotificationGroup[] {
  const { notifications } = useNotifications();

  return useMemo(() => {
    if (groupBy === NotificationGroupBy.NONE) {
      return [];
    }

    const groups = new Map<string, Notification[]>();

    notifications.forEach(notification => {
      let key: string;

      switch (groupBy) {
        case NotificationGroupBy.TYPE:
          key = notification.type;
          break;
        case NotificationGroupBy.SOURCE:
          key = notification.metadata?.source || 'unknown';
          break;
        case NotificationGroupBy.DATE:
          key = new Date(notification.createdAt).toDateString();
          break;
        case NotificationGroupBy.PRIORITY:
          key = notification.priority;
          break;
        default:
          key = 'default';
      }

      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key)!.push(notification);
    });

    return Array.from(groups.entries()).map(([key, notifs]) => {
      const sorted = notifs.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
      return {
        id: key,
        type: groupBy === NotificationGroupBy.TYPE ? (key as NotificationType) : NotificationType.INFO,
        source: groupBy === NotificationGroupBy.SOURCE ? key : undefined,
        count: notifs.length,
        notifications: sorted,
        latestNotification: sorted[0],
        allRead: notifs.every(n => n.status === NotificationStatus.READ || n.status === NotificationStatus.ARCHIVED),
        createdAt: sorted[sorted.length - 1].createdAt,
        updatedAt: sorted[0].updatedAt
      };
    }).sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime());
  }, [notifications, groupBy]);
}

/**
 * Hook for filtering notifications
 */
export function useFilteredNotifications(filter: NotificationFilter): Notification[] {
  const { notifications } = useNotifications();

  return useMemo(() => {
    let filtered = [...notifications];

    if (filter.status && filter.status.length > 0) {
      filtered = filtered.filter(n => filter.status!.includes(n.status));
    }

    if (filter.type && filter.type.length > 0) {
      filtered = filtered.filter(n => filter.type!.includes(n.type));
    }

    if (filter.priority && filter.priority.length > 0) {
      filtered = filtered.filter(n => filter.priority!.includes(n.priority));
    }

    if (filter.unreadOnly) {
      filtered = filtered.filter(n => n.status !== NotificationStatus.READ && n.status !== NotificationStatus.ARCHIVED);
    }

    if (filter.dateFrom) {
      filtered = filtered.filter(n => new Date(n.createdAt) >= filter.dateFrom!);
    }

    if (filter.dateTo) {
      filtered = filtered.filter(n => new Date(n.createdAt) <= filter.dateTo!);
    }

    if (filter.search) {
      const searchLower = filter.search.toLowerCase();
      filtered = filtered.filter(n =>
        n.title.toLowerCase().includes(searchLower) ||
        n.message.toLowerCase().includes(searchLower) ||
        n.metadata?.source?.toLowerCase().includes(searchLower)
      );
    }

    return filtered.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
  }, [notifications, filter]);
}

/**
 * Hook for Do Not Disturb mode
 */
export function useDoNotDisturb(): {
  enabled: boolean;
  isActive: boolean;
  toggle: () => Promise<void>;
  setSchedule: (schedule: { startTime: string; endTime: string; days?: number[] }) => Promise<void>;
} {
  const { preferences, updatePreferences } = useNotifications();
  const [isActive, setIsActive] = useState(false);

  useEffect(() => {
    if (!preferences?.doNotDisturb.enabled) {
      setIsActive(false);
      return;
    }

    const checkDND = () => {
      const now = new Date();
      const currentDay = now.getDay();
      const currentTime = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;

      const { startTime, endTime, days } = preferences.doNotDisturb;

      // Check if current day is in allowed days
      if (days && days.length > 0 && !days.includes(currentDay)) {
        setIsActive(false);
        return;
      }

      // Check if current time is in DND window
      if (startTime && endTime) {
        const active = currentTime >= startTime && currentTime <= endTime;
        setIsActive(active);
      } else {
        setIsActive(true); // If no time restrictions, DND is always active when enabled
      }
    };

    checkDND();
    const interval = setInterval(checkDND, 60000); // Check every minute

    return () => clearInterval(interval);
  }, [preferences]);

  const toggle = useCallback(async () => {
    if (!preferences) return;

    await updatePreferences({
      doNotDisturb: {
        ...preferences.doNotDisturb,
        enabled: !preferences.doNotDisturb.enabled
      }
    });
  }, [preferences, updatePreferences]);

  const setSchedule = useCallback(async (schedule: { startTime: string; endTime: string; days?: number[] }) => {
    if (!preferences) return;

    await updatePreferences({
      doNotDisturb: {
        ...preferences.doNotDisturb,
        ...schedule
      }
    });
  }, [preferences, updatePreferences]);

  return {
    enabled: preferences?.doNotDisturb.enabled || false,
    isActive,
    toggle,
    setSchedule
  };
}

/**
 * Hook for notification sound effects
 */
export function useNotificationSound(): {
  playSound: (priority: NotificationPriority) => void;
  enabled: boolean;
  toggleSound: () => Promise<void>;
} {
  const { preferences, updatePreferences } = useNotifications();

  const playSound = useCallback((priority: NotificationPriority) => {
    if (!preferences?.soundEnabled) return;

    const audioContext = new AudioContext();
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);

    // Different frequencies based on priority
    const frequencies: Record<NotificationPriority, number> = {
      [NotificationPriority.LOW]: 400,
      [NotificationPriority.MEDIUM]: 500,
      [NotificationPriority.HIGH]: 600,
      [NotificationPriority.URGENT]: 700,
      [NotificationPriority.CRITICAL]: 800
    };

    oscillator.frequency.value = frequencies[priority];
    oscillator.type = 'sine';

    gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.5);

    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.5);
  }, [preferences]);

  const toggleSound = useCallback(async () => {
    if (!preferences) return;

    await updatePreferences({
      soundEnabled: !preferences.soundEnabled
    });
  }, [preferences, updatePreferences]);

  return {
    playSound,
    enabled: preferences?.soundEnabled || false,
    toggleSound
  };
}

/**
 * Hook for desktop notifications
 */
export function useDesktopNotifications(): {
  permission: NotificationPermission;
  requestPermission: () => Promise<NotificationPermission>;
  showDesktopNotification: (notification: Notification) => void;
  enabled: boolean;
} {
  const [permission, setPermission] = useState<NotificationPermission>('default');
  const { preferences } = useNotifications();

  useEffect(() => {
    if ('Notification' in window) {
      setPermission(Notification.permission);
    }
  }, []);

  const requestPermission = useCallback(async (): Promise<NotificationPermission> => {
    if (!('Notification' in window)) {
      return 'denied';
    }

    const result = await Notification.requestPermission();
    setPermission(result);
    return result;
  }, []);

  const showDesktopNotification = useCallback((notification: Notification) => {
    if (!preferences?.desktopEnabled || permission !== 'granted') return;

    const options: NotificationOptions = {
      body: notification.message,
      icon: notification.metadata?.imageUrl,
      badge: notification.metadata?.avatarUrl,
      tag: notification.id,
      requireInteraction: notification.priority === NotificationPriority.CRITICAL || notification.priority === NotificationPriority.URGENT,
      silent: !preferences.soundEnabled
    };

    const desktopNotif = new Notification(notification.title, options);

    desktopNotif.onclick = () => {
      window.focus();
      if (notification.metadata?.url) {
        window.location.href = notification.metadata.url;
      }
      desktopNotif.close();
    };
  }, [permission, preferences]);

  return {
    permission,
    requestPermission,
    showDesktopNotification,
    enabled: preferences?.desktopEnabled || false
  };
}

/**
 * Hook for notification statistics
 */
export function useNotificationStats() {
  const { stats } = useNotifications();
  return stats;
}

/**
 * Hook for batch operations
 */
export function useBatchOperations() {
  const { markAsRead, markAsUnread, archiveNotification, deleteNotification } = useNotifications();
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  const toggleSelection = useCallback((id: string) => {
    setSelectedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const selectAll = useCallback((ids: string[]) => {
    setSelectedIds(new Set(ids));
  }, []);

  const clearSelection = useCallback(() => {
    setSelectedIds(new Set());
  }, []);

  const markSelectedAsRead = useCallback(async () => {
    await markAsRead(Array.from(selectedIds));
    clearSelection();
  }, [selectedIds, markAsRead, clearSelection]);

  const markSelectedAsUnread = useCallback(async () => {
    await markAsUnread(Array.from(selectedIds));
    clearSelection();
  }, [selectedIds, markAsUnread, clearSelection]);

  const archiveSelected = useCallback(async () => {
    await archiveNotification(Array.from(selectedIds));
    clearSelection();
  }, [selectedIds, archiveNotification, clearSelection]);

  const deleteSelected = useCallback(async () => {
    await deleteNotification(Array.from(selectedIds));
    clearSelection();
  }, [selectedIds, deleteNotification, clearSelection]);

  return {
    selectedIds,
    selectedCount: selectedIds.size,
    toggleSelection,
    selectAll,
    clearSelection,
    markSelectedAsRead,
    markSelectedAsUnread,
    archiveSelected,
    deleteSelected
  };
}
