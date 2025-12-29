/**
 * Enterprise Button Component
 * Features: Multiple variants, sizes, loading states, icons, full accessibility
 */

import React, { forwardRef, ButtonHTMLAttributes, ReactNode } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets, animationPresets } from './styles/animations';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'success';
export type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Button variant */
  variant?: ButtonVariant;
  /** Button size */
  size?: ButtonSize;
  /** Loading state */
  loading?: boolean;
  /** Disabled state */
  disabled?: boolean;
  /** Full width */
  fullWidth?: boolean;
  /** Icon before text */
  leftIcon?: ReactNode;
  /** Icon after text */
  rightIcon?: ReactNode;
  /** Children */
  children?: ReactNode;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      loading = false,
      disabled = false,
      fullWidth = false,
      leftIcon,
      rightIcon,
      children,
      className = '',
      style = {},
      ...props
    },
    ref
  ) => {
    const { theme } = useTheme();

    const sizeStyles = {
      sm: {
        padding: `${theme.spacing[1]} ${theme.spacing[3]}`,
        fontSize: theme.typography.fontSize.sm,
        height: '32px',
      },
      md: {
        padding: `${theme.spacing[2]} ${theme.spacing[4]}`,
        fontSize: theme.typography.fontSize.base,
        height: '40px',
      },
      lg: {
        padding: `${theme.spacing[3]} ${theme.spacing[5]}`,
        fontSize: theme.typography.fontSize.lg,
        height: '48px',
      },
    };

    const getVariantStyles = () => {
      const baseStyles = {
        border: `1px solid transparent`,
        cursor: disabled || loading ? 'not-allowed' : 'pointer',
        opacity: disabled ? 0.6 : 1,
      };

      switch (variant) {
        case 'primary':
          return {
            ...baseStyles,
            backgroundColor: theme.colors.interactive.primary,
            color: theme.colors.text.inverse,
            '&:hover': !disabled && !loading && {
              backgroundColor: theme.colors.interactive.primaryHover,
            },
            '&:active': !disabled && !loading && {
              backgroundColor: theme.colors.interactive.primaryActive,
            },
          };
        case 'secondary':
          return {
            ...baseStyles,
            backgroundColor: theme.colors.interactive.secondary,
            color: theme.colors.text.primary,
            borderColor: theme.colors.border.primary,
            '&:hover': !disabled && !loading && {
              backgroundColor: theme.colors.interactive.secondaryHover,
            },
          };
        case 'ghost':
          return {
            ...baseStyles,
            backgroundColor: 'transparent',
            color: theme.colors.text.primary,
            '&:hover': !disabled && !loading && {
              backgroundColor: theme.colors.background.secondary,
            },
          };
        case 'danger':
          return {
            ...baseStyles,
            backgroundColor: theme.colors.interactive.danger,
            color: theme.colors.text.inverse,
            '&:hover': !disabled && !loading && {
              backgroundColor: theme.colors.interactive.dangerHover,
            },
          };
        case 'success':
          return {
            ...baseStyles,
            backgroundColor: theme.colors.status.success,
            color: theme.colors.text.inverse,
            '&:hover': !disabled && !loading && {
              filter: 'brightness(1.1)',
            },
          };
        default:
          return baseStyles;
      }
    };

    const buttonStyles: React.CSSProperties = {
      display: 'inline-flex',
      alignItems: 'center',
      justifyContent: 'center',
      gap: theme.spacing[2],
      fontFamily: theme.typography.fontFamily.sans,
      fontWeight: theme.typography.fontWeight.medium,
      borderRadius: theme.borderRadius.md,
      transition: transitionPresets.colors,
      outline: 'none',
      position: 'relative',
      userSelect: 'none',
      width: fullWidth ? '100%' : 'auto',
      ...sizeStyles[size],
      ...getVariantStyles(),
      ...style,
    };

    const LoadingSpinner = () => (
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        style={{
          animation: animationPresets.spin,
        }}
      >
        <circle
          cx="8"
          cy="8"
          r="6"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeDasharray="30"
          opacity="0.25"
        />
        <circle
          cx="8"
          cy="8"
          r="6"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeDasharray="15"
          opacity="1"
        />
      </svg>
    );

    return (
      <button
        ref={ref}
        disabled={disabled || loading}
        className={className}
        style={buttonStyles}
        onFocus={(e) => {
          e.currentTarget.style.outline = `2px solid ${theme.colors.border.focus}`;
          e.currentTarget.style.outlineOffset = '2px';
          props.onFocus?.(e);
        }}
        onBlur={(e) => {
          e.currentTarget.style.outline = 'none';
          props.onBlur?.(e);
        }}
        aria-disabled={disabled || loading}
        aria-busy={loading}
        {...props}
      >
        {loading && <LoadingSpinner />}
        {!loading && leftIcon && <span style={{ display: 'flex', alignItems: 'center' }}>{leftIcon}</span>}
        {children && <span>{children}</span>}
        {!loading && rightIcon && <span style={{ display: 'flex', alignItems: 'center' }}>{rightIcon}</span>}
      </button>
    );
  }
);

Button.displayName = 'Button';
