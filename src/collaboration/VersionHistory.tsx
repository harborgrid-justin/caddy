/**
 * Version History - Version Timeline with Diff Visualization
 *
 * Displays document version history, allows browsing through versions,
 * creating branches, and viewing diffs between versions.
 */

import React, { useState, useEffect, useMemo } from 'react';
import { useVersioning } from './useCollaboration';
import type { DocumentVersion, User } from './useCollaboration';

/**
 * Diff change type
 */
interface DiffChange {
  type: 'added' | 'modified' | 'deleted';
  entityId: string;
  entityType?: string;
  property?: string;
  oldValue?: any;
  newValue?: any;
  description: string;
}

/**
 * Version diff data
 */
interface VersionDiff {
  fromVersion: DocumentVersion;
  toVersion: DocumentVersion;
  changes: DiffChange[];
  stats: {
    added: number;
    modified: number;
    deleted: number;
  };
}

/**
 * Version History Props
 */
export interface VersionHistoryProps {
  documentId: string;
  className?: string;
  style?: React.CSSProperties;
  maxHeight?: string | number;
  onVersionSelect?: (version: DocumentVersion) => void;
  onCreateBranch?: (name: string, version: DocumentVersion) => void;
  showBranches?: boolean;
  showTags?: boolean;
}

/**
 * Version Timeline Item Component
 */
function VersionTimelineItem({
  version,
  isSelected,
  onSelect,
  onCreateBranch,
  onCreateTag,
}: {
  version: DocumentVersion;
  isSelected: boolean;
  onSelect: () => void;
  onCreateBranch: () => void;
  onCreateTag: () => void;
}) {
  const [showActions, setShowActions] = useState(false);

  const formatDate = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const seconds = Math.floor(diff / 1000);

    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)} minutes ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)} hours ago`;
    if (seconds < 604800) return `${Math.floor(seconds / 86400)} days ago`;

    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
    });
  };

  const getCommitIcon = (message: string) => {
    if (message.toLowerCase().includes('merge')) return '🔀';
    if (message.toLowerCase().includes('initial')) return '🎯';
    if (message.toLowerCase().includes('fix')) return '🔧';
    return '📝';
  };

  return (
    <div
      style={{
        position: 'relative',
        paddingLeft: '24px',
        paddingBottom: '20px',
      }}
      onMouseEnter={() => setShowActions(true)}
      onMouseLeave={() => setShowActions(false)}
    >
      {/* Timeline line */}
      <div
        style={{
          position: 'absolute',
          left: '11px',
          top: '24px',
          bottom: '0',
          width: '2px',
          backgroundColor: '#e2e8f0',
        }}
      />

      {/* Timeline dot */}
      <div
        style={{
          position: 'absolute',
          left: '6px',
          top: '8px',
          width: '12px',
          height: '12px',
          borderRadius: '50%',
          backgroundColor: isSelected ? '#3b82f6' : '#cbd5e1',
          border: '2px solid #fff',
          boxShadow: '0 0 0 1px #e2e8f0',
        }}
      />

      {/* Content */}
      <div
        onClick={onSelect}
        style={{
          padding: '8px 12px',
          backgroundColor: isSelected ? '#eff6ff' : '#fff',
          border: `1px solid ${isSelected ? '#3b82f6' : '#e2e8f0'}`,
          borderRadius: '6px',
          cursor: 'pointer',
          transition: 'all 0.2s',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'flex-start', gap: '8px' }}>
          <span style={{ fontSize: '16px', marginTop: '2px' }}>
            {getCommitIcon(version.message)}
          </span>
          <div style={{ flex: 1, minWidth: 0 }}>
            <div
              style={{
                fontSize: '14px',
                fontWeight: '500',
                color: '#1e293b',
                marginBottom: '4px',
              }}
            >
              {version.message}
            </div>
            <div style={{ fontSize: '12px', color: '#64748b', marginBottom: '4px' }}>
              {version.author.name} · {formatDate(version.timestamp)}
            </div>
            {version.tags.length > 0 && (
              <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap', marginTop: '6px' }}>
                {version.tags.map(tag => (
                  <span
                    key={tag}
                    style={{
                      fontSize: '11px',
                      padding: '2px 6px',
                      backgroundColor: '#fef3c7',
                      color: '#92400e',
                      borderRadius: '3px',
                      fontWeight: '500',
                    }}
                  >
                    🏷️ {tag}
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Actions */}
        {showActions && (
          <div
            style={{
              marginTop: '8px',
              paddingTop: '8px',
              borderTop: '1px solid #e2e8f0',
              display: 'flex',
              gap: '8px',
            }}
          >
            <button
              onClick={e => {
                e.stopPropagation();
                onCreateBranch();
              }}
              style={{
                fontSize: '11px',
                padding: '4px 8px',
                backgroundColor: '#f8fafc',
                border: '1px solid #e2e8f0',
                borderRadius: '4px',
                cursor: 'pointer',
                color: '#475569',
              }}
            >
              🌿 Branch
            </button>
            <button
              onClick={e => {
                e.stopPropagation();
                onCreateTag();
              }}
              style={{
                fontSize: '11px',
                padding: '4px 8px',
                backgroundColor: '#f8fafc',
                border: '1px solid #e2e8f0',
                borderRadius: '4px',
                cursor: 'pointer',
                color: '#475569',
              }}
            >
              🏷️ Tag
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Diff Viewer Component
 */
function DiffViewer({ diff }: { diff: VersionDiff | null }) {
  if (!diff) {
    return (
      <div
        style={{
          padding: '32px',
          textAlign: 'center',
          color: '#94a3b8',
          fontSize: '14px',
        }}
      >
        Select two versions to compare
      </div>
    );
  }

  const getChangeIcon = (type: DiffChange['type']) => {
    switch (type) {
      case 'added':
        return '➕';
      case 'modified':
        return '✏️';
      case 'deleted':
        return '🗑️';
    }
  };

  const getChangeColor = (type: DiffChange['type']) => {
    switch (type) {
      case 'added':
        return '#dcfce7';
      case 'modified':
        return '#fef3c7';
      case 'deleted':
        return '#fee2e2';
    }
  };

  return (
    <div style={{ padding: '16px' }}>
      {/* Stats */}
      <div
        style={{
          display: 'flex',
          gap: '16px',
          padding: '12px',
          backgroundColor: '#f8fafc',
          borderRadius: '6px',
          marginBottom: '16px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <span style={{ color: '#22c55e', fontWeight: '600' }}>+{diff.stats.added}</span>
          <span style={{ fontSize: '12px', color: '#64748b' }}>added</span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <span style={{ color: '#eab308', fontWeight: '600' }}>~{diff.stats.modified}</span>
          <span style={{ fontSize: '12px', color: '#64748b' }}>modified</span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <span style={{ color: '#ef4444', fontWeight: '600' }}>-{diff.stats.deleted}</span>
          <span style={{ fontSize: '12px', color: '#64748b' }}>deleted</span>
        </div>
      </div>

      {/* Changes list */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        {diff.changes.map((change, index) => (
          <div
            key={index}
            style={{
              padding: '12px',
              backgroundColor: getChangeColor(change.type),
              border: '1px solid #e2e8f0',
              borderRadius: '6px',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
              <span style={{ fontSize: '16px' }}>{getChangeIcon(change.type)}</span>
              <span style={{ fontSize: '13px', fontWeight: '500', color: '#1e293b' }}>
                {change.description}
              </span>
            </div>
            {change.property && (
              <div style={{ fontSize: '12px', color: '#64748b', marginLeft: '24px' }}>
                Property: <code style={{ backgroundColor: '#fff', padding: '2px 4px', borderRadius: '3px' }}>{change.property}</code>
              </div>
            )}
          </div>
        ))}

        {diff.changes.length === 0 && (
          <div style={{ padding: '24px', textAlign: 'center', color: '#94a3b8', fontSize: '14px' }}>
            No changes between these versions
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Branch Selector Component
 */
function BranchSelector({
  branches,
  currentBranch,
  onSwitchBranch,
  onCreateBranch,
}: {
  branches: string[];
  currentBranch: string;
  onSwitchBranch: (branch: string) => void;
  onCreateBranch: () => void;
}) {
  return (
    <div style={{ padding: '12px', borderBottom: '1px solid #e2e8f0' }}>
      <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
        <select
          value={currentBranch}
          onChange={e => onSwitchBranch(e.target.value)}
          style={{
            flex: 1,
            padding: '6px 10px',
            border: '1px solid #e2e8f0',
            borderRadius: '6px',
            fontSize: '13px',
            backgroundColor: '#fff',
            cursor: 'pointer',
          }}
        >
          {branches.map(branch => (
            <option key={branch} value={branch}>
              🌿 {branch}
            </option>
          ))}
        </select>
        <button
          onClick={onCreateBranch}
          style={{
            padding: '6px 12px',
            backgroundColor: '#3b82f6',
            color: '#fff',
            border: 'none',
            borderRadius: '6px',
            fontSize: '13px',
            fontWeight: '500',
            cursor: 'pointer',
          }}
        >
          New
        </button>
      </div>
    </div>
  );
}

/**
 * Main Version History Component
 */
export function VersionHistory({
  documentId,
  className = '',
  style = {},
  maxHeight = '600px',
  onVersionSelect,
  onCreateBranch,
  showBranches = true,
  showTags = true,
}: VersionHistoryProps) {
  const { branches, currentBranch, versions, createBranch, switchBranch, createTag, refreshVersions } =
    useVersioning(documentId);

  const [selectedVersion, setSelectedVersion] = useState<DocumentVersion | null>(null);
  const [compareVersion, setCompareVersion] = useState<DocumentVersion | null>(null);
  const [showNewBranchDialog, setShowNewBranchDialog] = useState(false);
  const [showNewTagDialog, setShowNewTagDialog] = useState(false);
  const [newBranchName, setNewBranchName] = useState('');
  const [newTagName, setNewTagName] = useState('');
  const [activeView, setActiveView] = useState<'timeline' | 'diff'>('timeline');

  // Compute diff when comparing versions
  const diff = useMemo<VersionDiff | null>(() => {
    if (!selectedVersion || !compareVersion) return null;

    // Simplified diff computation - in production, this would call the backend
    const changes: DiffChange[] = [
      {
        type: 'modified',
        entityId: 'entity-1',
        entityType: 'line',
        property: 'color',
        description: 'Line color changed',
      },
      {
        type: 'added',
        entityId: 'entity-2',
        entityType: 'circle',
        description: 'New circle created',
      },
    ];

    return {
      fromVersion: compareVersion,
      toVersion: selectedVersion,
      changes,
      stats: {
        added: changes.filter(c => c.type === 'added').length,
        modified: changes.filter(c => c.type === 'modified').length,
        deleted: changes.filter(c => c.type === 'deleted').length,
      },
    };
  }, [selectedVersion, compareVersion]);

  const handleVersionSelect = (version: DocumentVersion) => {
    if (activeView === 'diff' && selectedVersion) {
      setCompareVersion(selectedVersion);
    }
    setSelectedVersion(version);
    onVersionSelect?.(version);
  };

  const handleCreateBranch = async (version?: DocumentVersion) => {
    if (!newBranchName.trim()) return;

    await createBranch(newBranchName);
    onCreateBranch?.(newBranchName, version || selectedVersion!);
    setNewBranchName('');
    setShowNewBranchDialog(false);
  };

  const handleCreateTag = async () => {
    if (!newTagName.trim() || !selectedVersion) return;

    await createTag(newTagName, selectedVersion.id);
    setNewTagName('');
    setShowNewTagDialog(false);
    refreshVersions();
  };

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
      {/* Header */}
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
          Version History
        </h3>
        <div style={{ display: 'flex', gap: '8px' }}>
          <button
            onClick={() => setActiveView('timeline')}
            style={{
              padding: '4px 12px',
              fontSize: '12px',
              backgroundColor: activeView === 'timeline' ? '#3b82f6' : '#f8fafc',
              color: activeView === 'timeline' ? '#fff' : '#64748b',
              border: '1px solid #e2e8f0',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            Timeline
          </button>
          <button
            onClick={() => setActiveView('diff')}
            style={{
              padding: '4px 12px',
              fontSize: '12px',
              backgroundColor: activeView === 'diff' ? '#3b82f6' : '#f8fafc',
              color: activeView === 'diff' ? '#fff' : '#64748b',
              border: '1px solid #e2e8f0',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            Diff
          </button>
        </div>
      </div>

      {/* Branch selector */}
      {showBranches && (
        <BranchSelector
          branches={branches}
          currentBranch={currentBranch}
          onSwitchBranch={switchBranch}
          onCreateBranch={() => setShowNewBranchDialog(true)}
        />
      )}

      {/* Content */}
      <div style={{ flex: 1, overflowY: 'auto' }}>
        {activeView === 'timeline' ? (
          <div style={{ padding: '16px' }}>
            {versions.map(version => (
              <VersionTimelineItem
                key={version.id}
                version={version}
                isSelected={selectedVersion?.id === version.id}
                onSelect={() => handleVersionSelect(version)}
                onCreateBranch={() => {
                  setSelectedVersion(version);
                  setShowNewBranchDialog(true);
                }}
                onCreateTag={() => {
                  setSelectedVersion(version);
                  setShowNewTagDialog(true);
                }}
              />
            ))}

            {versions.length === 0 && (
              <div style={{ padding: '32px', textAlign: 'center', color: '#94a3b8', fontSize: '14px' }}>
                No version history available
              </div>
            )}
          </div>
        ) : (
          <DiffViewer diff={diff} />
        )}
      </div>

      {/* New Branch Dialog */}
      {showNewBranchDialog && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0,0,0,0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
          onClick={() => setShowNewBranchDialog(false)}
        >
          <div
            onClick={e => e.stopPropagation()}
            style={{
              backgroundColor: '#fff',
              padding: '24px',
              borderRadius: '8px',
              minWidth: '320px',
            }}
          >
            <h3 style={{ margin: '0 0 16px 0', fontSize: '16px' }}>Create New Branch</h3>
            <input
              type="text"
              value={newBranchName}
              onChange={e => setNewBranchName(e.target.value)}
              placeholder="Branch name..."
              style={{
                width: '100%',
                padding: '8px 12px',
                border: '1px solid #e2e8f0',
                borderRadius: '6px',
                marginBottom: '16px',
              }}
            />
            <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
              <button
                onClick={() => setShowNewBranchDialog(false)}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#f8fafc',
                  border: '1px solid #e2e8f0',
                  borderRadius: '6px',
                  cursor: 'pointer',
                }}
              >
                Cancel
              </button>
              <button
                onClick={() => handleCreateBranch()}
                disabled={!newBranchName.trim()}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#3b82f6',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: newBranchName.trim() ? 'pointer' : 'not-allowed',
                }}
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}

      {/* New Tag Dialog */}
      {showNewTagDialog && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0,0,0,0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
          onClick={() => setShowNewTagDialog(false)}
        >
          <div
            onClick={e => e.stopPropagation()}
            style={{
              backgroundColor: '#fff',
              padding: '24px',
              borderRadius: '8px',
              minWidth: '320px',
            }}
          >
            <h3 style={{ margin: '0 0 16px 0', fontSize: '16px' }}>Create Tag</h3>
            <input
              type="text"
              value={newTagName}
              onChange={e => setNewTagName(e.target.value)}
              placeholder="Tag name (e.g., v1.0.0)..."
              style={{
                width: '100%',
                padding: '8px 12px',
                border: '1px solid #e2e8f0',
                borderRadius: '6px',
                marginBottom: '16px',
              }}
            />
            <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
              <button
                onClick={() => setShowNewTagDialog(false)}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#f8fafc',
                  border: '1px solid #e2e8f0',
                  borderRadius: '6px',
                  cursor: 'pointer',
                }}
              >
                Cancel
              </button>
              <button
                onClick={handleCreateTag}
                disabled={!newTagName.trim()}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#3b82f6',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: newTagName.trim() ? 'pointer' : 'not-allowed',
                }}
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default VersionHistory;
