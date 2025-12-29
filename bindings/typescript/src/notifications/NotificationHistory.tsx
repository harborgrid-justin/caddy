/**
 * CADDY v0.4.0 - Notification History
 * Delivery history and tracking across all channels
 */

import React, { useState, useCallback, useEffect } from 'react';
import { NotificationDelivery, NotificationChannel } from './types';

interface NotificationHistoryProps {
  tenantId?: string;
  userId?: string;
  apiUrl?: string;
}

export const NotificationHistory: React.FC<NotificationHistoryProps> = ({
  tenantId,
  userId,
  apiUrl = '/api/notifications/history'
}) => {
  const [deliveries, setDeliveries] = useState<NotificationDelivery[]>([]);
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState({
    channel: '',
    status: '',
    dateFrom: '',
    dateTo: '',
    search: ''
  });
  const [selectedDelivery, setSelectedDelivery] = useState<NotificationDelivery | null>(null);

  const fetchHistory = useCallback(async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams({
        ...(tenantId && { tenantId }),
        ...(userId && { userId }),
        ...(filter.channel && { channel: filter.channel }),
        ...(filter.status && { status: filter.status }),
        ...(filter.dateFrom && { dateFrom: filter.dateFrom }),
        ...(filter.dateTo && { dateTo: filter.dateTo }),
        ...(filter.search && { search: filter.search })
      });

      const response = await fetch(`${apiUrl}?${params}`, {
        credentials: 'include'
      });
      const data = await response.json();
      setDeliveries(data.deliveries || []);
    } catch (err) {
      console.error('Error fetching history:', err);
    } finally {
      setLoading(false);
    }
  }, [apiUrl, tenantId, userId, filter]);

  useEffect(() => {
    fetchHistory();
  }, [fetchHistory]);

  const handleRetry = useCallback(async (deliveryId: string) => {
    try {
      await fetch(`${apiUrl}/${deliveryId}/retry`, {
        method: 'POST',
        credentials: 'include'
      });
      await fetchHistory();
    } catch (err) {
      console.error('Error retrying delivery:', err);
      alert('Failed to retry delivery');
    }
  }, [apiUrl, fetchHistory]);

  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'delivered':
        return '#10b981';
      case 'sent':
        return '#3b82f6';
      case 'pending':
        return '#f59e0b';
      case 'failed':
      case 'bounced':
        return '#dc2626';
      default:
        return '#6b7280';
    }
  };

  const getChannelIcon = (channel: NotificationChannel): string => {
    const icons: Record<NotificationChannel, string> = {
      [NotificationChannel.IN_APP]: '🔔',
      [NotificationChannel.EMAIL]: '📧',
      [NotificationChannel.SMS]: '💬',
      [NotificationChannel.PUSH]: '📱',
      [NotificationChannel.SLACK]: '💼',
      [NotificationChannel.TEAMS]: '👥',
      [NotificationChannel.WEBHOOK]: '🔗'
    };
    return icons[channel];
  };

  const formatTimestamp = (date: Date): string => {
    return new Date(date).toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  const stats = {
    total: deliveries.length,
    delivered: deliveries.filter(d => d.status === 'delivered').length,
    failed: deliveries.filter(d => d.status === 'failed' || d.status === 'bounced').length,
    pending: deliveries.filter(d => d.status === 'pending').length,
    deliveryRate: deliveries.length > 0
      ? ((deliveries.filter(d => d.status === 'delivered').length / deliveries.length) * 100).toFixed(1)
      : '0.0'
  };

  return (
    <div style={{ padding: '24px', maxWidth: '1400px', margin: '0 auto' }}>
      <div style={{ marginBottom: '24px' }}>
        <h2 style={{ margin: '0 0 4px 0', fontSize: '20px', fontWeight: '600', color: '#111827' }}>
          Notification History
        </h2>
        <p style={{ margin: 0, fontSize: '14px', color: '#6b7280' }}>
          Track delivery status across all channels
        </p>
      </div>

      {/* Stats */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '16px', marginBottom: '24px' }}>
        <div style={{ padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#ffffff' }}>
          <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
            TOTAL DELIVERIES
          </div>
          <div style={{ fontSize: '28px', fontWeight: '700', color: '#111827' }}>
            {stats.total}
          </div>
        </div>
        <div style={{ padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#ffffff' }}>
          <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
            DELIVERED
          </div>
          <div style={{ fontSize: '28px', fontWeight: '700', color: '#10b981' }}>
            {stats.delivered}
          </div>
        </div>
        <div style={{ padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#ffffff' }}>
          <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
            FAILED
          </div>
          <div style={{ fontSize: '28px', fontWeight: '700', color: '#dc2626' }}>
            {stats.failed}
          </div>
        </div>
        <div style={{ padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#ffffff' }}>
          <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
            DELIVERY RATE
          </div>
          <div style={{ fontSize: '28px', fontWeight: '700', color: '#111827' }}>
            {stats.deliveryRate}%
          </div>
        </div>
      </div>

      {/* Filters */}
      <div style={{ padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#f9fafb', marginBottom: '24px' }}>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px' }}>
          <div>
            <label style={{ display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
              Channel
            </label>
            <select
              value={filter.channel}
              onChange={(e) => setFilter({ ...filter, channel: e.target.value })}
              style={{
                width: '100%',
                padding: '8px',
                fontSize: '13px',
                border: '1px solid #d1d5db',
                borderRadius: '4px',
                backgroundColor: '#ffffff'
              }}
            >
              <option value="">All Channels</option>
              {Object.values(NotificationChannel).map(channel => (
                <option key={channel} value={channel}>{channel}</option>
              ))}
            </select>
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
              Status
            </label>
            <select
              value={filter.status}
              onChange={(e) => setFilter({ ...filter, status: e.target.value })}
              style={{
                width: '100%',
                padding: '8px',
                fontSize: '13px',
                border: '1px solid #d1d5db',
                borderRadius: '4px',
                backgroundColor: '#ffffff'
              }}
            >
              <option value="">All Status</option>
              <option value="pending">Pending</option>
              <option value="sent">Sent</option>
              <option value="delivered">Delivered</option>
              <option value="failed">Failed</option>
              <option value="bounced">Bounced</option>
            </select>
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
              From Date
            </label>
            <input
              type="date"
              value={filter.dateFrom}
              onChange={(e) => setFilter({ ...filter, dateFrom: e.target.value })}
              style={{
                width: '100%',
                padding: '8px',
                fontSize: '13px',
                border: '1px solid #d1d5db',
                borderRadius: '4px',
                backgroundColor: '#ffffff'
              }}
            />
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
              To Date
            </label>
            <input
              type="date"
              value={filter.dateTo}
              onChange={(e) => setFilter({ ...filter, dateTo: e.target.value })}
              style={{
                width: '100%',
                padding: '8px',
                fontSize: '13px',
                border: '1px solid #d1d5db',
                borderRadius: '4px',
                backgroundColor: '#ffffff'
              }}
            />
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
              Search
            </label>
            <input
              type="text"
              value={filter.search}
              onChange={(e) => setFilter({ ...filter, search: e.target.value })}
              placeholder="Recipient address..."
              style={{
                width: '100%',
                padding: '8px',
                fontSize: '13px',
                border: '1px solid #d1d5db',
                borderRadius: '4px',
                backgroundColor: '#ffffff'
              }}
            />
          </div>
        </div>
      </div>

      {/* History Table */}
      {loading ? (
        <div style={{ padding: '48px', textAlign: 'center', color: '#6b7280' }}>
          Loading history...
        </div>
      ) : deliveries.length === 0 ? (
        <div style={{ padding: '48px', textAlign: 'center', color: '#6b7280' }}>
          <div style={{ fontSize: '48px', marginBottom: '16px' }}>📊</div>
          <div style={{ fontSize: '16px', fontWeight: '500', marginBottom: '8px' }}>
            No delivery history
          </div>
          <div style={{ fontSize: '14px' }}>
            Delivery records will appear here
          </div>
        </div>
      ) : (
        <div style={{ border: '1px solid #e5e7eb', borderRadius: '8px', overflow: 'hidden' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', backgroundColor: '#ffffff' }}>
            <thead style={{ backgroundColor: '#f9fafb' }}>
              <tr>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' }}>
                  CHANNEL
                </th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' }}>
                  RECIPIENT
                </th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' }}>
                  STATUS
                </th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' }}>
                  ATTEMPTS
                </th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' }}>
                  CREATED
                </th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' }}>
                  DELIVERED
                </th>
                <th style={{ padding: '12px 16px', textAlign: 'right', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' }}>
                  ACTIONS
                </th>
              </tr>
            </thead>
            <tbody>
              {deliveries.map((delivery) => (
                <tr
                  key={delivery.id}
                  style={{ cursor: 'pointer', transition: 'background-color 0.2s' }}
                  onClick={() => setSelectedDelivery(delivery)}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.backgroundColor = '#f9fafb';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.backgroundColor = '#ffffff';
                  }}
                >
                  <td style={{ padding: '12px 16px', borderBottom: '1px solid #e5e7eb' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <span style={{ fontSize: '20px' }}>{getChannelIcon(delivery.channel)}</span>
                      <span style={{ fontSize: '13px', color: '#111827', textTransform: 'capitalize' }}>
                        {delivery.channel.replace('_', ' ')}
                      </span>
                    </div>
                  </td>
                  <td style={{ padding: '12px 16px', fontSize: '13px', color: '#111827', borderBottom: '1px solid #e5e7eb' }}>
                    {delivery.recipientAddress}
                  </td>
                  <td style={{ padding: '12px 16px', borderBottom: '1px solid #e5e7eb' }}>
                    <span
                      style={{
                        padding: '4px 8px',
                        fontSize: '11px',
                        fontWeight: '600',
                        borderRadius: '12px',
                        backgroundColor: `${getStatusColor(delivery.status)}20`,
                        color: getStatusColor(delivery.status),
                        textTransform: 'uppercase'
                      }}
                    >
                      {delivery.status}
                    </span>
                  </td>
                  <td style={{ padding: '12px 16px', fontSize: '13px', color: '#6b7280', borderBottom: '1px solid #e5e7eb' }}>
                    {delivery.attempts} / {delivery.maxAttempts}
                  </td>
                  <td style={{ padding: '12px 16px', fontSize: '13px', color: '#6b7280', borderBottom: '1px solid #e5e7eb' }}>
                    {formatTimestamp(delivery.createdAt)}
                  </td>
                  <td style={{ padding: '12px 16px', fontSize: '13px', color: '#6b7280', borderBottom: '1px solid #e5e7eb' }}>
                    {delivery.deliveredAt ? formatTimestamp(delivery.deliveredAt) : '-'}
                  </td>
                  <td style={{ padding: '12px 16px', borderBottom: '1px solid #e5e7eb', textAlign: 'right' }}>
                    {(delivery.status === 'failed' || delivery.status === 'bounced') && delivery.attempts < delivery.maxAttempts && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRetry(delivery.id);
                        }}
                        style={{
                          padding: '4px 12px',
                          fontSize: '12px',
                          fontWeight: '500',
                          border: '1px solid #3b82f6',
                          borderRadius: '4px',
                          backgroundColor: '#ffffff',
                          color: '#3b82f6',
                          cursor: 'pointer'
                        }}
                      >
                        Retry
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Detail Modal */}
      {selectedDelivery && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 10000
          }}
          onClick={() => setSelectedDelivery(null)}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              backgroundColor: '#ffffff',
              borderRadius: '8px',
              padding: '24px',
              maxWidth: '600px',
              width: '90%',
              maxHeight: '80vh',
              overflowY: 'auto',
              boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
            }}
          >
            <h3 style={{ margin: '0 0 24px 0', fontSize: '20px', fontWeight: '600', color: '#111827' }}>
              Delivery Details
            </h3>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
              <div>
                <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                  DELIVERY ID
                </div>
                <div style={{ fontSize: '13px', color: '#111827', fontFamily: 'monospace' }}>
                  {selectedDelivery.id}
                </div>
              </div>

              <div>
                <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                  NOTIFICATION ID
                </div>
                <div style={{ fontSize: '13px', color: '#111827', fontFamily: 'monospace' }}>
                  {selectedDelivery.notificationId}
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' }}>
                <div>
                  <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                    CHANNEL
                  </div>
                  <div style={{ fontSize: '13px', color: '#111827', textTransform: 'capitalize' }}>
                    {selectedDelivery.channel.replace('_', ' ')}
                  </div>
                </div>
                <div>
                  <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                    STATUS
                  </div>
                  <span
                    style={{
                      padding: '4px 8px',
                      fontSize: '11px',
                      fontWeight: '600',
                      borderRadius: '12px',
                      backgroundColor: `${getStatusColor(selectedDelivery.status)}20`,
                      color: getStatusColor(selectedDelivery.status),
                      textTransform: 'uppercase'
                    }}
                  >
                    {selectedDelivery.status}
                  </span>
                </div>
              </div>

              <div>
                <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                  RECIPIENT
                </div>
                <div style={{ fontSize: '13px', color: '#111827' }}>
                  {selectedDelivery.recipientAddress}
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' }}>
                <div>
                  <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                    ATTEMPTS
                  </div>
                  <div style={{ fontSize: '13px', color: '#111827' }}>
                    {selectedDelivery.attempts} / {selectedDelivery.maxAttempts}
                  </div>
                </div>
                <div>
                  <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                    LAST ATTEMPT
                  </div>
                  <div style={{ fontSize: '13px', color: '#111827' }}>
                    {selectedDelivery.lastAttemptAt ? formatTimestamp(selectedDelivery.lastAttemptAt) : '-'}
                  </div>
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' }}>
                <div>
                  <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                    CREATED
                  </div>
                  <div style={{ fontSize: '13px', color: '#111827' }}>
                    {formatTimestamp(selectedDelivery.createdAt)}
                  </div>
                </div>
                <div>
                  <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                    DELIVERED
                  </div>
                  <div style={{ fontSize: '13px', color: '#111827' }}>
                    {selectedDelivery.deliveredAt ? formatTimestamp(selectedDelivery.deliveredAt) : '-'}
                  </div>
                </div>
              </div>

              {selectedDelivery.errorMessage && (
                <div>
                  <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                    ERROR MESSAGE
                  </div>
                  <div
                    style={{
                      padding: '12px',
                      fontSize: '13px',
                      color: '#dc2626',
                      backgroundColor: '#fee2e2',
                      borderRadius: '4px',
                      fontFamily: 'monospace'
                    }}
                  >
                    {selectedDelivery.errorMessage}
                  </div>
                </div>
              )}

              {selectedDelivery.metadata && Object.keys(selectedDelivery.metadata).length > 0 && (
                <div>
                  <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                    METADATA
                  </div>
                  <pre
                    style={{
                      padding: '12px',
                      fontSize: '12px',
                      backgroundColor: '#f9fafb',
                      borderRadius: '4px',
                      overflow: 'auto',
                      fontFamily: 'monospace',
                      margin: 0
                    }}
                  >
                    {JSON.stringify(selectedDelivery.metadata, null, 2)}
                  </pre>
                </div>
              )}
            </div>

            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' }}>
              <button
                onClick={() => setSelectedDelivery(null)}
                style={{
                  padding: '10px 20px',
                  fontSize: '14px',
                  fontWeight: '500',
                  border: '1px solid #d1d5db',
                  borderRadius: '6px',
                  backgroundColor: '#ffffff',
                  color: '#374151',
                  cursor: 'pointer'
                }}
              >
                Close
              </button>
              {(selectedDelivery.status === 'failed' || selectedDelivery.status === 'bounced') &&
                selectedDelivery.attempts < selectedDelivery.maxAttempts && (
                  <button
                    onClick={() => {
                      handleRetry(selectedDelivery.id);
                      setSelectedDelivery(null);
                    }}
                    style={{
                      padding: '10px 20px',
                      fontSize: '14px',
                      fontWeight: '500',
                      border: 'none',
                      borderRadius: '6px',
                      backgroundColor: '#3b82f6',
                      color: '#ffffff',
                      cursor: 'pointer'
                    }}
                  >
                    Retry Delivery
                  </button>
                )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
