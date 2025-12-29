import { Notification, NotificationFilter, NotificationGroup, NotificationPriority, NotificationType, NotificationGroupBy, NotificationContextValue } from './types';
export declare function useNotifications(): NotificationContextValue;
export declare function useUnreadCount(filter?: {
    type?: NotificationType;
    priority?: NotificationPriority;
}): number;
export declare function useUrgentNotifications(): Notification[];
export declare function useGroupedNotifications(groupBy?: NotificationGroupBy): NotificationGroup[];
export declare function useFilteredNotifications(filter: NotificationFilter): Notification[];
export declare function useDoNotDisturb(): {
    enabled: boolean;
    isActive: boolean;
    toggle: () => Promise<void>;
    setSchedule: (schedule: {
        startTime: string;
        endTime: string;
        days?: number[];
    }) => Promise<void>;
};
export declare function useNotificationSound(): {
    playSound: (priority: NotificationPriority) => void;
    enabled: boolean;
    toggleSound: () => Promise<void>;
};
export declare function useDesktopNotifications(): {
    permission: NotificationPermission;
    requestPermission: () => Promise<NotificationPermission>;
    showDesktopNotification: (notification: Notification) => void;
    enabled: boolean;
};
export declare function useNotificationStats(): import("./types").NotificationStats;
export declare function useBatchOperations(): {
    selectedIds: Set<string>;
    selectedCount: number;
    toggleSelection: (id: string) => void;
    selectAll: (ids: string[]) => void;
    clearSelection: () => void;
    markSelectedAsRead: () => Promise<void>;
    markSelectedAsUnread: () => Promise<void>;
    archiveSelected: () => Promise<void>;
    deleteSelected: () => Promise<void>;
};
//# sourceMappingURL=useNotifications.d.ts.map