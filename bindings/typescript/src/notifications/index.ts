/**
 * CADDY v0.4.0 - Notification System
 * Complete multi-channel notification system with real-time delivery
 *
 * @module notifications
 * @author Agent 6 - Notification Center Specialist
 * @version 0.4.0
 */

// Components
export { NotificationProvider } from './NotificationProvider';
export { NotificationCenter } from './NotificationCenter';
export { NotificationList } from './NotificationList';
export { NotificationItem } from './NotificationItem';
export { NotificationBell } from './NotificationBell';
export { NotificationPreferences } from './NotificationPreferences';
export { NotificationTemplates } from './NotificationTemplates';
export { NotificationChannels } from './NotificationChannels';
export { NotificationRules } from './NotificationRules';
export { NotificationHistory } from './NotificationHistory';

// Hooks
export {
  useNotifications,
  useUnreadCount,
  useUrgentNotifications,
  useGroupedNotifications,
  useFilteredNotifications,
  useDoNotDisturb,
  useNotificationSound,
  useDesktopNotifications,
  useNotificationStats,
  useBatchOperations
} from './useNotifications';

// Types
export type {
  Notification,
  NotificationGroup,
  NotificationPreference,
  NotificationTemplate,
  NotificationRule,
  NotificationDelivery,
  NotificationChannelConfig,
  NotificationFilter,
  NotificationStats,
  NotificationContextValue,
  NotificationAction,
  NotificationMetadata,
  WebSocketNotificationEvent
} from './types';

export {
  NotificationPriority,
  NotificationStatus,
  NotificationType,
  NotificationChannel,
  NotificationGroupBy
} from './types';
