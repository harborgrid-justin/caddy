/**
 * CADDY v0.4.0 - File Share Component
 * File sharing with permissions, links, and access control
 */

import React, { useState, useCallback, useEffect } from 'react';
import {
  FileItem,
  FileShare,
  ShareLink,
  ShareRecipient,
  SharePermission,
} from './types';

interface FileShareProps {
  file: FileItem;
  tenantId: string;
  userId: string;
  onClose: () => void;
  onShareUpdate?: (share: FileShare) => void;
  className?: string;
}

export const FileShareComponent: React.FC<FileShareProps> = ({
  file,
  tenantId,
  userId,
  onClose,
  onShareUpdate,
  className = '',
}) => {
  const [share, setShare] = useState<FileShare | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'users' | 'links'>('users');
  const [recipientEmail, setRecipientEmail] = useState('');
  const [recipientPermission, setRecipientPermission] = useState<SharePermission>('view');
  const [linkPassword, setLinkPassword] = useState('');
  const [linkExpiration, setLinkExpiration] = useState('');
  const [linkMaxDownloads, setLinkMaxDownloads] = useState('');
  const [linkPermission, setLinkPermission] = useState<SharePermission>('view');
  const [showLinkSettings, setShowLinkSettings] = useState(false);
  const [copiedLinkId, setCopiedLinkId] = useState<string | null>(null);

  // Load share data
  useEffect(() => {
    loadShare();
  }, [file.id]);

  const loadShare = async () => {
    setLoading(true);
    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/share`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        setShare(data);
      } else if (response.status === 404) {
        // No share exists yet
        setShare({
          id: '',
          fileId: file.id,
          sharedWith: [],
          links: [],
          createdAt: new Date(),
          updatedAt: new Date(),
        });
      }
    } catch (error) {
      console.error('Failed to load share:', error);
    } finally {
      setLoading(false);
    }
  };

  // Add recipient
  const addRecipient = async () => {
    if (!recipientEmail.trim()) return;

    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/share/recipients`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email: recipientEmail,
          permission: recipientPermission,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to add recipient');
      }

      const updatedShare = await response.json();
      setShare(updatedShare);
      setRecipientEmail('');
      onShareUpdate?.(updatedShare);
    } catch (error) {
      console.error('Failed to add recipient:', error);
      alert(error instanceof Error ? error.message : 'Failed to add recipient');
    }
  };

  // Remove recipient
  const removeRecipient = async (recipientId: string) => {
    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/${file.id}/share/recipients/${recipientId}`,
        {
          method: 'DELETE',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error('Failed to remove recipient');
      }

      const updatedShare = await response.json();
      setShare(updatedShare);
      onShareUpdate?.(updatedShare);
    } catch (error) {
      console.error('Failed to remove recipient:', error);
      alert(error instanceof Error ? error.message : 'Failed to remove recipient');
    }
  };

  // Update recipient permission
  const updateRecipientPermission = async (recipientId: string, permission: SharePermission) => {
    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/${file.id}/share/recipients/${recipientId}`,
        {
          method: 'PATCH',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ permission }),
        }
      );

      if (!response.ok) {
        throw new Error('Failed to update permission');
      }

      const updatedShare = await response.json();
      setShare(updatedShare);
      onShareUpdate?.(updatedShare);
    } catch (error) {
      console.error('Failed to update permission:', error);
      alert(error instanceof Error ? error.message : 'Failed to update permission');
    }
  };

  // Create share link
  const createShareLink = async () => {
    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/share/links`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          permission: linkPermission,
          password: linkPassword || undefined,
          expiresAt: linkExpiration ? new Date(linkExpiration).toISOString() : undefined,
          maxDownloads: linkMaxDownloads ? parseInt(linkMaxDownloads) : undefined,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to create share link');
      }

      const updatedShare = await response.json();
      setShare(updatedShare);
      setShowLinkSettings(false);
      setLinkPassword('');
      setLinkExpiration('');
      setLinkMaxDownloads('');
      onShareUpdate?.(updatedShare);
    } catch (error) {
      console.error('Failed to create share link:', error);
      alert(error instanceof Error ? error.message : 'Failed to create share link');
    }
  };

  // Delete share link
  const deleteShareLink = async (linkId: string) => {
    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/${file.id}/share/links/${linkId}`,
        {
          method: 'DELETE',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error('Failed to delete share link');
      }

      const updatedShare = await response.json();
      setShare(updatedShare);
      onShareUpdate?.(updatedShare);
    } catch (error) {
      console.error('Failed to delete share link:', error);
      alert(error instanceof Error ? error.message : 'Failed to delete share link');
    }
  };

  // Copy link to clipboard
  const copyLink = async (link: ShareLink) => {
    try {
      await navigator.clipboard.writeText(link.url);
      setCopiedLinkId(link.id);
      setTimeout(() => setCopiedLinkId(null), 2000);
    } catch (error) {
      console.error('Failed to copy link:', error);
    }
  };

  // Get permission label
  const getPermissionLabel = (permission: SharePermission): string => {
    const labels: Record<SharePermission, string> = {
      view: 'Can view',
      comment: 'Can comment',
      edit: 'Can edit',
      admin: 'Full access',
    };
    return labels[permission];
  };

  // Get permission icon
  const getPermissionIcon = (permission: SharePermission): string => {
    const icons: Record<SharePermission, string> = {
      view: '👁️',
      comment: '💬',
      edit: '✏️',
      admin: '⚙️',
    };
    return icons[permission];
  };

  if (loading) {
    return (
      <div className={`file-share-modal ${className}`}>
        <div className="share-overlay" onClick={onClose}></div>
        <div className="share-container">
          <div className="loading">Loading share settings...</div>
        </div>
      </div>
    );
  }

  return (
    <div className={`file-share-modal ${className}`}>
      <div className="share-overlay" onClick={onClose}></div>
      <div className="share-container">
        {/* Header */}
        <div className="share-header">
          <h2>Share "{file.name}"</h2>
          <button onClick={onClose} className="btn-close">
            ✕
          </button>
        </div>

        {/* Tabs */}
        <div className="share-tabs">
          <button
            className={`tab ${activeTab === 'users' ? 'active' : ''}`}
            onClick={() => setActiveTab('users')}
          >
            People ({share?.sharedWith.length || 0})
          </button>
          <button
            className={`tab ${activeTab === 'links' ? 'active' : ''}`}
            onClick={() => setActiveTab('links')}
          >
            Links ({share?.links.length || 0})
          </button>
        </div>

        {/* Content */}
        <div className="share-content">
          {activeTab === 'users' && (
            <div className="share-users">
              {/* Add Recipient */}
              <div className="add-recipient">
                <input
                  type="email"
                  placeholder="Enter email address"
                  value={recipientEmail}
                  onChange={(e) => setRecipientEmail(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && addRecipient()}
                  className="form-input"
                />
                <select
                  value={recipientPermission}
                  onChange={(e) => setRecipientPermission(e.target.value as SharePermission)}
                  className="form-select"
                >
                  <option value="view">Can view</option>
                  <option value="comment">Can comment</option>
                  <option value="edit">Can edit</option>
                  <option value="admin">Full access</option>
                </select>
                <button onClick={addRecipient} className="btn btn-primary">
                  Add
                </button>
              </div>

              {/* Recipients List */}
              <div className="recipients-list">
                {share?.sharedWith.length === 0 ? (
                  <div className="empty-state">
                    No one has access yet. Add people above to share this file.
                  </div>
                ) : (
                  share?.sharedWith.map(recipient => (
                    <div key={recipient.id} className="recipient-item">
                      <div className="recipient-info">
                        <div className="recipient-avatar">
                          {recipient.name.charAt(0).toUpperCase()}
                        </div>
                        <div className="recipient-details">
                          <div className="recipient-name">{recipient.name}</div>
                          <div className="recipient-email">{recipient.email}</div>
                        </div>
                      </div>
                      <div className="recipient-actions">
                        <select
                          value={recipient.permission}
                          onChange={(e) =>
                            updateRecipientPermission(recipient.id, e.target.value as SharePermission)
                          }
                          className="form-select form-select-sm"
                        >
                          <option value="view">Can view</option>
                          <option value="comment">Can comment</option>
                          <option value="edit">Can edit</option>
                          <option value="admin">Full access</option>
                        </select>
                        <button
                          onClick={() => removeRecipient(recipient.id)}
                          className="btn btn-sm btn-danger"
                        >
                          Remove
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          )}

          {activeTab === 'links' && (
            <div className="share-links">
              {/* Create Link Button */}
              {!showLinkSettings && (
                <button
                  onClick={() => setShowLinkSettings(true)}
                  className="btn btn-primary btn-block"
                >
                  Create Share Link
                </button>
              )}

              {/* Link Settings */}
              {showLinkSettings && (
                <div className="link-settings">
                  <h3>Link Settings</h3>

                  <div className="form-group">
                    <label>Permission Level</label>
                    <select
                      value={linkPermission}
                      onChange={(e) => setLinkPermission(e.target.value as SharePermission)}
                      className="form-select"
                    >
                      <option value="view">Can view</option>
                      <option value="comment">Can comment</option>
                      <option value="edit">Can edit</option>
                    </select>
                  </div>

                  <div className="form-group">
                    <label>Password (optional)</label>
                    <input
                      type="password"
                      placeholder="Set password for link"
                      value={linkPassword}
                      onChange={(e) => setLinkPassword(e.target.value)}
                      className="form-input"
                    />
                  </div>

                  <div className="form-group">
                    <label>Expiration Date (optional)</label>
                    <input
                      type="datetime-local"
                      value={linkExpiration}
                      onChange={(e) => setLinkExpiration(e.target.value)}
                      className="form-input"
                    />
                  </div>

                  <div className="form-group">
                    <label>Maximum Downloads (optional)</label>
                    <input
                      type="number"
                      min="1"
                      placeholder="Unlimited"
                      value={linkMaxDownloads}
                      onChange={(e) => setLinkMaxDownloads(e.target.value)}
                      className="form-input"
                    />
                  </div>

                  <div className="form-actions">
                    <button onClick={createShareLink} className="btn btn-primary">
                      Create Link
                    </button>
                    <button
                      onClick={() => setShowLinkSettings(false)}
                      className="btn"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              )}

              {/* Links List */}
              <div className="links-list">
                {share?.links.length === 0 ? (
                  <div className="empty-state">
                    No share links created yet. Create one above to share this file.
                  </div>
                ) : (
                  share?.links.map(link => (
                    <div key={link.id} className={`link-item ${!link.isActive ? 'inactive' : ''}`}>
                      <div className="link-header">
                        <div className="link-permission">
                          <span className="permission-icon">{getPermissionIcon(link.permission)}</span>
                          <span className="permission-label">{getPermissionLabel(link.permission)}</span>
                        </div>
                        <div className="link-status">
                          {link.isActive ? (
                            <span className="status-badge status-active">Active</span>
                          ) : (
                            <span className="status-badge status-inactive">Inactive</span>
                          )}
                        </div>
                      </div>

                      <div className="link-url">
                        <input
                          type="text"
                          value={link.url}
                          readOnly
                          className="form-input"
                        />
                        <button
                          onClick={() => copyLink(link)}
                          className="btn btn-sm"
                        >
                          {copiedLinkId === link.id ? 'Copied!' : 'Copy'}
                        </button>
                      </div>

                      <div className="link-details">
                        {link.password && (
                          <div className="link-detail">
                            <span className="detail-icon">🔒</span>
                            <span>Password protected</span>
                          </div>
                        )}
                        {link.expiresAt && (
                          <div className="link-detail">
                            <span className="detail-icon">⏰</span>
                            <span>Expires {formatDate(link.expiresAt)}</span>
                          </div>
                        )}
                        {link.maxDownloads && (
                          <div className="link-detail">
                            <span className="detail-icon">📥</span>
                            <span>
                              {link.downloadCount}/{link.maxDownloads} downloads
                            </span>
                          </div>
                        )}
                        <div className="link-detail">
                          <span className="detail-icon">📅</span>
                          <span>Created {formatDate(link.createdAt)}</span>
                        </div>
                      </div>

                      <div className="link-actions">
                        <button
                          onClick={() => deleteShareLink(link.id)}
                          className="btn btn-sm btn-danger"
                        >
                          Delete Link
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="share-footer">
          <div className="share-info">
            {file.permissions.canShare ? (
              <span>You can share this file with others</span>
            ) : (
              <span className="warning">You have limited sharing permissions</span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

function formatDate(date: Date): string {
  const d = new Date(date);
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return 'today';
  if (diffDays === 1) return 'yesterday';
  if (diffDays < 7) return `${diffDays} days ago`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;

  return d.toLocaleDateString();
}

export default FileShareComponent;
