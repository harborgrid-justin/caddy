/**
 * Enterprise Tabs Component
 * Features: Accessible tabs with keyboard navigation, icons, badges, vertical orientation
 */

import React, { useState, useRef, useEffect, ReactNode, CSSProperties } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets } from './styles/animations';

export interface Tab {
  id: string;
  label: ReactNode;
  content: ReactNode;
  icon?: ReactNode;
  badge?: number | string;
  disabled?: boolean;
}

export interface TabsProps {
  /** Tab items */
  tabs: Tab[];
  /** Active tab ID */
  activeTab?: string;
  /** Tab change handler */
  onChange?: (tabId: string) => void;
  /** Vertical orientation */
  vertical?: boolean;
  /** Variant */
  variant?: 'line' | 'enclosed' | 'pills';
  /** Size */
  size?: 'sm' | 'md' | 'lg';
  /** Full width tabs */
  fullWidth?: boolean;
}

export const Tabs: React.FC<TabsProps> = ({
  tabs,
  activeTab: controlledActiveTab,
  onChange,
  vertical = false,
  variant = 'line',
  size = 'md',
  fullWidth = false,
}) => {
  const { theme } = useTheme();
  const [internalActiveTab, setInternalActiveTab] = useState(tabs[0]?.id);
  const [indicatorStyle, setIndicatorStyle] = useState<CSSProperties>({});
  const tabRefs = useRef<Map<string, HTMLButtonElement>>(new Map());

  const activeTab = controlledActiveTab ?? internalActiveTab;
  const activeTabData = tabs.find((tab) => tab.id === activeTab) || tabs[0];

  const sizeStyles = {
    sm: {
      padding: `${theme.spacing[2]} ${theme.spacing[3]}`,
      fontSize: theme.typography.fontSize.sm,
    },
    md: {
      padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
      fontSize: theme.typography.fontSize.base,
    },
    lg: {
      padding: `${theme.spacing[4]} ${theme.spacing[5]}`,
      fontSize: theme.typography.fontSize.lg,
    },
  };

  // Update indicator position
  useEffect(() => {
    const activeButton = tabRefs.current.get(activeTab);
    if (!activeButton || variant !== 'line') return;

    const updateIndicator = () => {
      const rect = activeButton.getBoundingClientRect();
      const containerRect = activeButton.parentElement?.getBoundingClientRect();

      if (!containerRect) return;

      if (vertical) {
        setIndicatorStyle({
          height: `${rect.height}px`,
          top: `${rect.top - containerRect.top}px`,
          transition: transitionPresets.transform,
        });
      } else {
        setIndicatorStyle({
          width: `${rect.width}px`,
          left: `${rect.left - containerRect.left}px`,
          transition: transitionPresets.transform,
        });
      }
    };

    updateIndicator();
    window.addEventListener('resize', updateIndicator);
    return () => window.removeEventListener('resize', updateIndicator);
  }, [activeTab, variant, vertical]);

  const handleTabClick = (tab: Tab) => {
    if (tab.disabled) return;

    if (onChange) {
      onChange(tab.id);
    } else {
      setInternalActiveTab(tab.id);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent, currentIndex: number) => {
    const enabledTabs = tabs.filter((tab) => !tab.disabled);
    const currentEnabledIndex = enabledTabs.findIndex((tab) => tab.id === tabs[currentIndex].id);

    let nextIndex = currentEnabledIndex;

    switch (e.key) {
      case 'ArrowLeft':
      case 'ArrowUp':
        e.preventDefault();
        nextIndex = currentEnabledIndex - 1;
        if (nextIndex < 0) nextIndex = enabledTabs.length - 1;
        break;
      case 'ArrowRight':
      case 'ArrowDown':
        e.preventDefault();
        nextIndex = currentEnabledIndex + 1;
        if (nextIndex >= enabledTabs.length) nextIndex = 0;
        break;
      case 'Home':
        e.preventDefault();
        nextIndex = 0;
        break;
      case 'End':
        e.preventDefault();
        nextIndex = enabledTabs.length - 1;
        break;
      default:
        return;
    }

    const nextTab = enabledTabs[nextIndex];
    if (nextTab) {
      handleTabClick(nextTab);
      tabRefs.current.get(nextTab.id)?.focus();
    }
  };

  const containerStyles: CSSProperties = {
    display: 'flex',
    flexDirection: vertical ? 'row' : 'column',
    gap: vertical ? theme.spacing[4] : 0,
  };

  const tabListStyles: CSSProperties = {
    display: 'flex',
    flexDirection: vertical ? 'column' : 'row',
    gap: variant === 'pills' ? theme.spacing[2] : 0,
    position: 'relative',
    borderBottom: variant === 'line' && !vertical ? `2px solid ${theme.colors.border.primary}` : undefined,
    borderRight: variant === 'line' && vertical ? `2px solid ${theme.colors.border.primary}` : undefined,
    backgroundColor: variant === 'enclosed' ? theme.colors.background.secondary : undefined,
    padding: variant === 'enclosed' ? theme.spacing[1] : undefined,
    borderRadius: variant === 'enclosed' ? theme.borderRadius.md : undefined,
    width: vertical ? '200px' : fullWidth ? '100%' : undefined,
  };

  const getTabStyles = (tab: Tab, isActive: boolean): CSSProperties => {
    const base: CSSProperties = {
      ...sizeStyles[size],
      display: 'flex',
      alignItems: 'center',
      gap: theme.spacing[2],
      border: 'none',
      background: 'transparent',
      cursor: tab.disabled ? 'not-allowed' : 'pointer',
      color: tab.disabled
        ? theme.colors.text.disabled
        : isActive
        ? theme.colors.interactive.primary
        : theme.colors.text.secondary,
      fontWeight: isActive ? theme.typography.fontWeight.semibold : theme.typography.fontWeight.normal,
      transition: transitionPresets.colors,
      position: 'relative',
      outline: 'none',
      whiteSpace: 'nowrap',
      opacity: tab.disabled ? 0.5 : 1,
      flex: fullWidth ? 1 : undefined,
      justifyContent: fullWidth ? 'center' : undefined,
    };

    switch (variant) {
      case 'line':
        return {
          ...base,
          borderBottom: vertical ? undefined : 'none',
          borderRight: vertical ? 'none' : undefined,
        };
      case 'enclosed':
        return {
          ...base,
          backgroundColor: isActive ? theme.colors.background.primary : 'transparent',
          borderRadius: theme.borderRadius.base,
        };
      case 'pills':
        return {
          ...base,
          backgroundColor: isActive ? theme.colors.interactive.primary : theme.colors.background.secondary,
          color: isActive ? theme.colors.text.inverse : theme.colors.text.primary,
          borderRadius: theme.borderRadius.full,
          paddingLeft: theme.spacing[4],
          paddingRight: theme.spacing[4],
        };
      default:
        return base;
    }
  };

  const indicatorStyles: CSSProperties = {
    position: 'absolute',
    bottom: vertical ? undefined : 0,
    left: vertical ? undefined : 0,
    right: vertical ? 0 : undefined,
    width: vertical ? '2px' : indicatorStyle.width,
    height: vertical ? indicatorStyle.height : '2px',
    backgroundColor: theme.colors.interactive.primary,
    top: vertical ? indicatorStyle.top : undefined,
    left: !vertical ? indicatorStyle.left : undefined,
    ...indicatorStyle,
  };

  const badgeStyles: CSSProperties = {
    backgroundColor: theme.colors.status.error,
    color: theme.colors.text.inverse,
    fontSize: theme.typography.fontSize.xs,
    fontWeight: theme.typography.fontWeight.bold,
    padding: `${theme.spacing[0]} ${theme.spacing[1]}`,
    borderRadius: theme.borderRadius.full,
    minWidth: '20px',
    height: '20px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  };

  const contentStyles: CSSProperties = {
    flex: 1,
    padding: theme.spacing[4],
  };

  return (
    <div style={containerStyles}>
      <div role="tablist" aria-orientation={vertical ? 'vertical' : 'horizontal'} style={tabListStyles}>
        {variant === 'line' && <div style={indicatorStyles} />}

        {tabs.map((tab, index) => {
          const isActive = tab.id === activeTab;

          return (
            <button
              key={tab.id}
              ref={(el) => {
                if (el) tabRefs.current.set(tab.id, el);
              }}
              role="tab"
              aria-selected={isActive}
              aria-controls={`tabpanel-${tab.id}`}
              aria-disabled={tab.disabled}
              tabIndex={isActive ? 0 : -1}
              style={getTabStyles(tab, isActive)}
              onClick={() => handleTabClick(tab)}
              onKeyDown={(e) => handleKeyDown(e, index)}
              onMouseEnter={(e) => {
                if (!tab.disabled && !isActive && variant !== 'pills') {
                  e.currentTarget.style.color = theme.colors.text.primary;
                }
              }}
              onMouseLeave={(e) => {
                if (!tab.disabled && !isActive && variant !== 'pills') {
                  e.currentTarget.style.color = theme.colors.text.secondary;
                }
              }}
              onFocus={(e) => {
                if (variant === 'enclosed') {
                  e.currentTarget.style.outline = `2px solid ${theme.colors.border.focus}`;
                  e.currentTarget.style.outlineOffset = '2px';
                }
              }}
              onBlur={(e) => {
                e.currentTarget.style.outline = 'none';
              }}
            >
              {tab.icon && <span style={{ display: 'flex' }}>{tab.icon}</span>}
              <span>{tab.label}</span>
              {tab.badge !== undefined && <span style={badgeStyles}>{tab.badge}</span>}
            </button>
          );
        })}
      </div>

      <div
        role="tabpanel"
        id={`tabpanel-${activeTab}`}
        aria-labelledby={activeTab}
        style={contentStyles}
      >
        {activeTabData?.content}
      </div>
    </div>
  );
};

Tabs.displayName = 'Tabs';
