/**
 * CADDY Enterprise Realtime Feed Component v0.4.0
 *
 * Live activity feed with WebSocket support for real-time updates.
 * Includes filtering, search, notifications, and accessibility features.
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import type { ActivityFeedItem, AlertSeverity, WebSocketMessage } from './types';
import { useDashboard } from './DashboardLayout';

/**
 * Realtime feed props
 */
export interface RealtimeFeedProps {
  /** WebSocket URL */
  wsUrl: string;
  /** WebSocket channel */
  channel?: string;
  /** Initial feed items */
  initialItems?: ActivityFeedItem[];
  /** Max items to display */
  maxItems?: number;
  /** Enable notifications */
  enableNotifications?: boolean;
  /** Enable sound */
  enableSound?: boolean;
  /** Enable filters */
  showFilters?: boolean;
  /** Enable search */
  showSearch?: boolean;
  /** Auto-scroll to new items */
  autoScroll?: boolean;
  /** On item click */
  onItemClick?: (item: ActivityFeedItem) => void;
  /** Custom class name */
  className?: string;
}

/**
 * Realtime feed component
 */
export const RealtimeFeed: React.FC<RealtimeFeedProps> = ({
  wsUrl,
  channel = 'activity',
  initialItems = [],
  maxItems = 100,
  enableNotifications = true,
  enableSound = false,
  showFilters = true,
  showSearch = true,
  autoScroll = true,
  onItemClick,
  className = '',
}) => {
  const [items, setItems] = useState<ActivityFeedItem[]>(initialItems);
  const [filteredItems, setFilteredItems] = useState<ActivityFeedItem[]>(initialItems);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<string>('all');
  const [filterSeverity, setFilterSeverity] = useState<string>('all');
  const [isConnected, setIsConnected] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const feedRef = useRef<HTMLDivElement>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const { theme, accessibility } = useDashboard();

  /**
   * Connect to WebSocket
   */
  useEffect(() => {
    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      setIsConnected(true);
      // Subscribe to channel
      ws.send(JSON.stringify({ type: 'subscribe', channel }));
    };

    ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        handleWebSocketMessage(message);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    ws.onclose = () => {
      setIsConnected(false);
      // Attempt to reconnect after 5 seconds
      setTimeout(() => {
        if (wsRef.current === ws) {
          wsRef.current = null;
        }
      }, 5000);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      setIsConnected(false);
    };

    wsRef.current = ws;

    return () => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.close();
      }
    };
  }, [wsUrl, channel]);

  /**
   * Handle WebSocket message
   */
  const handleWebSocketMessage = useCallback(
    (message: WebSocketMessage) => {
      if (isPaused) return;

      switch (message.type) {
        case 'activity':
          addActivityItem(message.payload as ActivityFeedItem);
          break;
        case 'alert':
          addActivityItem({
            ...message.payload,
            type: 'alert',
          } as ActivityFeedItem);
          break;
        default:
          break;
      }
    },
    [isPaused]
  );

  /**
   * Add new activity item
   */
  const addActivityItem = useCallback(
    (item: ActivityFeedItem) => {
      setItems((prev) => {
        const updated = [item, ...prev];
        // Limit to maxItems
        return updated.slice(0, maxItems);
      });

      // Show notification
      if (enableNotifications && item.severity && ['error', 'critical'].includes(item.severity)) {
        showNotification(item);
      }

      // Play sound
      if (enableSound) {
        playSound();
      }

      // Auto-scroll
      if (autoScroll && feedRef.current) {
        feedRef.current.scrollTop = 0;
      }
    },
    [maxItems, enableNotifications, enableSound, autoScroll]
  );

  /**
   * Show browser notification
   */
  const showNotification = useCallback((item: ActivityFeedItem) => {
    if ('Notification' in window && Notification.permission === 'granted') {
      new Notification(item.title, {
        body: item.description,
        icon: '/favicon.ico',
        tag: item.id,
      });
    }
  }, []);

  /**
   * Play notification sound
   */
  const playSound = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.play().catch(() => {
        // Ignore errors (user interaction required)
      });
    }
  }, []);

  /**
   * Request notification permission
   */
  useEffect(() => {
    if (enableNotifications && 'Notification' in window) {
      if (Notification.permission === 'default') {
        Notification.requestPermission();
      }
    }
  }, [enableNotifications]);

  /**
   * Filter items based on search and filters
   */
  useEffect(() => {
    let filtered = [...items];

    // Search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (item) =>
          item.title.toLowerCase().includes(query) ||
          item.description.toLowerCase().includes(query)
      );
    }

    // Type filter
    if (filterType !== 'all') {
      filtered = filtered.filter((item) => item.type === filterType);
    }

    // Severity filter
    if (filterSeverity !== 'all') {
      filtered = filtered.filter((item) => item.severity === filterSeverity);
    }

    setFilteredItems(filtered);
  }, [items, searchQuery, filterType, filterSeverity]);

  /**
   * Mark item as read
   */
  const markAsRead = useCallback((itemId: string) => {
    setItems((prev) =>
      prev.map((item) =>
        item.id === itemId ? { ...item, read: true } : item
      )
    );
  }, []);

  /**
   * Mark all as read
   */
  const markAllAsRead = useCallback(() => {
    setItems((prev) => prev.map((item) => ({ ...item, read: true })));
  }, []);

  /**
   * Clear all items
   */
  const clearAll = useCallback(() => {
    setItems([]);
  }, []);

  /**
   * Toggle pause
   */
  const togglePause = useCallback(() => {
    setIsPaused((prev) => !prev);
  }, []);

  /**
   * Get unread count
   */
  const unreadCount = items.filter((item) => !item.read).length;

  return (
    <div
      className={`realtime-feed ${className}`}
      style={styles.container}
      role="region"
      aria-label="Real-time activity feed"
      aria-live="polite"
    >
      {/* Header */}
      <div style={styles.header}>
        <div style={styles.headerLeft}>
          <h3 style={styles.title}>
            Live Activity Feed
            {unreadCount > 0 && (
              <span style={styles.unreadBadge} aria-label={`${unreadCount} unread items`}>
                {unreadCount}
              </span>
            )}
          </h3>
          <div style={styles.connectionStatus}>
            <span
              style={{
                ...styles.connectionDot,
                backgroundColor: isConnected ? '#4caf50' : '#f44336',
              }}
              aria-hidden="true"
            />
            <span style={styles.connectionText}>
              {isConnected ? 'Connected' : 'Disconnected'}
            </span>
          </div>
        </div>

        <div style={styles.headerRight}>
          {/* Pause button */}
          <button
            onClick={togglePause}
            style={styles.iconButton}
            aria-label={isPaused ? 'Resume feed' : 'Pause feed'}
            title={isPaused ? 'Resume' : 'Pause'}
          >
            {isPaused ? '▶' : '⏸'}
          </button>

          {/* Mark all read button */}
          <button
            onClick={markAllAsRead}
            style={styles.iconButton}
            aria-label="Mark all as read"
            title="Mark all as read"
            disabled={unreadCount === 0}
          >
            ✓
          </button>

          {/* Clear all button */}
          <button
            onClick={clearAll}
            style={styles.iconButton}
            aria-label="Clear all items"
            title="Clear all"
          >
            🗑️
          </button>
        </div>
      </div>

      {/* Search and Filters */}
      {(showSearch || showFilters) && (
        <div style={styles.controls}>
          {/* Search */}
          {showSearch && (
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search activities..."
              style={styles.searchInput}
              aria-label="Search activities"
            />
          )}

          {/* Filters */}
          {showFilters && (
            <div style={styles.filters}>
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value)}
                style={styles.filterSelect}
                aria-label="Filter by type"
              >
                <option value="all">All Types</option>
                <option value="user">User</option>
                <option value="system">System</option>
                <option value="alert">Alert</option>
                <option value="metric">Metric</option>
                <option value="event">Event</option>
              </select>

              <select
                value={filterSeverity}
                onChange={(e) => setFilterSeverity(e.target.value)}
                style={styles.filterSelect}
                aria-label="Filter by severity"
              >
                <option value="all">All Severities</option>
                <option value="info">Info</option>
                <option value="warning">Warning</option>
                <option value="error">Error</option>
                <option value="critical">Critical</option>
              </select>
            </div>
          )}
        </div>
      )}

      {/* Feed items */}
      <div
        ref={feedRef}
        style={styles.feed}
        role="feed"
        aria-busy={!isConnected}
      >
        {filteredItems.length === 0 && (
          <div style={styles.emptyState} role="status">
            <p style={styles.emptyText}>
              {items.length === 0 ? 'No activities yet' : 'No activities match your filters'}
            </p>
          </div>
        )}

        {filteredItems.map((item) => (
          <FeedItem
            key={item.id}
            item={item}
            onClick={() => {
              markAsRead(item.id);
              if (onItemClick) {
                onItemClick(item);
              }
            }}
            accessibility={accessibility}
          />
        ))}
      </div>

      {/* Hidden audio element for notifications */}
      {enableSound && (
        <audio
          ref={audioRef}
          src="data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1fdJivrJBhNjVgodDbq2EcBj+a2/LDciUFLIHO8tiJNwgZaLvt559NEAxQp+PwtmMcBjiR1/LMeSwFJHfH8N2QQAoUXrTp66hVFApGn+DyvmwhBTWL0fPTgjMGHm7A7+OZURE="
        />
      )}
    </div>
  );
};

/**
 * Feed item component
 */
interface FeedItemProps {
  item: ActivityFeedItem;
  onClick: () => void;
  accessibility: any;
}

const FeedItem: React.FC<FeedItemProps> = ({ item, onClick, accessibility }) => {
  const getTypeColor = (type: string): string => {
    switch (type) {
      case 'user':
        return '#2196f3';
      case 'system':
        return '#9e9e9e';
      case 'alert':
        return '#f44336';
      case 'metric':
        return '#4caf50';
      case 'event':
        return '#ff9800';
      default:
        return '#666';
    }
  };

  const getSeverityColor = (severity?: AlertSeverity): string => {
    if (!severity) return 'transparent';
    switch (severity) {
      case 'critical':
        return '#d32f2f';
      case 'error':
        return '#f44336';
      case 'warning':
        return '#ff9800';
      case 'info':
        return '#2196f3';
      default:
        return 'transparent';
    }
  };

  const getRelativeTime = (timestamp: string): string => {
    const now = new Date();
    const time = new Date(timestamp);
    const diff = now.getTime() - time.getTime();

    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    if (minutes > 0) return `${minutes}m ago`;
    return 'Just now';
  };

  return (
    <article
      style={{
        ...styles.feedItem,
        ...(item.read ? {} : styles.feedItemUnread),
        borderLeftColor: item.severity
          ? getSeverityColor(item.severity)
          : getTypeColor(item.type),
      }}
      onClick={onClick}
      role="article"
      tabIndex={accessibility.keyboardNavigation ? 0 : undefined}
      aria-label={`${item.title}. ${item.description}`}
    >
      {/* Avatar/Icon */}
      <div style={styles.feedItemAvatar}>
        {item.user?.avatar ? (
          <img
            src={item.user.avatar}
            alt={item.user.name}
            style={styles.avatarImage}
          />
        ) : (
          <div
            style={{
              ...styles.avatarPlaceholder,
              backgroundColor: getTypeColor(item.type),
            }}
            aria-hidden="true"
          >
            {item.icon || getDefaultIcon(item.type)}
          </div>
        )}
      </div>

      {/* Content */}
      <div style={styles.feedItemContent}>
        <div style={styles.feedItemHeader}>
          <h4 style={styles.feedItemTitle}>{item.title}</h4>
          <span style={styles.feedItemTime} aria-label={`Time: ${getRelativeTime(item.timestamp)}`}>
            {getRelativeTime(item.timestamp)}
          </span>
        </div>

        <p style={styles.feedItemDescription}>{item.description}</p>

        {/* Resource info */}
        {item.resource && (
          <div style={styles.feedItemResource}>
            <span style={styles.resourceType}>{item.resource.type}</span>
            <span style={styles.resourceName}>{item.resource.name}</span>
          </div>
        )}

        {/* Action button */}
        {item.actionUrl && (
          <a
            href={item.actionUrl}
            style={styles.feedItemAction}
            onClick={(e) => e.stopPropagation()}
            aria-label={item.actionLabel || 'View details'}
          >
            {item.actionLabel || 'View Details'} →
          </a>
        )}

        {/* Severity badge */}
        {item.severity && (
          <span
            style={{
              ...styles.severityBadge,
              backgroundColor: getSeverityColor(item.severity),
            }}
            aria-label={`Severity: ${item.severity}`}
          >
            {item.severity}
          </span>
        )}
      </div>

      {/* Unread indicator */}
      {!item.read && (
        <div
          style={styles.unreadIndicator}
          aria-label="Unread"
          role="status"
        />
      )}
    </article>
  );
};

/**
 * Get default icon for activity type
 */
function getDefaultIcon(type: string): string {
  switch (type) {
    case 'user':
      return '👤';
    case 'system':
      return '⚙️';
    case 'alert':
      return '⚠️';
    case 'metric':
      return '📊';
    case 'event':
      return '📅';
    default:
      return '•';
  }
}

/**
 * Component styles
 */
const styles: Record<string, React.CSSProperties> = {
  container: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    border: '1px solid var(--color-border, #e0e0e0)',
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    overflow: 'hidden',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    padding: '16px 20px',
    borderBottom: '1px solid var(--color-divider, #e0e0e0)',
  },
  headerLeft: {
    flex: 1,
  },
  headerRight: {
    display: 'flex',
    gap: 8,
  },
  title: {
    margin: '0 0 8px 0',
    fontSize: 18,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
    display: 'flex',
    alignItems: 'center',
    gap: 8,
  },
  unreadBadge: {
    display: 'inline-block',
    minWidth: 20,
    height: 20,
    padding: '0 6px',
    backgroundColor: '#f44336',
    color: '#fff',
    borderRadius: 10,
    fontSize: 12,
    fontWeight: 600,
    lineHeight: '20px',
    textAlign: 'center',
  },
  connectionStatus: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
  },
  connectionDot: {
    width: 8,
    height: 8,
    borderRadius: '50%',
  },
  connectionText: {
    fontSize: 12,
    color: 'var(--color-text-secondary, #666)',
  },
  iconButton: {
    width: 32,
    height: 32,
    border: 'none',
    backgroundColor: 'transparent',
    color: 'var(--color-text-secondary, #666)',
    cursor: 'pointer',
    borderRadius: 4,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: 16,
    transition: 'background-color var(--animation-duration, 200ms)',
  },
  controls: {
    padding: '12px 20px',
    borderBottom: '1px solid var(--color-divider, #e0e0e0)',
    display: 'flex',
    flexDirection: 'column',
    gap: 12,
  },
  searchInput: {
    width: '100%',
    padding: '8px 12px',
    border: '1px solid var(--color-border, #e0e0e0)',
    borderRadius: 4,
    fontSize: 14,
    backgroundColor: 'var(--color-background, #f5f5f5)',
    color: 'var(--color-text, #333)',
  },
  filters: {
    display: 'flex',
    gap: 12,
  },
  filterSelect: {
    flex: 1,
    padding: '8px 12px',
    border: '1px solid var(--color-border, #e0e0e0)',
    borderRadius: 4,
    fontSize: 13,
    backgroundColor: 'var(--color-surface, #fff)',
    color: 'var(--color-text, #333)',
    cursor: 'pointer',
  },
  feed: {
    flex: 1,
    overflowY: 'auto',
    padding: 12,
  },
  emptyState: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    minHeight: 200,
  },
  emptyText: {
    color: 'var(--color-text-secondary, #999)',
    fontSize: 14,
  },
  feedItem: {
    display: 'flex',
    gap: 12,
    padding: 12,
    marginBottom: 8,
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    border: '1px solid var(--color-border, #e0e0e0)',
    borderLeft: '4px solid',
    cursor: 'pointer',
    transition: 'background-color var(--animation-duration, 200ms), transform var(--animation-duration, 200ms)',
    position: 'relative',
  },
  feedItemUnread: {
    backgroundColor: '#f5f9ff',
  },
  feedItemAvatar: {
    flexShrink: 0,
  },
  avatarImage: {
    width: 40,
    height: 40,
    borderRadius: '50%',
    objectFit: 'cover',
  },
  avatarPlaceholder: {
    width: 40,
    height: 40,
    borderRadius: '50%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    color: '#fff',
    fontSize: 20,
    fontWeight: 600,
  },
  feedItemContent: {
    flex: 1,
    minWidth: 0,
  },
  feedItemHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 4,
  },
  feedItemTitle: {
    margin: 0,
    fontSize: 14,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
    flex: 1,
  },
  feedItemTime: {
    fontSize: 12,
    color: 'var(--color-text-secondary, #999)',
    flexShrink: 0,
    marginLeft: 8,
  },
  feedItemDescription: {
    margin: '0 0 8px 0',
    fontSize: 13,
    color: 'var(--color-text-secondary, #666)',
    lineHeight: 1.5,
  },
  feedItemResource: {
    display: 'flex',
    gap: 8,
    marginBottom: 8,
    fontSize: 12,
  },
  resourceType: {
    padding: '2px 8px',
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 12,
    color: 'var(--color-text-secondary, #666)',
    fontWeight: 500,
  },
  resourceName: {
    color: 'var(--color-text, #333)',
    fontWeight: 500,
  },
  feedItemAction: {
    display: 'inline-block',
    fontSize: 13,
    color: 'var(--color-primary, #1976d2)',
    textDecoration: 'none',
    fontWeight: 500,
    marginTop: 4,
  },
  severityBadge: {
    display: 'inline-block',
    padding: '2px 8px',
    borderRadius: 12,
    fontSize: 11,
    fontWeight: 600,
    color: '#fff',
    textTransform: 'uppercase',
    marginTop: 8,
  },
  unreadIndicator: {
    position: 'absolute',
    top: 16,
    right: 16,
    width: 8,
    height: 8,
    backgroundColor: '#2196f3',
    borderRadius: '50%',
  },
};

export default RealtimeFeed;
