/**
 * Member Management Component
 *
 * Comprehensive interface for managing team members, invitations, and roles.
 */

import React, { useState, useEffect, useMemo } from 'react';
import type {
  Member,
  MemberInvitation,
  MemberRole,
  MemberStatus,
  InvitationStatus,
  MemberActivity,
} from './types';

interface MemberManagementProps {
  workspaceId: string;
  currentUserId: string;
  currentUserRole: MemberRole;
}

type ViewMode = 'members' | 'invitations' | 'activity';

export function MemberManagement({
  workspaceId,
  currentUserId,
  currentUserRole,
}: MemberManagementProps) {
  const [viewMode, setViewMode] = useState<ViewMode>('members');
  const [members, setMembers] = useState<Member[]>([]);
  const [invitations, setInvitations] = useState<MemberInvitation[]>([]);
  const [activities, setActivities] = useState<Map<string, MemberActivity>>(new Map());
  const [selectedMember, setSelectedMember] = useState<Member | null>(null);
  const [showInviteDialog, setShowInviteDialog] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, [workspaceId]);

  const loadData = async () => {
    setLoading(true);
    try {
      await Promise.all([loadMembers(), loadInvitations(), loadActivities()]);
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
        joined_at: new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString(),
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
        joined_at: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000).toISOString(),
        last_active_at: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
        manager_id: 'user1',
        metadata: {},
      },
    ];
    setMembers(mockMembers);
  };

  const loadInvitations = async () => {
    // Mock implementation
    const mockInvitations: MemberInvitation[] = [
      {
        id: '1',
        workspace_id: workspaceId,
        user_id: 'user3',
        email: 'carol@example.com',
        role: 'Developer',
        token: 'invite-token-123',
        invited_by: 'user1',
        created_at: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
        expires_at: new Date(Date.now() + 5 * 24 * 60 * 60 * 1000).toISOString(),
        status: 'Pending',
      },
    ];
    setInvitations(mockInvitations);
  };

  const loadActivities = async () => {
    // Mock implementation
    const mockActivities = new Map<string, MemberActivity>([
      [
        'user1',
        {
          member_id: 'user1',
          last_login: new Date().toISOString(),
          total_actions: 342,
          weekly_actions: 45,
          daily_actions: 12,
          last_action_at: new Date().toISOString(),
          last_action_type: 'IssueCreated',
        },
      ],
      [
        'user2',
        {
          member_id: 'user2',
          last_login: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
          total_actions: 156,
          weekly_actions: 28,
          daily_actions: 6,
          last_action_at: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
          last_action_type: 'CommentCreated',
        },
      ],
    ]);
    setActivities(mockActivities);
  };

  const canManageMembers = useMemo(() => {
    return ['Owner', 'Admin', 'Manager'].includes(currentUserRole);
  }, [currentUserRole]);

  if (loading) {
    return (
      <div className="member-management loading">
        <div className="loading-spinner" />
        <p>Loading members...</p>
      </div>
    );
  }

  return (
    <div className="member-management">
      {/* Header */}
      <header className="management-header">
        <div>
          <h1>Team Members</h1>
          <p className="header-subtitle">
            Manage team members, invitations, and permissions
          </p>
        </div>
        {canManageMembers && (
          <button
            className="btn-primary"
            onClick={() => setShowInviteDialog(true)}
          >
            Invite Member
          </button>
        )}
      </header>

      {/* View Mode Tabs */}
      <div className="view-mode-tabs">
        <button
          className={viewMode === 'members' ? 'active' : ''}
          onClick={() => setViewMode('members')}
        >
          Members ({members.length})
        </button>
        <button
          className={viewMode === 'invitations' ? 'active' : ''}
          onClick={() => setViewMode('invitations')}
        >
          Invitations ({invitations.filter((i) => i.status === 'Pending').length})
        </button>
        <button
          className={viewMode === 'activity' ? 'active' : ''}
          onClick={() => setViewMode('activity')}
        >
          Activity
        </button>
      </div>

      {/* Content */}
      <div className="management-content">
        {viewMode === 'members' && (
          <MembersView
            members={members}
            activities={activities}
            canManage={canManageMembers}
            onSelect={setSelectedMember}
            onUpdate={loadMembers}
          />
        )}

        {viewMode === 'invitations' && (
          <InvitationsView
            invitations={invitations}
            canManage={canManageMembers}
            onUpdate={loadInvitations}
          />
        )}

        {viewMode === 'activity' && (
          <ActivityView members={members} activities={activities} />
        )}
      </div>

      {/* Invite Dialog */}
      {showInviteDialog && (
        <InviteMemberDialog
          workspaceId={workspaceId}
          onClose={() => setShowInviteDialog(false)}
          onInvite={() => {
            setShowInviteDialog(false);
            loadInvitations();
          }}
        />
      )}

      {/* Member Details Dialog */}
      {selectedMember && (
        <MemberDetailsDialog
          member={selectedMember}
          activity={activities.get(selectedMember.user_id)}
          canManage={canManageMembers}
          onClose={() => setSelectedMember(null)}
          onUpdate={loadMembers}
        />
      )}
    </div>
  );
}

// ============================================================================
// Sub-components
// ============================================================================

interface MembersViewProps {
  members: Member[];
  activities: Map<string, MemberActivity>;
  canManage: boolean;
  onSelect: (member: Member) => void;
  onUpdate: () => void;
}

function MembersView({
  members,
  activities,
  canManage,
  onSelect,
  onUpdate,
}: MembersViewProps) {
  const [filter, setFilter] = useState<MemberStatus | 'all'>('all');
  const [search, setSearch] = useState('');

  const filteredMembers = useMemo(() => {
    let filtered = members;

    if (filter !== 'all') {
      filtered = filtered.filter((m) => m.status === filter);
    }

    if (search) {
      const searchLower = search.toLowerCase();
      filtered = filtered.filter(
        (m) =>
          m.display_name?.toLowerCase().includes(searchLower) ||
          m.email.toLowerCase().includes(searchLower)
      );
    }

    return filtered;
  }, [members, filter, search]);

  return (
    <div className="members-view">
      {/* Filters */}
      <div className="members-filters">
        <input
          type="text"
          placeholder="Search members..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="search-input"
        />
        <select value={filter} onChange={(e) => setFilter(e.target.value as any)}>
          <option value="all">All Status</option>
          <option value="Active">Active</option>
          <option value="Pending">Pending</option>
          <option value="Inactive">Inactive</option>
          <option value="Suspended">Suspended</option>
        </select>
      </div>

      {/* Members Table */}
      <div className="members-table">
        <table>
          <thead>
            <tr>
              <th>Member</th>
              <th>Role</th>
              <th>Status</th>
              <th>Joined</th>
              <th>Last Active</th>
              <th>Activity</th>
              {canManage && <th>Actions</th>}
            </tr>
          </thead>
          <tbody>
            {filteredMembers.map((member) => {
              const activity = activities.get(member.user_id);
              return (
                <tr key={member.id} onClick={() => onSelect(member)}>
                  <td>
                    <div className="member-cell">
                      <div className="member-avatar">
                        {member.display_name?.charAt(0) || '?'}
                      </div>
                      <div className="member-info">
                        <div className="member-name">
                          {member.display_name || member.email}
                        </div>
                        <div className="member-email">{member.email}</div>
                      </div>
                    </div>
                  </td>
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
                      ? formatRelativeTime(member.last_active_at)
                      : 'Never'}
                  </td>
                  <td>
                    {activity && (
                      <div className="activity-stats">
                        <span title="Daily actions">{activity.daily_actions}d</span>
                        <span title="Weekly actions">{activity.weekly_actions}w</span>
                      </div>
                    )}
                  </td>
                  {canManage && (
                    <td>
                      <button
                        className="btn-small"
                        onClick={(e) => {
                          e.stopPropagation();
                          onSelect(member);
                        }}
                      >
                        Manage
                      </button>
                    </td>
                  )}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

interface InvitationsViewProps {
  invitations: MemberInvitation[];
  canManage: boolean;
  onUpdate: () => void;
}

function InvitationsView({
  invitations,
  canManage,
  onUpdate,
}: InvitationsViewProps) {
  const handleResend = async (invitation: MemberInvitation) => {
    // Implementation
    console.log('Resending invitation:', invitation.id);
  };

  const handleCancel = async (invitation: MemberInvitation) => {
    if (confirm('Cancel this invitation?')) {
      // Implementation
      console.log('Cancelling invitation:', invitation.id);
      onUpdate();
    }
  };

  return (
    <div className="invitations-view">
      {invitations.length === 0 ? (
        <div className="empty-state">
          <p>No pending invitations</p>
        </div>
      ) : (
        <div className="invitations-list">
          {invitations.map((invitation) => (
            <div key={invitation.id} className="invitation-card">
              <div className="invitation-header">
                <div>
                  <h3>{invitation.email}</h3>
                  <span className={`role-badge ${invitation.role.toLowerCase()}`}>
                    {invitation.role}
                  </span>
                </div>
                <span
                  className={`status-badge ${invitation.status.toLowerCase()}`}
                >
                  {invitation.status}
                </span>
              </div>

              <div className="invitation-details">
                <div className="detail-row">
                  <span className="label">Invited by:</span>
                  <span>{invitation.invited_by}</span>
                </div>
                <div className="detail-row">
                  <span className="label">Created:</span>
                  <span>{formatDate(invitation.created_at)}</span>
                </div>
                <div className="detail-row">
                  <span className="label">Expires:</span>
                  <span>{formatDate(invitation.expires_at)}</span>
                </div>
              </div>

              {canManage && invitation.status === 'Pending' && (
                <div className="invitation-actions">
                  <button
                    className="btn-secondary btn-small"
                    onClick={() => handleResend(invitation)}
                  >
                    Resend
                  </button>
                  <button
                    className="btn-danger btn-small"
                    onClick={() => handleCancel(invitation)}
                  >
                    Cancel
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

interface ActivityViewProps {
  members: Member[];
  activities: Map<string, MemberActivity>;
}

function ActivityView({ members, activities }: ActivityViewProps) {
  const sortedMembers = useMemo(() => {
    return [...members].sort((a, b) => {
      const activityA = activities.get(a.user_id);
      const activityB = activities.get(b.user_id);
      return (activityB?.weekly_actions || 0) - (activityA?.weekly_actions || 0);
    });
  }, [members, activities]);

  return (
    <div className="activity-view">
      <h2>Member Activity (Last 7 Days)</h2>

      <div className="activity-list">
        {sortedMembers.map((member) => {
          const activity = activities.get(member.user_id);
          if (!activity) return null;

          return (
            <div key={member.id} className="activity-card">
              <div className="activity-header">
                <div className="member-info">
                  <div className="member-avatar">
                    {member.display_name?.charAt(0) || '?'}
                  </div>
                  <div>
                    <div className="member-name">{member.display_name}</div>
                    <div className="member-role">{member.role}</div>
                  </div>
                </div>
              </div>

              <div className="activity-stats-grid">
                <div className="stat">
                  <div className="stat-label">Total Actions</div>
                  <div className="stat-value">{activity.total_actions}</div>
                </div>
                <div className="stat">
                  <div className="stat-label">This Week</div>
                  <div className="stat-value">{activity.weekly_actions}</div>
                </div>
                <div className="stat">
                  <div className="stat-label">Today</div>
                  <div className="stat-value">{activity.daily_actions}</div>
                </div>
                <div className="stat">
                  <div className="stat-label">Last Action</div>
                  <div className="stat-value">
                    {activity.last_action_at
                      ? formatRelativeTime(activity.last_action_at)
                      : 'Never'}
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

// ============================================================================
// Dialogs
// ============================================================================

interface InviteMemberDialogProps {
  workspaceId: string;
  onClose: () => void;
  onInvite: () => void;
}

function InviteMemberDialog({
  workspaceId,
  onClose,
  onInvite,
}: InviteMemberDialogProps) {
  const [email, setEmail] = useState('');
  const [role, setRole] = useState<MemberRole>('Developer');
  const [message, setMessage] = useState('');
  const [sending, setSending] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSending(true);

    try {
      // In production, would call API
      await new Promise((resolve) => setTimeout(resolve, 1000));
      onInvite();
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>Invite Team Member</h2>
          <button className="dialog-close" onClick={onClose}>
            ×
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="invite-email">Email Address</label>
            <input
              id="invite-email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="user@example.com"
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="invite-role">Role</label>
            <select
              id="invite-role"
              value={role}
              onChange={(e) => setRole(e.target.value as MemberRole)}
            >
              <option value="Viewer">Viewer</option>
              <option value="Developer">Developer</option>
              <option value="Designer">Designer</option>
              <option value="Reviewer">Reviewer</option>
              <option value="Manager">Manager</option>
              <option value="Admin">Admin</option>
            </select>
          </div>

          <div className="form-group">
            <label htmlFor="invite-message">Message (Optional)</label>
            <textarea
              id="invite-message"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              placeholder="Add a personal message..."
              rows={3}
            />
          </div>

          <div className="dialog-actions">
            <button
              type="button"
              className="btn-secondary"
              onClick={onClose}
              disabled={sending}
            >
              Cancel
            </button>
            <button type="submit" className="btn-primary" disabled={sending}>
              {sending ? 'Sending...' : 'Send Invitation'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

interface MemberDetailsDialogProps {
  member: Member;
  activity?: MemberActivity;
  canManage: boolean;
  onClose: () => void;
  onUpdate: () => void;
}

function MemberDetailsDialog({
  member,
  activity,
  canManage,
  onClose,
  onUpdate,
}: MemberDetailsDialogProps) {
  const [editingRole, setEditingRole] = useState(false);
  const [newRole, setNewRole] = useState(member.role);

  const handleUpdateRole = async () => {
    // Implementation
    console.log('Updating role to:', newRole);
    setEditingRole(false);
    onUpdate();
  };

  const handleRemove = async () => {
    if (confirm(`Remove ${member.display_name} from workspace?`)) {
      // Implementation
      console.log('Removing member:', member.id);
      onClose();
      onUpdate();
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog large" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>Member Details</h2>
          <button className="dialog-close" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="member-details">
          <div className="member-profile">
            <div className="member-avatar large">
              {member.display_name?.charAt(0) || '?'}
            </div>
            <h3>{member.display_name || member.email}</h3>
            <p>{member.email}</p>
          </div>

          <div className="details-grid">
            <div className="detail-section">
              <h4>Role</h4>
              {editingRole && canManage ? (
                <div className="edit-role">
                  <select
                    value={newRole}
                    onChange={(e) => setNewRole(e.target.value as MemberRole)}
                  >
                    <option value="Viewer">Viewer</option>
                    <option value="Developer">Developer</option>
                    <option value="Designer">Designer</option>
                    <option value="Reviewer">Reviewer</option>
                    <option value="Manager">Manager</option>
                    <option value="Admin">Admin</option>
                    <option value="Owner">Owner</option>
                  </select>
                  <button className="btn-small" onClick={handleUpdateRole}>
                    Save
                  </button>
                  <button
                    className="btn-small"
                    onClick={() => setEditingRole(false)}
                  >
                    Cancel
                  </button>
                </div>
              ) : (
                <div>
                  <span className={`role-badge ${member.role.toLowerCase()}`}>
                    {member.role}
                  </span>
                  {canManage && (
                    <button
                      className="btn-link"
                      onClick={() => setEditingRole(true)}
                    >
                      Change
                    </button>
                  )}
                </div>
              )}
            </div>

            <div className="detail-section">
              <h4>Status</h4>
              <span className={`status-badge ${member.status.toLowerCase()}`}>
                {member.status}
              </span>
            </div>

            <div className="detail-section">
              <h4>Joined</h4>
              <p>{formatDate(member.joined_at)}</p>
            </div>

            <div className="detail-section">
              <h4>Last Active</h4>
              <p>
                {member.last_active_at
                  ? formatRelativeTime(member.last_active_at)
                  : 'Never'}
              </p>
            </div>
          </div>

          {activity && (
            <div className="detail-section">
              <h4>Activity Statistics</h4>
              <div className="stats-grid">
                <div>
                  <span className="stat-label">Total Actions:</span>
                  <span className="stat-value">{activity.total_actions}</span>
                </div>
                <div>
                  <span className="stat-label">This Week:</span>
                  <span className="stat-value">{activity.weekly_actions}</span>
                </div>
                <div>
                  <span className="stat-label">Today:</span>
                  <span className="stat-value">{activity.daily_actions}</span>
                </div>
              </div>
            </div>
          )}

          {canManage && member.role !== 'Owner' && (
            <div className="dialog-actions">
              <button className="btn-danger" onClick={handleRemove}>
                Remove from Workspace
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Utility Functions
// ============================================================================

function formatDate(timestamp: string): string {
  return new Date(timestamp).toLocaleDateString();
}

function formatRelativeTime(timestamp: string): string {
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
