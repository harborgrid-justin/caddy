import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useDashboard } from './DashboardLayout';
export const RealtimeFeed = ({ wsUrl, channel = 'activity', initialItems = [], maxItems = 100, enableNotifications = true, enableSound = false, showFilters = true, showSearch = true, autoScroll = true, onItemClick, className = '', }) => {
    const [items, setItems] = useState(initialItems);
    const [filteredItems, setFilteredItems] = useState(initialItems);
    const [searchQuery, setSearchQuery] = useState('');
    const [filterType, setFilterType] = useState('all');
    const [filterSeverity, setFilterSeverity] = useState('all');
    const [isConnected, setIsConnected] = useState(false);
    const [isPaused, setIsPaused] = useState(false);
    const wsRef = useRef(null);
    const feedRef = useRef(null);
    const audioRef = useRef(null);
    const { theme, accessibility } = useDashboard();
    useEffect(() => {
        const ws = new WebSocket(wsUrl);
        ws.onopen = () => {
            setIsConnected(true);
            ws.send(JSON.stringify({ type: 'subscribe', channel }));
        };
        ws.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                handleWebSocketMessage(message);
            }
            catch (error) {
                console.error('Failed to parse WebSocket message:', error);
            }
        };
        ws.onclose = () => {
            setIsConnected(false);
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
    const handleWebSocketMessage = useCallback((message) => {
        if (isPaused)
            return;
        switch (message.type) {
            case 'activity':
                addActivityItem(message.payload);
                break;
            case 'alert':
                addActivityItem({
                    ...message.payload,
                    type: 'alert',
                });
                break;
            default:
                break;
        }
    }, [isPaused]);
    const addActivityItem = useCallback((item) => {
        setItems((prev) => {
            const updated = [item, ...prev];
            return updated.slice(0, maxItems);
        });
        if (enableNotifications && item.severity && ['error', 'critical'].includes(item.severity)) {
            showNotification(item);
        }
        if (enableSound) {
            playSound();
        }
        if (autoScroll && feedRef.current) {
            feedRef.current.scrollTop = 0;
        }
    }, [maxItems, enableNotifications, enableSound, autoScroll]);
    const showNotification = useCallback((item) => {
        if ('Notification' in window && Notification.permission === 'granted') {
            new Notification(item.title, {
                body: item.description,
                icon: '/favicon.ico',
                tag: item.id,
            });
        }
    }, []);
    const playSound = useCallback(() => {
        if (audioRef.current) {
            audioRef.current.play().catch(() => {
            });
        }
    }, []);
    useEffect(() => {
        if (enableNotifications && 'Notification' in window) {
            if (Notification.permission === 'default') {
                Notification.requestPermission();
            }
        }
    }, [enableNotifications]);
    useEffect(() => {
        let filtered = [...items];
        if (searchQuery) {
            const query = searchQuery.toLowerCase();
            filtered = filtered.filter((item) => item.title.toLowerCase().includes(query) ||
                item.description.toLowerCase().includes(query));
        }
        if (filterType !== 'all') {
            filtered = filtered.filter((item) => item.type === filterType);
        }
        if (filterSeverity !== 'all') {
            filtered = filtered.filter((item) => item.severity === filterSeverity);
        }
        setFilteredItems(filtered);
    }, [items, searchQuery, filterType, filterSeverity]);
    const markAsRead = useCallback((itemId) => {
        setItems((prev) => prev.map((item) => item.id === itemId ? { ...item, read: true } : item));
    }, []);
    const markAllAsRead = useCallback(() => {
        setItems((prev) => prev.map((item) => ({ ...item, read: true })));
    }, []);
    const clearAll = useCallback(() => {
        setItems([]);
    }, []);
    const togglePause = useCallback(() => {
        setIsPaused((prev) => !prev);
    }, []);
    const unreadCount = items.filter((item) => !item.read).length;
    return (React.createElement("div", { className: `realtime-feed ${className}`, style: styles.container, role: "region", "aria-label": "Real-time activity feed", "aria-live": "polite" },
        React.createElement("div", { style: styles.header },
            React.createElement("div", { style: styles.headerLeft },
                React.createElement("h3", { style: styles.title },
                    "Live Activity Feed",
                    unreadCount > 0 && (React.createElement("span", { style: styles.unreadBadge, "aria-label": `${unreadCount} unread items` }, unreadCount))),
                React.createElement("div", { style: styles.connectionStatus },
                    React.createElement("span", { style: {
                            ...styles.connectionDot,
                            backgroundColor: isConnected ? '#4caf50' : '#f44336',
                        }, "aria-hidden": "true" }),
                    React.createElement("span", { style: styles.connectionText }, isConnected ? 'Connected' : 'Disconnected'))),
            React.createElement("div", { style: styles.headerRight },
                React.createElement("button", { onClick: togglePause, style: styles.iconButton, "aria-label": isPaused ? 'Resume feed' : 'Pause feed', title: isPaused ? 'Resume' : 'Pause' }, isPaused ? '▶' : '⏸'),
                React.createElement("button", { onClick: markAllAsRead, style: styles.iconButton, "aria-label": "Mark all as read", title: "Mark all as read", disabled: unreadCount === 0 }, "\u2713"),
                React.createElement("button", { onClick: clearAll, style: styles.iconButton, "aria-label": "Clear all items", title: "Clear all" }, "\uD83D\uDDD1\uFE0F"))),
        (showSearch || showFilters) && (React.createElement("div", { style: styles.controls },
            showSearch && (React.createElement("input", { type: "text", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value), placeholder: "Search activities...", style: styles.searchInput, "aria-label": "Search activities" })),
            showFilters && (React.createElement("div", { style: styles.filters },
                React.createElement("select", { value: filterType, onChange: (e) => setFilterType(e.target.value), style: styles.filterSelect, "aria-label": "Filter by type" },
                    React.createElement("option", { value: "all" }, "All Types"),
                    React.createElement("option", { value: "user" }, "User"),
                    React.createElement("option", { value: "system" }, "System"),
                    React.createElement("option", { value: "alert" }, "Alert"),
                    React.createElement("option", { value: "metric" }, "Metric"),
                    React.createElement("option", { value: "event" }, "Event")),
                React.createElement("select", { value: filterSeverity, onChange: (e) => setFilterSeverity(e.target.value), style: styles.filterSelect, "aria-label": "Filter by severity" },
                    React.createElement("option", { value: "all" }, "All Severities"),
                    React.createElement("option", { value: "info" }, "Info"),
                    React.createElement("option", { value: "warning" }, "Warning"),
                    React.createElement("option", { value: "error" }, "Error"),
                    React.createElement("option", { value: "critical" }, "Critical")))))),
        React.createElement("div", { ref: feedRef, style: styles.feed, role: "feed", "aria-busy": !isConnected },
            filteredItems.length === 0 && (React.createElement("div", { style: styles.emptyState, role: "status" },
                React.createElement("p", { style: styles.emptyText }, items.length === 0 ? 'No activities yet' : 'No activities match your filters'))),
            filteredItems.map((item) => (React.createElement(FeedItem, { key: item.id, item: item, onClick: () => {
                    markAsRead(item.id);
                    if (onItemClick) {
                        onItemClick(item);
                    }
                }, accessibility: accessibility })))),
        enableSound && (React.createElement("audio", { ref: audioRef, src: "data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1fdJivrJBhNjVgodDbq2EcBj+a2/LDciUFLIHO8tiJNwgZaLvt559NEAxQp+PwtmMcBjiR1/LMeSwFJHfH8N2QQAoUXrTp66hVFApGn+DyvmwhBTWL0fPTgjMGHm7A7+OZURE=" }))));
};
const FeedItem = ({ item, onClick, accessibility }) => {
    const getTypeColor = (type) => {
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
    const getSeverityColor = (severity) => {
        if (!severity)
            return 'transparent';
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
    const getRelativeTime = (timestamp) => {
        const now = new Date();
        const time = new Date(timestamp);
        const diff = now.getTime() - time.getTime();
        const seconds = Math.floor(diff / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);
        if (days > 0)
            return `${days}d ago`;
        if (hours > 0)
            return `${hours}h ago`;
        if (minutes > 0)
            return `${minutes}m ago`;
        return 'Just now';
    };
    return (React.createElement("article", { style: {
            ...styles.feedItem,
            ...(item.read ? {} : styles.feedItemUnread),
            borderLeftColor: item.severity
                ? getSeverityColor(item.severity)
                : getTypeColor(item.type),
        }, onClick: onClick, role: "article", tabIndex: accessibility.keyboardNavigation ? 0 : undefined, "aria-label": `${item.title}. ${item.description}` },
        React.createElement("div", { style: styles.feedItemAvatar }, item.user?.avatar ? (React.createElement("img", { src: item.user.avatar, alt: item.user.name, style: styles.avatarImage })) : (React.createElement("div", { style: {
                ...styles.avatarPlaceholder,
                backgroundColor: getTypeColor(item.type),
            }, "aria-hidden": "true" }, item.icon || getDefaultIcon(item.type)))),
        React.createElement("div", { style: styles.feedItemContent },
            React.createElement("div", { style: styles.feedItemHeader },
                React.createElement("h4", { style: styles.feedItemTitle }, item.title),
                React.createElement("span", { style: styles.feedItemTime, "aria-label": `Time: ${getRelativeTime(item.timestamp)}` }, getRelativeTime(item.timestamp))),
            React.createElement("p", { style: styles.feedItemDescription }, item.description),
            item.resource && (React.createElement("div", { style: styles.feedItemResource },
                React.createElement("span", { style: styles.resourceType }, item.resource.type),
                React.createElement("span", { style: styles.resourceName }, item.resource.name))),
            item.actionUrl && (React.createElement("a", { href: item.actionUrl, style: styles.feedItemAction, onClick: (e) => e.stopPropagation(), "aria-label": item.actionLabel || 'View details' },
                item.actionLabel || 'View Details',
                " \u2192")),
            item.severity && (React.createElement("span", { style: {
                    ...styles.severityBadge,
                    backgroundColor: getSeverityColor(item.severity),
                }, "aria-label": `Severity: ${item.severity}` }, item.severity))),
        !item.read && (React.createElement("div", { style: styles.unreadIndicator, "aria-label": "Unread", role: "status" }))));
};
function getDefaultIcon(type) {
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
const styles = {
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
//# sourceMappingURL=RealtimeFeed.js.map