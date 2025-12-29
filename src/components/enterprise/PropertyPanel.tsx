/**
 * Enterprise Property Panel Component
 * Features: CAD property inspector with categorized properties, inline editing, validation
 */

import React, { useState, ReactNode, CSSProperties } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets } from './styles/animations';

export type PropertyValue = string | number | boolean | null;

export interface Property {
  id: string;
  label: string;
  value: PropertyValue;
  type: 'text' | 'number' | 'boolean' | 'color' | 'select' | 'readonly';
  options?: { value: PropertyValue; label: string }[];
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  editable?: boolean;
  validate?: (value: PropertyValue) => string | null;
  format?: (value: PropertyValue) => string;
}

export interface PropertyCategory {
  id: string;
  title: string;
  properties: Property[];
  collapsed?: boolean;
  icon?: ReactNode;
}

export interface PropertyPanelProps {
  /** Property categories */
  categories: PropertyCategory[];
  /** Property change handler */
  onChange?: (propertyId: string, value: PropertyValue) => void;
  /** Width of the panel */
  width?: number;
  /** Collapsible categories */
  collapsible?: boolean;
}

export const PropertyPanel: React.FC<PropertyPanelProps> = ({
  categories,
  onChange,
  width = 280,
  collapsible = true,
}) => {
  const { theme } = useTheme();
  const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(
    new Set(categories.filter((c) => c.collapsed).map((c) => c.id))
  );
  const [editingProperty, setEditingProperty] = useState<string | null>(null);

  const toggleCategory = (categoryId: string) => {
    if (!collapsible) return;

    setCollapsedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(categoryId)) {
        next.delete(categoryId);
      } else {
        next.add(categoryId);
      }
      return next;
    });
  };

  const handlePropertyChange = (property: Property, value: PropertyValue) => {
    // Validate if validator provided
    if (property.validate) {
      const error = property.validate(value);
      if (error) {
        console.warn(`Validation error for ${property.id}: ${error}`);
        return;
      }
    }

    onChange?.(property.id, value);
    setEditingProperty(null);
  };

  const renderPropertyValue = (property: Property) => {
    const isEditing = editingProperty === property.id;
    const isEditable = property.editable !== false && property.type !== 'readonly';

    const inputStyles: CSSProperties = {
      width: '100%',
      padding: `${theme.spacing[1]} ${theme.spacing[2]}`,
      border: `1px solid ${theme.colors.border.primary}`,
      borderRadius: theme.borderRadius.base,
      backgroundColor: theme.colors.background.primary,
      color: theme.colors.text.primary,
      fontSize: theme.typography.fontSize.sm,
      outline: 'none',
      transition: transitionPresets.colors,
    };

    const valueDisplayStyles: CSSProperties = {
      padding: `${theme.spacing[1]} ${theme.spacing[2]}`,
      backgroundColor: isEditable ? 'transparent' : theme.colors.background.secondary,
      borderRadius: theme.borderRadius.base,
      cursor: isEditable ? 'pointer' : 'default',
      fontSize: theme.typography.fontSize.sm,
      color: theme.colors.text.primary,
      border: `1px solid transparent`,
      transition: transitionPresets.colors,
    };

    switch (property.type) {
      case 'text':
        return isEditing ? (
          <input
            type="text"
            value={String(property.value || '')}
            onChange={(e) => handlePropertyChange(property, e.target.value)}
            onBlur={() => setEditingProperty(null)}
            autoFocus
            style={inputStyles}
          />
        ) : (
          <div
            style={valueDisplayStyles}
            onClick={() => isEditable && setEditingProperty(property.id)}
            onMouseEnter={(e) => {
              if (isEditable) {
                e.currentTarget.style.borderColor = theme.colors.border.primary;
              }
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.borderColor = 'transparent';
            }}
          >
            {property.format ? property.format(property.value) : property.value || '—'}
          </div>
        );

      case 'number':
        return isEditing ? (
          <input
            type="number"
            value={Number(property.value || 0)}
            onChange={(e) => handlePropertyChange(property, Number(e.target.value))}
            onBlur={() => setEditingProperty(null)}
            min={property.min}
            max={property.max}
            step={property.step || 1}
            autoFocus
            style={inputStyles}
          />
        ) : (
          <div
            style={valueDisplayStyles}
            onClick={() => isEditable && setEditingProperty(property.id)}
            onMouseEnter={(e) => {
              if (isEditable) {
                e.currentTarget.style.borderColor = theme.colors.border.primary;
              }
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.borderColor = 'transparent';
            }}
          >
            {property.format
              ? property.format(property.value)
              : `${property.value || 0}${property.unit || ''}`}
          </div>
        );

      case 'boolean':
        return (
          <input
            type="checkbox"
            checked={Boolean(property.value)}
            onChange={(e) => handlePropertyChange(property, e.target.checked)}
            disabled={!isEditable}
            style={{
              cursor: isEditable ? 'pointer' : 'not-allowed',
              width: '18px',
              height: '18px',
            }}
          />
        );

      case 'color':
        return (
          <div style={{ display: 'flex', gap: theme.spacing[2], alignItems: 'center' }}>
            <input
              type="color"
              value={String(property.value || '#000000')}
              onChange={(e) => handlePropertyChange(property, e.target.value)}
              disabled={!isEditable}
              style={{
                width: '40px',
                height: '30px',
                border: `1px solid ${theme.colors.border.primary}`,
                borderRadius: theme.borderRadius.base,
                cursor: isEditable ? 'pointer' : 'not-allowed',
              }}
            />
            <span style={{ fontSize: theme.typography.fontSize.sm, color: theme.colors.text.secondary }}>
              {property.value}
            </span>
          </div>
        );

      case 'select':
        return (
          <select
            value={String(property.value || '')}
            onChange={(e) => {
              const option = property.options?.find((o) => String(o.value) === e.target.value);
              if (option) {
                handlePropertyChange(property, option.value);
              }
            }}
            disabled={!isEditable}
            style={{
              ...inputStyles,
              cursor: isEditable ? 'pointer' : 'not-allowed',
            }}
          >
            {property.options?.map((option) => (
              <option key={String(option.value)} value={String(option.value)}>
                {option.label}
              </option>
            ))}
          </select>
        );

      case 'readonly':
        return (
          <div style={valueDisplayStyles}>
            {property.format ? property.format(property.value) : property.value || '—'}
          </div>
        );

      default:
        return null;
    }
  };

  const panelStyles: CSSProperties = {
    width: `${width}px`,
    height: '100%',
    backgroundColor: theme.colors.background.primary,
    borderLeft: `1px solid ${theme.colors.border.primary}`,
    display: 'flex',
    flexDirection: 'column',
    overflow: 'hidden',
  };

  const headerStyles: CSSProperties = {
    padding: theme.spacing[4],
    borderBottom: `1px solid ${theme.colors.border.primary}`,
    backgroundColor: theme.colors.background.secondary,
    fontWeight: theme.typography.fontWeight.semibold,
    fontSize: theme.typography.fontSize.base,
    color: theme.colors.text.primary,
  };

  const contentStyles: CSSProperties = {
    flex: 1,
    overflowY: 'auto',
    overflowX: 'hidden',
  };

  const categoryHeaderStyles = (isCollapsed: boolean): CSSProperties => ({
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing[2],
    padding: theme.spacing[3],
    backgroundColor: theme.colors.background.secondary,
    borderBottom: `1px solid ${theme.colors.border.secondary}`,
    cursor: collapsible ? 'pointer' : 'default',
    userSelect: 'none',
    transition: transitionPresets.colors,
  });

  const categoryContentStyles: CSSProperties = {
    padding: theme.spacing[2],
  };

  const propertyRowStyles: CSSProperties = {
    display: 'grid',
    gridTemplateColumns: '1fr 1.5fr',
    gap: theme.spacing[2],
    padding: `${theme.spacing[2]} ${theme.spacing[1]}`,
    alignItems: 'center',
    borderBottom: `1px solid ${theme.colors.border.secondary}`,
  };

  const propertyLabelStyles: CSSProperties = {
    fontSize: theme.typography.fontSize.sm,
    color: theme.colors.text.secondary,
    fontWeight: theme.typography.fontWeight.medium,
  };

  const ChevronIcon = ({ isCollapsed }: { isCollapsed: boolean }) => (
    <svg
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="currentColor"
      style={{
        transform: isCollapsed ? 'rotate(-90deg)' : 'rotate(0deg)',
        transition: transitionPresets.transform,
      }}
    >
      <path d="M4 6l4 4 4-4" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" />
    </svg>
  );

  return (
    <div style={panelStyles}>
      <div style={headerStyles}>Properties</div>

      <div style={contentStyles}>
        {categories.map((category) => {
          const isCollapsed = collapsedCategories.has(category.id);

          return (
            <div key={category.id}>
              <div
                style={categoryHeaderStyles(isCollapsed)}
                onClick={() => toggleCategory(category.id)}
                onMouseEnter={(e) => {
                  if (collapsible) {
                    e.currentTarget.style.backgroundColor = theme.colors.background.tertiary;
                  }
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor = theme.colors.background.secondary;
                }}
              >
                {collapsible && <ChevronIcon isCollapsed={isCollapsed} />}
                {category.icon && <span style={{ display: 'flex' }}>{category.icon}</span>}
                <span style={{ fontSize: theme.typography.fontSize.sm, fontWeight: theme.typography.fontWeight.semibold }}>
                  {category.title}
                </span>
              </div>

              {!isCollapsed && (
                <div style={categoryContentStyles}>
                  {category.properties.map((property) => (
                    <div key={property.id} style={propertyRowStyles}>
                      <label style={propertyLabelStyles}>{property.label}</label>
                      <div>{renderPropertyValue(property)}</div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};

PropertyPanel.displayName = 'PropertyPanel';
