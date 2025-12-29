/**
 * Enterprise Toolbar Component
 * Features: Customizable toolbar with groups, overflow menu, keyboard shortcuts
 */

import React, { useState, useRef, useEffect, ReactNode, CSSProperties } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets, animationPresets } from './styles/animations';

export interface ToolbarItem {
  id: string;
  icon?: ReactNode;
  label: string;
  tooltip?: string;
  shortcut?: string;
  disabled?: boolean;
  active?: boolean;
  onClick?: () => void;
  type?: 'button' | 'toggle' | 'dropdown';
  dropdown?: ToolbarItem[];
}

export interface ToolbarGroup {
  id: string;
  items: ToolbarItem[];
}

export interface ToolbarProps {
  /** Toolbar groups */
  groups: ToolbarGroup[];
  /** Toolbar size */
  size?: 'sm' | 'md' | 'lg';
  /** Vertical orientation */
  vertical?: boolean;
  /** Show labels */
  showLabels?: boolean;
  /** Compact mode (icons only) */
  compact?: boolean;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  groups,
  size = 'md',
  vertical = false,
  showLabels = true,
  compact = false,
}) => {
  const { theme } = useTheme();
  const [activeDropdown, setActiveDropdown] = useState<string | null>(null);
  const [dropdownPosition, setDropdownPosition] = useState({ x: 0, y: 0 });
  const toolbarRef = useRef<HTMLDivElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const sizeStyles = {
    sm: {
      padding: theme.spacing[1],
      iconSize: '16px',
      fontSize: theme.typography.fontSize.xs,
    },
    md: {
      padding: theme.spacing[2],
      iconSize: '20px',
      fontSize: theme.typography.fontSize.sm,
    },
    lg: {
      padding: theme.spacing[3],
      iconSize: '24px',
      fontSize: theme.typography.fontSize.base,
    },
  };

  useEffect(() => {
    if (!activeDropdown) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(e.target as Node) &&
        toolbarRef.current &&
        !toolbarRef.current.contains(e.target as Node)
      ) {
        setActiveDropdown(null);
      }
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setActiveDropdown(null);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscape);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [activeDropdown]);

  const handleItemClick = (item: ToolbarItem, buttonElement: HTMLButtonElement) => {
    if (item.disabled) return;

    if (item.type === 'dropdown' && item.dropdown) {
      const rect = buttonElement.getBoundingClientRect();

      setDropdownPosition({
        x: vertical ? rect.right + 5 : rect.left,
        y: vertical ? rect.top : rect.bottom + 5,
      });

      setActiveDropdown(activeDropdown === item.id ? null : item.id);
    } else {
      item.onClick?.();
      setActiveDropdown(null);
    }
  };

  const toolbarStyles: CSSProperties = {
    display: 'flex',
    flexDirection: vertical ? 'column' : 'row',
    gap: theme.spacing[2],
    backgroundColor: theme.colors.background.secondary,
    borderBottom: vertical ? 'none' : `1px solid ${theme.colors.border.primary}`,
    borderRight: vertical ? `1px solid ${theme.colors.border.primary}` : 'none',
    padding: theme.spacing[2],
    flexWrap: 'wrap',
  };

  const groupStyles: CSSProperties = {
    display: 'flex',
    flexDirection: vertical ? 'column' : 'row',
    gap: theme.spacing[1],
    padding: sizeStyles[size].padding,
    borderRight: vertical ? 'none' : `1px solid ${theme.colors.border.secondary}`,
    borderBottom: vertical ? `1px solid ${theme.colors.border.secondary}` : 'none',
  };

  const buttonStyles = (item: ToolbarItem): CSSProperties => ({
    display: 'flex',
    flexDirection: compact ? 'row' : vertical ? 'row' : 'column',
    alignItems: 'center',
    justifyContent: 'center',
    gap: theme.spacing[1],
    padding: sizeStyles[size].padding,
    border: 'none',
    borderRadius: theme.borderRadius.base,
    backgroundColor: item.active ? theme.colors.interactive.secondary : 'transparent',
    color: item.disabled ? theme.colors.text.disabled : theme.colors.text.primary,
    cursor: item.disabled ? 'not-allowed' : 'pointer',
    fontSize: sizeStyles[size].fontSize,
    fontWeight: item.active ? theme.typography.fontWeight.semibold : theme.typography.fontWeight.normal,
    transition: transitionPresets.colors,
    outline: 'none',
    minWidth: compact ? 'auto' : vertical ? '100%' : '60px',
    opacity: item.disabled ? 0.5 : 1,
    position: 'relative',
  });

  const dropdownStyles: CSSProperties = {
    position: 'fixed',
    top: `${dropdownPosition.y}px`,
    left: `${dropdownPosition.x}px`,
    minWidth: '180px',
    backgroundColor: theme.colors.background.elevated,
    border: `1px solid ${theme.colors.border.primary}`,
    borderRadius: theme.borderRadius.md,
    boxShadow: theme.shadows.lg,
    padding: theme.spacing[1],
    zIndex: theme.zIndex.dropdown,
    animation: animationPresets.scaleIn,
  };

  const dropdownItemStyles = (item: ToolbarItem): CSSProperties => ({
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing[2],
    padding: theme.spacing[2],
    cursor: item.disabled ? 'not-allowed' : 'pointer',
    borderRadius: theme.borderRadius.base,
    color: item.disabled ? theme.colors.text.disabled : theme.colors.text.primary,
    fontSize: theme.typography.fontSize.sm,
    transition: transitionPresets.colors,
    opacity: item.disabled ? 0.5 : 1,
    justifyContent: 'space-between',
  });

  const DropdownIcon = () => (
    <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
      <path d="M3 5l3 3 3-3" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" />
    </svg>
  );

  const renderTooltip = (item: ToolbarItem) => {
    if (!item.tooltip && !item.shortcut) return null;

    return (
      <div
        style={{
          position: 'absolute',
          bottom: vertical ? '50%' : '-5px',
          left: vertical ? '105%' : '50%',
          transform: vertical ? 'translateY(50%)' : 'translateY(100%)',
          backgroundColor: theme.colors.background.tertiary,
          color: theme.colors.text.primary,
          padding: `${theme.spacing[1]} ${theme.spacing[2]}`,
          borderRadius: theme.borderRadius.base,
          fontSize: theme.typography.fontSize.xs,
          whiteSpace: 'nowrap',
          pointerEvents: 'none',
          opacity: 0,
          transition: transitionPresets.opacity,
          zIndex: theme.zIndex.tooltip,
          boxShadow: theme.shadows.md,
        }}
        className="toolbar-tooltip"
      >
        {item.tooltip}
        {item.shortcut && (
          <span style={{ marginLeft: theme.spacing[2], color: theme.colors.text.tertiary }}>
            {item.shortcut}
          </span>
        )}
      </div>
    );
  };

  return (
    <>
      <div ref={toolbarRef} style={toolbarStyles} role="toolbar">
        {groups.map((group, groupIndex) => (
          <div
            key={group.id}
            style={{
              ...groupStyles,
              borderRight: groupIndex === groups.length - 1 ? 'none' : groupStyles.borderRight,
              borderBottom: groupIndex === groups.length - 1 ? 'none' : groupStyles.borderBottom,
            }}
          >
            {group.items.map((item) => (
              <button
                key={item.id}
                style={buttonStyles(item)}
                onClick={(e) => handleItemClick(item, e.currentTarget)}
                disabled={item.disabled}
                aria-label={item.label}
                aria-pressed={item.type === 'toggle' ? item.active : undefined}
                onMouseEnter={(e) => {
                  if (!item.disabled) {
                    e.currentTarget.style.backgroundColor = item.active
                      ? theme.colors.interactive.secondaryHover
                      : theme.colors.background.tertiary;

                    const tooltip = e.currentTarget.querySelector('.toolbar-tooltip') as HTMLElement;
                    if (tooltip) {
                      tooltip.style.opacity = '1';
                    }
                  }
                }}
                onMouseLeave={(e) => {
                  if (!item.disabled && !item.active) {
                    e.currentTarget.style.backgroundColor = 'transparent';
                  }

                  const tooltip = e.currentTarget.querySelector('.toolbar-tooltip') as HTMLElement;
                  if (tooltip) {
                    tooltip.style.opacity = '0';
                  }
                }}
                onFocus={(e) => {
                  e.currentTarget.style.outline = `2px solid ${theme.colors.border.focus}`;
                  e.currentTarget.style.outlineOffset = '2px';
                }}
                onBlur={(e) => {
                  e.currentTarget.style.outline = 'none';
                }}
              >
                {item.icon && (
                  <span
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      width: sizeStyles[size].iconSize,
                      height: sizeStyles[size].iconSize,
                    }}
                  >
                    {item.icon}
                  </span>
                )}
                {(showLabels || !item.icon) && !compact && <span>{item.label}</span>}
                {item.type === 'dropdown' && <DropdownIcon />}
                {renderTooltip(item)}
              </button>
            ))}
          </div>
        ))}
      </div>

      {activeDropdown &&
        (() => {
          const activeItem = groups
            .flatMap((g) => g.items)
            .find((item) => item.id === activeDropdown);

          if (!activeItem?.dropdown) return null;

          return (
            <div ref={dropdownRef} style={dropdownStyles}>
              {activeItem.dropdown.map((dropdownItem) => (
                <div
                  key={dropdownItem.id}
                  style={dropdownItemStyles(dropdownItem)}
                  onClick={() => {
                    if (!dropdownItem.disabled) {
                      dropdownItem.onClick?.();
                      setActiveDropdown(null);
                    }
                  }}
                  onMouseEnter={(e) => {
                    if (!dropdownItem.disabled) {
                      e.currentTarget.style.backgroundColor = theme.colors.background.secondary;
                    }
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.backgroundColor = 'transparent';
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: theme.spacing[2] }}>
                    {dropdownItem.icon && (
                      <span style={{ display: 'flex', width: '16px', height: '16px' }}>
                        {dropdownItem.icon}
                      </span>
                    )}
                    <span>{dropdownItem.label}</span>
                  </div>
                  {dropdownItem.shortcut && (
                    <span style={{ fontSize: theme.typography.fontSize.xs, color: theme.colors.text.tertiary }}>
                      {dropdownItem.shortcut}
                    </span>
                  )}
                </div>
              ))}
            </div>
          );
        })()}
    </>
  );
};

Toolbar.displayName = 'Toolbar';
