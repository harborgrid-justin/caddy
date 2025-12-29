/**
 * Enterprise Input Component
 * Features: Validation, masking, prefix/suffix, error states, accessibility
 */

import React, { forwardRef, InputHTMLAttributes, ReactNode, useState, useCallback } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets } from './styles/animations';

export interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  /** Input label */
  label?: string;
  /** Error message */
  error?: string;
  /** Helper text */
  helperText?: string;
  /** Size variant */
  size?: 'sm' | 'md' | 'lg';
  /** Full width */
  fullWidth?: boolean;
  /** Prefix content */
  prefix?: ReactNode;
  /** Suffix content */
  suffix?: ReactNode;
  /** Input mask pattern */
  mask?: (value: string) => string;
  /** Validation function */
  validate?: (value: string) => string | null;
  /** Show character count */
  showCount?: boolean;
  /** Required field indicator */
  showRequired?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      label,
      error,
      helperText,
      size = 'md',
      fullWidth = false,
      prefix,
      suffix,
      mask,
      validate,
      showCount = false,
      showRequired = false,
      required = false,
      disabled = false,
      maxLength,
      value: controlledValue,
      onChange,
      onBlur,
      className = '',
      style = {},
      ...props
    },
    ref
  ) => {
    const { theme } = useTheme();
    const [internalValue, setInternalValue] = useState('');
    const [touched, setTouched] = useState(false);
    const [isFocused, setIsFocused] = useState(false);

    const value = controlledValue !== undefined ? String(controlledValue) : internalValue;
    const hasError = Boolean(error);

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

    const handleChange = useCallback(
      (e: React.ChangeEvent<HTMLInputElement>) => {
        let newValue = e.target.value;

        // Apply mask if provided
        if (mask) {
          newValue = mask(newValue);
        }

        // Apply maxLength if specified
        if (maxLength && newValue.length > maxLength) {
          newValue = newValue.slice(0, maxLength);
        }

        if (controlledValue === undefined) {
          setInternalValue(newValue);
        }

        // Create synthetic event with masked value
        const syntheticEvent = {
          ...e,
          target: { ...e.target, value: newValue },
        };
        onChange?.(syntheticEvent as React.ChangeEvent<HTMLInputElement>);
      },
      [mask, maxLength, controlledValue, onChange]
    );

    const handleBlur = useCallback(
      (e: React.FocusEvent<HTMLInputElement>) => {
        setTouched(true);
        setIsFocused(false);
        onBlur?.(e);
      },
      [onBlur]
    );

    const handleFocus = useCallback(() => {
      setIsFocused(true);
    }, []);

    const validationError = validate && touched ? validate(value) : null;
    const displayError = error || validationError;

    const containerStyles: React.CSSProperties = {
      display: 'flex',
      flexDirection: 'column',
      gap: theme.spacing[1],
      width: fullWidth ? '100%' : 'auto',
      ...style,
    };

    const labelStyles: React.CSSProperties = {
      fontSize: theme.typography.fontSize.sm,
      fontWeight: theme.typography.fontWeight.medium,
      color: theme.colors.text.primary,
      marginBottom: theme.spacing[1],
    };

    const inputWrapperStyles: React.CSSProperties = {
      display: 'flex',
      alignItems: 'center',
      gap: theme.spacing[2],
      backgroundColor: theme.colors.background.primary,
      border: `1px solid ${
        hasError
          ? theme.colors.border.error
          : isFocused
          ? theme.colors.border.focus
          : theme.colors.border.primary
      }`,
      borderRadius: theme.borderRadius.md,
      transition: transitionPresets.colors,
      opacity: disabled ? 0.6 : 1,
      cursor: disabled ? 'not-allowed' : 'text',
    };

    const inputStyles: React.CSSProperties = {
      flex: 1,
      border: 'none',
      outline: 'none',
      backgroundColor: 'transparent',
      color: theme.colors.text.primary,
      fontFamily: theme.typography.fontFamily.sans,
      ...sizeStyles[size],
      padding: prefix || suffix ? `${sizeStyles[size].padding.split(' ')[0]} 0` : sizeStyles[size].padding,
    };

    const helperStyles: React.CSSProperties = {
      fontSize: theme.typography.fontSize.xs,
      color: displayError ? theme.colors.status.error : theme.colors.text.secondary,
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'center',
    };

    return (
      <div className={className} style={containerStyles}>
        {label && (
          <label htmlFor={props.id} style={labelStyles}>
            {label}
            {(required || showRequired) && (
              <span style={{ color: theme.colors.status.error, marginLeft: theme.spacing[1] }}>*</span>
            )}
          </label>
        )}

        <div style={inputWrapperStyles}>
          {prefix && (
            <span
              style={{
                display: 'flex',
                alignItems: 'center',
                paddingLeft: theme.spacing[3],
                color: theme.colors.text.secondary,
              }}
            >
              {prefix}
            </span>
          )}

          <input
            ref={ref}
            value={value}
            onChange={handleChange}
            onBlur={handleBlur}
            onFocus={handleFocus}
            disabled={disabled}
            required={required}
            maxLength={maxLength}
            aria-invalid={hasError}
            aria-describedby={displayError ? `${props.id}-error` : helperText ? `${props.id}-helper` : undefined}
            style={inputStyles}
            {...props}
          />

          {suffix && (
            <span
              style={{
                display: 'flex',
                alignItems: 'center',
                paddingRight: theme.spacing[3],
                color: theme.colors.text.secondary,
              }}
            >
              {suffix}
            </span>
          )}
        </div>

        {(displayError || helperText || showCount) && (
          <div style={helperStyles}>
            <span id={displayError ? `${props.id}-error` : `${props.id}-helper`}>
              {displayError || helperText || ' '}
            </span>
            {showCount && maxLength && (
              <span style={{ color: theme.colors.text.tertiary }}>
                {value.length}/{maxLength}
              </span>
            )}
          </div>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';

// Common mask functions
export const InputMasks = {
  phone: (value: string): string => {
    const digits = value.replace(/\D/g, '');
    if (digits.length <= 3) return digits;
    if (digits.length <= 6) return `(${digits.slice(0, 3)}) ${digits.slice(3)}`;
    return `(${digits.slice(0, 3)}) ${digits.slice(3, 6)}-${digits.slice(6, 10)}`;
  },

  creditCard: (value: string): string => {
    const digits = value.replace(/\D/g, '');
    return digits.match(/.{1,4}/g)?.join(' ') || digits;
  },

  date: (value: string): string => {
    const digits = value.replace(/\D/g, '');
    if (digits.length <= 2) return digits;
    if (digits.length <= 4) return `${digits.slice(0, 2)}/${digits.slice(2)}`;
    return `${digits.slice(0, 2)}/${digits.slice(2, 4)}/${digits.slice(4, 8)}`;
  },

  currency: (value: string): string => {
    const digits = value.replace(/[^\d.]/g, '');
    const parts = digits.split('.');
    parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ',');
    return parts.length > 1 ? `${parts[0]}.${parts[1].slice(0, 2)}` : parts[0];
  },
};
