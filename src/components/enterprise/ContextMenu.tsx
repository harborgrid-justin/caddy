/**
 * Enterprise Context Menu Component
 * Features: Right-click menus, submenus, keyboard navigation, accessibility
 */

import React, { useState, useEffect, useRef, ReactNode, CSSProperties } from 'react';
import { createPortal } from 'react-dom';
import { useTheme } from './styles/theme';
import { animationPresets, transitionPresets } from './styles/animations';

export interface ContextMenuItem {
  id: string;
  label: ReactNode;
  icon?: ReactNode;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
  divider?: boolean;
  submenu?: ContextMenuItem[];
  onClick?: () => void;
}

export interface ContextMenuProps {
  /** Menu items */
  items: ContextMenuItem[];
  /** Children (trigger element) */
  children: React.ReactElement;
  /** Disable context menu */
  disabled?: boolean;
}

export const ContextMenu: React.FC<ContextMenuProps> = ({ items, children, disabled = false }) => {
  const { theme } = useTheme();
  const [isOpen, setIsOpen] = useState(false);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [activeSubmenu, setActiveSubmenu] = useState<string | null>(null);
  const [submenuPosition, setSubmenuPosition] = useState({ x: 0, y: 0 });
  const menuRef = useRef<HTMLDivElement>(null);
  const submenuRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (
        menuRef.current &&
        !menuRef.current.contains(e.target as Node) &&
        submenuRef.current &&
        !submenuRef.current.contains(e.target as Node)
      ) {
        setIsOpen(false);
        setActiveSubmenu(null);
      }
    };

    const handleScroll = () => {
      setIsOpen(false);
      setActiveSubmenu(null);
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setIsOpen(false);
        setActiveSubmenu(null);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('scroll', handleScroll, true);
    document.addEventListener('keydown', handleEscape);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('scroll', handleScroll, true);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [isOpen]);

  const handleContextMenu = (e: React.MouseEvent) => {
    if (disabled) return;

    e.preventDefault();
    e.stopPropagation();

    const viewport = {
      width: window.innerWidth,
      height: window.innerHeight,
    };

    let x = e.clientX;
    let y = e.clientY;

    // Rough estimate of menu size
    const menuWidth = 250;
    const menuHeight = Math.min(items.length * 40, 400);

    // Adjust position if menu would overflow viewport
    if (x + menuWidth > viewport.width) {
      x = viewport.width - menuWidth - 10;
    }

    if (y + menuHeight > viewport.height) {
      y = viewport.height - menuHeight - 10;
    }

    setPosition({ x, y });
    setIsOpen(true);
  };

  const handleItemClick = (item: ContextMenuItem) => {
    if (item.disabled || item.divider) return;

    if (item.submenu) {
      setActiveSubmenu(activeSubmenu === item.id ? null : item.id);
    } else {
      item.onClick?.();
      setIsOpen(false);
      setActiveSubmenu(null);
    }
  };

  const handleSubmenuHover = (item: ContextMenuItem, index: number) => {
    if (!item.submenu || item.disabled) return;

    setActiveSubmenu(item.id);

    // Calculate submenu position
    if (menuRef.current) {
      const menuRect = menuRef.current.getBoundingClientRect();
      const itemHeight = 40;
      const submenuWidth = 250;

      let x = menuRect.right + 5;
      let y = menuRect.top + index * itemHeight;

      // Flip to left if overflows
      if (x + submenuWidth > window.innerWidth) {
        x = menuRect.left - submenuWidth - 5;
      }

      setSubmenuPosition({ x, y });
    }
  };

  const menuStyles: CSSProperties = {
    position: 'fixed',
    top: `${position.y}px`,
    left: `${position.x}px`,
    minWidth: '200px',
    maxWidth: '300px',
    backgroundColor: theme.colors.background.elevated,
    border: `1px solid ${theme.colors.border.primary}`,
    borderRadius: theme.borderRadius.md,
    boxShadow: theme.shadows.lg,
    padding: theme.spacing[1],
    zIndex: theme.zIndex.popover,
    animation: animationPresets.scaleIn,
  };

  const itemStyles = (item: ContextMenuItem, isActive: boolean): CSSProperties => ({
    display: item.divider ? 'block' : 'flex',
    alignItems: 'center',
    gap: theme.spacing[3],
    padding: item.divider ? 0 : theme.spacing[2],
    margin: item.divider ? `${theme.spacing[1]} 0` : 0,
    borderTop: item.divider ? `1px solid ${theme.colors.border.secondary}` : undefined,
    cursor: item.disabled || item.divider ? 'default' : 'pointer',
    borderRadius: theme.borderRadius.base,
    color: item.disabled
      ? theme.colors.text.disabled
      : item.danger
      ? theme.colors.status.error
      : theme.colors.text.primary,
    backgroundColor: isActive && !item.disabled && !item.divider ? theme.colors.background.secondary : 'transparent',
    fontSize: theme.typography.fontSize.sm,
    transition: transitionPresets.colors,
    position: 'relative',
    userSelect: 'none',
  });

  const shortcutStyles: CSSProperties = {
    marginLeft: 'auto',
    fontSize: theme.typography.fontSize.xs,
    color: theme.colors.text.tertiary,
  };

  const ChevronIcon = () => (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
      <path d="M6 4l4 4-4 4" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" />
    </svg>
  );

  const renderMenu = (menuItems: ContextMenuItem[], isSubmenu = false) => (
    <div ref={isSubmenu ? submenuRef : menuRef} style={menuStyles}>
      {menuItems.map((item, index) => {
        if (item.divider) {
          return <div key={`divider-${index}`} style={itemStyles(item, false)} />;
        }

        const isActive = activeSubmenu === item.id;

        return (
          <div
            key={item.id}
            role="menuitem"
            aria-disabled={item.disabled}
            tabIndex={item.disabled ? -1 : 0}
            style={itemStyles(item, isActive)}
            onClick={() => handleItemClick(item)}
            onMouseEnter={() => !isSubmenu && handleSubmenuHover(item, index)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                handleItemClick(item);
              }
            }}
          >
            {item.icon && (
              <span style={{ display: 'flex', width: '16px', height: '16px' }}>{item.icon}</span>
            )}
            <span style={{ flex: 1 }}>{item.label}</span>
            {item.shortcut && <span style={shortcutStyles}>{item.shortcut}</span>}
            {item.submenu && <ChevronIcon />}
          </div>
        );
      })}
    </div>
  );

  const trigger = React.cloneElement(children, {
    ref: triggerRef,
    onContextMenu: handleContextMenu,
  });

  return (
    <>
      {trigger}
      {isOpen && createPortal(renderMenu(items), document.body)}
      {activeSubmenu &&
        (() => {
          const item = items.find((i) => i.id === activeSubmenu);
          if (!item?.submenu) return null;

          const submenuStyles: CSSProperties = {
            ...menuStyles,
            top: `${submenuPosition.y}px`,
            left: `${submenuPosition.x}px`,
          };

          return createPortal(
            <div ref={submenuRef} style={submenuStyles}>
              {item.submenu.map((subItem, index) => {
                if (subItem.divider) {
                  return <div key={`divider-${index}`} style={itemStyles(subItem, false)} />;
                }

                return (
                  <div
                    key={subItem.id}
                    role="menuitem"
                    aria-disabled={subItem.disabled}
                    tabIndex={subItem.disabled ? -1 : 0}
                    style={itemStyles(subItem, false)}
                    onClick={() => {
                      if (!subItem.disabled) {
                        subItem.onClick?.();
                        setIsOpen(false);
                        setActiveSubmenu(null);
                      }
                    }}
                  >
                    {subItem.icon && (
                      <span style={{ display: 'flex', width: '16px', height: '16px' }}>{subItem.icon}</span>
                    )}
                    <span style={{ flex: 1 }}>{subItem.label}</span>
                    {subItem.shortcut && <span style={shortcutStyles}>{subItem.shortcut}</span>}
                  </div>
                );
              })}
            </div>,
            document.body
          );
        })()}
    </>
  );
};

ContextMenu.displayName = 'ContextMenu';
