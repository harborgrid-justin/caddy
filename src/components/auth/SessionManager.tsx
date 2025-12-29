/**
 * Session Manager Component
 * View and manage user sessions
 */

import React, { useState, useEffect } from 'react';
import { Button, Table, Modal, Tooltip } from '../enterprise';
import type { Session, SessionStats, SessionManagerProps } from './types';

const formatDate = (dateString: string): string => {
  const date = new Date(dateString);
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
};

const formatRelativeTime = (dateString: string): string => {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
  if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
  return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
};

const getDeviceInfo = (userAgent: string): { type: string; name: string } => {
  const ua = userAgent.toLowerCase();

  let type = 'desktop';
  if (ua.includes('mobile') || ua.includes('android') || ua.includes('iphone')) {
    type = 'mobile';
  } else if (ua.includes('ipad') || ua.includes('tablet')) {
    type = 'tablet';
  }

  let name = 'Unknown Device';
  if (ua.includes('chrome')) name = 'Chrome';
  else if (ua.includes('firefox')) name = 'Firefox';
  else if (ua.includes('safari')) name = 'Safari';
  else if (ua.includes('edge')) name = 'Edge';

  return { type, name };
};

const getDeviceIcon = (type: string): string => {
  switch (type) {
    case 'mobile':
      return '📱';
    case 'tablet':
      return '💻';
    case 'desktop':
      return '🖥️';
    default:
      return '📟';
  }
};

export const SessionManager: React.FC<SessionManagerProps> = ({
  userId,
  showAllSessions = false,
  onSessionRevoked,
}) => {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [stats, setStats] = useState<SessionStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [revokeModalOpen, setRevokeModalOpen] = useState(false);
  const [sessionToRevoke, setSessionToRevoke] = useState<Session | null>(null);
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
  const [filter, setFilter] = useState<'all' | 'active' | 'expired'>('all');

  useEffect(() => {
    loadSessions();
    loadStats();

    // Refresh every 30 seconds
    const interval = setInterval(() => {
      loadSessions();
      loadStats();
    }, 30000);

    return () => clearInterval(interval);
  }, [userId, showAllSessions]);

  const loadSessions = async () => {
    setLoading(true);
    try {
      const url = showAllSessions
        ? '/api/sessions'
        : `/api/sessions?user_id=${userId}`;

      const response = await fetch(url);
      const data = await response.json();

      setSessions(data.sessions || []);
      setCurrentSessionId(data.current_session_id || null);
    } catch (error) {
      console.error('Failed to load sessions:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadStats = async () => {
    try {
      const url = showAllSessions
        ? '/api/sessions/stats'
        : `/api/sessions/stats?user_id=${userId}`;

      const response = await fetch(url);
      const data = await response.json();
      setStats(data);
    } catch (error) {
      console.error('Failed to load stats:', error);
    }
  };

  const handleRevokeClick = (session: Session) => {
    setSessionToRevoke(session);
    setRevokeModalOpen(true);
  };

  const confirmRevoke = async () => {
    if (!sessionToRevoke) return;

    try {
      await fetch(`/api/sessions/${sessionToRevoke.id}/revoke`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reason: 'Manually revoked by user' }),
      });

      setSessions((prev) =>
        prev.map((s) =>
          s.id === sessionToRevoke.id
            ? { ...s, is_active: false, revoked: true }
            : s
        )
      );

      onSessionRevoked?.(sessionToRevoke.id);
      setRevokeModalOpen(false);
      setSessionToRevoke(null);
    } catch (error) {
      console.error('Failed to revoke session:', error);
    }
  };

  const handleRevokeAll = async () => {
    if (!confirm('Are you sure you want to revoke all sessions except the current one?')) {
      return;
    }

    try {
      await fetch(`/api/sessions/revoke-all?user_id=${userId}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ except_current: true }),
      });

      loadSessions();
    } catch (error) {
      console.error('Failed to revoke all sessions:', error);
    }
  };

  const filteredSessions = sessions.filter((session) => {
    if (filter === 'active') return session.is_active && !session.revoked;
    if (filter === 'expired') return !session.is_active || session.revoked;
    return true;
  });

  const columns = [
    {
      key: 'device',
      header: 'Device',
      render: (session: Session) => {
        const device = getDeviceInfo(session.user_agent);
        const isCurrent = session.id === currentSessionId;

        return (
          <div className="flex items-center space-x-3">
            <span className="text-2xl">{getDeviceIcon(device.type)}</span>
            <div>
              <div className="flex items-center space-x-2">
                <span className="font-medium">{device.name}</span>
                {isCurrent && (
                  <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
                    Current
                  </span>
                )}
              </div>
              <div className="text-xs text-gray-500">
                {session.device_name || 'Unnamed Device'}
              </div>
            </div>
          </div>
        );
      },
    },
    {
      key: 'location',
      header: 'Location',
      render: (session: Session) => (
        <div>
          <div className="text-sm">{session.ip_address}</div>
          <div className="text-xs text-gray-500">
            {/* In production, use IP geolocation service */}
            Unknown Location
          </div>
        </div>
      ),
    },
    {
      key: 'activity',
      header: 'Last Activity',
      render: (session: Session) => (
        <div>
          <div className="text-sm">
            {formatRelativeTime(session.last_accessed)}
          </div>
          <div className="text-xs text-gray-500">
            {formatDate(session.last_accessed)}
          </div>
        </div>
      ),
    },
    {
      key: 'created',
      header: 'Created',
      render: (session: Session) => (
        <div className="text-sm">{formatDate(session.created_at)}</div>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (session: Session) => {
        if (session.revoked) {
          return (
            <span className="px-2 py-1 rounded text-xs bg-red-100 text-red-800">
              Revoked
            </span>
          );
        }

        if (!session.is_active) {
          return (
            <span className="px-2 py-1 rounded text-xs bg-gray-100 text-gray-800">
              Expired
            </span>
          );
        }

        const expiresAt = new Date(session.expires_at);
        const now = new Date();
        const hoursUntilExpiry = Math.floor(
          (expiresAt.getTime() - now.getTime()) / 3600000
        );

        return (
          <div>
            <span className="px-2 py-1 rounded text-xs bg-green-100 text-green-800">
              Active
            </span>
            {session.mfa_verified && (
              <span className="ml-1 px-2 py-1 rounded text-xs bg-blue-100 text-blue-800">
                MFA ✓
              </span>
            )}
            {hoursUntilExpiry < 24 && (
              <div className="text-xs text-gray-500 mt-1">
                Expires in {hoursUntilExpiry}h
              </div>
            )}
          </div>
        );
      },
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (session: Session) => {
        const isCurrent = session.id === currentSessionId;

        if (session.revoked || !session.is_active) {
          return <span className="text-gray-400 text-sm">-</span>;
        }

        return (
          <Tooltip content={isCurrent ? 'Cannot revoke current session' : 'End this session'}>
            <Button
              size="sm"
              variant="danger"
              onClick={() => handleRevokeClick(session)}
              disabled={isCurrent}
            >
              Revoke
            </Button>
          </Tooltip>
        );
      },
    },
  ];

  return (
    <div className="p-6">
      {/* Header */}
      <div className="mb-6">
        <div className="flex justify-between items-center mb-4">
          <div>
            <h2 className="text-2xl font-bold">Active Sessions</h2>
            <p className="text-gray-600 mt-1">
              Manage your active login sessions across devices
            </p>
          </div>
          <Button variant="danger" onClick={handleRevokeAll}>
            Revoke All Other Sessions
          </Button>
        </div>

        {/* Stats Cards */}
        {stats && (
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
            <div className="bg-white p-4 rounded-lg border">
              <div className="text-sm text-gray-600">Total Sessions</div>
              <div className="text-2xl font-bold">{stats.total_sessions}</div>
            </div>
            <div className="bg-green-50 p-4 rounded-lg border border-green-200">
              <div className="text-sm text-green-600">Active</div>
              <div className="text-2xl font-bold text-green-700">
                {stats.active_sessions}
              </div>
            </div>
            <div className="bg-gray-50 p-4 rounded-lg border border-gray-200">
              <div className="text-sm text-gray-600">Expired</div>
              <div className="text-2xl font-bold text-gray-700">
                {stats.expired_sessions}
              </div>
            </div>
            <div className="bg-red-50 p-4 rounded-lg border border-red-200">
              <div className="text-sm text-red-600">Revoked</div>
              <div className="text-2xl font-bold text-red-700">
                {stats.revoked_sessions}
              </div>
            </div>
          </div>
        )}

        {/* Filters */}
        <div className="flex space-x-2">
          <button
            onClick={() => setFilter('all')}
            className={`px-4 py-2 rounded ${
              filter === 'all'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            All
          </button>
          <button
            onClick={() => setFilter('active')}
            className={`px-4 py-2 rounded ${
              filter === 'active'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            Active
          </button>
          <button
            onClick={() => setFilter('expired')}
            className={`px-4 py-2 rounded ${
              filter === 'expired'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            Expired/Revoked
          </button>
        </div>
      </div>

      {/* Sessions Table */}
      <Table
        columns={columns}
        data={filteredSessions}
        loading={loading}
        emptyMessage="No sessions found"
      />

      {/* Security Tips */}
      <div className="mt-6 bg-blue-50 p-4 rounded-lg border border-blue-200">
        <h4 className="font-medium text-blue-900 mb-2">Security Tips</h4>
        <ul className="text-sm text-blue-800 space-y-1 list-disc list-inside">
          <li>Review your sessions regularly and revoke any you don't recognize</li>
          <li>If you see suspicious activity, revoke all sessions and change your password</li>
          <li>Use unique devices names to easily identify your sessions</li>
          <li>Enable MFA for additional security</li>
        </ul>
      </div>

      {/* Revoke Confirmation Modal */}
      <Modal
        isOpen={revokeModalOpen}
        onClose={() => setRevokeModalOpen(false)}
        title="Revoke Session"
      >
        {sessionToRevoke && (
          <div className="space-y-4">
            <p>Are you sure you want to revoke this session?</p>

            <div className="bg-gray-50 p-4 rounded">
              <dl className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <dt className="text-gray-600">Device:</dt>
                  <dd className="font-medium">
                    {getDeviceInfo(sessionToRevoke.user_agent).name}
                  </dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-gray-600">IP Address:</dt>
                  <dd className="font-medium">{sessionToRevoke.ip_address}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-gray-600">Last Active:</dt>
                  <dd className="font-medium">
                    {formatRelativeTime(sessionToRevoke.last_accessed)}
                  </dd>
                </div>
              </dl>
            </div>

            <p className="text-sm text-gray-600">
              This will immediately log out this session. The user will need to
              log in again to access their account.
            </p>

            <div className="flex justify-end space-x-2">
              <Button
                variant="secondary"
                onClick={() => setRevokeModalOpen(false)}
              >
                Cancel
              </Button>
              <Button variant="danger" onClick={confirmRevoke}>
                Revoke Session
              </Button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
};
