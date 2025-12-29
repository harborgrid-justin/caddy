/**
 * Team Dashboard Component
 *
 * Main dashboard for team collaboration showing overview, members, and activity.
 */

import React, { useState, useEffect, useMemo } from 'react';
import type {
  Workspace,
  Member,
  Activity,
  TeamMetrics,
  Assignment,
  MemberRole,
  AssignmentStatus,
} from './types';

interface TeamDashboardProps {
  workspaceId: string;
  currentUserId: string;
  onNavigate?: (view: string, data?: any) => void;
}

type DashboardTab = 'overview' | 'members' | 'assignments' | 'activity';

export function TeamDashboard({
  workspaceId,
  currentUserId,
  onNavigate,
}: TeamDashboardProps) {
  const [activeTab, setActiveTab] = useState<DashboardTab>('overview');
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [members, setMembers] = useState<Member[]>([]);
  const [activities, setActivities] = useState<Activity[]>([]);
  const [assignments, setAssignments] = useState<Assignment[]>([]);
  const [metrics, setMetrics] = useState<TeamMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Load workspace data
  useEffect(() => {
    loadWorkspaceData();
  }, [workspaceId]);

  const loadWorkspaceData = async () => {
    setLoading(true);
    setError(null);

    try {
      // In production, these would be API calls
      // For now, using mock data
      await Promise.all([
        loadWorkspace(),
        loadMembers(),
        loadActivities(),
        loadAssignments(),
        loadMetrics(),
      ]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const loadWorkspace = async () => {
    // Mock implementation
    const mockWorkspace: Workspace = {
      id: workspaceId,
      name: 'Engineering Team',
      slug: 'engineering',
      owner_id: 'user1',
      status: 'Active',
      visibility: 'Private',
      settings: {
        auto_assignment_enabled: true,
        require_member_approval: false,
        allow_guest_access: true,
        default_member_role: 'developer',
        activity_tracking_enabled: true,
        email_notifications_enabled: true,
        slack_integration_enabled: false,
        notification_settings: {},
        timezone: 'UTC',
        working_hours: {
          start_hour: 9,
          end_hour: 17,
          working_days: [1, 2, 3, 4, 5],
        },
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      metadata: {},
    };
    setWorkspace(mockWorkspace);
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
        last_active_at: new Date().toISOString(),
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
        last_active_at: new Date().toISOString(),
        metadata: {},
      },
    ];
    setMembers(mockMembers);
  };

  const loadActivities = async () => {
    // Mock implementation
    const mockActivities: Activity[] = [
      {
        id: '1',
        workspace_id: workspaceId,
        user_id: 'user1',
        activity_type: 'IssueCreated',
        description: 'Created issue #123',
        timestamp: new Date().toISOString(),
        metadata: {},
      },
    ];
    setActivities(mockActivities);
  };

  const loadAssignments = async () => {
    // Mock implementation
    setAssignments([]);
  };

  const loadMetrics = async () => {
    // Mock implementation
    const mockMetrics: TeamMetrics = {
      workspace_id: workspaceId,
      period_start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
      period_end: new Date().toISOString(),
      total_activities: 45,
      active_members: 8,
      issues_created: 12,
      issues_completed: 10,
      comments_created: 28,
      completion_rate: 0.83,
      top_users: [
        ['user1', 15],
        ['user2', 12],
      ],
      activity_trend: [],
    };
    setMetrics(mockMetrics);
  };

  // Calculated stats
  const stats = useMemo(() => {
    const activeMembers = members.filter((m) => m.status === 'Active').length;
    const activeAssignments = assignments.filter((a) =>
      ['Pending', 'Accepted', 'InProgress'].includes(a.status)
    ).length;

    return {
      totalMembers: members.length,
      activeMembers,
      activeAssignments,
      recentActivities: activities.length,
    };
  }, [members, assignments, activities]);

  if (loading) {
    return (
      <div className="team-dashboard loading">
        <div className="loading-spinner" />
        <p>Loading team dashboard...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="team-dashboard error">
        <h2>Error Loading Dashboard</h2>
        <p>{error}</p>
        <button onClick={loadWorkspaceData}>Retry</button>
      </div>
    );
  }

  return (
    <div className="team-dashboard">
      {/* Header */}
      <header className="dashboard-header">
        <div className="header-content">
          <h1>{workspace?.name || 'Team Dashboard'}</h1>
          <p className="workspace-description">{workspace?.description}</p>
        </div>
        <div className="header-actions">
          <button
            className="btn-primary"
            onClick={() => onNavigate?.('invite-member')}
          >
            Invite Member
          </button>
          <button
            className="btn-secondary"
            onClick={() => onNavigate?.('settings')}
          >
            Settings
          </button>
        </div>
      </header>

      {/* Stats Cards */}
      <div className="stats-grid">
        <StatCard
          title="Team Members"
          value={stats.totalMembers}
          subtitle={`${stats.activeMembers} active`}
          icon="👥"
          trend={+5}
        />
        <StatCard
          title="Active Assignments"
          value={stats.activeAssignments}
          subtitle="In progress"
          icon="📋"
        />
        <StatCard
          title="Completion Rate"
          value={`${Math.round((metrics?.completion_rate || 0) * 100)}%`}
          subtitle="Last 7 days"
          icon="✓"
          trend={+8}
        />
        <StatCard
          title="Recent Activity"
          value={stats.recentActivities}
          subtitle="Today"
          icon="⚡"
        />
      </div>

      {/* Tabs */}
      <div className="dashboard-tabs">
        <button
          className={activeTab === 'overview' ? 'active' : ''}
          onClick={() => setActiveTab('overview')}
        >
          Overview
        </button>
        <button
          className={activeTab === 'members' ? 'active' : ''}
          onClick={() => setActiveTab('members')}
        >
          Members ({members.length})
        </button>
        <button
          className={activeTab === 'assignments' ? 'active' : ''}
          onClick={() => setActiveTab('assignments')}
        >
          Assignments ({assignments.length})
        </button>
        <button
          className={activeTab === 'activity' ? 'active' : ''}
          onClick={() => setActiveTab('activity')}
        >
          Activity
        </button>
      </div>

      {/* Tab Content */}
      <div className="dashboard-content">
        {activeTab === 'overview' && (
          <OverviewTab
            workspace={workspace}
            members={members}
            metrics={metrics}
            activities={activities.slice(0, 10)}
          />
        )}

        {activeTab === 'members' && (
          <MembersTab members={members} onNavigate={onNavigate} />
        )}

        {activeTab === 'assignments' && (
          <AssignmentsTab assignments={assignments} members={members} />
        )}

        {activeTab === 'activity' && (
          <ActivityTab activities={activities} members={members} />
        )}
      </div>
    </div>
  );
}

// ============================================================================
// Sub-components
// ============================================================================

interface StatCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  icon?: string;
  trend?: number;
}

function StatCard({ title, value, subtitle, icon, trend }: StatCardProps) {
  return (
    <div className="stat-card">
      <div className="stat-icon">{icon}</div>
      <div className="stat-content">
        <h3>{title}</h3>
        <div className="stat-value">{value}</div>
        {subtitle && <p className="stat-subtitle">{subtitle}</p>}
        {trend !== undefined && (
          <div className={`stat-trend ${trend >= 0 ? 'positive' : 'negative'}`}>
            {trend >= 0 ? '↑' : '↓'} {Math.abs(trend)}%
          </div>
        )}
      </div>
    </div>
  );
}

interface OverviewTabProps {
  workspace: Workspace | null;
  members: Member[];
  metrics: TeamMetrics | null;
  activities: Activity[];
}

function OverviewTab({
  workspace,
  members,
  metrics,
  activities,
}: OverviewTabProps) {
  return (
    <div className="overview-tab">
      <div className="overview-grid">
        {/* Team Members Preview */}
        <div className="overview-section">
          <h2>Team Members</h2>
          <div className="member-list">
            {members.slice(0, 5).map((member) => (
              <div key={member.id} className="member-item">
                <div className="member-avatar">
                  {member.display_name?.charAt(0) || '?'}
                </div>
                <div className="member-info">
                  <div className="member-name">{member.display_name}</div>
                  <div className="member-role">{member.role}</div>
                </div>
                <div className={`member-status ${member.status.toLowerCase()}`}>
                  {member.status}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Recent Activity */}
        <div className="overview-section">
          <h2>Recent Activity</h2>
          <div className="activity-list">
            {activities.map((activity) => (
              <div key={activity.id} className="activity-item">
                <div className="activity-icon">⚡</div>
                <div className="activity-content">
                  <p>{activity.description}</p>
                  <span className="activity-time">
                    {formatTimestamp(activity.timestamp)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Metrics */}
        {metrics && (
          <div className="overview-section full-width">
            <h2>Team Metrics</h2>
            <div className="metrics-grid">
              <div className="metric">
                <div className="metric-label">Total Activities</div>
                <div className="metric-value">{metrics.total_activities}</div>
              </div>
              <div className="metric">
                <div className="metric-label">Issues Created</div>
                <div className="metric-value">{metrics.issues_created}</div>
              </div>
              <div className="metric">
                <div className="metric-label">Issues Completed</div>
                <div className="metric-value">{metrics.issues_completed}</div>
              </div>
              <div className="metric">
                <div className="metric-label">Comments</div>
                <div className="metric-value">{metrics.comments_created}</div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

interface MembersTabProps {
  members: Member[];
  onNavigate?: (view: string, data?: any) => void;
}

function MembersTab({ members, onNavigate }: MembersTabProps) {
  const [filter, setFilter] = useState<string>('all');

  const filteredMembers = useMemo(() => {
    if (filter === 'all') return members;
    return members.filter((m) => m.status.toLowerCase() === filter.toLowerCase());
  }, [members, filter]);

  return (
    <div className="members-tab">
      <div className="members-header">
        <select value={filter} onChange={(e) => setFilter(e.target.value)}>
          <option value="all">All Members</option>
          <option value="active">Active</option>
          <option value="pending">Pending</option>
          <option value="inactive">Inactive</option>
        </select>
      </div>

      <div className="members-table">
        <table>
          <thead>
            <tr>
              <th>Member</th>
              <th>Email</th>
              <th>Role</th>
              <th>Status</th>
              <th>Joined</th>
              <th>Last Active</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {filteredMembers.map((member) => (
              <tr key={member.id}>
                <td>
                  <div className="member-cell">
                    <div className="member-avatar">
                      {member.display_name?.charAt(0) || '?'}
                    </div>
                    {member.display_name || member.email}
                  </div>
                </td>
                <td>{member.email}</td>
                <td>
                  <span className={`role-badge ${member.role.toLowerCase()}`}>
                    {member.role}
                  </span>
                </td>
                <td>
                  <span className={`status-badge ${member.status.toLowerCase()}`}>
                    {member.status}
                  </span>
                </td>
                <td>{formatDate(member.joined_at)}</td>
                <td>
                  {member.last_active_at
                    ? formatTimestamp(member.last_active_at)
                    : 'Never'}
                </td>
                <td>
                  <button
                    className="btn-small"
                    onClick={() => onNavigate?.('member-details', member)}
                  >
                    View
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

interface AssignmentsTabProps {
  assignments: Assignment[];
  members: Member[];
}

function AssignmentsTab({ assignments, members }: AssignmentsTabProps) {
  const getMemberName = (userId: string) => {
    const member = members.find((m) => m.user_id === userId);
    return member?.display_name || member?.email || userId;
  };

  return (
    <div className="assignments-tab">
      {assignments.length === 0 ? (
        <div className="empty-state">
          <p>No assignments yet</p>
        </div>
      ) : (
        <div className="assignments-list">
          {assignments.map((assignment) => (
            <div key={assignment.id} className="assignment-card">
              <div className="assignment-header">
                <h3>Issue #{assignment.issue_id}</h3>
                <span
                  className={`priority-badge ${assignment.priority.toLowerCase()}`}
                >
                  {assignment.priority}
                </span>
              </div>
              <div className="assignment-body">
                <div className="assignment-field">
                  <label>Assignee:</label>
                  <span>{getMemberName(assignment.assignee_id)}</span>
                </div>
                <div className="assignment-field">
                  <label>Status:</label>
                  <span className={`status-badge ${assignment.status.toLowerCase()}`}>
                    {assignment.status}
                  </span>
                </div>
                {assignment.due_date && (
                  <div className="assignment-field">
                    <label>Due Date:</label>
                    <span>{formatDate(assignment.due_date)}</span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

interface ActivityTabProps {
  activities: Activity[];
  members: Member[];
}

function ActivityTab({ activities, members }: ActivityTabProps) {
  const getMemberName = (userId: string) => {
    const member = members.find((m) => m.user_id === userId);
    return member?.display_name || member?.email || userId;
  };

  return (
    <div className="activity-tab">
      <div className="activity-feed">
        {activities.map((activity) => (
          <div key={activity.id} className="activity-entry">
            <div className="activity-avatar">
              {getMemberName(activity.user_id).charAt(0)}
            </div>
            <div className="activity-details">
              <div className="activity-header">
                <strong>{getMemberName(activity.user_id)}</strong>
                <span className="activity-type">
                  {formatActivityType(activity.activity_type)}
                </span>
              </div>
              <p className="activity-description">{activity.description}</p>
              <span className="activity-timestamp">
                {formatTimestamp(activity.timestamp)}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ============================================================================
// Utility Functions
// ============================================================================

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return 'Just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;

  return date.toLocaleDateString();
}

function formatDate(timestamp: string): string {
  return new Date(timestamp).toLocaleDateString();
}

function formatActivityType(type: any): string {
  if (typeof type === 'string') {
    return type.replace(/([A-Z])/g, ' $1').trim();
  }
  if (typeof type === 'object' && type.Custom) {
    return type.Custom;
  }
  return 'Unknown';
}
