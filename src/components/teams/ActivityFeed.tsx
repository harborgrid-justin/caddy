/**
 * Activity Feed Component
 *
 * Real-time activity stream with filtering and search capabilities.
 */

import React, { useState, useEffect, useMemo, useRef } from 'react';
import type {
  Activity,
  ActivityType,
  ActivityFilter,
  Member,
} from './types';

interface ActivityFeedProps {
  workspaceId: string;
  realtime?: boolean;
  compact?: boolean;
  limit?: number;
  onActivityClick?: (activity: Activity) => void;
}

export function ActivityFeed({
  workspaceId,
  realtime = true,
  compact = false,
  limit = 50,
  onActivityClick,
}: ActivityFeedProps) {
  const [activities, setActivities] = useState<Activity[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [filter, setFilter] = useState<ActivityFilter>({ workspace_id: workspaceId });
  const [loading, setLoading] = useState(true);
  const [autoScroll, setAutoScroll] = useState(true);
  const feedRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadActivities();
    loadMembers();

    // Set up real-time updates
    if (realtime) {
      const interval = setInterval(() => {
        loadActivities();
      }, 5000); // Poll every 5 seconds

      return () => clearInterval(interval);
    }
  }, [workspaceId, realtime]);

  const loadActivities = async () => {
    setLoading(true);
    try {
      // Mock implementation
      const mockActivities: Activity[] = generateMockActivities(workspaceId);
      setActivities(mockActivities.slice(0, limit));
    } finally {
      setLoading(false);
    }
  };

  const loadMembers = async () => {
    // Mock implementation
    const mockMembers: Member[] = [
      {
        id: '1',
        workspace_id: workspaceId,
        user_id: 'user1',
        email: 'alice@example.com',
        display_name: 'Alice Johnson',
        role: 'Owner',
        status: 'Active',
        joined_at: new Date().toISOString(),
        metadata: {},
      },
      {
        id: '2',
        workspace_id: workspaceId,
        user_id: 'user2',
        email: 'bob@example.com',
        display_name: 'Bob Smith',
        role: 'Developer',
        status: 'Active',
        joined_at: new Date().toISOString(),
        metadata: {},
      },
    ];
    setMembers(mockMembers);
  };

  const filteredActivities = useMemo(() => {
    let filtered = activities;

    if (filter.user_id) {
      filtered = filtered.filter((a) => a.user_id === filter.user_id);
    }

    if (filter.activity_types && filter.activity_types.length > 0) {
      filtered = filtered.filter((a) =>
        filter.activity_types!.includes(a.activity_type)
      );
    }

    if (filter.security_only) {
      filtered = filtered.filter((a) => isSecurityEvent(a.activity_type));
    }

    return filtered;
  }, [activities, filter]);

  const getMemberName = (userId: string): string => {
    const member = members.find((m) => m.user_id === userId);
    return member?.display_name || member?.email || userId;
  };

  useEffect(() => {
    if (autoScroll && feedRef.current) {
      feedRef.current.scrollTop = 0;
    }
  }, [activities, autoScroll]);

  return (
    <div className={`activity-feed ${compact ? 'compact' : ''}`}>
      {/* Header */}
      <div className="feed-header">
        <div className="feed-title">
          <h2>Activity Feed</h2>
          {realtime && (
            <span className="live-indicator">
              <span className="pulse"></span>
              Live
            </span>
          )}
        </div>

        <div className="feed-controls">
          <label className="auto-scroll-toggle">
            <input
              type="checkbox"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
            />
            <span>Auto-scroll</span>
          </label>
        </div>
      </div>

      {/* Filters */}
      <ActivityFilters
        filter={filter}
        members={members}
        onChange={setFilter}
      />

      {/* Activity List */}
      <div className="feed-content" ref={feedRef}>
        {loading && activities.length === 0 ? (
          <div className="feed-loading">
            <div className="loading-spinner" />
            <p>Loading activity...</p>
          </div>
        ) : filteredActivities.length === 0 ? (
          <div className="feed-empty">
            <p>No activity to display</p>
          </div>
        ) : (
          <div className="activity-list">
            {filteredActivities.map((activity) => (
              <ActivityItem
                key={activity.id}
                activity={activity}
                memberName={getMemberName(activity.user_id)}
                compact={compact}
                onClick={() => onActivityClick?.(activity)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// Sub-components
// ============================================================================

interface ActivityFiltersProps {
  filter: ActivityFilter;
  members: Member[];
  onChange: (filter: ActivityFilter) => void;
}

function ActivityFilters({ filter, members, onChange }: ActivityFiltersProps) {
  const [showAdvanced, setShowAdvanced] = useState(false);

  return (
    <div className="activity-filters">
      <div className="filters-row">
        <select
          value={filter.user_id || ''}
          onChange={(e) =>
            onChange({
              ...filter,
              user_id: e.target.value || undefined,
            })
          }
        >
          <option value="">All Members</option>
          {members.map((member) => (
            <option key={member.id} value={member.user_id}>
              {member.display_name || member.email}
            </option>
          ))}
        </select>

        <button
          className="btn-small"
          onClick={() => setShowAdvanced(!showAdvanced)}
        >
          {showAdvanced ? 'Hide' : 'More'} Filters
        </button>
      </div>

      {showAdvanced && (
        <div className="advanced-filters">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={filter.security_only || false}
              onChange={(e) =>
                onChange({ ...filter, security_only: e.target.checked })
              }
            />
            <span>Security events only</span>
          </label>

          <div className="filter-group">
            <label>Activity Types:</label>
            <div className="checkbox-group">
              {[
                'WorkspaceCreated',
                'MemberJoined',
                'IssueCreated',
                'IssueAssigned',
                'CommentCreated',
              ].map((type) => (
                <label key={type} className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={filter.activity_types?.includes(type as ActivityType) || false}
                    onChange={(e) => {
                      const types = filter.activity_types || [];
                      onChange({
                        ...filter,
                        activity_types: e.target.checked
                          ? [...types, type as ActivityType]
                          : types.filter((t) => t !== type),
                      });
                    }}
                  />
                  <span>{formatActivityType(type)}</span>
                </label>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

interface ActivityItemProps {
  activity: Activity;
  memberName: string;
  compact?: boolean;
  onClick?: () => void;
}

function ActivityItem({
  activity,
  memberName,
  compact = false,
  onClick,
}: ActivityItemProps) {
  const activityIcon = getActivityIcon(activity.activity_type);
  const activityColor = getActivityColor(activity.activity_type);

  return (
    <div
      className={`activity-item ${compact ? 'compact' : ''} ${activityColor}`}
      onClick={onClick}
    >
      <div className="activity-icon">{activityIcon}</div>

      <div className="activity-content">
        <div className="activity-header">
          <span className="activity-user">{memberName}</span>
          <span className="activity-type">
            {formatActivityType(activity.activity_type)}
          </span>
        </div>

        <p className="activity-description">{activity.description}</p>

        {!compact && activity.resource_id && (
          <div className="activity-meta">
            <span className="resource-type">{activity.resource_type}</span>
            <span className="resource-id">{activity.resource_id}</span>
          </div>
        )}

        <div className="activity-footer">
          <span className="activity-time">
            {formatTimestamp(activity.timestamp)}
          </span>

          {activity.changes && (
            <span className="activity-changes">
              {Object.keys(activity.changes.after).length} changes
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Utility Functions
// ============================================================================

function generateMockActivities(workspaceId: string): Activity[] {
  const types: ActivityType[] = [
    'WorkspaceCreated',
    'MemberJoined',
    'IssueCreated',
    'IssueAssigned',
    'IssueCompleted',
    'CommentCreated',
  ];

  const users = ['user1', 'user2', 'user3'];

  return Array.from({ length: 30 }, (_, i) => ({
    id: `activity-${i}`,
    workspace_id: workspaceId,
    user_id: users[i % users.length],
    activity_type: types[i % types.length],
    description: `Mock activity ${i + 1}`,
    timestamp: new Date(Date.now() - i * 5 * 60 * 1000).toISOString(),
    metadata: {},
  }));
}

function formatActivityType(type: ActivityType | string): string {
  if (typeof type === 'string') {
    return type.replace(/([A-Z])/g, ' $1').trim();
  }
  if (typeof type === 'object' && 'Custom' in type) {
    return type.Custom;
  }
  return 'Unknown';
}

function getActivityIcon(type: ActivityType): string {
  const typeStr = typeof type === 'string' ? type : 'Custom';

  const iconMap: Record<string, string> = {
    WorkspaceCreated: '🏢',
    WorkspaceUpdated: '✏️',
    MemberInvited: '📧',
    MemberJoined: '👋',
    IssueCreated: '📝',
    IssueAssigned: '👤',
    IssueCompleted: '✅',
    CommentCreated: '💬',
    LoginSuccess: '🔐',
    LoginFailure: '⚠️',
  };

  return iconMap[typeStr] || '⚡';
}

function getActivityColor(type: ActivityType): string {
  const typeStr = typeof type === 'string' ? type : 'Custom';

  if (typeStr.includes('Success') || typeStr.includes('Completed')) {
    return 'success';
  }
  if (typeStr.includes('Failure') || typeStr.includes('Error')) {
    return 'error';
  }
  if (typeStr.includes('Created') || typeStr.includes('Invited')) {
    return 'info';
  }

  return 'default';
}

function isSecurityEvent(type: ActivityType): boolean {
  const typeStr = typeof type === 'string' ? type : '';
  return (
    typeStr.includes('Login') ||
    typeStr.includes('Password') ||
    typeStr.includes('TwoFactor') ||
    typeStr.includes('Permission')
  );
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (seconds < 60) return 'Just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;

  return date.toLocaleDateString();
}
