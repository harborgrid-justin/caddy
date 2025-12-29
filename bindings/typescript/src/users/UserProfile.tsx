/**
 * CADDY v0.4.0 - User Profile Component
 *
 * Comprehensive user profile with:
 * - Personal information editing
 * - Security settings management
 * - Role and team assignments
 * - Activity history
 * - Session management
 * - GDPR data export/deletion
 * - Audit trail
 */

import React, { useState, useCallback, useEffect } from 'react';
import {
  User,
  UpdateUserRequest,
  UserStatus,
  UserPreferences,
  UserSecuritySettings,
} from './types';
import {
  useUser,
  useRoles,
  useTeams,
  useUserActivity,
  useUserSessions,
  usePermissions,
} from './UserHooks';

interface UserProfileProps {
  userId: string;
  onUpdate?: (user: User) => void;
  onClose?: () => void;
  editable?: boolean;
  showSessions?: boolean;
  showActivity?: boolean;
  className?: string;
}

type TabType = 'profile' | 'security' | 'roles' | 'teams' | 'activity' | 'sessions' | 'gdpr';

export const UserProfile: React.FC<UserProfileProps> = ({
  userId,
  onUpdate,
  onClose,
  editable = true,
  showSessions = true,
  showActivity = true,
  className = '',
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('profile');
  const [isEditing, setIsEditing] = useState(false);
  const [formData, setFormData] = useState<Partial<UpdateUserRequest>>({});
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

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!user || !editable) return;

      try {
        setSaving(true);
        const updated = await updateUser(formData);
        setIsEditing(false);
        onUpdate?.(updated);
      } catch (err) {
        console.error('Failed to update user:', err);
      } finally {
        setSaving(false);
      }
    },
    [user, editable, formData, updateUser, onUpdate]
  );

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
    } catch (err) {
      console.error('Failed to export data:', err);
    }
  }, [userId]);

  const handleDeleteAccount = useCallback(async () => {
    if (
      window.confirm(
        'Are you sure you want to delete this account? This action cannot be undone.'
      )
    ) {
      try {
        await fetch(`/api/users/${userId}`, { method: 'DELETE' });
        onClose?.();
      } catch (err) {
        console.error('Failed to delete account:', err);
      }
    }
  }, [userId, onClose]);

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  if (error || !user) {
    return (
      <div className="rounded-md bg-red-50 p-4">
        <div className="flex">
          <div className="ml-3">
            <h3 className="text-sm font-medium text-red-800">Error loading user profile</h3>
            <div className="mt-2 text-sm text-red-700">
              {error?.message || 'User not found'}
            </div>
          </div>
        </div>
      </div>
    );
  }

  const tabs: Array<{ id: TabType; label: string; count?: number }> = [
    { id: 'profile', label: 'Profile' },
    { id: 'security', label: 'Security' },
    { id: 'roles', label: 'Roles', count: user.roles.length },
    { id: 'teams', label: 'Teams', count: user.teams.length },
    { id: 'activity', label: 'Activity', count: activity.length },
    { id: 'sessions', label: 'Sessions', count: sessions.length },
    { id: 'gdpr', label: 'Privacy & Data' },
  ];

  return (
    <div className={`bg-white shadow overflow-hidden sm:rounded-lg ${className}`}>
      <div className="px-4 py-5 sm:px-6 flex items-center justify-between">
        <div className="flex items-center space-x-4">
          {user.avatar ? (
            <img
              className="h-16 w-16 rounded-full"
              src={user.avatar}
              alt={user.displayName}
            />
          ) : (
            <div className="h-16 w-16 rounded-full bg-indigo-100 flex items-center justify-center">
              <span className="text-indigo-700 font-medium text-xl">
                {user.firstName[0]}
                {user.lastName[0]}
              </span>
            </div>
          )}
          <div>
            <h3 className="text-lg leading-6 font-medium text-gray-900">
              {user.displayName}
            </h3>
            <p className="mt-1 max-w-2xl text-sm text-gray-500">{user.email}</p>
            <div className="mt-1 flex items-center space-x-2">
              <span
                className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${
                  user.status === 'active'
                    ? 'bg-green-100 text-green-800'
                    : user.status === 'inactive'
                    ? 'bg-gray-100 text-gray-800'
                    : 'bg-red-100 text-red-800'
                }`}
              >
                {user.status}
              </span>
              {user.metadata.mfaEnabled && (
                <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                  MFA Enabled
                </span>
              )}
              {user.metadata.ssoEnabled && (
                <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800">
                  SSO: {user.metadata.ssoProvider}
                </span>
              )}
            </div>
          </div>
        </div>
        <div className="flex space-x-2">
          {editable && !isEditing && (
            <button
              onClick={() => setIsEditing(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
            >
              Edit
            </button>
          )}
          {onClose && (
            <button
              onClick={onClose}
              className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
            >
              Close
            </button>
          )}
        </div>
      </div>

      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8 px-6" aria-label="Tabs">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`${
                activeTab === tab.id
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              } whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
            >
              {tab.label}
              {tab.count !== undefined && (
                <span
                  className={`${
                    activeTab === tab.id
                      ? 'bg-indigo-100 text-indigo-600'
                      : 'bg-gray-100 text-gray-900'
                  } ml-2 py-0.5 px-2.5 rounded-full text-xs font-medium`}
                >
                  {tab.count}
                </span>
              )}
            </button>
          ))}
        </nav>
      </div>

      <div className="px-4 py-5 sm:p-6">
        {activeTab === 'profile' && (
          <form onSubmit={handleSubmit}>
            <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  First Name
                </label>
                <input
                  type="text"
                  disabled={!isEditing}
                  value={formData.firstName || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, firstName: e.target.value })
                  }
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">Last Name</label>
                <input
                  type="text"
                  disabled={!isEditing}
                  value={formData.lastName || ''}
                  onChange={(e) => setFormData({ ...formData, lastName: e.target.value })}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">Email</label>
                <input
                  type="email"
                  disabled={!isEditing}
                  value={formData.email || ''}
                  onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Phone Number
                </label>
                <input
                  type="tel"
                  disabled={!isEditing}
                  value={formData.phoneNumber || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, phoneNumber: e.target.value })
                  }
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">Timezone</label>
                <select
                  disabled={!isEditing}
                  value={formData.timezone || ''}
                  onChange={(e) => setFormData({ ...formData, timezone: e.target.value })}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100"
                >
                  <option value="UTC">UTC</option>
                  <option value="America/New_York">America/New York</option>
                  <option value="America/Los_Angeles">America/Los Angeles</option>
                  <option value="Europe/London">Europe/London</option>
                  <option value="Asia/Tokyo">Asia/Tokyo</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">Status</label>
                <select
                  disabled={!isEditing}
                  value={formData.status || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, status: e.target.value as UserStatus })
                  }
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm disabled:bg-gray-100"
                >
                  <option value="active">Active</option>
                  <option value="inactive">Inactive</option>
                  <option value="pending">Pending</option>
                  <option value="suspended">Suspended</option>
                  <option value="locked">Locked</option>
                </select>
              </div>
            </div>

            <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2">
              <div className="bg-gray-50 p-4 rounded-md">
                <h4 className="text-sm font-medium text-gray-700 mb-2">Metadata</h4>
                <dl className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <dt className="text-gray-500">User ID:</dt>
                    <dd className="text-gray-900 font-mono">{user.id}</dd>
                  </div>
                  <div className="flex justify-between text-sm">
                    <dt className="text-gray-500">Created:</dt>
                    <dd className="text-gray-900">
                      {new Date(user.createdAt).toLocaleDateString()}
                    </dd>
                  </div>
                  <div className="flex justify-between text-sm">
                    <dt className="text-gray-500">Last Login:</dt>
                    <dd className="text-gray-900">
                      {user.lastLoginAt
                        ? new Date(user.lastLoginAt).toLocaleString()
                        : 'Never'}
                    </dd>
                  </div>
                  <div className="flex justify-between text-sm">
                    <dt className="text-gray-500">Source:</dt>
                    <dd className="text-gray-900">{user.metadata.source}</dd>
                  </div>
                </dl>
              </div>
              <div className="bg-gray-50 p-4 rounded-md">
                <h4 className="text-sm font-medium text-gray-700 mb-2">
                  Organization Info
                </h4>
                <dl className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <dt className="text-gray-500">Department:</dt>
                    <dd className="text-gray-900">{user.metadata.department || '-'}</dd>
                  </div>
                  <div className="flex justify-between text-sm">
                    <dt className="text-gray-500">Job Title:</dt>
                    <dd className="text-gray-900">{user.metadata.jobTitle || '-'}</dd>
                  </div>
                  <div className="flex justify-between text-sm">
                    <dt className="text-gray-500">Manager:</dt>
                    <dd className="text-gray-900">{user.metadata.manager || '-'}</dd>
                  </div>
                  <div className="flex justify-between text-sm">
                    <dt className="text-gray-500">Employee ID:</dt>
                    <dd className="text-gray-900">{user.metadata.employeeId || '-'}</dd>
                  </div>
                </dl>
              </div>
            </div>

            {isEditing && (
              <div className="mt-6 flex justify-end space-x-3">
                <button
                  type="button"
                  onClick={() => setIsEditing(false)}
                  className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={saving}
                  className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
                >
                  {saving ? 'Saving...' : 'Save Changes'}
                </button>
              </div>
            )}
          </form>
        )}

        {activeTab === 'security' && (
          <div className="space-y-6">
            <div>
              <h4 className="text-lg font-medium text-gray-900 mb-4">
                Security Settings
              </h4>
              <dl className="space-y-4">
                <div className="flex justify-between items-center">
                  <div>
                    <dt className="text-sm font-medium text-gray-700">
                      Multi-Factor Authentication
                    </dt>
                    <dd className="text-sm text-gray-500">
                      Add an extra layer of security
                    </dd>
                  </div>
                  <span
                    className={`px-2 py-1 rounded text-xs font-semibold ${
                      user.metadata.mfaEnabled
                        ? 'bg-green-100 text-green-800'
                        : 'bg-gray-100 text-gray-800'
                    }`}
                  >
                    {user.metadata.mfaEnabled ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <div>
                    <dt className="text-sm font-medium text-gray-700">
                      Password Last Changed
                    </dt>
                    <dd className="text-sm text-gray-500">
                      {user.metadata.passwordLastChanged
                        ? new Date(user.metadata.passwordLastChanged).toLocaleDateString()
                        : 'Never'}
                    </dd>
                  </div>
                </div>
                <div className="flex justify-between items-center">
                  <div>
                    <dt className="text-sm font-medium text-gray-700">
                      Failed Login Attempts
                    </dt>
                    <dd className="text-sm text-gray-500">
                      {user.metadata.failedLoginAttempts} attempts
                    </dd>
                  </div>
                </div>
                <div className="flex justify-between items-center">
                  <div>
                    <dt className="text-sm font-medium text-gray-700">
                      Session Timeout
                    </dt>
                    <dd className="text-sm text-gray-500">
                      {user.securitySettings.sessionTimeout} minutes
                    </dd>
                  </div>
                </div>
              </dl>
            </div>
          </div>
        )}

        {activeTab === 'roles' && (
          <div>
            <h4 className="text-lg font-medium text-gray-900 mb-4">Assigned Roles</h4>
            <div className="space-y-3">
              {user.roles.map((roleId) => {
                const role = roles.find((r) => r.id === roleId);
                return (
                  <div
                    key={roleId}
                    className="border border-gray-200 rounded-md p-4 flex justify-between items-start"
                  >
                    <div>
                      <h5 className="text-sm font-medium text-gray-900">
                        {role?.displayName || roleId}
                      </h5>
                      <p className="text-sm text-gray-500">{role?.description}</p>
                      <div className="mt-2 flex flex-wrap gap-1">
                        {role?.permissions.slice(0, 5).map((perm, idx) => (
                          <span
                            key={idx}
                            className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800"
                          >
                            {perm.resource}:{perm.action}
                          </span>
                        ))}
                        {role && role.permissions.length > 5 && (
                          <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                            +{role.permissions.length - 5} more
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {activeTab === 'teams' && (
          <div>
            <h4 className="text-lg font-medium text-gray-900 mb-4">Team Memberships</h4>
            <div className="space-y-3">
              {user.teams.map((teamId) => {
                const team = teams.find((t) => t.id === teamId);
                return (
                  <div
                    key={teamId}
                    className="border border-gray-200 rounded-md p-4 flex justify-between items-start"
                  >
                    <div>
                      <h5 className="text-sm font-medium text-gray-900">
                        {team?.displayName || teamId}
                      </h5>
                      <p className="text-sm text-gray-500">{team?.description}</p>
                      <div className="mt-2 text-xs text-gray-500">
                        {team?.members.length} members
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {activeTab === 'activity' && (
          <div>
            <h4 className="text-lg font-medium text-gray-900 mb-4">Recent Activity</h4>
            <div className="flow-root">
              <ul className="-mb-8">
                {activity.slice(0, 20).map((log, idx) => (
                  <li key={log.id}>
                    <div className="relative pb-8">
                      {idx !== activity.length - 1 && (
                        <span
                          className="absolute top-4 left-4 -ml-px h-full w-0.5 bg-gray-200"
                          aria-hidden="true"
                        />
                      )}
                      <div className="relative flex space-x-3">
                        <div>
                          <span
                            className={`h-8 w-8 rounded-full flex items-center justify-center ring-8 ring-white ${
                              log.severity === 'error' || log.severity === 'critical'
                                ? 'bg-red-500'
                                : log.severity === 'warning'
                                ? 'bg-yellow-500'
                                : 'bg-green-500'
                            }`}
                          >
                            <svg
                              className="h-5 w-5 text-white"
                              fill="currentColor"
                              viewBox="0 0 20 20"
                            >
                              <path
                                fillRule="evenodd"
                                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                                clipRule="evenodd"
                              />
                            </svg>
                          </span>
                        </div>
                        <div className="min-w-0 flex-1 pt-1.5 flex justify-between space-x-4">
                          <div>
                            <p className="text-sm text-gray-500">
                              {log.details.description}
                            </p>
                            <p className="text-xs text-gray-400 mt-1">
                              {log.resource} • {log.action}
                            </p>
                          </div>
                          <div className="text-right text-sm whitespace-nowrap text-gray-500">
                            {new Date(log.timestamp).toLocaleString()}
                          </div>
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        )}

        {activeTab === 'sessions' && (
          <div>
            <h4 className="text-lg font-medium text-gray-900 mb-4">Active Sessions</h4>
            <div className="space-y-3">
              {sessions.map((session) => (
                <div
                  key={session.id}
                  className="border border-gray-200 rounded-md p-4 flex justify-between items-start"
                >
                  <div className="flex-1">
                    <div className="flex items-center space-x-2">
                      <h5 className="text-sm font-medium text-gray-900">
                        {session.deviceName}
                      </h5>
                      <span
                        className={`px-2 py-0.5 rounded text-xs font-semibold ${
                          session.status === 'active'
                            ? 'bg-green-100 text-green-800'
                            : 'bg-gray-100 text-gray-800'
                        }`}
                      >
                        {session.status}
                      </span>
                    </div>
                    <p className="text-sm text-gray-500 mt-1">
                      {session.browser} on {session.os}
                    </p>
                    <p className="text-xs text-gray-400 mt-1">
                      IP: {session.ipAddress} • Last active:{' '}
                      {new Date(session.lastActivityAt).toLocaleString()}
                    </p>
                  </div>
                  {session.status === 'active' && (
                    <button
                      onClick={() => terminateSession(session.id)}
                      className="ml-4 text-sm text-red-600 hover:text-red-900"
                    >
                      Terminate
                    </button>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'gdpr' && (
          <div className="space-y-6">
            <div>
              <h4 className="text-lg font-medium text-gray-900 mb-4">Privacy & Data</h4>
              <dl className="space-y-4">
                <div>
                  <dt className="text-sm font-medium text-gray-700">Data Processing</dt>
                  <dd className="text-sm text-gray-500 mt-1">
                    Consent given on{' '}
                    {new Date(user.gdprConsent.consentDate).toLocaleDateString()}
                  </dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-gray-700">
                    Marketing Communications
                  </dt>
                  <dd className="text-sm text-gray-500 mt-1">
                    {user.gdprConsent.marketing ? 'Opted in' : 'Opted out'}
                  </dd>
                </div>
              </dl>
            </div>
            <div className="border-t border-gray-200 pt-6">
              <h4 className="text-sm font-medium text-gray-900 mb-4">Data Management</h4>
              <div className="space-y-3">
                <button
                  onClick={handleExportData}
                  className="w-full inline-flex justify-center items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
                >
                  Export My Data
                </button>
                <button
                  onClick={handleDeleteAccount}
                  className="w-full inline-flex justify-center items-center px-4 py-2 border border-red-300 shadow-sm text-sm font-medium rounded-md text-red-700 bg-white hover:bg-red-50"
                >
                  Delete Account
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default UserProfile;
