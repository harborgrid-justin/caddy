/**
 * Enterprise Tree Component
 * Features: Virtualization for large hierarchies, lazy loading, selection, keyboard navigation
 */

import React, { useState, useCallback, useMemo, CSSProperties, ReactNode } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets } from './styles/animations';

export interface TreeNode<T = any> {
  id: string;
  label: ReactNode;
  children?: TreeNode<T>[];
  icon?: ReactNode;
  data?: T;
  disabled?: boolean;
  isLeaf?: boolean;
}

export interface TreeProps<T = any> {
  /** Tree data */
  data: TreeNode<T>[];
  /** Selected node IDs */
  selectedIds?: string[];
  /** Expanded node IDs */
  expandedIds?: string[];
  /** Selection change handler */
  onSelect?: (nodeId: string, node: TreeNode<T>) => void;
  /** Expansion change handler */
  onExpand?: (nodeId: string, expanded: boolean) => void;
  /** Multiple selection */
  multiSelect?: boolean;
  /** Show checkbox for selection */
  showCheckbox?: boolean;
  /** Enable virtualization */
  virtualized?: boolean;
  /** Height for virtualized tree */
  height?: number;
  /** Row height for virtualization */
  rowHeight?: number;
  /** Custom render for node */
  renderNode?: (node: TreeNode<T>, depth: number) => ReactNode;
  /** Lazy load children */
  onLoadChildren?: (node: TreeNode<T>) => Promise<TreeNode<T>[]>;
}

interface FlatNode<T = any> extends TreeNode<T> {
  depth: number;
  parentId?: string;
}

export const Tree = <T,>({
  data,
  selectedIds = [],
  expandedIds: controlledExpandedIds,
  onSelect,
  onExpand,
  multiSelect = false,
  showCheckbox = false,
  virtualized = false,
  height = 400,
  rowHeight = 32,
  renderNode,
  onLoadChildren,
}: TreeProps<T>) => {
  const { theme } = useTheme();
  const [internalExpandedIds, setInternalExpandedIds] = useState<Set<string>>(new Set());
  const [loadingIds, setLoadingIds] = useState<Set<string>>(new Set());

  const expandedIds = controlledExpandedIds
    ? new Set(controlledExpandedIds)
    : internalExpandedIds;

  // Flatten tree for rendering
  const flattenTree = useCallback(
    (nodes: TreeNode<T>[], depth = 0, parentId?: string): FlatNode<T>[] => {
      const result: FlatNode<T>[] = [];

      for (const node of nodes) {
        result.push({ ...node, depth, parentId });

        if (node.children && expandedIds.has(node.id)) {
          result.push(...flattenTree(node.children, depth + 1, node.id));
        }
      }

      return result;
    },
    [expandedIds]
  );

  const flatNodes = useMemo(() => flattenTree(data), [data, flattenTree]);

  const handleToggle = async (node: FlatNode<T>) => {
    const isExpanded = expandedIds.has(node.id);

    // Lazy load children if needed
    if (!isExpanded && !node.children && onLoadChildren && !node.isLeaf) {
      setLoadingIds((prev) => new Set(prev).add(node.id));
      try {
        const children = await onLoadChildren(node);
        node.children = children;
      } catch (error) {
        console.error('Failed to load children:', error);
      } finally {
        setLoadingIds((prev) => {
          const next = new Set(prev);
          next.delete(node.id);
          return next;
        });
      }
    }

    if (controlledExpandedIds) {
      onExpand?.(node.id, !isExpanded);
    } else {
      setInternalExpandedIds((prev) => {
        const next = new Set(prev);
        if (isExpanded) {
          next.delete(node.id);
        } else {
          next.add(node.id);
        }
        return next;
      });
    }
  };

  const handleSelect = (node: FlatNode<T>) => {
    if (node.disabled) return;
    onSelect?.(node.id, node);
  };

  const handleKeyDown = (e: React.KeyboardEvent, node: FlatNode<T>, index: number) => {
    switch (e.key) {
      case 'Enter':
      case ' ':
        e.preventDefault();
        handleSelect(node);
        break;
      case 'ArrowRight':
        e.preventDefault();
        if (!expandedIds.has(node.id) && (node.children || !node.isLeaf)) {
          handleToggle(node);
        }
        break;
      case 'ArrowLeft':
        e.preventDefault();
        if (expandedIds.has(node.id)) {
          handleToggle(node);
        }
        break;
      case 'ArrowDown':
      case 'ArrowUp':
        e.preventDefault();
        // Focus next/previous node (simplified - full implementation would handle focus management)
        break;
    }
  };

  const nodeStyles = (node: FlatNode<T>, isSelected: boolean): CSSProperties => ({
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing[2],
    padding: `${theme.spacing[1]} ${theme.spacing[2]}`,
    paddingLeft: `calc(${theme.spacing[2]} + ${node.depth * 20}px)`,
    cursor: node.disabled ? 'not-allowed' : 'pointer',
    backgroundColor: isSelected ? theme.colors.interactive.secondary : 'transparent',
    color: node.disabled ? theme.colors.text.disabled : theme.colors.text.primary,
    height: `${rowHeight}px`,
    transition: transitionPresets.colors,
    userSelect: 'none',
  });

  const ChevronIcon = ({ isExpanded, isLoading }: { isExpanded: boolean; isLoading: boolean }) => {
    if (isLoading) {
      return (
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
          style={{ animation: 'spin 1s linear infinite' }}
        >
          <circle
            cx="8"
            cy="8"
            r="6"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeDasharray="30"
          />
        </svg>
      );
    }

    return (
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="currentColor"
        style={{
          transform: isExpanded ? 'rotate(90deg)' : 'rotate(0deg)',
          transition: transitionPresets.transform,
        }}
      >
        <path d="M6 4l4 4-4 4" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" />
      </svg>
    );
  };

  const renderTreeNode = (node: FlatNode<T>, index: number) => {
    const isSelected = selectedIds.includes(node.id);
    const isExpanded = expandedIds.has(node.id);
    const isLoading = loadingIds.has(node.id);
    const hasChildren = (node.children && node.children.length > 0) || (!node.isLeaf && onLoadChildren);

    return (
      <div
        key={node.id}
        role="treeitem"
        aria-expanded={hasChildren ? isExpanded : undefined}
        aria-selected={isSelected}
        aria-disabled={node.disabled}
        tabIndex={0}
        style={nodeStyles(node, isSelected)}
        onClick={() => handleSelect(node)}
        onKeyDown={(e) => handleKeyDown(e, node, index)}
        onMouseEnter={(e) => {
          if (!node.disabled) {
            e.currentTarget.style.backgroundColor = theme.colors.background.secondary;
          }
        }}
        onMouseLeave={(e) => {
          if (!isSelected) {
            e.currentTarget.style.backgroundColor = 'transparent';
          }
        }}
      >
        {hasChildren ? (
          <div
            onClick={(e) => {
              e.stopPropagation();
              handleToggle(node);
            }}
            style={{ display: 'flex', cursor: 'pointer' }}
          >
            <ChevronIcon isExpanded={isExpanded} isLoading={isLoading} />
          </div>
        ) : (
          <div style={{ width: '16px' }} />
        )}

        {showCheckbox && (
          <input
            type="checkbox"
            checked={isSelected}
            onChange={() => handleSelect(node)}
            disabled={node.disabled}
            onClick={(e) => e.stopPropagation()}
            style={{ cursor: node.disabled ? 'not-allowed' : 'pointer' }}
          />
        )}

        {node.icon && <span style={{ display: 'flex' }}>{node.icon}</span>}

        {renderNode ? (
          renderNode(node, node.depth)
        ) : (
          <span style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            {node.label}
          </span>
        )}
      </div>
    );
  };

  const containerStyles: CSSProperties = {
    backgroundColor: theme.colors.background.primary,
    border: `1px solid ${theme.colors.border.primary}`,
    borderRadius: theme.borderRadius.md,
    overflow: virtualized ? 'hidden' : 'auto',
    height: virtualized ? `${height}px` : 'auto',
  };

  if (virtualized) {
    // Simple virtualization - for production, use react-window or react-virtualized
    const [scrollTop, setScrollTop] = useState(0);
    const visibleStart = Math.floor(scrollTop / rowHeight);
    const visibleEnd = Math.ceil((scrollTop + height) / rowHeight);
    const visibleNodes = flatNodes.slice(visibleStart, visibleEnd);
    const totalHeight = flatNodes.length * rowHeight;

    return (
      <div
        role="tree"
        aria-multiselectable={multiSelect}
        style={containerStyles}
        onScroll={(e) => setScrollTop(e.currentTarget.scrollTop)}
      >
        <div style={{ height: `${totalHeight}px`, position: 'relative' }}>
          <div style={{ transform: `translateY(${visibleStart * rowHeight}px)` }}>
            {visibleNodes.map((node, index) => renderTreeNode(node, visibleStart + index))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div role="tree" aria-multiselectable={multiSelect} style={containerStyles}>
      {flatNodes.map((node, index) => renderTreeNode(node, index))}
    </div>
  );
};

Tree.displayName = 'Tree';
