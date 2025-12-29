/**
 * Collaboration Panel - User List, Activity Feed, and Chat
 *
 * Displays active collaborators, their status, recent activity, and provides
 * a built-in chat interface for team communication.
 */

import React, { useState, useRef, useEffect } from 'react';
import { usePresence, useActivity, useSync } from './useCollaboration';
import type { UserPresence } from './useCollaboration';

/**
 * Chat message interface
 */
interface ChatMessage {
  id: string;
  userId: string;
  userName: string;
  message: string;
  timestamp: Date;
  type: 'message' | 'system' | 'mention';
}

/**
 * Activity item interface
 */
interface ActivityItem {
  id: string;
  userId: string;
  userName: string;
  action: string;
  details?: string;
  timestamp: Date;
  icon: string;
}

/**
 * Panel tab type
 */
type PanelTab = 'users' | 'activity' | 'chat';

/**
 * Collaboration Panel Props
 */
export interface CollaborationPanelProps {
  className?: string;
  style?: React.CSSProperties;
  defaultTab?: PanelTab;
  showChat?: boolean;
  maxHeight?: string | number;
}

/**
 * User Avatar Component
 */
function UserAvatar({ user, size = 32 }: { user: UserPresence; size?: number }) {
  const initials = user.user.name
    .split(' ')
    .map(n => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2);

  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: '50%',
        backgroundColor: user.user.color,
        color: '#fff',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: size * 0.4,
        fontWeight: 'bold',
        position: 'relative',
      }}
    >
      {user.user.avatar ? (
        <img
          src={user.user.avatar}
          alt={user.user.name}
          style={{ width: '100%', height: '100%', borderRadius: '50%' }}
        />
      ) : (
        initials
      )}
      <div
        style={{
          position: 'absolute',
          bottom: 0,
          right: 0,
          width: size * 0.25,
          height: size * 0.25,
          borderRadius: '50%',
          backgroundColor: user.isActive ? '#22c55e' : '#94a3b8',
          border: '2px solid white',
        }}
      />
    </div>
  );
}

/**
 * User List Component
 */
function UserList({ users }: { users: UserPresence[] }) {
  const formatLastActive = (date: Date) => {
    const seconds = Math.floor((Date.now() - date.getTime()) / 1000);

    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    return `${Math.floor(seconds / 86400)}d ago`;
  };

  return (
    <div style={{ padding: '12px' }}>
      <div style={{ marginBottom: '12px', color: '#64748b', fontSize: '12px', fontWeight: '600' }}>
        {users.length} ACTIVE {users.length === 1 ? 'USER' : 'USERS'}
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        {users.map(user => (
          <div
            key={user.userId}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              padding: '8px',
              borderRadius: '6px',
              backgroundColor: user.isActive ? '#f8fafc' : '#fff',
              border: '1px solid #e2e8f0',
            }}
          >
            <UserAvatar user={user} size={36} />
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontWeight: '500', fontSize: '14px', color: '#1e293b' }}>
                {user.user.name}
              </div>
              <div style={{ fontSize: '12px', color: '#64748b' }}>
                {user.isActive ? 'Active' : `Last seen ${formatLastActive(user.lastActive)}`}
              </div>
            </div>
            {user.selection && user.selection.length > 0 && (
              <div
                style={{
                  fontSize: '11px',
                  color: '#64748b',
                  backgroundColor: '#f1f5f9',
                  padding: '2px 6px',
                  borderRadius: '4px',
                }}
              >
                {user.selection.length} selected
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

/**
 * Activity Feed Component
 */
function ActivityFeed({ activities }: { activities: ActivityItem[] }) {
  const getActivityIcon = (icon: string) => {
    const icons: Record<string, string> = {
      create: '➕',
      edit: '✏️',
      delete: '🗑️',
      move: '↔️',
      comment: '💬',
      version: '📌',
    };
    return icons[icon] || '•';
  };

  const formatTime = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const seconds = Math.floor(diff / 1000);

    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;

    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div style={{ padding: '12px' }}>
      <div style={{ marginBottom: '12px', color: '#64748b', fontSize: '12px', fontWeight: '600' }}>
        RECENT ACTIVITY
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
        {activities.length === 0 ? (
          <div style={{ padding: '24px', textAlign: 'center', color: '#94a3b8', fontSize: '14px' }}>
            No recent activity
          </div>
        ) : (
          activities.map(activity => (
            <div
              key={activity.id}
              style={{
                display: 'flex',
                gap: '12px',
                padding: '8px',
                borderRadius: '4px',
                fontSize: '13px',
              }}
            >
              <div style={{ fontSize: '16px' }}>{getActivityIcon(activity.icon)}</div>
              <div style={{ flex: 1, minWidth: 0 }}>
                <div>
                  <span style={{ fontWeight: '600', color: '#1e293b' }}>{activity.userName}</span>
                  <span style={{ color: '#64748b' }}> {activity.action}</span>
                </div>
                {activity.details && (
                  <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                    {activity.details}
                  </div>
                )}
              </div>
              <div style={{ fontSize: '11px', color: '#94a3b8', whiteSpace: 'nowrap' }}>
                {formatTime(activity.timestamp)}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

/**
 * Chat Component
 */
function Chat({ messages, onSendMessage }: {
  messages: ChatMessage[];
  onSendMessage: (message: string) => void;
}) {
  const [inputValue, setInputValue] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSend = () => {
    if (inputValue.trim()) {
      onSendMessage(inputValue);
      setInputValue('');
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ flex: 1, overflowY: 'auto', padding: '12px' }}>
        {messages.length === 0 ? (
          <div style={{ padding: '24px', textAlign: 'center', color: '#94a3b8', fontSize: '14px' }}>
            No messages yet. Start the conversation!
          </div>
        ) : (
          messages.map(msg => (
            <div
              key={msg.id}
              style={{
                marginBottom: '12px',
                padding: msg.type === 'system' ? '8px' : '0',
                textAlign: msg.type === 'system' ? 'center' : 'left',
              }}
            >
              {msg.type === 'system' ? (
                <div style={{ fontSize: '12px', color: '#94a3b8' }}>{msg.message}</div>
              ) : (
                <>
                  <div style={{ display: 'flex', gap: '8px', marginBottom: '4px' }}>
                    <span style={{ fontWeight: '600', fontSize: '13px', color: '#1e293b' }}>
                      {msg.userName}
                    </span>
                    <span style={{ fontSize: '11px', color: '#94a3b8' }}>
                      {msg.timestamp.toLocaleTimeString('en-US', {
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                    </span>
                  </div>
                  <div
                    style={{
                      fontSize: '13px',
                      color: '#475569',
                      backgroundColor: msg.type === 'mention' ? '#fef3c7' : '#f8fafc',
                      padding: '8px 12px',
                      borderRadius: '8px',
                      borderLeft: msg.type === 'mention' ? '3px solid #fbbf24' : 'none',
                    }}
                  >
                    {msg.message}
                  </div>
                </>
              )}
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>
      <div style={{ borderTop: '1px solid #e2e8f0', padding: '12px' }}>
        <div style={{ display: 'flex', gap: '8px' }}>
          <input
            type="text"
            value={inputValue}
            onChange={e => setInputValue(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Type a message..."
            style={{
              flex: 1,
              padding: '8px 12px',
              border: '1px solid #e2e8f0',
              borderRadius: '6px',
              fontSize: '14px',
              outline: 'none',
            }}
          />
          <button
            onClick={handleSend}
            disabled={!inputValue.trim()}
            style={{
              padding: '8px 16px',
              backgroundColor: inputValue.trim() ? '#3b82f6' : '#e2e8f0',
              color: inputValue.trim() ? '#fff' : '#94a3b8',
              border: 'none',
              borderRadius: '6px',
              fontSize: '14px',
              fontWeight: '500',
              cursor: inputValue.trim() ? 'pointer' : 'not-allowed',
            }}
          >
            Send
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * Main Collaboration Panel Component
 */
export function CollaborationPanel({
  className = '',
  style = {},
  defaultTab = 'users',
  showChat = true,
  maxHeight = '600px',
}: CollaborationPanelProps) {
  const { users } = usePresence();
  const { syncState } = useSync();
  const [activeTab, setActiveTab] = useState<PanelTab>(defaultTab);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [activities] = useState<ActivityItem[]>([
    {
      id: '1',
      userId: 'user1',
      userName: 'John Doe',
      action: 'created a new layer',
      details: 'Electrical Components',
      timestamp: new Date(Date.now() - 5 * 60 * 1000),
      icon: 'create',
    },
    {
      id: '2',
      userId: 'user2',
      userName: 'Jane Smith',
      action: 'modified dimensions',
      details: '5 entities affected',
      timestamp: new Date(Date.now() - 15 * 60 * 1000),
      icon: 'edit',
    },
  ]);

  const handleSendMessage = (message: string) => {
    const newMessage: ChatMessage = {
      id: Date.now().toString(),
      userId: 'current-user',
      userName: 'You',
      message,
      timestamp: new Date(),
      type: 'message',
    };
    setChatMessages(prev => [...prev, newMessage]);
  };

  const tabs: { id: PanelTab; label: string; count?: number }[] = [
    { id: 'users', label: 'Users', count: users.length },
    { id: 'activity', label: 'Activity' },
  ];

  if (showChat) {
    tabs.push({ id: 'chat', label: 'Chat', count: chatMessages.length || undefined });
  }

  return (
    <div
      className={className}
      style={{
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: '#fff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        overflow: 'hidden',
        maxHeight,
        ...style,
      }}
    >
      {/* Header with sync status */}
      <div
        style={{
          padding: '12px 16px',
          borderBottom: '1px solid #e2e8f0',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <h3 style={{ margin: 0, fontSize: '16px', fontWeight: '600', color: '#1e293b' }}>
          Collaboration
        </h3>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <div
            style={{
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              backgroundColor:
                syncState === 'synchronized'
                  ? '#22c55e'
                  : syncState === 'syncing'
                  ? '#eab308'
                  : syncState === 'offline'
                  ? '#94a3b8'
                  : '#ef4444',
            }}
          />
          <span style={{ fontSize: '12px', color: '#64748b', textTransform: 'capitalize' }}>
            {syncState}
          </span>
        </div>
      </div>

      {/* Tabs */}
      <div
        style={{
          display: 'flex',
          borderBottom: '1px solid #e2e8f0',
          backgroundColor: '#f8fafc',
        }}
      >
        {tabs.map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            style={{
              flex: 1,
              padding: '12px',
              border: 'none',
              backgroundColor: 'transparent',
              color: activeTab === tab.id ? '#3b82f6' : '#64748b',
              borderBottom: activeTab === tab.id ? '2px solid #3b82f6' : '2px solid transparent',
              fontSize: '14px',
              fontWeight: '500',
              cursor: 'pointer',
              transition: 'all 0.2s',
            }}
          >
            {tab.label}
            {tab.count !== undefined && (
              <span
                style={{
                  marginLeft: '6px',
                  fontSize: '12px',
                  backgroundColor: activeTab === tab.id ? '#dbeafe' : '#f1f5f9',
                  color: activeTab === tab.id ? '#3b82f6' : '#94a3b8',
                  padding: '2px 6px',
                  borderRadius: '10px',
                }}
              >
                {tab.count}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflowY: 'auto' }}>
        {activeTab === 'users' && <UserList users={users} />}
        {activeTab === 'activity' && <ActivityFeed activities={activities} />}
        {activeTab === 'chat' && (
          <Chat messages={chatMessages} onSendMessage={handleSendMessage} />
        )}
      </div>
    </div>
  );
}

export default CollaborationPanel;
