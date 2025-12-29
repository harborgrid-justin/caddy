/**
 * Enterprise Modal Component
 * Features: Accessible dialog, animations, backdrop, keyboard handling, focus trap
 */

import React, { useEffect, useRef, ReactNode, CSSProperties } from 'react';
import { createPortal } from 'react-dom';
import { useTheme } from './styles/theme';
import { animationPresets, transitionPresets } from './styles/animations';

export type ModalSize = 'sm' | 'md' | 'lg' | 'xl' | 'full';

export interface ModalProps {
  /** Whether modal is open */
  isOpen: boolean;
  /** Close handler */
  onClose: () => void;
  /** Modal title */
  title?: ReactNode;
  /** Modal content */
  children: ReactNode;
  /** Modal footer */
  footer?: ReactNode;
  /** Modal size */
  size?: ModalSize;
  /** Close on backdrop click */
  closeOnBackdropClick?: boolean;
  /** Close on escape key */
  closeOnEscape?: boolean;
  /** Show close button */
  showCloseButton?: boolean;
  /** Custom className */
  className?: string;
  /** Custom styles */
  style?: CSSProperties;
  /** Prevent body scroll when open */
  preventScroll?: boolean;
}

export const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  children,
  footer,
  size = 'md',
  closeOnBackdropClick = true,
  closeOnEscape = true,
  showCloseButton = true,
  className = '',
  style = {},
  preventScroll = true,
}) => {
  const { theme } = useTheme();
  const modalRef = useRef<HTMLDivElement>(null);
  const previousActiveElement = useRef<HTMLElement | null>(null);

  const sizeStyles: Record<ModalSize, CSSProperties> = {
    sm: { maxWidth: '400px', width: '90%' },
    md: { maxWidth: '600px', width: '90%' },
    lg: { maxWidth: '800px', width: '90%' },
    xl: { maxWidth: '1200px', width: '90%' },
    full: { width: '100%', height: '100%', maxWidth: 'none', borderRadius: 0 },
  };

  // Handle escape key
  useEffect(() => {
    if (!isOpen || !closeOnEscape) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, closeOnEscape, onClose]);

  // Prevent body scroll
  useEffect(() => {
    if (!isOpen || !preventScroll) return;

    const originalOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';

    return () => {
      document.body.style.overflow = originalOverflow;
    };
  }, [isOpen, preventScroll]);

  // Focus trap and restoration
  useEffect(() => {
    if (!isOpen) return;

    // Save previously focused element
    previousActiveElement.current = document.activeElement as HTMLElement;

    // Focus modal
    if (modalRef.current) {
      modalRef.current.focus();
    }

    // Restore focus on close
    return () => {
      if (previousActiveElement.current) {
        previousActiveElement.current.focus();
      }
    };
  }, [isOpen]);

  // Focus trap
  useEffect(() => {
    if (!isOpen || !modalRef.current) return;

    const modal = modalRef.current;
    const focusableElements = modal.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );

    if (focusableElements.length === 0) return;

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    const handleTab = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;

      if (e.shiftKey) {
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement.focus();
        }
      } else {
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement.focus();
        }
      }
    };

    modal.addEventListener('keydown', handleTab);
    return () => modal.removeEventListener('keydown', handleTab);
  }, [isOpen]);

  if (!isOpen) return null;

  const backdropStyles: CSSProperties = {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: theme.colors.background.overlay,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: theme.zIndex.modalBackdrop,
    animation: animationPresets.fadeIn,
    padding: theme.spacing[4],
  };

  const modalStyles: CSSProperties = {
    backgroundColor: theme.colors.background.elevated,
    borderRadius: theme.borderRadius.lg,
    boxShadow: theme.shadows['2xl'],
    display: 'flex',
    flexDirection: 'column',
    maxHeight: size === 'full' ? '100%' : '90vh',
    outline: 'none',
    animation: animationPresets.scaleIn,
    ...sizeStyles[size],
    ...style,
  };

  const headerStyles: CSSProperties = {
    padding: theme.spacing[6],
    borderBottom: `1px solid ${theme.colors.border.secondary}`,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    flexShrink: 0,
  };

  const titleStyles: CSSProperties = {
    fontSize: theme.typography.fontSize['2xl'],
    fontWeight: theme.typography.fontWeight.semibold,
    color: theme.colors.text.primary,
    margin: 0,
  };

  const closeButtonStyles: CSSProperties = {
    background: 'transparent',
    border: 'none',
    cursor: 'pointer',
    padding: theme.spacing[1],
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: theme.borderRadius.base,
    color: theme.colors.text.secondary,
    transition: transitionPresets.colors,
  };

  const contentStyles: CSSProperties = {
    padding: theme.spacing[6],
    overflowY: 'auto',
    flex: 1,
    color: theme.colors.text.primary,
  };

  const footerStyles: CSSProperties = {
    padding: theme.spacing[6],
    borderTop: `1px solid ${theme.colors.border.secondary}`,
    display: 'flex',
    gap: theme.spacing[3],
    justifyContent: 'flex-end',
    flexShrink: 0,
  };

  const CloseIcon = () => (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M18 6L6 18M6 6l12 12" strokeLinecap="round" />
    </svg>
  );

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (closeOnBackdropClick && e.target === e.currentTarget) {
      onClose();
    }
  };

  const modal = (
    <div
      style={backdropStyles}
      onClick={handleBackdropClick}
      aria-modal="true"
      role="dialog"
      aria-labelledby={title ? 'modal-title' : undefined}
    >
      <div
        ref={modalRef}
        className={className}
        style={modalStyles}
        onClick={(e) => e.stopPropagation()}
        tabIndex={-1}
      >
        {(title || showCloseButton) && (
          <div style={headerStyles}>
            {title && (
              <h2 id="modal-title" style={titleStyles}>
                {title}
              </h2>
            )}
            {showCloseButton && (
              <button
                onClick={onClose}
                aria-label="Close modal"
                style={closeButtonStyles}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = theme.colors.background.secondary;
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor = 'transparent';
                }}
              >
                <CloseIcon />
              </button>
            )}
          </div>
        )}

        <div style={contentStyles}>{children}</div>

        {footer && <div style={footerStyles}>{footer}</div>}
      </div>
    </div>
  );

  return createPortal(modal, document.body);
};

Modal.displayName = 'Modal';
