/**
 * CADDY v0.4.0 - Notification Bell Component
 * Header notification bell with badge and dropdown
 */

import React, { useState, useRef, useEffect, useCallback } from 'react';
import { NotificationList } from './NotificationList';
import { useNotifications, useUnreadCount, useUrgentNotifications } from './useNotifications';
import { NotificationPriority } from './types';

interface NotificationBellProps {
  position?: 'left' | 'right';
  showPreview?: boolean;
  previewCount?: number;
  animateOnNew?: boolean;
}

export const NotificationBell: React.FC<NotificationBellProps> = ({
  position = 'right',
  showPreview = true,
  previewCount = 5,
  animateOnNew = true
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [isAnimating, setIsAnimating] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const bellRef = useRef<HTMLButtonElement>(null);
  const prevUnreadCount = useRef<number>(0);

  const { notifications } = useNotifications();
  const unreadCount = useUnreadCount();
  const urgentNotifications = useUrgentNotifications();
  const hasUrgent = urgentNotifications.length > 0;

  // Handle click outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node) &&
        bellRef.current &&
        !bellRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  // Animate on new notification
  useEffect(() => {
    if (animateOnNew && unreadCount > prevUnreadCount.current && prevUnreadCount.current > 0) {
      setIsAnimating(true);
      setTimeout(() => setIsAnimating(false), 1000);
    }
    prevUnreadCount.current = unreadCount;
  }, [unreadCount, animateOnNew]);

  const handleToggle = useCallback(() => {
    setIsOpen(prev => !prev);
  }, []);

  const handleViewAll = useCallback(() => {
    setIsOpen(false);
    // Navigate to full notifications page
    window.location.href = '/notifications';
  }, []);

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      {/* Bell Button */}
      <button
        ref={bellRef}
        onClick={handleToggle}
        style={{
          position: 'relative',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          width: '40px',
          height: '40px',
          padding: 0,
          border: 'none',
          borderRadius: '8px',
          backgroundColor: isOpen ? '#f3f4f6' : 'transparent',
          cursor: 'pointer',
          transition: 'background-color 0.2s ease',
          animation: isAnimating ? 'bellRing 0.5s ease-in-out' : undefined
        }}
        onMouseEnter={(e) => {
          if (!isOpen) e.currentTarget.style.backgroundColor = '#f3f4f6';
        }}
        onMouseLeave={(e) => {
          if (!isOpen) e.currentTarget.style.backgroundColor = 'transparent';
        }}
        aria-label={`Notifications ${unreadCount > 0 ? `(${unreadCount} unread)` : ''}`}
      >
        {/* Bell Icon */}
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke={hasUrgent ? '#dc2626' : '#374151'}
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
          <path d="M13.73 21a2 2 0 0 1-3.46 0" />
        </svg>

        {/* Badge */}
        {unreadCount > 0 && (
          <div
            style={{
              position: 'absolute',
              top: '4px',
              right: '4px',
              minWidth: '18px',
              height: '18px',
              padding: '0 4px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              borderRadius: '9px',
              backgroundColor: hasUrgent ? '#dc2626' : '#3b82f6',
              color: '#ffffff',
              fontSize: '11px',
              fontWeight: '600',
              lineHeight: 1,
              animation: hasUrgent ? 'pulse 2s ease-in-out infinite' : undefined
            }}
          >
            {unreadCount > 99 ? '99+' : unreadCount}
          </div>
        )}

        {/* Urgent indicator */}
        {hasUrgent && (
          <div
            style={{
              position: 'absolute',
              bottom: '4px',
              right: '4px',
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              backgroundColor: '#dc2626',
              animation: 'pulse 2s ease-in-out infinite'
            }}
          />
        )}
      </button>

      {/* Dropdown */}
      {isOpen && (
        <div
          ref={dropdownRef}
          style={{
            position: 'absolute',
            top: '48px',
            [position]: '0',
            width: '400px',
            maxWidth: '90vw',
            backgroundColor: '#ffffff',
            borderRadius: '8px',
            boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
            border: '1px solid #e5e7eb',
            zIndex: 9999,
            animation: 'slideDown 0.2s ease-out'
          }}
        >
          {showPreview && notifications.length > 0 ? (
            <>
              <NotificationList
                showFilters={false}
                showGrouping={false}
                showBulkActions={false}
                compact={true}
                maxHeight="400px"
              />
              <div
                style={{
                  padding: '12px',
                  borderTop: '1px solid #e5e7eb',
                  textAlign: 'center'
                }}
              >
                <button
                  onClick={handleViewAll}
                  style={{
                    width: '100%',
                    padding: '8px 16px',
                    fontSize: '14px',
                    fontWeight: '500',
                    border: '1px solid #d1d5db',
                    borderRadius: '6px',
                    backgroundColor: '#ffffff',
                    color: '#374151',
                    cursor: 'pointer',
                    transition: 'background-color 0.2s'
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.backgroundColor = '#f9fafb';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.backgroundColor = '#ffffff';
                  }}
                >
                  View all notifications
                </button>
              </div>
            </>
          ) : (
            <div style={{ padding: '48px 24px', textAlign: 'center', color: '#6b7280' }}>
              <div style={{ fontSize: '48px', marginBottom: '16px' }}>🔔</div>
              <div style={{ fontSize: '16px', fontWeight: '500', marginBottom: '8px' }}>
                No notifications
              </div>
              <div style={{ fontSize: '14px' }}>You're all caught up!</div>
            </div>
          )}
        </div>
      )}

      {/* CSS Animations */}
      <style>{`
        @keyframes bellRing {
          0%, 100% { transform: rotate(0deg); }
          10%, 30%, 50%, 70%, 90% { transform: rotate(-10deg); }
          20%, 40%, 60%, 80% { transform: rotate(10deg); }
        }

        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.5; }
        }

        @keyframes slideDown {
          from {
            opacity: 0;
            transform: translateY(-10px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
      `}</style>
    </div>
  );
};
