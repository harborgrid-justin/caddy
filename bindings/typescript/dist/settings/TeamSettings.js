import React, { useState, useCallback } from 'react';
const AVAILABLE_PERMISSIONS = [
    { resource: 'users', actions: ['create', 'read', 'update', 'delete', 'manage'] },
    { resource: 'projects', actions: ['create', 'read', 'update', 'delete', 'manage'] },
    { resource: 'settings', actions: ['read', 'update', 'manage'] },
    { resource: 'billing', actions: ['read', 'update', 'manage'] },
    { resource: 'analytics', actions: ['read', 'manage'] },
    { resource: 'api', actions: ['create', 'read', 'delete', 'manage'] },
];
const TeamSettings = ({ onSave, onConfirm, addToast, addToHistory, }) => {
    const [settings, setSettings] = useState({
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
    const inviteMember = useCallback(async () => {
        if (!inviteEmail || !inviteEmail.includes('@')) {
            addToast({ type: 'error', message: 'Please enter a valid email address' });
            return;
        }
        try {
            const newInvitation = {
                id: `inv-${Date.now()}`,
                email: inviteEmail,
                role: inviteRole,
                invitedBy: 'current-user@example.com',
                invitedAt: new Date(),
                expiresAt: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000),
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
        }
        catch (error) {
            addToast({
                type: 'error',
                message: 'Failed to send invitation',
            });
        }
    }, [inviteEmail, inviteRole, addToast, addToHistory]);
    const revokeInvitation = useCallback((id) => {
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
            onCancel: () => { },
        });
    }, [onConfirm, addToast]);
    const removeMember = useCallback((id) => {
        const member = settings.members.find((m) => m.id === id);
        if (!member)
            return;
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
            onCancel: () => { },
        });
    }, [settings.members, onConfirm, addToast, addToHistory]);
    const changeMemberRole = useCallback((memberId, newRole) => {
        setSettings((prev) => ({
            ...prev,
            members: prev.members.map((m) => m.id === memberId ? { ...m, role: newRole } : m),
        }));
        const member = settings.members.find((m) => m.id === memberId);
        addToast({
            type: 'success',
            message: `${member?.name}'s role updated to ${newRole}`,
        });
    }, [settings.members, addToast]);
    const toggleMemberStatus = useCallback((memberId) => {
        const member = settings.members.find((m) => m.id === memberId);
        if (!member)
            return;
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
                    members: prev.members.map((m) => m.id === memberId ? { ...m, status: newStatus } : m),
                }));
                addToast({
                    type: 'success',
                    message: `${member.name} ${newStatus === 'suspended' ? 'suspended' : 'activated'}`,
                });
            },
            onCancel: () => { },
        });
    }, [settings.members, onConfirm, addToast]);
    return (React.createElement("div", { style: { maxWidth: '1000px' } },
        React.createElement("div", { style: { marginBottom: '2rem' } },
            React.createElement("h2", { style: { fontSize: '1.5rem', marginBottom: '0.5rem' } }, "Team Settings"),
            React.createElement("p", { style: { color: '#666', margin: 0 } }, "Manage team members, roles, and permissions")),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' } },
                React.createElement("h3", { style: { fontSize: '1.125rem', margin: 0 } },
                    "Team Members (",
                    settings.members.length,
                    ")"),
                React.createElement("button", { onClick: () => setShowInviteModal(true), style: {
                        padding: '0.5rem 1rem',
                        backgroundColor: '#1976d2',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                    } }, "+ Invite Member")),
            React.createElement("div", { style: { overflowX: 'auto' } },
                React.createElement("table", { style: { width: '100%', borderCollapse: 'collapse', fontSize: '0.875rem' } },
                    React.createElement("thead", null,
                        React.createElement("tr", { style: { borderBottom: '2px solid #e0e0e0' } },
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'left', fontWeight: 600 } }, "Name"),
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'left', fontWeight: 600 } }, "Email"),
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'left', fontWeight: 600 } }, "Role"),
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'left', fontWeight: 600 } }, "Status"),
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'left', fontWeight: 600 } }, "Last Active"),
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'right', fontWeight: 600 } }, "Actions"))),
                    React.createElement("tbody", null, settings.members.map((member) => (React.createElement("tr", { key: member.id, style: { borderBottom: '1px solid #f0f0f0' } },
                        React.createElement("td", { style: { padding: '0.75rem' } },
                            React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '0.5rem' } },
                                React.createElement("div", { style: {
                                        width: '32px',
                                        height: '32px',
                                        borderRadius: '50%',
                                        backgroundColor: '#e0e0e0',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontWeight: 600,
                                        fontSize: '0.875rem',
                                    } }, member.name.charAt(0).toUpperCase()),
                                React.createElement("span", { style: { fontWeight: 500 } }, member.name))),
                        React.createElement("td", { style: { padding: '0.75rem' } }, member.email),
                        React.createElement("td", { style: { padding: '0.75rem' } },
                            React.createElement("select", { value: member.role, onChange: (e) => changeMemberRole(member.id, e.target.value), style: {
                                    padding: '0.25rem 0.5rem',
                                    border: '1px solid #d0d0d0',
                                    borderRadius: '4px',
                                    fontSize: '0.875rem',
                                } }, settings.roles.map((role) => (React.createElement("option", { key: role.id, value: role.name.toLowerCase() }, role.name))))),
                        React.createElement("td", { style: { padding: '0.75rem' } },
                            React.createElement("span", { style: {
                                    padding: '0.25rem 0.5rem',
                                    backgroundColor: member.status === 'active' ? '#e8f5e9' : '#ffebee',
                                    color: member.status === 'active' ? '#2e7d32' : '#c62828',
                                    borderRadius: '4px',
                                    fontSize: '0.75rem',
                                    textTransform: 'capitalize',
                                } }, member.status)),
                        React.createElement("td", { style: { padding: '0.75rem' } }, member.lastActive?.toLocaleDateString() || 'Never'),
                        React.createElement("td", { style: { padding: '0.75rem', textAlign: 'right' } },
                            React.createElement("div", { style: { display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' } },
                                React.createElement("button", { onClick: () => toggleMemberStatus(member.id), style: {
                                        padding: '0.25rem 0.75rem',
                                        backgroundColor: '#fff',
                                        color: member.status === 'active' ? '#ed6c02' : '#2e7d32',
                                        border: `1px solid ${member.status === 'active' ? '#ed6c02' : '#2e7d32'}`,
                                        borderRadius: '4px',
                                        cursor: 'pointer',
                                        fontSize: '0.75rem',
                                    } }, member.status === 'active' ? 'Suspend' : 'Activate'),
                                React.createElement("button", { onClick: () => removeMember(member.id), style: {
                                        padding: '0.25rem 0.75rem',
                                        backgroundColor: '#fff',
                                        color: '#d32f2f',
                                        border: '1px solid #d32f2f',
                                        borderRadius: '4px',
                                        cursor: 'pointer',
                                        fontSize: '0.75rem',
                                    } }, "Remove")))))))))),
        settings.invitations.length > 0 && (React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } },
                "Pending Invitations (",
                settings.invitations.length,
                ")"),
            React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '1rem' } }, settings.invitations.map((invitation) => (React.createElement("div", { key: invitation.id, style: {
                    padding: '1rem',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                } },
                React.createElement("div", null,
                    React.createElement("div", { style: { fontWeight: 600, marginBottom: '0.25rem' } }, invitation.email),
                    React.createElement("div", { style: { fontSize: '0.875rem', color: '#666' } },
                        "Role: ",
                        invitation.role,
                        " \u2022 Invited by ",
                        invitation.invitedBy,
                        " \u2022 Expires ",
                        invitation.expiresAt.toLocaleDateString())),
                React.createElement("button", { onClick: () => revokeInvitation(invitation.id), style: {
                        padding: '0.25rem 0.75rem',
                        backgroundColor: '#fff',
                        color: '#d32f2f',
                        border: '1px solid #d32f2f',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '0.875rem',
                    } }, "Revoke"))))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' } },
                React.createElement("h3", { style: { fontSize: '1.125rem', margin: 0 } }, "Roles & Permissions"),
                React.createElement("button", { onClick: () => setShowRoleModal(true), style: {
                        padding: '0.5rem 1rem',
                        backgroundColor: '#1976d2',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                    } }, "+ Create Role")),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))', gap: '1rem' } }, settings.roles.map((role) => (React.createElement("div", { key: role.id, style: {
                    padding: '1rem',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                } },
                React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '0.5rem' } },
                    React.createElement("div", null,
                        React.createElement("div", { style: { fontWeight: 600, fontSize: '1.125rem', marginBottom: '0.25rem' } }, role.name),
                        role.isSystem && (React.createElement("span", { style: {
                                padding: '0.125rem 0.5rem',
                                backgroundColor: '#f5f5f5',
                                color: '#666',
                                fontSize: '0.75rem',
                                borderRadius: '4px',
                            } }, "System Role"))),
                    !role.isSystem && (React.createElement("button", { style: {
                            padding: '0.25rem',
                            backgroundColor: 'transparent',
                            border: 'none',
                            cursor: 'pointer',
                            color: '#666',
                        } }, "\u22EE"))),
                React.createElement("p", { style: { margin: '0.5rem 0', fontSize: '0.875rem', color: '#666' } }, role.description),
                React.createElement("div", { style: { fontSize: '0.875rem', color: '#666' } },
                    role.memberCount,
                    " member",
                    role.memberCount !== 1 ? 's' : '')))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' } },
                React.createElement("h3", { style: { fontSize: '1.125rem', margin: 0 } }, "Groups"),
                React.createElement("button", { style: {
                        padding: '0.5rem 1rem',
                        backgroundColor: '#1976d2',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                    } }, "+ Create Group")),
            React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '1rem' } }, settings.groups.map((group) => (React.createElement("div", { key: group.id, style: {
                    padding: '1rem',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                } },
                React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'start' } },
                    React.createElement("div", null,
                        React.createElement("div", { style: { fontWeight: 600, fontSize: '1.125rem', marginBottom: '0.25rem' } }, group.name),
                        React.createElement("p", { style: { margin: '0.25rem 0', fontSize: '0.875rem', color: '#666' } }, group.description),
                        React.createElement("div", { style: { fontSize: '0.875rem', color: '#666' } },
                            group.memberIds.length,
                            " member",
                            group.memberIds.length !== 1 ? 's' : '')),
                    React.createElement("button", { style: {
                            padding: '0.5rem 1rem',
                            backgroundColor: '#fff',
                            color: '#1976d2',
                            border: '1px solid #1976d2',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '0.875rem',
                        } }, "Manage"))))))),
        showInviteModal && (React.createElement("div", { style: {
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
            }, onClick: () => setShowInviteModal(false) },
            React.createElement("div", { onClick: (e) => e.stopPropagation(), style: {
                    backgroundColor: '#fff',
                    borderRadius: '8px',
                    padding: '2rem',
                    maxWidth: '500px',
                    width: '90%',
                } },
                React.createElement("h2", { style: { marginTop: 0 } }, "Invite Team Member"),
                React.createElement("div", { style: { marginBottom: '1rem' } },
                    React.createElement("label", { htmlFor: "inviteEmail", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Email Address"),
                    React.createElement("input", { id: "inviteEmail", type: "email", value: inviteEmail, onChange: (e) => setInviteEmail(e.target.value), placeholder: "name@example.com", style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } })),
                React.createElement("div", { style: { marginBottom: '1.5rem' } },
                    React.createElement("label", { htmlFor: "inviteRole", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Role"),
                    React.createElement("select", { id: "inviteRole", value: inviteRole, onChange: (e) => setInviteRole(e.target.value), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } }, settings.roles.map((role) => (React.createElement("option", { key: role.id, value: role.name.toLowerCase() }, role.name))))),
                React.createElement("div", { style: { display: 'flex', gap: '1rem', justifyContent: 'flex-end' } },
                    React.createElement("button", { onClick: () => setShowInviteModal(false), style: {
                            padding: '0.5rem 1.5rem',
                            backgroundColor: '#f5f5f5',
                            border: '1px solid #e0e0e0',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, "Cancel"),
                    React.createElement("button", { onClick: inviteMember, style: {
                            padding: '0.5rem 1.5rem',
                            backgroundColor: '#1976d2',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, "Send Invitation")))))));
};
export default TeamSettings;
//# sourceMappingURL=TeamSettings.js.map