/**
 * CADDY v0.4.0 Enterprise Team Settings
 * Roles, permissions, team management
 */

import React, { useState, useCallback } from 'react';
import {
  TeamSettings as TeamSettingsType,
  TeamMember,
  Role,
  Invitation,
  Permission,
  ToastNotification,
  ConfirmationDialog,
  SettingsHistory,
} from './types';

interface TeamSettingsProps {
  onSave: (section: string, data: TeamSettingsType) => Promise<void>;
  onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
  addToast: (toast: Omit<ToastNotification, 'id'>) => void;
  addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}

const AVAILABLE_PERMISSIONS = [
  { resource: 'users', actions: ['create', 'read', 'update', 'delete', 'manage'] },
  { resource: 'projects', actions: ['create', 'read', 'update', 'delete', 'manage'] },
  { resource: 'settings', actions: ['read', 'update', 'manage'] },
  { resource: 'billing', actions: ['read', 'update', 'manage'] },
  { resource: 'analytics', actions: ['read', 'manage'] },
  { resource: 'api', actions: ['create', 'read', 'delete', 'manage'] },
];

const TeamSettings: React.FC<TeamSettingsProps> = ({
  onSave,
  onConfirm,
  addToast,
  addToHistory,
}) => {
  const [settings, setSettings] = useState<TeamSettingsType>({
    id: 'team-1',
    version: 1,
    updatedAt: new Date(),
    updatedBy: 'current-user',
    members: [
      {
        id: 'user-1',
        email: 'admin@example.com',
        name: 'Admin User',
        role: 'admin',
        status: 'active',
        joinedAt: new Date('2024-01-01'),
        lastActive: new Date(),
        permissions: [],
        groups: [],
      },
      {
        id: 'user-2',
        email: 'developer@example.com',
        name: 'Developer User',
        role: 'developer',
        status: 'active',
        joinedAt: new Date('2024-02-15'),
        lastActive: new Date('2025-01-28'),
        permissions: [],
        groups: ['engineering'],
      },
    ],
    roles: [
      {
        id: 'role-admin',
        name: 'Admin',
        description: 'Full system access',
        permissions: [],
        isSystem: true,
        memberCount: 1,
      },
      {
        id: 'role-developer',
        name: 'Developer',
        description: 'Development and API access',
        permissions: [],
        isSystem: true,
        memberCount: 1,
      },
      {
        id: 'role-viewer',
        name: 'Viewer',
        description: 'Read-only access',
        permissions: [],
        isSystem: true,
        memberCount: 0,
      },
    ],
    invitations: [
      {
        id: 'inv-1',
        email: 'newuser@example.com',
        role: 'developer',
        invitedBy: 'admin@example.com',
        invitedAt: new Date('2025-01-25'),
        expiresAt: new Date('2025-02-01'),
        status: 'pending',
      },
    ],
    groups: [
      {
        id: 'group-eng',
        name: 'Engineering',
        description: 'Engineering team',
        memberIds: ['user-2'],
        permissions: [],
      },
    ],
  });

  const [showInviteModal, setShowInviteModal] = useState(false);
  const [showRoleModal, setShowRoleModal] = useState(false);
  const [inviteEmail, setInviteEmail] = useState('');
  const [inviteRole, setInviteRole] = useState('developer');

  // Invite member
  const inviteMember = useCallback(async () => {
    if (!inviteEmail || !inviteEmail.includes('@')) {
      addToast({ type: 'error', message: 'Please enter a valid email address' });
      return;
    }

    try {
      const newInvitation: Invitation = {
        id: `inv-${Date.now()}`,
        email: inviteEmail,
        role: inviteRole,
        invitedBy: 'current-user@example.com',
        invitedAt: new Date(),
        expiresAt: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000), // 7 days
        status: 'pending',
      };

      setSettings((prev) => ({
        ...prev,
        invitations: [...prev.invitations, newInvitation],
      }));

      addToast({
        type: 'success',
        message: `Invitation sent to ${inviteEmail}`,
      });

      addToHistory({
        section: 'Team Settings',
        action: 'create',
        changes: [{ field: 'invitation', oldValue: null, newValue: inviteEmail }],
        userId: 'current-user',
        userName: 'Current User',
      });

      setInviteEmail('');
      setShowInviteModal(false);
    } catch (error) {
      addToast({
        type: 'error',
        message: 'Failed to send invitation',
      });
    }
  }, [inviteEmail, inviteRole, addToast, addToHistory]);

  // Revoke invitation
  const revokeInvitation = useCallback(
    (id: string) => {
      onConfirm({
        title: 'Revoke Invitation',
        message: 'Are you sure you want to revoke this invitation?',
        severity: 'warning',
        confirmText: 'Revoke',
        cancelText: 'Cancel',
        onConfirm: () => {
          setSettings((prev) => ({
            ...prev,
            invitations: prev.invitations.filter((inv) => inv.id !== id),
          }));
          addToast({ type: 'success', message: 'Invitation revoked' });
        },
        onCancel: () => {},
      });
    },
    [onConfirm, addToast]
  );

  // Remove member
  const removeMember = useCallback(
    (id: string) => {
      const member = settings.members.find((m) => m.id === id);
      if (!member) return;

      onConfirm({
        title: 'Remove Team Member',
        message: `Are you sure you want to remove ${member.name} from the team? This action cannot be undone.`,
        severity: 'error',
        confirmText: 'Remove',
        cancelText: 'Cancel',
        onConfirm: () => {
          setSettings((prev) => ({
            ...prev,
            members: prev.members.filter((m) => m.id !== id),
          }));
          addToast({ type: 'success', message: `${member.name} removed from team` });
          addToHistory({
            section: 'Team Settings',
            action: 'delete',
            changes: [{ field: 'member', oldValue: member.email, newValue: null }],
            userId: 'current-user',
            userName: 'Current User',
          });
        },
        onCancel: () => {},
      });
    },
    [settings.members, onConfirm, addToast, addToHistory]
  );

  // Change member role
  const changeMemberRole = useCallback(
    (memberId: string, newRole: string) => {
      setSettings((prev) => ({
        ...prev,
        members: prev.members.map((m) =>
          m.id === memberId ? { ...m, role: newRole } : m
        ),
      }));

      const member = settings.members.find((m) => m.id === memberId);
      addToast({
        type: 'success',
        message: `${member?.name}'s role updated to ${newRole}`,
      });
    },
    [settings.members, addToast]
  );

  // Suspend/Activate member
  const toggleMemberStatus = useCallback(
    (memberId: string) => {
      const member = settings.members.find((m) => m.id === memberId);
      if (!member) return;

      const newStatus = member.status === 'active' ? 'suspended' : 'active';

      onConfirm({
        title: newStatus === 'suspended' ? 'Suspend Member' : 'Activate Member',
        message: `Are you sure you want to ${newStatus === 'suspended' ? 'suspend' : 'activate'} ${member.name}?`,
        severity: newStatus === 'suspended' ? 'warning' : 'info',
        confirmText: newStatus === 'suspended' ? 'Suspend' : 'Activate',
        cancelText: 'Cancel',
        onConfirm: () => {
          setSettings((prev) => ({
            ...prev,
            members: prev.members.map((m) =>
              m.id === memberId ? { ...m, status: newStatus } : m
            ),
          }));
          addToast({
            type: 'success',
            message: `${member.name} ${newStatus === 'suspended' ? 'suspended' : 'activated'}`,
          });
        },
        onCancel: () => {},
      });
    },
    [settings.members, onConfirm, addToast]
  );

  return (
    <div style={{ maxWidth: '1000px' }}>
      <div style={{ marginBottom: '2rem' }}>
        <h2 style={{ fontSize: '1.5rem', marginBottom: '0.5rem' }}>Team Settings</h2>
        <p style={{ color: '#666', margin: 0 }}>
          Manage team members, roles, and permissions
        </p>
      </div>

      {/* Team Members */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <h3 style={{ fontSize: '1.125rem', margin: 0 }}>
            Team Members ({settings.members.length})
          </h3>
          <button
            onClick={() => setShowInviteModal(true)}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#1976d2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            + Invite Member
          </button>
        </div>

        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.875rem' }}>
            <thead>
              <tr style={{ borderBottom: '2px solid #e0e0e0' }}>
                <th style={{ padding: '0.75rem', textAlign: 'left', fontWeight: 600 }}>Name</th>
                <th style={{ padding: '0.75rem', textAlign: 'left', fontWeight: 600 }}>Email</th>
                <th style={{ padding: '0.75rem', textAlign: 'left', fontWeight: 600 }}>Role</th>
                <th style={{ padding: '0.75rem', textAlign: 'left', fontWeight: 600 }}>Status</th>
                <th style={{ padding: '0.75rem', textAlign: 'left', fontWeight: 600 }}>Last Active</th>
                <th style={{ padding: '0.75rem', textAlign: 'right', fontWeight: 600 }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {settings.members.map((member) => (
                <tr key={member.id} style={{ borderBottom: '1px solid #f0f0f0' }}>
                  <td style={{ padding: '0.75rem' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                      <div
                        style={{
                          width: '32px',
                          height: '32px',
                          borderRadius: '50%',
                          backgroundColor: '#e0e0e0',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          fontWeight: 600,
                          fontSize: '0.875rem',
                        }}
                      >
                        {member.name.charAt(0).toUpperCase()}
                      </div>
                      <span style={{ fontWeight: 500 }}>{member.name}</span>
                    </div>
                  </td>
                  <td style={{ padding: '0.75rem' }}>{member.email}</td>
                  <td style={{ padding: '0.75rem' }}>
                    <select
                      value={member.role}
                      onChange={(e) => changeMemberRole(member.id, e.target.value)}
                      style={{
                        padding: '0.25rem 0.5rem',
                        border: '1px solid #d0d0d0',
                        borderRadius: '4px',
                        fontSize: '0.875rem',
                      }}
                    >
                      {settings.roles.map((role) => (
                        <option key={role.id} value={role.name.toLowerCase()}>
                          {role.name}
                        </option>
                      ))}
                    </select>
                  </td>
                  <td style={{ padding: '0.75rem' }}>
                    <span
                      style={{
                        padding: '0.25rem 0.5rem',
                        backgroundColor: member.status === 'active' ? '#e8f5e9' : '#ffebee',
                        color: member.status === 'active' ? '#2e7d32' : '#c62828',
                        borderRadius: '4px',
                        fontSize: '0.75rem',
                        textTransform: 'capitalize',
                      }}
                    >
                      {member.status}
                    </span>
                  </td>
                  <td style={{ padding: '0.75rem' }}>
                    {member.lastActive?.toLocaleDateString() || 'Never'}
                  </td>
                  <td style={{ padding: '0.75rem', textAlign: 'right' }}>
                    <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
                      <button
                        onClick={() => toggleMemberStatus(member.id)}
                        style={{
                          padding: '0.25rem 0.75rem',
                          backgroundColor: '#fff',
                          color: member.status === 'active' ? '#ed6c02' : '#2e7d32',
                          border: `1px solid ${member.status === 'active' ? '#ed6c02' : '#2e7d32'}`,
                          borderRadius: '4px',
                          cursor: 'pointer',
                          fontSize: '0.75rem',
                        }}
                      >
                        {member.status === 'active' ? 'Suspend' : 'Activate'}
                      </button>
                      <button
                        onClick={() => removeMember(member.id)}
                        style={{
                          padding: '0.25rem 0.75rem',
                          backgroundColor: '#fff',
                          color: '#d32f2f',
                          border: '1px solid #d32f2f',
                          borderRadius: '4px',
                          cursor: 'pointer',
                          fontSize: '0.75rem',
                        }}
                      >
                        Remove
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Pending Invitations */}
      {settings.invitations.length > 0 && (
        <section
          style={{
            backgroundColor: '#fff',
            borderRadius: '8px',
            padding: '1.5rem',
            marginBottom: '1.5rem',
            border: '1px solid #e0e0e0',
          }}
        >
          <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>
            Pending Invitations ({settings.invitations.length})
          </h3>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            {settings.invitations.map((invitation) => (
              <div
                key={invitation.id}
                style={{
                  padding: '1rem',
                  border: '1px solid #e0e0e0',
                  borderRadius: '4px',
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <div>
                  <div style={{ fontWeight: 600, marginBottom: '0.25rem' }}>
                    {invitation.email}
                  </div>
                  <div style={{ fontSize: '0.875rem', color: '#666' }}>
                    Role: {invitation.role} • Invited by {invitation.invitedBy} •
                    Expires {invitation.expiresAt.toLocaleDateString()}
                  </div>
                </div>
                <button
                  onClick={() => revokeInvitation(invitation.id)}
                  style={{
                    padding: '0.25rem 0.75rem',
                    backgroundColor: '#fff',
                    color: '#d32f2f',
                    border: '1px solid #d32f2f',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '0.875rem',
                  }}
                >
                  Revoke
                </button>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Roles */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <h3 style={{ fontSize: '1.125rem', margin: 0 }}>Roles & Permissions</h3>
          <button
            onClick={() => setShowRoleModal(true)}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#1976d2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            + Create Role
          </button>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))', gap: '1rem' }}>
          {settings.roles.map((role) => (
            <div
              key={role.id}
              style={{
                padding: '1rem',
                border: '1px solid #e0e0e0',
                borderRadius: '4px',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '0.5rem' }}>
                <div>
                  <div style={{ fontWeight: 600, fontSize: '1.125rem', marginBottom: '0.25rem' }}>
                    {role.name}
                  </div>
                  {role.isSystem && (
                    <span
                      style={{
                        padding: '0.125rem 0.5rem',
                        backgroundColor: '#f5f5f5',
                        color: '#666',
                        fontSize: '0.75rem',
                        borderRadius: '4px',
                      }}
                    >
                      System Role
                    </span>
                  )}
                </div>
                {!role.isSystem && (
                  <button
                    style={{
                      padding: '0.25rem',
                      backgroundColor: 'transparent',
                      border: 'none',
                      cursor: 'pointer',
                      color: '#666',
                    }}
                  >
                    ⋮
                  </button>
                )}
              </div>
              <p style={{ margin: '0.5rem 0', fontSize: '0.875rem', color: '#666' }}>
                {role.description}
              </p>
              <div style={{ fontSize: '0.875rem', color: '#666' }}>
                {role.memberCount} member{role.memberCount !== 1 ? 's' : ''}
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Groups */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <h3 style={{ fontSize: '1.125rem', margin: 0 }}>Groups</h3>
          <button
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#1976d2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            + Create Group
          </button>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          {settings.groups.map((group) => (
            <div
              key={group.id}
              style={{
                padding: '1rem',
                border: '1px solid #e0e0e0',
                borderRadius: '4px',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start' }}>
                <div>
                  <div style={{ fontWeight: 600, fontSize: '1.125rem', marginBottom: '0.25rem' }}>
                    {group.name}
                  </div>
                  <p style={{ margin: '0.25rem 0', fontSize: '0.875rem', color: '#666' }}>
                    {group.description}
                  </p>
                  <div style={{ fontSize: '0.875rem', color: '#666' }}>
                    {group.memberIds.length} member{group.memberIds.length !== 1 ? 's' : ''}
                  </div>
                </div>
                <button
                  style={{
                    padding: '0.5rem 1rem',
                    backgroundColor: '#fff',
                    color: '#1976d2',
                    border: '1px solid #1976d2',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '0.875rem',
                  }}
                >
                  Manage
                </button>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Invite Modal */}
      {showInviteModal && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
          onClick={() => setShowInviteModal(false)}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              backgroundColor: '#fff',
              borderRadius: '8px',
              padding: '2rem',
              maxWidth: '500px',
              width: '90%',
            }}
          >
            <h2 style={{ marginTop: 0 }}>Invite Team Member</h2>

            <div style={{ marginBottom: '1rem' }}>
              <label
                htmlFor="inviteEmail"
                style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
              >
                Email Address
              </label>
              <input
                id="inviteEmail"
                type="email"
                value={inviteEmail}
                onChange={(e) => setInviteEmail(e.target.value)}
                placeholder="name@example.com"
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  border: '1px solid #d0d0d0',
                  borderRadius: '4px',
                }}
              />
            </div>

            <div style={{ marginBottom: '1.5rem' }}>
              <label
                htmlFor="inviteRole"
                style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
              >
                Role
              </label>
              <select
                id="inviteRole"
                value={inviteRole}
                onChange={(e) => setInviteRole(e.target.value)}
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  border: '1px solid #d0d0d0',
                  borderRadius: '4px',
                }}
              >
                {settings.roles.map((role) => (
                  <option key={role.id} value={role.name.toLowerCase()}>
                    {role.name}
                  </option>
                ))}
              </select>
            </div>

            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
              <button
                onClick={() => setShowInviteModal(false)}
                style={{
                  padding: '0.5rem 1.5rem',
                  backgroundColor: '#f5f5f5',
                  border: '1px solid #e0e0e0',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                Cancel
              </button>
              <button
                onClick={inviteMember}
                style={{
                  padding: '0.5rem 1.5rem',
                  backgroundColor: '#1976d2',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                Send Invitation
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default TeamSettings;
