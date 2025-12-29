/**
 * Conflict Dialog - Conflict Resolution UI
 *
 * Provides an interactive interface for viewing and resolving conflicts
 * that occur during collaborative editing with visual diff and resolution options.
 */

import React, { useState } from 'react';
import { useConflicts } from './useCollaboration';
import type { Conflict } from './useCollaboration';

/**
 * Conflict Dialog Props
 */
export interface ConflictDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onResolve?: (conflictId: string, strategy: string) => void;
  className?: string;
  style?: React.CSSProperties;
}

/**
 * Resolution Strategy Option
 */
interface ResolutionOption {
  value: string;
  label: string;
  description: string;
  icon: string;
  recommended?: boolean;
}

/**
 * Get available resolution strategies for a conflict
 */
function getResolutionStrategies(conflict: Conflict): ResolutionOption[] {
  const baseStrategies: ResolutionOption[] = [
    {
      value: 'last-write-wins',
      label: 'Last Write Wins',
      description: 'Keep the most recent change',
      icon: '⏰',
      recommended: conflict.severity === 'low',
    },
    {
      value: 'first-write-wins',
      label: 'First Write Wins',
      description: 'Keep the earliest change',
      icon: '🥇',
    },
    {
      value: 'manual',
      label: 'Manual Resolution',
      description: 'Review and choose changes manually',
      icon: '✋',
      recommended: conflict.severity === 'high',
    },
  ];

  // Add type-specific strategies
  if (conflict.type === 'transform') {
    baseStrategies.push({
      value: 'merge',
      label: 'Merge Transformations',
      description: 'Combine both transformations',
      icon: '🔀',
      recommended: true,
    });
  }

  if (conflict.type === 'property') {
    baseStrategies.push({
      value: 'user-priority',
      label: 'User Priority',
      description: 'Prefer changes from higher-priority user',
      icon: '👑',
    });
  }

  return baseStrategies;
}

/**
 * Conflict Severity Badge
 */
function SeverityBadge({ severity }: { severity: Conflict['severity'] }) {
  const config = {
    low: { color: '#22c55e', bg: '#dcfce7', label: 'Low' },
    medium: { color: '#eab308', bg: '#fef3c7', label: 'Medium' },
    high: { color: '#ef4444', bg: '#fee2e2', label: 'High' },
  };

  const { color, bg, label } = config[severity];

  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '4px',
        padding: '4px 8px',
        backgroundColor: bg,
        color,
        borderRadius: '4px',
        fontSize: '12px',
        fontWeight: '600',
      }}
    >
      <span style={{ fontSize: '10px' }}>⚠️</span>
      {label}
    </span>
  );
}

/**
 * Conflict Type Icon
 */
function ConflictTypeIcon({ type }: { type: Conflict['type'] }) {
  const icons = {
    property: '🎨',
    'delete-modify': '⚔️',
    layer: '📚',
    transform: '↔️',
    constraint: '🔗',
    structural: '🏗️',
  };

  return <span style={{ fontSize: '24px' }}>{icons[type]}</span>;
}

/**
 * Individual Conflict Card
 */
function ConflictCard({
  conflict,
  onResolve,
  isExpanded,
  onToggleExpand,
}: {
  conflict: Conflict;
  onResolve: (strategy: string) => void;
  isExpanded: boolean;
  onToggleExpand: () => void;
}) {
  const [selectedStrategy, setSelectedStrategy] = useState<string>('');
  const strategies = getResolutionStrategies(conflict);

  const handleResolve = () => {
    if (selectedStrategy) {
      onResolve(selectedStrategy);
    }
  };

  return (
    <div
      style={{
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        overflow: 'hidden',
        marginBottom: '12px',
      }}
    >
      {/* Header */}
      <div
        onClick={onToggleExpand}
        style={{
          padding: '16px',
          backgroundColor: '#f8fafc',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'flex-start',
          gap: '12px',
        }}
      >
        <ConflictTypeIcon type={conflict.type} />
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '6px' }}>
            <h4 style={{ margin: 0, fontSize: '14px', fontWeight: '600', color: '#1e293b' }}>
              {conflict.description}
            </h4>
            <SeverityBadge severity={conflict.severity} />
          </div>
          <div style={{ fontSize: '12px', color: '#64748b' }}>
            {conflict.entityIds.length} {conflict.entityIds.length === 1 ? 'entity' : 'entities'} affected ·{' '}
            {conflict.operations.length} conflicting operations
          </div>
        </div>
        <div style={{ fontSize: '12px', color: '#94a3b8' }}>
          {isExpanded ? '▲' : '▼'}
        </div>
      </div>

      {/* Expanded Content */}
      {isExpanded && (
        <div style={{ padding: '16px', borderTop: '1px solid #e2e8f0' }}>
          {/* Operations */}
          <div style={{ marginBottom: '16px' }}>
            <h5 style={{ fontSize: '13px', fontWeight: '600', color: '#475569', marginBottom: '8px' }}>
              Conflicting Operations
            </h5>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {conflict.operations.map((op, index) => (
                <div
                  key={index}
                  style={{
                    padding: '12px',
                    backgroundColor: '#f8fafc',
                    borderRadius: '6px',
                    fontSize: '13px',
                  }}
                >
                  <div style={{ fontWeight: '500', color: '#1e293b', marginBottom: '4px' }}>
                    Operation {index + 1}
                  </div>
                  <div style={{ fontSize: '12px', color: '#64748b' }}>
                    User: {/* User name would come from operation data */}
                    <code style={{ marginLeft: '8px', backgroundColor: '#fff', padding: '2px 6px', borderRadius: '3px' }}>
                      {JSON.stringify(op).slice(0, 100)}...
                    </code>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Resolution Strategies */}
          <div style={{ marginBottom: '16px' }}>
            <h5 style={{ fontSize: '13px', fontWeight: '600', color: '#475569', marginBottom: '8px' }}>
              Choose Resolution Strategy
            </h5>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {strategies.map(strategy => (
                <label
                  key={strategy.value}
                  style={{
                    display: 'flex',
                    alignItems: 'flex-start',
                    gap: '12px',
                    padding: '12px',
                    border: `2px solid ${
                      selectedStrategy === strategy.value ? '#3b82f6' : '#e2e8f0'
                    }`,
                    borderRadius: '6px',
                    cursor: 'pointer',
                    backgroundColor:
                      selectedStrategy === strategy.value ? '#eff6ff' : '#fff',
                    transition: 'all 0.2s',
                  }}
                >
                  <input
                    type="radio"
                    name={`strategy-${conflict.id}`}
                    value={strategy.value}
                    checked={selectedStrategy === strategy.value}
                    onChange={e => setSelectedStrategy(e.target.value)}
                    style={{ marginTop: '2px' }}
                  />
                  <div style={{ flex: 1 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '6px', marginBottom: '4px' }}>
                      <span style={{ fontSize: '16px' }}>{strategy.icon}</span>
                      <span style={{ fontSize: '13px', fontWeight: '600', color: '#1e293b' }}>
                        {strategy.label}
                      </span>
                      {strategy.recommended && (
                        <span
                          style={{
                            fontSize: '10px',
                            padding: '2px 6px',
                            backgroundColor: '#dbeafe',
                            color: '#3b82f6',
                            borderRadius: '3px',
                            fontWeight: '600',
                          }}
                        >
                          RECOMMENDED
                        </span>
                      )}
                    </div>
                    <div style={{ fontSize: '12px', color: '#64748b' }}>
                      {strategy.description}
                    </div>
                  </div>
                </label>
              ))}
            </div>
          </div>

          {/* Actions */}
          <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
            <button
              onClick={handleResolve}
              disabled={!selectedStrategy}
              style={{
                padding: '8px 16px',
                backgroundColor: selectedStrategy ? '#3b82f6' : '#e2e8f0',
                color: selectedStrategy ? '#fff' : '#94a3b8',
                border: 'none',
                borderRadius: '6px',
                fontSize: '13px',
                fontWeight: '500',
                cursor: selectedStrategy ? 'pointer' : 'not-allowed',
              }}
            >
              Resolve Conflict
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

/**
 * Main Conflict Dialog Component
 */
export function ConflictDialog({
  isOpen,
  onClose,
  onResolve,
  className = '',
  style = {},
}: ConflictDialogProps) {
  const { conflicts, pendingConflicts, autoResolvableConflicts, resolve } = useConflicts();
  const [expandedConflictId, setExpandedConflictId] = useState<string | null>(null);
  const [resolving, setResolving] = useState(false);

  if (!isOpen || conflicts.length === 0) return null;

  const handleResolveConflict = async (conflictId: string, strategy: string) => {
    setResolving(true);
    try {
      await resolve(conflictId, strategy as any);
      onResolve?.(conflictId, strategy);
    } catch (error) {
      console.error('Failed to resolve conflict:', error);
    } finally {
      setResolving(false);
    }
  };

  const handleAutoResolveAll = async () => {
    setResolving(true);
    try {
      for (const conflict of autoResolvableConflicts) {
        await resolve(conflict.id, 'last-write-wins');
      }
    } catch (error) {
      console.error('Failed to auto-resolve conflicts:', error);
    } finally {
      setResolving(false);
    }
  };

  return (
    <div
      className={className}
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.6)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 10000,
        padding: '20px',
        ...style,
      }}
      onClick={onClose}
    >
      <div
        onClick={e => e.stopPropagation()}
        style={{
          backgroundColor: '#fff',
          borderRadius: '12px',
          boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
          maxWidth: '700px',
          width: '100%',
          maxHeight: '80vh',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        {/* Header */}
        <div
          style={{
            padding: '20px 24px',
            borderBottom: '1px solid #e2e8f0',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <div>
            <h2 style={{ margin: '0 0 4px 0', fontSize: '18px', fontWeight: '600', color: '#1e293b' }}>
              Resolve Conflicts
            </h2>
            <p style={{ margin: 0, fontSize: '13px', color: '#64748b' }}>
              {pendingConflicts.length} pending · {autoResolvableConflicts.length} auto-resolvable
            </p>
          </div>
          <button
            onClick={onClose}
            style={{
              padding: '8px',
              backgroundColor: 'transparent',
              border: 'none',
              fontSize: '20px',
              cursor: 'pointer',
              color: '#94a3b8',
              lineHeight: 1,
            }}
          >
            ✕
          </button>
        </div>

        {/* Auto-resolve banner */}
        {autoResolvableConflicts.length > 0 && (
          <div
            style={{
              padding: '12px 24px',
              backgroundColor: '#eff6ff',
              borderBottom: '1px solid #bfdbfe',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <div style={{ fontSize: '13px', color: '#1e40af' }}>
              <strong>{autoResolvableConflicts.length}</strong> conflicts can be resolved automatically
            </div>
            <button
              onClick={handleAutoResolveAll}
              disabled={resolving}
              style={{
                padding: '6px 12px',
                backgroundColor: '#3b82f6',
                color: '#fff',
                border: 'none',
                borderRadius: '6px',
                fontSize: '12px',
                fontWeight: '500',
                cursor: resolving ? 'not-allowed' : 'pointer',
                opacity: resolving ? 0.6 : 1,
              }}
            >
              {resolving ? 'Resolving...' : 'Auto-Resolve All'}
            </button>
          </div>
        )}

        {/* Conflict List */}
        <div style={{ flex: 1, overflowY: 'auto', padding: '16px 24px' }}>
          {conflicts.length === 0 ? (
            <div
              style={{
                padding: '48px 24px',
                textAlign: 'center',
                color: '#94a3b8',
              }}
            >
              <div style={{ fontSize: '48px', marginBottom: '16px' }}>✅</div>
              <div style={{ fontSize: '16px', fontWeight: '500', marginBottom: '8px' }}>
                No Conflicts
              </div>
              <div style={{ fontSize: '13px' }}>All changes are synchronized</div>
            </div>
          ) : (
            conflicts.map(conflict => (
              <ConflictCard
                key={conflict.id}
                conflict={conflict}
                onResolve={strategy => handleResolveConflict(conflict.id, strategy)}
                isExpanded={expandedConflictId === conflict.id}
                onToggleExpand={() =>
                  setExpandedConflictId(expandedConflictId === conflict.id ? null : conflict.id)
                }
              />
            ))
          )}
        </div>

        {/* Footer */}
        <div
          style={{
            padding: '16px 24px',
            borderTop: '1px solid #e2e8f0',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <div style={{ fontSize: '12px', color: '#64748b' }}>
            Tip: Most conflicts can be automatically resolved using smart strategies
          </div>
          <button
            onClick={onClose}
            style={{
              padding: '8px 16px',
              backgroundColor: '#f8fafc',
              border: '1px solid #e2e8f0',
              borderRadius: '6px',
              fontSize: '13px',
              fontWeight: '500',
              cursor: 'pointer',
              color: '#475569',
            }}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * Conflict Notification Badge
 * Small badge to show conflict count
 */
export function ConflictBadge({ onClick }: { onClick?: () => void }) {
  const { conflicts, hasConflicts } = useConflicts();

  if (!hasConflicts) return null;

  return (
    <button
      onClick={onClick}
      style={{
        position: 'fixed',
        bottom: '20px',
        right: '20px',
        padding: '12px 16px',
        backgroundColor: '#ef4444',
        color: '#fff',
        border: 'none',
        borderRadius: '8px',
        fontSize: '14px',
        fontWeight: '600',
        cursor: 'pointer',
        boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        zIndex: 9999,
      }}
    >
      <span style={{ fontSize: '18px' }}>⚠️</span>
      <span>{conflicts.length} Conflict{conflicts.length !== 1 ? 's' : ''}</span>
    </button>
  );
}

export default ConflictDialog;
