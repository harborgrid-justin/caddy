/**
 * Constraint Manager Component
 *
 * Manages and visualizes geometric constraints in the 3D model
 */

import React, { useState, useCallback } from 'react';
import { useConstraints } from './use3DEngine';
import type { Constraint, ConstraintType } from './types';

/**
 * Props for ConstraintManager component
 */
interface ConstraintManagerProps {
  className?: string;
}

/**
 * Props for ConstraintItem component
 */
interface ConstraintItemProps {
  constraint: Constraint;
  onToggleEnabled: (id: string) => void;
  onDelete: (id: string) => void;
}

/**
 * Individual constraint item component
 */
function ConstraintItem({ constraint, onToggleEnabled, onDelete }: ConstraintItemProps) {
  const getConstraintIcon = (type: ConstraintType): string => {
    const icons: Record<ConstraintType, string> = {
      [ConstraintType.FixedPoint]: '📍',
      [ConstraintType.Distance]: '📏',
      [ConstraintType.Angle]: '📐',
      [ConstraintType.Parallel]: '⫴',
      [ConstraintType.Perpendicular]: '⊥',
      [ConstraintType.Coincident]: '⚬',
      [ConstraintType.Horizontal]: '⟷',
      [ConstraintType.Vertical]: '⟼',
      [ConstraintType.PointOnLine]: '⋯',
      [ConstraintType.Tangent]: '⤳',
    };
    return icons[type] || '🔗';
  };

  const getConstraintLabel = (type: ConstraintType): string => {
    return type.charAt(0).toUpperCase() + type.slice(1).replace(/([A-Z])/g, ' $1');
  };

  const getSatisfiedStatus = (constraint: Constraint) => {
    if (!constraint.enabled) return 'disabled';
    if (constraint.satisfied) return 'satisfied';
    if (constraint.error < 0.01) return 'nearly-satisfied';
    return 'unsatisfied';
  };

  const status = getSatisfiedStatus(constraint);

  return (
    <div className={`constraint-item ${status}`}>
      <div className="constraint-header">
        <span className="constraint-icon">{getConstraintIcon(constraint.type)}</span>
        <span className="constraint-label">{getConstraintLabel(constraint.type)}</span>

        <div className="constraint-actions">
          <button
            className="action-button"
            onClick={() => onToggleEnabled(constraint.id)}
            title={constraint.enabled ? 'Disable' : 'Enable'}
          >
            {constraint.enabled ? '✓' : '○'}
          </button>
          <button
            className="action-button delete"
            onClick={() => onDelete(constraint.id)}
            title="Delete constraint"
          >
            ×
          </button>
        </div>
      </div>

      <div className="constraint-details">
        {constraint.value !== undefined && (
          <div className="detail-row">
            <span className="detail-label">Value:</span>
            <span className="detail-value">{constraint.value.toFixed(3)}</span>
          </div>
        )}

        {constraint.enabled && (
          <div className="detail-row">
            <span className="detail-label">Error:</span>
            <span className={`detail-value error-${status}`}>{constraint.error.toFixed(6)}</span>
          </div>
        )}

        <div className="detail-row">
          <span className="detail-label">Entities:</span>
          <span className="detail-value">{constraint.entityIds.length}</span>
        </div>
      </div>

      <style jsx>{`
        .constraint-item {
          background: #2a2a2a;
          border-left: 3px solid #666;
          border-radius: 4px;
          padding: 12px;
          margin-bottom: 8px;
          transition: all 0.15s;
        }

        .constraint-item:hover {
          background: #333;
        }

        .constraint-item.satisfied {
          border-left-color: #4caf50;
        }

        .constraint-item.nearly-satisfied {
          border-left-color: #ff9800;
        }

        .constraint-item.unsatisfied {
          border-left-color: #f44336;
        }

        .constraint-item.disabled {
          opacity: 0.5;
          border-left-color: #666;
        }

        .constraint-header {
          display: flex;
          align-items: center;
          margin-bottom: 8px;
        }

        .constraint-icon {
          font-size: 18px;
          margin-right: 8px;
        }

        .constraint-label {
          flex: 1;
          font-size: 13px;
          font-weight: 500;
        }

        .constraint-actions {
          display: flex;
          gap: 4px;
        }

        .action-button {
          background: transparent;
          border: none;
          color: #cccccc;
          cursor: pointer;
          padding: 2px 6px;
          font-size: 16px;
          opacity: 0.7;
        }

        .action-button:hover {
          opacity: 1;
        }

        .action-button.delete:hover {
          color: #f44336;
        }

        .constraint-details {
          font-size: 12px;
          color: #999;
        }

        .detail-row {
          display: flex;
          justify-content: space-between;
          margin-bottom: 4px;
        }

        .detail-label {
          font-weight: 500;
        }

        .error-satisfied {
          color: #4caf50;
        }

        .error-nearly-satisfied {
          color: #ff9800;
        }

        .error-unsatisfied {
          color: #f44336;
        }
      `}</style>
    </div>
  );
}

/**
 * Main ConstraintManager component
 */
export function ConstraintManager({ className = '' }: ConstraintManagerProps) {
  const { constraints, unsatisfiedConstraints, updateConstraint, deleteConstraint, solveConstraints } =
    useConstraints();

  const [showOnlyUnsatisfied, setShowOnlyUnsatisfied] = useState(false);

  const handleToggleEnabled = useCallback(
    (id: string) => {
      const constraint = constraints.find((c) => c.id === id);
      if (constraint) {
        updateConstraint(id, { enabled: !constraint.enabled });
      }
    },
    [constraints, updateConstraint]
  );

  const handleDelete = useCallback(
    (id: string) => {
      if (confirm('Delete this constraint?')) {
        deleteConstraint(id);
      }
    },
    [deleteConstraint]
  );

  const handleSolve = useCallback(async () => {
    await solveConstraints();
  }, [solveConstraints]);

  const displayedConstraints = showOnlyUnsatisfied ? unsatisfiedConstraints : constraints;
  const enabledCount = constraints.filter((c) => c.enabled).length;
  const satisfiedCount = constraints.filter((c) => c.enabled && c.satisfied).length;

  return (
    <div className={`constraint-manager ${className}`}>
      <div className="panel-header">
        <h3>Constraints</h3>
        <div className="header-stats">
          <span className="stat">
            {satisfiedCount}/{enabledCount} satisfied
          </span>
        </div>
      </div>

      <div className="panel-toolbar">
        <button className="toolbar-button primary" onClick={handleSolve} disabled={constraints.length === 0}>
          Solve Constraints
        </button>

        <label className="toolbar-checkbox">
          <input
            type="checkbox"
            checked={showOnlyUnsatisfied}
            onChange={(e) => setShowOnlyUnsatisfied(e.target.checked)}
          />
          Show Unsatisfied Only
        </label>
      </div>

      <div className="panel-content">
        {displayedConstraints.length === 0 ? (
          <div className="empty-state">
            {showOnlyUnsatisfied
              ? 'All constraints are satisfied!'
              : 'No constraints yet. Add constraints to control geometry.'}
          </div>
        ) : (
          <div className="constraints-list">
            {displayedConstraints.map((constraint) => (
              <ConstraintItem
                key={constraint.id}
                constraint={constraint}
                onToggleEnabled={handleToggleEnabled}
                onDelete={handleDelete}
              />
            ))}
          </div>
        )}
      </div>

      <div className="panel-footer">
        <button className="footer-button">+ Add Constraint</button>
      </div>

      <style jsx>{`
        .constraint-manager {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: #1e1e1e;
          color: #cccccc;
        }

        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          border-bottom: 1px solid #333;
        }

        .panel-header h3 {
          margin: 0;
          font-size: 14px;
          font-weight: 600;
        }

        .header-stats {
          font-size: 12px;
          color: #999;
        }

        .panel-toolbar {
          display: flex;
          flex-direction: column;
          gap: 8px;
          padding: 12px 16px;
          border-bottom: 1px solid #333;
        }

        .toolbar-button {
          background: #4fc3f7;
          color: #000;
          border: none;
          padding: 8px 16px;
          font-size: 13px;
          font-weight: 600;
          border-radius: 4px;
          cursor: pointer;
          transition: background 0.15s;
        }

        .toolbar-button:hover:not(:disabled) {
          background: #29b6f6;
        }

        .toolbar-button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .toolbar-checkbox {
          display: flex;
          align-items: center;
          font-size: 12px;
          cursor: pointer;
        }

        .toolbar-checkbox input {
          margin-right: 8px;
        }

        .panel-content {
          flex: 1;
          overflow-y: auto;
          padding: 12px 16px;
        }

        .empty-state {
          padding: 32px 16px;
          text-align: center;
          color: #666;
          font-size: 13px;
        }

        .constraints-list {
          /* Styles inherited from children */
        }

        .panel-footer {
          padding: 12px 16px;
          border-top: 1px solid #333;
        }

        .footer-button {
          width: 100%;
          background: transparent;
          color: #4fc3f7;
          border: 1px solid #4fc3f7;
          padding: 8px 16px;
          font-size: 13px;
          font-weight: 600;
          border-radius: 4px;
          cursor: pointer;
          transition: all 0.15s;
        }

        .footer-button:hover {
          background: #4fc3f7;
          color: #000;
        }
      `}</style>
    </div>
  );
}
