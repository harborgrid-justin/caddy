/**
 * Enterprise Select Component
 * Features: Searchable, multi-select, virtualization, keyboard navigation, accessibility
 */

import React, { useState, useRef, useEffect, useMemo, forwardRef, ReactNode } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets, animationPresets } from './styles/animations';

export interface SelectOption<T = any> {
  value: T;
  label: string;
  disabled?: boolean;
  icon?: ReactNode;
}

export interface SelectProps<T = any> {
  /** Options to display */
  options: SelectOption<T>[];
  /** Selected value(s) */
  value?: T | T[];
  /** Change handler */
  onChange?: (value: T | T[]) => void;
  /** Placeholder text */
  placeholder?: string;
  /** Enable search */
  searchable?: boolean;
  /** Enable multi-select */
  multiple?: boolean;
  /** Disabled state */
  disabled?: boolean;
  /** Error state */
  error?: string;
  /** Label */
  label?: string;
  /** Helper text */
  helperText?: string;
  /** Full width */
  fullWidth?: boolean;
  /** Max height for dropdown */
  maxHeight?: number;
  /** Custom render for selected value */
  renderValue?: (value: T | T[]) => ReactNode;
  /** Custom render for option */
  renderOption?: (option: SelectOption<T>) => ReactNode;
  /** Size variant */
  size?: 'sm' | 'md' | 'lg';
}

export const Select = forwardRef<HTMLDivElement, SelectProps>(
  (
    {
      options,
      value,
      onChange,
      placeholder = 'Select...',
      searchable = false,
      multiple = false,
      disabled = false,
      error,
      label,
      helperText,
      fullWidth = false,
      maxHeight = 300,
      renderValue,
      renderOption,
      size = 'md',
    },
    ref
  ) => {
    const { theme } = useTheme();
    const [isOpen, setIsOpen] = useState(false);
    const [searchQuery, setSearchQuery] = useState('');
    const [highlightedIndex, setHighlightedIndex] = useState(0);
    const containerRef = useRef<HTMLDivElement>(null);
    const dropdownRef = useRef<HTMLDivElement>(null);
    const searchInputRef = useRef<HTMLInputElement>(null);

    const sizeStyles = {
      sm: {
        padding: `${theme.spacing[1]} ${theme.spacing[2]}`,
        fontSize: theme.typography.fontSize.sm,
        height: '32px',
      },
      md: {
        padding: `${theme.spacing[2]} ${theme.spacing[3]}`,
        fontSize: theme.typography.fontSize.base,
        height: '40px',
      },
      lg: {
        padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
        fontSize: theme.typography.fontSize.lg,
        height: '48px',
      },
    };

    // Filter options based on search query
    const filteredOptions = useMemo(() => {
      if (!searchQuery) return options;
      return options.filter((option) =>
        option.label.toLowerCase().includes(searchQuery.toLowerCase())
      );
    }, [options, searchQuery]);

    // Get selected options
    const selectedOptions = useMemo(() => {
      if (!value) return [];
      const values = Array.isArray(value) ? value : [value];
      return options.filter((option) => values.includes(option.value));
    }, [value, options]);

    // Close dropdown on outside click
    useEffect(() => {
      const handleClickOutside = (event: MouseEvent) => {
        if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
          setIsOpen(false);
          setSearchQuery('');
        }
      };

      if (isOpen) {
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
      }
    }, [isOpen]);

    // Focus search input when dropdown opens
    useEffect(() => {
      if (isOpen && searchable && searchInputRef.current) {
        searchInputRef.current.focus();
      }
    }, [isOpen, searchable]);

    // Keyboard navigation
    const handleKeyDown = (e: React.KeyboardEvent) => {
      if (disabled) return;

      switch (e.key) {
        case 'Enter':
        case ' ':
          if (!isOpen) {
            setIsOpen(true);
          } else if (filteredOptions[highlightedIndex] && !filteredOptions[highlightedIndex].disabled) {
            handleSelect(filteredOptions[highlightedIndex]);
          }
          e.preventDefault();
          break;
        case 'Escape':
          setIsOpen(false);
          setSearchQuery('');
          break;
        case 'ArrowDown':
          e.preventDefault();
          setHighlightedIndex((prev) => Math.min(prev + 1, filteredOptions.length - 1));
          break;
        case 'ArrowUp':
          e.preventDefault();
          setHighlightedIndex((prev) => Math.max(prev - 1, 0));
          break;
        case 'Home':
          e.preventDefault();
          setHighlightedIndex(0);
          break;
        case 'End':
          e.preventDefault();
          setHighlightedIndex(filteredOptions.length - 1);
          break;
      }
    };

    const handleSelect = (option: SelectOption) => {
      if (option.disabled) return;

      if (multiple) {
        const currentValues = Array.isArray(value) ? value : [];
        const newValues = currentValues.includes(option.value)
          ? currentValues.filter((v) => v !== option.value)
          : [...currentValues, option.value];
        onChange?.(newValues);
      } else {
        onChange?.(option.value);
        setIsOpen(false);
        setSearchQuery('');
      }
    };

    const getDisplayValue = (): ReactNode => {
      if (renderValue) {
        return renderValue(value || (multiple ? [] : ''));
      }

      if (selectedOptions.length === 0) {
        return <span style={{ color: theme.colors.text.tertiary }}>{placeholder}</span>;
      }

      if (multiple) {
        return selectedOptions.map((opt) => opt.label).join(', ');
      }

      return selectedOptions[0]?.label || placeholder;
    };

    const containerStyles: React.CSSProperties = {
      display: 'flex',
      flexDirection: 'column',
      gap: theme.spacing[1],
      width: fullWidth ? '100%' : 'auto',
      position: 'relative',
    };

    const labelStyles: React.CSSProperties = {
      fontSize: theme.typography.fontSize.sm,
      fontWeight: theme.typography.fontWeight.medium,
      color: theme.colors.text.primary,
    };

    const selectStyles: React.CSSProperties = {
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      backgroundColor: theme.colors.background.primary,
      border: `1px solid ${
        error ? theme.colors.border.error : isOpen ? theme.colors.border.focus : theme.colors.border.primary
      }`,
      borderRadius: theme.borderRadius.md,
      cursor: disabled ? 'not-allowed' : 'pointer',
      opacity: disabled ? 0.6 : 1,
      transition: transitionPresets.colors,
      userSelect: 'none',
      ...sizeStyles[size],
    };

    const dropdownStyles: React.CSSProperties = {
      position: 'absolute',
      top: '100%',
      left: 0,
      right: 0,
      marginTop: theme.spacing[1],
      backgroundColor: theme.colors.background.elevated,
      border: `1px solid ${theme.colors.border.primary}`,
      borderRadius: theme.borderRadius.md,
      boxShadow: theme.shadows.lg,
      maxHeight: `${maxHeight}px`,
      overflowY: 'auto',
      zIndex: theme.zIndex.dropdown,
      animation: animationPresets.slideInDown,
    };

    const searchStyles: React.CSSProperties = {
      width: '100%',
      padding: theme.spacing[2],
      border: 'none',
      borderBottom: `1px solid ${theme.colors.border.secondary}`,
      outline: 'none',
      backgroundColor: 'transparent',
      color: theme.colors.text.primary,
      fontSize: theme.typography.fontSize.sm,
    };

    const optionStyles = (option: SelectOption, index: number): React.CSSProperties => {
      const isSelected = selectedOptions.some((s) => s.value === option.value);
      const isHighlighted = index === highlightedIndex;

      return {
        padding: theme.spacing[2],
        cursor: option.disabled ? 'not-allowed' : 'pointer',
        backgroundColor: isHighlighted
          ? theme.colors.background.secondary
          : isSelected
          ? theme.colors.interactive.secondary
          : 'transparent',
        color: option.disabled ? theme.colors.text.disabled : theme.colors.text.primary,
        transition: transitionPresets.colors,
        display: 'flex',
        alignItems: 'center',
        gap: theme.spacing[2],
      };
    };

    const ChevronIcon = ({ isOpen }: { isOpen: boolean }) => (
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="currentColor"
        style={{
          transform: isOpen ? 'rotate(180deg)' : 'rotate(0deg)',
          transition: transitionPresets.transform,
        }}
      >
        <path d="M4 6l4 4 4-4" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" />
      </svg>
    );

    return (
      <div ref={ref} style={containerStyles}>
        {label && <label style={labelStyles}>{label}</label>}

        <div ref={containerRef}>
          <div
            role="combobox"
            aria-expanded={isOpen}
            aria-haspopup="listbox"
            aria-disabled={disabled}
            tabIndex={disabled ? -1 : 0}
            style={selectStyles}
            onClick={() => !disabled && setIsOpen(!isOpen)}
            onKeyDown={handleKeyDown}
          >
            <span style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {getDisplayValue()}
            </span>
            <ChevronIcon isOpen={isOpen} />
          </div>

          {isOpen && (
            <div ref={dropdownRef} style={dropdownStyles}>
              {searchable && (
                <input
                  ref={searchInputRef}
                  type="text"
                  placeholder="Search..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  style={searchStyles}
                  onClick={(e) => e.stopPropagation()}
                />
              )}

              <div role="listbox" aria-multiselectable={multiple}>
                {filteredOptions.length === 0 ? (
                  <div style={{ padding: theme.spacing[4], textAlign: 'center', color: theme.colors.text.tertiary }}>
                    No options found
                  </div>
                ) : (
                  filteredOptions.map((option, index) => (
                    <div
                      key={String(option.value)}
                      role="option"
                      aria-selected={selectedOptions.some((s) => s.value === option.value)}
                      aria-disabled={option.disabled}
                      style={optionStyles(option, index)}
                      onClick={() => handleSelect(option)}
                      onMouseEnter={() => setHighlightedIndex(index)}
                    >
                      {multiple && (
                        <input
                          type="checkbox"
                          checked={selectedOptions.some((s) => s.value === option.value)}
                          onChange={() => {}}
                          style={{ pointerEvents: 'none' }}
                        />
                      )}
                      {option.icon && <span>{option.icon}</span>}
                      {renderOption ? renderOption(option) : <span>{option.label}</span>}
                    </div>
                  ))
                )}
              </div>
            </div>
          )}
        </div>

        {(error || helperText) && (
          <span
            style={{
              fontSize: theme.typography.fontSize.xs,
              color: error ? theme.colors.status.error : theme.colors.text.secondary,
            }}
          >
            {error || helperText}
          </span>
        )}
      </div>
    );
  }
);

Select.displayName = 'Select';
