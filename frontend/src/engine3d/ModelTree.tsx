/**
 * Model Tree Component
 *
 * Displays the feature tree / model browser for navigating and managing 3D features
 */

import React, { useState, useCallback } from 'react';
import { useFeatures, useSelection } from './use3DEngine';
import type { ModelFeature } from './types';

/**
 * Props for ModelTree component
 */
interface ModelTreeProps {
  className?: string;
  onFeatureDoubleClick?: (feature: ModelFeature) => void;
}

/**
 * Props for TreeNode component
 */
interface TreeNodeProps {
  feature: ModelFeature;
  level: number;
  isSelected: boolean;
  onSelect: (id: string, multi: boolean) => void;
  onToggleVisibility: (id: string) => void;
  onToggleLock: (id: string) => void;
  onDelete: (id: string) => void;
  onDoubleClick?: (feature: ModelFeature) => void;
}

/**
 * Individual tree node component
 */
function TreeNode({
  feature,
  level,
  isSelected,
  onSelect,
  onToggleVisibility,
  onToggleLock,
  onDelete,
  onDoubleClick,
}: TreeNodeProps) {
  const [isExpanded, setIsExpanded] = useState(true);
  const hasChildren = feature.children && feature.children.length > 0;

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      onSelect(feature.id, e.ctrlKey || e.metaKey);
    },
    [feature.id, onSelect]
  );

  const handleDoubleClick = useCallback(() => {
    if (onDoubleClick) {
      onDoubleClick(feature);
    }
  }, [feature, onDoubleClick]);

  const handleToggleExpand = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      setIsExpanded(!isExpanded);
    },
    [isExpanded]
  );

  return (
    <div className="tree-node">
      <div
        className={`tree-node-content ${isSelected ? 'selected' : ''} ${feature.locked ? 'locked' : ''}`}
        style={{ paddingLeft: `${level * 20}px` }}
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
      >
        {hasChildren && (
          <button className="expand-button" onClick={handleToggleExpand}>
            {isExpanded ? '▼' : '▶'}
          </button>
        )}

        <span className="feature-icon">{getFeatureIcon(feature.type)}</span>

        <span className="feature-name">{feature.name}</span>

        <div className="feature-actions">
          <button
            className="action-button"
            onClick={(e) => {
              e.stopPropagation();
              onToggleVisibility(feature.id);
            }}
            title={feature.visible ? 'Hide' : 'Show'}
          >
            {feature.visible ? '👁️' : '👁️‍🗨️'}
          </button>

          <button
            className="action-button"
            onClick={(e) => {
              e.stopPropagation();
              onToggleLock(feature.id);
            }}
            title={feature.locked ? 'Unlock' : 'Lock'}
          >
            {feature.locked ? '🔒' : '🔓'}
          </button>

          <button
            className="action-button delete"
            onClick={(e) => {
              e.stopPropagation();
              if (confirm(`Delete feature "${feature.name}"?`)) {
                onDelete(feature.id);
              }
            }}
            title="Delete"
          >
            🗑️
          </button>
        </div>
      </div>

      {hasChildren && isExpanded && (
        <div className="tree-children">
          {feature.children!.map((child) => (
            <TreeNode
              key={child.id}
              feature={child}
              level={level + 1}
              isSelected={isSelected}
              onSelect={onSelect}
              onToggleVisibility={onToggleVisibility}
              onToggleLock={onToggleLock}
              onDelete={onDelete}
              onDoubleClick={onDoubleClick}
            />
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Get icon for feature type
 */
function getFeatureIcon(type: string): string {
  const icons: Record<string, string> = {
    primitive: '📦',
    extrude: '↕️',
    revolve: '🔄',
    sweep: '🌀',
    loft: '🎭',
    shell: '🐚',
    union: '➕',
    intersection: '✖️',
    difference: '➖',
  };
  return icons[type] || '📐';
}

/**
 * Main ModelTree component
 */
export function ModelTree({ className = '', onFeatureDoubleClick }: ModelTreeProps) {
  const { features, updateFeature, deleteFeature } = useFeatures();
  const { selectedFeatureIds, selectFeatures, toggleSelection } = useSelection();

  const handleSelect = useCallback(
    (id: string, multi: boolean) => {
      if (multi) {
        toggleSelection(id);
      } else {
        selectFeatures([id]);
      }
    },
    [selectFeatures, toggleSelection]
  );

  const handleToggleVisibility = useCallback(
    (id: string) => {
      const feature = features.find((f) => f.id === id);
      if (feature) {
        updateFeature(id, { visible: !feature.visible });
      }
    },
    [features, updateFeature]
  );

  const handleToggleLock = useCallback(
    (id: string) => {
      const feature = features.find((f) => f.id === id);
      if (feature) {
        updateFeature(id, { locked: !feature.locked });
      }
    },
    [features, updateFeature]
  );

  return (
    <div className={`model-tree ${className}`}>
      <div className="model-tree-header">
        <h3>Model Tree</h3>
        <div className="header-actions">
          <button className="icon-button" title="Collapse All">
            ⊟
          </button>
          <button className="icon-button" title="Expand All">
            ⊞
          </button>
        </div>
      </div>

      <div className="model-tree-content">
        {features.length === 0 ? (
          <div className="empty-state">No features yet. Create your first feature!</div>
        ) : (
          features.map((feature) => (
            <TreeNode
              key={feature.id}
              feature={feature}
              level={0}
              isSelected={selectedFeatureIds.includes(feature.id)}
              onSelect={handleSelect}
              onToggleVisibility={handleToggleVisibility}
              onToggleLock={handleToggleLock}
              onDelete={deleteFeature}
              onDoubleClick={onFeatureDoubleClick}
            />
          ))
        )}
      </div>

      <style jsx>{`
        .model-tree {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: #1e1e1e;
          color: #cccccc;
        }

        .model-tree-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 12px;
          border-bottom: 1px solid #333;
        }

        .model-tree-header h3 {
          margin: 0;
          font-size: 14px;
          font-weight: 600;
        }

        .header-actions {
          display: flex;
          gap: 4px;
        }

        .icon-button {
          background: transparent;
          border: none;
          color: #cccccc;
          cursor: pointer;
          padding: 4px 8px;
          border-radius: 4px;
        }

        .icon-button:hover {
          background: #333;
        }

        .model-tree-content {
          flex: 1;
          overflow-y: auto;
          padding: 8px 0;
        }

        .tree-node {
          user-select: none;
        }

        .tree-node-content {
          display: flex;
          align-items: center;
          padding: 6px 12px;
          cursor: pointer;
          transition: background 0.15s;
        }

        .tree-node-content:hover {
          background: #2a2a2a;
        }

        .tree-node-content.selected {
          background: #094771;
        }

        .tree-node-content.locked {
          opacity: 0.6;
        }

        .expand-button {
          background: none;
          border: none;
          color: #cccccc;
          cursor: pointer;
          padding: 0 4px;
          margin-right: 4px;
          font-size: 12px;
        }

        .feature-icon {
          margin-right: 8px;
          font-size: 16px;
        }

        .feature-name {
          flex: 1;
          font-size: 13px;
        }

        .feature-actions {
          display: flex;
          gap: 4px;
          opacity: 0;
          transition: opacity 0.15s;
        }

        .tree-node-content:hover .feature-actions {
          opacity: 1;
        }

        .action-button {
          background: transparent;
          border: none;
          cursor: pointer;
          padding: 2px 4px;
          font-size: 14px;
          opacity: 0.7;
        }

        .action-button:hover {
          opacity: 1;
        }

        .action-button.delete:hover {
          color: #f44336;
        }

        .empty-state {
          padding: 24px;
          text-align: center;
          color: #666;
          font-size: 13px;
        }
      `}</style>
    </div>
  );
}
