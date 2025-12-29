import React, { useState, useCallback, useEffect } from 'react';
import { useUser, useRoles, useTeams, useUserActivity, useUserSessions, usePermissions, } from './UserHooks';
export const UserProfile = ({ userId, onUpdate, onClose, editable = true, showSessions = true, showActivity = true, className = '', }) => {
    const [activeTab, setActiveTab] = useState('profile');
    const [isEditing, setIsEditing] = useState(false);
    const [formData, setFormData] = useState({});
    const [saving, setSaving] = useState(false);
    const { user, loading, error, updateUser } = useUser(userId);
    const { roles } = useRoles();
    const { teams } = useTeams();
    const { activity } = useUserActivity(userId, 100);
    const { sessions, terminateSession } = useUserSessions(userId);
    const { permissions } = usePermissions(userId);
    useEffect(() => {
        if (user) {
            setFormData({
                firstName: user.firstName,
                lastName: user.lastName,
                displayName: user.displayName,
                email: user.email,
                phoneNumber: user.phoneNumber,
                timezone: user.timezone,
                locale: user.locale,
                status: user.status,
            });
        }
    }, [user]);
    const handleSubmit = useCallback(async (e) => {
        e.preventDefault();
        if (!user || !editable)
            return;
        try {
            setSaving(true);
            const updated = await updateUser(formData);
            setIsEditing(false);
            onUpdate?.(updated);
        }
        catch (err) {
            console.error('Failed to update user:', err);
        }
        finally {
            setSaving(false);
        }
    }, [user, editable, formData, updateUser, onUpdate]);
    const handleExportData = useCallback(async () => {
        try {
            const response = await fetch(`/api/users/${userId}/export-data`, {
                method: 'POST',
            });
            const blob = await response.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `user-data-${userId}-${new Date().toISOString()}.json`;
            a.click();
            window.URL.revokeObjectURL(url);
        }
        catch (err) {
            console.error('Failed to export data:', err);
        }
    }, [userId]);
    const handleDeleteAccount = useCallback(async () => {
        if (window.confirm('Are you sure you want to delete this account? This action cannot be undone.')) {
            try {
                await fetch(`/api/users/${userId}`, { method: 'DELETE' });
                onClose?.();
            }
            catch (err) {
                console.error('Failed to delete account:', err);
            }
        }
    }, [userId, onClose]);
    if (loading) {
        return (React.createElement("div", { className: "flex justify-center items-center h-64" },
            React.createElement("div", { className: "animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600" })));
    }
    if (error || !user) {
        return (React.createElement("div", { className: "rounded-md bg-red-50 p-4" },
            React.createElement("div", { className: "flex" },
                React.createElement("div", { className: "ml-3" },
                    React.createElement("h3", { className: "text-sm font-medium text-red-800" }, "Error loading user profile"),
                    React.createElement("div", { className: "mt-2 text-sm text-red-700" }, error?.message || 'User not found')))));
    }
    const tabs = [
        { id: 'profile', label: 'Profile' },
        { id: 'security', label: 'Security' },
        { id: 'roles', label: 'Roles', count: user.roles.length },
        { id: 'teams', label: 'Teams', count: user.teams.length },
        { id: 'activity', label: 'Activity', count: activity.length },
        { id: 'sessions', label: 'Sessions', count: sessions.length },
        { id: 'gdpr', label: 'Privacy & Data' },
    ];
    return (React.createElement("div", { className: `bg-white shadow overflow-hidden sm:rounded-lg ${className}` },
        React.createElement("div", { className: "px-4 py-5 sm:px-6 flex items-center justify-between" },
            React.createElement("div", { className: "flex items-center space-x-4" },
                user.avatar ? (React.createElement("img", { className: "h-16 w-16 rounded-full", src: user.avatar, alt: user.displayName })) : (React.createElement("div", { className: "h-16 w-16 rounded-full bg-indigo-100 flex items-center justify-center" },
                    React.createElement("span", { className: "text-indigo-700 font-medium text-xl" },
                        user.firstName[0],
                        user.lastName[0]))),
                React.createElement("div", null,
                    React.createElement("h3", { className: "text-lg leading-6 font-medium text-gray-900" }, user.displayName),
                    React.createElement("p", { className: "mt-1 max-w-2xl text-sm text-gray-500" }, user.email),
                    React.createElement("div", { className: "mt-1 flex items-center space-x-2" },
                        React.createElement("span", { className: `inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${user.status === 'active'
                                ? 'bg-green-100 text-green-800'
                                : user.status === 'inactive'
                                    ? 'bg-gray-100 text-gray-800'
                                    : 'bg-red-100 text-red-800'}` }, user.status),
                        user.metadata.mfaEnabled && (React.createElement("span", { className: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800" }, "MFA Enabled")),
                        user.metadata.ssoEnabled && (React.createElement("span", { className: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800" },
                            "SSO: ",
                            user.metadata.ssoProvider))))),
            React.createElement("div", { className: "flex space-x-2" },
                editable && !isEditing && (React.createElement("button", { onClick: () => setIsEditing(true), className: "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700" }, "Edit")),
                onClose && (React.createElement("button", { onClick: onClose, className: "inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Close")))),
        React.createElement("div", { className: "border-b border-gray-200" },
            React.createElement("nav", { className: "-mb-px flex space-x-8 px-6", "aria-label": "Tabs" }, tabs.map((tab) => (React.createElement("button", { key: tab.id, onClick: () => setActiveTab(tab.id), className: `${activeTab === tab.id
                    ? 'border-indigo-500 text-indigo-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm` },
                tab.label,
                tab.count !== undefined && (React.createElement("span", { className: `${activeTab === tab.id
                        ? 'bg-indigo-100 text-indigo-600'
                        : 'bg-gray-100 text-gray-900'} ml-2 py-0.5 px-2.5 rounded-full text-xs font-medium` }, tab.count))))))),
        React.createElement("div", { className: "px-4 py-5 sm:p-6" },
            activeTab === 'profile' && (React.createElement("form", { onSubmit: handleSubmit },
                React.createElement("div", { className: "grid grid-cols-1 gap-6 sm:grid-cols-2" },
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "First Name"),
                        React.createElement("input", { type: "text", disabled: !isEditing, value: formData.firstName || '', onChange: (e) => setFormData({ ...formData, firstName: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100" })),
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Last Name"),
                        React.createElement("input", { type: "text", disabled: !isEditing, value: formData.lastName || '', onChange: (e) => setFormData({ ...formData, lastName: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100" })),
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Email"),
                        React.createElement("input", { type: "email", disabled: !isEditing, value: formData.email || '', onChange: (e) => setFormData({ ...formData, email: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100" })),
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Phone Number"),
                        React.createElement("input", { type: "tel", disabled: !isEditing, value: formData.phoneNumber || '', onChange: (e) => setFormData({ ...formData, phoneNumber: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100" })),
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Timezone"),
                        React.createElement("select", { disabled: !isEditing, value: formData.timezone || '', onChange: (e) => setFormData({ ...formData, timezone: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100" },
                            React.createElement("option", { value: "UTC" }, "UTC"),
                            React.createElement("option", { value: "America/New_York" }, "America/New York"),
                            React.createElement("option", { value: "America/Los_Angeles" }, "America/Los Angeles"),
                            React.createElement("option", { value: "Europe/London" }, "Europe/London"),
                            React.createElement("option", { value: "Asia/Tokyo" }, "Asia/Tokyo"))),
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Status"),
                        React.createElement("select", { disabled: !isEditing, value: formData.status || '', onChange: (e) => setFormData({ ...formData, status: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100" },
                            React.createElement("option", { value: "active" }, "Active"),
                            React.createElement("option", { value: "inactive" }, "Inactive"),
                            React.createElement("option", { value: "pending" }, "Pending"),
                            React.createElement("option", { value: "suspended" }, "Suspended"),
                            React.createElement("option", { value: "locked" }, "Locked")))),
                React.createElement("div", { className: "mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2" },
                    React.createElement("div", { className: "bg-gray-50 p-4 rounded-md" },
                        React.createElement("h4", { className: "text-sm font-medium text-gray-700 mb-2" }, "Metadata"),
                        React.createElement("dl", { className: "space-y-2" },
                            React.createElement("div", { className: "flex justify-between text-sm" },
                                React.createElement("dt", { className: "text-gray-500" }, "User ID:"),
                                React.createElement("dd", { className: "text-gray-900 font-mono" }, user.id)),
                            React.createElement("div", { className: "flex justify-between text-sm" },
                                React.createElement("dt", { className: "text-gray-500" }, "Created:"),
                                React.createElement("dd", { className: "text-gray-900" }, new Date(user.createdAt).toLocaleDateString())),
                            React.createElement("div", { className: "flex justify-between text-sm" },
                                React.createElement("dt", { className: "text-gray-500" }, "Last Login:"),
                                React.createElement("dd", { className: "text-gray-900" }, user.lastLoginAt
                                    ? new Date(user.lastLoginAt).toLocaleString()
                                    : 'Never')),
                            React.createElement("div", { className: "flex justify-between text-sm" },
                                React.createElement("dt", { className: "text-gray-500" }, "Source:"),
                                React.createElement("dd", { className: "text-gray-900" }, user.metadata.source)))),
                    React.createElement("div", { className: "bg-gray-50 p-4 rounded-md" },
                        React.createElement("h4", { className: "text-sm font-medium text-gray-700 mb-2" }, "Organization Info"),
                        React.createElement("dl", { className: "space-y-2" },
                            React.createElement("div", { className: "flex justify-between text-sm" },
                                React.createElement("dt", { className: "text-gray-500" }, "Department:"),
                                React.createElement("dd", { className: "text-gray-900" }, user.metadata.department || '-')),
                            React.createElement("div", { className: "flex justify-between text-sm" },
                                React.createElement("dt", { className: "text-gray-500" }, "Job Title:"),
                                React.createElement("dd", { className: "text-gray-900" }, user.metadata.jobTitle || '-')),
                            React.createElement("div", { className: "flex justify-between text-sm" },
                                React.createElement("dt", { className: "text-gray-500" }, "Manager:"),
                                React.createElement("dd", { className: "text-gray-900" }, user.metadata.manager || '-')),
                            React.createElement("div", { className: "flex justify-between text-sm" },
                                React.createElement("dt", { className: "text-gray-500" }, "Employee ID:"),
                                React.createElement("dd", { className: "text-gray-900" }, user.metadata.employeeId || '-'))))),
                isEditing && (React.createElement("div", { className: "mt-6 flex justify-end space-x-3" },
                    React.createElement("button", { type: "button", onClick: () => setIsEditing(false), className: "inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Cancel"),
                    React.createElement("button", { type: "submit", disabled: saving, className: "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50" }, saving ? 'Saving...' : 'Save Changes'))))),
            activeTab === 'security' && (React.createElement("div", { className: "space-y-6" },
                React.createElement("div", null,
                    React.createElement("h4", { className: "text-lg font-medium text-gray-900 mb-4" }, "Security Settings"),
                    React.createElement("dl", { className: "space-y-4" },
                        React.createElement("div", { className: "flex justify-between items-center" },
                            React.createElement("div", null,
                                React.createElement("dt", { className: "text-sm font-medium text-gray-700" }, "Multi-Factor Authentication"),
                                React.createElement("dd", { className: "text-sm text-gray-500" }, "Add an extra layer of security")),
                            React.createElement("span", { className: `px-2 py-1 rounded text-xs font-semibold ${user.metadata.mfaEnabled
                                    ? 'bg-green-100 text-green-800'
                                    : 'bg-gray-100 text-gray-800'}` }, user.metadata.mfaEnabled ? 'Enabled' : 'Disabled')),
                        React.createElement("div", { className: "flex justify-between items-center" },
                            React.createElement("div", null,
                                React.createElement("dt", { className: "text-sm font-medium text-gray-700" }, "Password Last Changed"),
                                React.createElement("dd", { className: "text-sm text-gray-500" }, user.metadata.passwordLastChanged
                                    ? new Date(user.metadata.passwordLastChanged).toLocaleDateString()
                                    : 'Never'))),
                        React.createElement("div", { className: "flex justify-between items-center" },
                            React.createElement("div", null,
                                React.createElement("dt", { className: "text-sm font-medium text-gray-700" }, "Failed Login Attempts"),
                                React.createElement("dd", { className: "text-sm text-gray-500" },
                                    user.metadata.failedLoginAttempts,
                                    " attempts"))),
                        React.createElement("div", { className: "flex justify-between items-center" },
                            React.createElement("div", null,
                                React.createElement("dt", { className: "text-sm font-medium text-gray-700" }, "Session Timeout"),
                                React.createElement("dd", { className: "text-sm text-gray-500" },
                                    user.securitySettings.sessionTimeout,
                                    " minutes"))))))),
            activeTab === 'roles' && (React.createElement("div", null,
                React.createElement("h4", { className: "text-lg font-medium text-gray-900 mb-4" }, "Assigned Roles"),
                React.createElement("div", { className: "space-y-3" }, user.roles.map((roleId) => {
                    const role = roles.find((r) => r.id === roleId);
                    return (React.createElement("div", { key: roleId, className: "border border-gray-200 rounded-md p-4 flex justify-between items-start" },
                        React.createElement("div", null,
                            React.createElement("h5", { className: "text-sm font-medium text-gray-900" }, role?.displayName || roleId),
                            React.createElement("p", { className: "text-sm text-gray-500" }, role?.description),
                            React.createElement("div", { className: "mt-2 flex flex-wrap gap-1" },
                                role?.permissions.slice(0, 5).map((perm, idx) => (React.createElement("span", { key: idx, className: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800" },
                                    perm.resource,
                                    ":",
                                    perm.action))),
                                role && role.permissions.length > 5 && (React.createElement("span", { className: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800" },
                                    "+",
                                    role.permissions.length - 5,
                                    " more"))))));
                })))),
            activeTab === 'teams' && (React.createElement("div", null,
                React.createElement("h4", { className: "text-lg font-medium text-gray-900 mb-4" }, "Team Memberships"),
                React.createElement("div", { className: "space-y-3" }, user.teams.map((teamId) => {
                    const team = teams.find((t) => t.id === teamId);
                    return (React.createElement("div", { key: teamId, className: "border border-gray-200 rounded-md p-4 flex justify-between items-start" },
                        React.createElement("div", null,
                            React.createElement("h5", { className: "text-sm font-medium text-gray-900" }, team?.displayName || teamId),
                            React.createElement("p", { className: "text-sm text-gray-500" }, team?.description),
                            React.createElement("div", { className: "mt-2 text-xs text-gray-500" },
                                team?.members.length,
                                " members"))));
                })))),
            activeTab === 'activity' && (React.createElement("div", null,
                React.createElement("h4", { className: "text-lg font-medium text-gray-900 mb-4" }, "Recent Activity"),
                React.createElement("div", { className: "flow-root" },
                    React.createElement("ul", { className: "-mb-8" }, activity.slice(0, 20).map((log, idx) => (React.createElement("li", { key: log.id },
                        React.createElement("div", { className: "relative pb-8" },
                            idx !== activity.length - 1 && (React.createElement("span", { className: "absolute top-4 left-4 -ml-px h-full w-0.5 bg-gray-200", "aria-hidden": "true" })),
                            React.createElement("div", { className: "relative flex space-x-3" },
                                React.createElement("div", null,
                                    React.createElement("span", { className: `h-8 w-8 rounded-full flex items-center justify-center ring-8 ring-white ${log.severity === 'error' || log.severity === 'critical'
                                            ? 'bg-red-500'
                                            : log.severity === 'warning'
                                                ? 'bg-yellow-500'
                                                : 'bg-green-500'}` },
                                        React.createElement("svg", { className: "h-5 w-5 text-white", fill: "currentColor", viewBox: "0 0 20 20" },
                                            React.createElement("path", { fillRule: "evenodd", d: "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z", clipRule: "evenodd" })))),
                                React.createElement("div", { className: "min-w-0 flex-1 pt-1.5 flex justify-between space-x-4" },
                                    React.createElement("div", null,
                                        React.createElement("p", { className: "text-sm text-gray-500" }, log.details.description),
                                        React.createElement("p", { className: "text-xs text-gray-400 mt-1" },
                                            log.resource,
                                            " \u2022 ",
                                            log.action)),
                                    React.createElement("div", { className: "text-right text-sm whitespace-nowrap text-gray-500" }, new Date(log.timestamp).toLocaleString()))))))))))),
            activeTab === 'sessions' && (React.createElement("div", null,
                React.createElement("h4", { className: "text-lg font-medium text-gray-900 mb-4" }, "Active Sessions"),
                React.createElement("div", { className: "space-y-3" }, sessions.map((session) => (React.createElement("div", { key: session.id, className: "border border-gray-200 rounded-md p-4 flex justify-between items-start" },
                    React.createElement("div", { className: "flex-1" },
                        React.createElement("div", { className: "flex items-center space-x-2" },
                            React.createElement("h5", { className: "text-sm font-medium text-gray-900" }, session.deviceName),
                            React.createElement("span", { className: `px-2 py-0.5 rounded text-xs font-semibold ${session.status === 'active'
                                    ? 'bg-green-100 text-green-800'
                                    : 'bg-gray-100 text-gray-800'}` }, session.status)),
                        React.createElement("p", { className: "text-sm text-gray-500 mt-1" },
                            session.browser,
                            " on ",
                            session.os),
                        React.createElement("p", { className: "text-xs text-gray-400 mt-1" },
                            "IP: ",
                            session.ipAddress,
                            " \u2022 Last active:",
                            ' ',
                            new Date(session.lastActivityAt).toLocaleString())),
                    session.status === 'active' && (React.createElement("button", { onClick: () => terminateSession(session.id), className: "ml-4 text-sm text-red-600 hover:text-red-900" }, "Terminate")))))))),
            activeTab === 'gdpr' && (React.createElement("div", { className: "space-y-6" },
                React.createElement("div", null,
                    React.createElement("h4", { className: "text-lg font-medium text-gray-900 mb-4" }, "Privacy & Data"),
                    React.createElement("dl", { className: "space-y-4" },
                        React.createElement("div", null,
                            React.createElement("dt", { className: "text-sm font-medium text-gray-700" }, "Data Processing"),
                            React.createElement("dd", { className: "text-sm text-gray-500 mt-1" },
                                "Consent given on",
                                ' ',
                                new Date(user.gdprConsent.consentDate).toLocaleDateString())),
                        React.createElement("div", null,
                            React.createElement("dt", { className: "text-sm font-medium text-gray-700" }, "Marketing Communications"),
                            React.createElement("dd", { className: "text-sm text-gray-500 mt-1" }, user.gdprConsent.marketing ? 'Opted in' : 'Opted out')))),
                React.createElement("div", { className: "border-t border-gray-200 pt-6" },
                    React.createElement("h4", { className: "text-sm font-medium text-gray-900 mb-4" }, "Data Management"),
                    React.createElement("div", { className: "space-y-3" },
                        React.createElement("button", { onClick: handleExportData, className: "w-full inline-flex justify-center items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Export My Data"),
                        React.createElement("button", { onClick: handleDeleteAccount, className: "w-full inline-flex justify-center items-center px-4 py-2 border border-red-300 shadow-sm text-sm font-medium rounded-md text-red-700 bg-white hover:bg-red-50" }, "Delete Account"))))))));
};
export default UserProfile;
//# sourceMappingURL=UserProfile.js.map