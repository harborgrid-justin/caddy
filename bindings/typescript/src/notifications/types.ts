/**
 * CADDY v0.4.0 - Notification System Types
 * Comprehensive type definitions for multi-channel notification system
 */

export enum NotificationPriority {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  URGENT = 'urgent',
  CRITICAL = 'critical'
}

export enum NotificationStatus {
  PENDING = 'pending',
  SENT = 'sent',
  DELIVERED = 'delivered',
  READ = 'read',
  FAILED = 'failed',
  ARCHIVED = 'archived'
}

export enum NotificationType {
  INFO = 'info',
  SUCCESS = 'success',
  WARNING = 'warning',
  ERROR = 'error',
  SYSTEM = 'system',
  TASK = 'task',
  MENTION = 'mention',
  COMMENT = 'comment',
  APPROVAL = 'approval',
  REMINDER = 'reminder',
  ALERT = 'alert'
}

export enum NotificationChannel {
  IN_APP = 'in_app',
  EMAIL = 'email',
  SMS = 'sms',
  PUSH = 'push',
  SLACK = 'slack',
  TEAMS = 'teams',
  WEBHOOK = 'webhook'
}

export enum NotificationGroupBy {
  NONE = 'none',
  TYPE = 'type',
  SOURCE = 'source',
  DATE = 'date',
  PRIORITY = 'priority'
}

export interface NotificationAction {
  id: string;
  label: string;
  type: 'primary' | 'secondary' | 'danger';
  action: string;
  payload?: Record<string, unknown>;
  requiresConfirmation?: boolean;
  confirmationMessage?: string;
}

export interface NotificationMetadata {
  source?: string;
  sourceId?: string;
  entityType?: string;
  entityId?: string;
  url?: string;
  imageUrl?: string;
  avatarUrl?: string;
  tags?: string[];
  customData?: Record<string, unknown>;
}

export interface Notification {
  id: string;
  tenantId: string;
  userId: string;
  type: NotificationType;
  priority: NotificationPriority;
  status: NotificationStatus;
  title: string;
  message: string;
  shortMessage?: string;
  channels: NotificationChannel[];
  actions?: NotificationAction[];
  metadata?: NotificationMetadata;
  groupId?: string;
  parentId?: string;
  readAt?: Date;
  archivedAt?: Date;
  expiresAt?: Date;
  createdAt: Date;
  updatedAt: Date;
  scheduledFor?: Date;
}

export interface NotificationGroup {
  id: string;
  type: NotificationType;
  source?: string;
  count: number;
  notifications: Notification[];
  latestNotification: Notification;
  allRead: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface NotificationPreference {
  id: string;
  userId: string;
  tenantId: string;
  enabled: boolean;
  channels: {
    [key in NotificationChannel]?: boolean;
  };
  types: {
    [key in NotificationType]?: {
      enabled: boolean;
      channels: NotificationChannel[];
      minPriority?: NotificationPriority;
    };
  };
  doNotDisturb: {
    enabled: boolean;
    startTime?: string; // HH:mm format
    endTime?: string; // HH:mm format
    days?: number[]; // 0-6 (Sunday-Saturday)
    allowUrgent?: boolean;
    allowCritical?: boolean;
  };
  emailDigest: {
    enabled: boolean;
    frequency: 'realtime' | 'hourly' | 'daily' | 'weekly';
    time?: string; // HH:mm format
  };
  soundEnabled: boolean;
  desktopEnabled: boolean;
  mobileEnabled: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface NotificationTemplate {
  id: string;
  tenantId: string;
  name: string;
  description?: string;
  type: NotificationType;
  priority: NotificationPriority;
  channels: NotificationChannel[];
  titleTemplate: string;
  messageTemplate: string;
  shortMessageTemplate?: string;
  emailTemplate?: string;
  smsTemplate?: string;
  variables: string[];
  actions?: Omit<NotificationAction, 'id'>[];
  metadata?: Partial<NotificationMetadata>;
  active: boolean;
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
}

export interface NotificationRule {
  id: string;
  tenantId: string;
  name: string;
  description?: string;
  enabled: boolean;
  priority: number;
  conditions: {
    field: string;
    operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'in' | 'nin' | 'contains' | 'matches';
    value: unknown;
  }[];
  conditionLogic: 'AND' | 'OR';
  actions: {
    type: 'route' | 'escalate' | 'suppress' | 'transform' | 'delay';
    config: Record<string, unknown>;
  }[];
  schedule?: {
    startDate?: Date;
    endDate?: Date;
    daysOfWeek?: number[];
    timeRanges?: Array<{ start: string; end: string }>;
  };
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
}

export interface NotificationDelivery {
  id: string;
  notificationId: string;
  channel: NotificationChannel;
  status: 'pending' | 'sent' | 'delivered' | 'failed' | 'bounced';
  recipientAddress: string;
  attempts: number;
  maxAttempts: number;
  lastAttemptAt?: Date;
  deliveredAt?: Date;
  failedAt?: Date;
  errorMessage?: string;
  metadata?: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
}

export interface NotificationChannelConfig {
  id: string;
  tenantId: string;
  channel: NotificationChannel;
  enabled: boolean;
  config: {
    // Email
    smtpHost?: string;
    smtpPort?: number;
    smtpSecure?: boolean;
    smtpUser?: string;
    smtpPassword?: string;
    fromEmail?: string;
    fromName?: string;

    // SMS
    smsProvider?: 'twilio' | 'nexmo' | 'aws-sns';
    smsApiKey?: string;
    smsApiSecret?: string;
    smsFromNumber?: string;

    // Push
    pushProvider?: 'fcm' | 'apns' | 'onesignal';
    pushApiKey?: string;
    pushAppId?: string;

    // Slack
    slackWebhookUrl?: string;
    slackBotToken?: string;
    slackChannel?: string;

    // Teams
    teamsWebhookUrl?: string;

    // Webhook
    webhookUrl?: string;
    webhookHeaders?: Record<string, string>;
    webhookMethod?: 'POST' | 'PUT' | 'PATCH';
  };
  rateLimit?: {
    maxPerMinute?: number;
    maxPerHour?: number;
    maxPerDay?: number;
  };
  retryPolicy?: {
    maxAttempts: number;
    backoffMultiplier: number;
    initialDelay: number;
    maxDelay: number;
  };
  createdAt: Date;
  updatedAt: Date;
}

export interface NotificationFilter {
  status?: NotificationStatus[];
  type?: NotificationType[];
  priority?: NotificationPriority[];
  channels?: NotificationChannel[];
  dateFrom?: Date;
  dateTo?: Date;
  search?: string;
  unreadOnly?: boolean;
  groupBy?: NotificationGroupBy;
}

export interface NotificationStats {
  total: number;
  unread: number;
  byType: Record<NotificationType, number>;
  byPriority: Record<NotificationPriority, number>;
  byStatus: Record<NotificationStatus, number>;
  byChannel: Record<NotificationChannel, number>;
  todayCount: number;
  weekCount: number;
  monthCount: number;
}

export interface NotificationContextValue {
  notifications: Notification[];
  groups: NotificationGroup[];
  stats: NotificationStats;
  preferences: NotificationPreference | null;
  loading: boolean;
  error: Error | null;
  filter: NotificationFilter;

  // Actions
  fetchNotifications: (filter?: NotificationFilter) => Promise<void>;
  markAsRead: (id: string | string[]) => Promise<void>;
  markAsUnread: (id: string | string[]) => Promise<void>;
  markAllAsRead: () => Promise<void>;
  archiveNotification: (id: string | string[]) => Promise<void>;
  deleteNotification: (id: string | string[]) => Promise<void>;
  executeAction: (notificationId: string, actionId: string) => Promise<void>;
  updatePreferences: (preferences: Partial<NotificationPreference>) => Promise<void>;
  setFilter: (filter: NotificationFilter) => void;
  subscribe: () => void;
  unsubscribe: () => void;
}

export interface WebSocketNotificationEvent {
  type: 'notification.created' | 'notification.updated' | 'notification.deleted' | 'notification.read' | 'notification.archived';
  data: Notification | { id: string };
  timestamp: Date;
}
