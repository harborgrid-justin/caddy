import React, { useState, useCallback, useEffect } from 'react';
import { NotificationChannel } from './types';
export const NotificationHistory = ({ tenantId, userId, apiUrl = '/api/notifications/history' }) => {
    const [deliveries, setDeliveries] = useState([]);
    const [loading, setLoading] = useState(false);
    const [filter, setFilter] = useState({
        channel: '',
        status: '',
        dateFrom: '',
        dateTo: '',
        search: ''
    });
    const [selectedDelivery, setSelectedDelivery] = useState(null);
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
        }
        catch (err) {
            console.error('Error fetching history:', err);
        }
        finally {
            setLoading(false);
        }
    }, [apiUrl, tenantId, userId, filter]);
    useEffect(() => {
        fetchHistory();
    }, [fetchHistory]);
    const handleRetry = useCallback(async (deliveryId) => {
        try {
            await fetch(`${apiUrl}/${deliveryId}/retry`, {
                method: 'POST',
                credentials: 'include'
            });
            await fetchHistory();
        }
        catch (err) {
            console.error('Error retrying delivery:', err);
            alert('Failed to retry delivery');
        }
    }, [apiUrl, fetchHistory]);
    const getStatusColor = (status) => {
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
    const getChannelIcon = (channel) => {
        const icons = {
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
    const formatTimestamp = (date) => {
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
    return (React.createElement("div", { style: { padding: '24px', maxWidth: '1400px', margin: '0 auto' } },
        React.createElement("div", { style: { marginBottom: '24px' } },
            React.createElement("h2", { style: { margin: '0 0 4px 0', fontSize: '20px', fontWeight: '600', color: '#111827' } }, "Notification History"),
            React.createElement("p", { style: { margin: 0, fontSize: '14px', color: '#6b7280' } }, "Track delivery status across all channels")),
        React.createElement("div", { style: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '16px', marginBottom: '24px' } },
            React.createElement("div", { style: { padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#ffffff' } },
                React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "TOTAL DELIVERIES"),
                React.createElement("div", { style: { fontSize: '28px', fontWeight: '700', color: '#111827' } }, stats.total)),
            React.createElement("div", { style: { padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#ffffff' } },
                React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "DELIVERED"),
                React.createElement("div", { style: { fontSize: '28px', fontWeight: '700', color: '#10b981' } }, stats.delivered)),
            React.createElement("div", { style: { padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#ffffff' } },
                React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "FAILED"),
                React.createElement("div", { style: { fontSize: '28px', fontWeight: '700', color: '#dc2626' } }, stats.failed)),
            React.createElement("div", { style: { padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#ffffff' } },
                React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "DELIVERY RATE"),
                React.createElement("div", { style: { fontSize: '28px', fontWeight: '700', color: '#111827' } },
                    stats.deliveryRate,
                    "%"))),
        React.createElement("div", { style: { padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#f9fafb', marginBottom: '24px' } },
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px' } },
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Channel"),
                    React.createElement("select", { value: filter.channel, onChange: (e) => setFilter({ ...filter, channel: e.target.value }), style: {
                            width: '100%',
                            padding: '8px',
                            fontSize: '13px',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff'
                        } },
                        React.createElement("option", { value: "" }, "All Channels"),
                        Object.values(NotificationChannel).map(channel => (React.createElement("option", { key: channel, value: channel }, channel))))),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Status"),
                    React.createElement("select", { value: filter.status, onChange: (e) => setFilter({ ...filter, status: e.target.value }), style: {
                            width: '100%',
                            padding: '8px',
                            fontSize: '13px',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff'
                        } },
                        React.createElement("option", { value: "" }, "All Status"),
                        React.createElement("option", { value: "pending" }, "Pending"),
                        React.createElement("option", { value: "sent" }, "Sent"),
                        React.createElement("option", { value: "delivered" }, "Delivered"),
                        React.createElement("option", { value: "failed" }, "Failed"),
                        React.createElement("option", { value: "bounced" }, "Bounced"))),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "From Date"),
                    React.createElement("input", { type: "date", value: filter.dateFrom, onChange: (e) => setFilter({ ...filter, dateFrom: e.target.value }), style: {
                            width: '100%',
                            padding: '8px',
                            fontSize: '13px',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff'
                        } })),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "To Date"),
                    React.createElement("input", { type: "date", value: filter.dateTo, onChange: (e) => setFilter({ ...filter, dateTo: e.target.value }), style: {
                            width: '100%',
                            padding: '8px',
                            fontSize: '13px',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff'
                        } })),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Search"),
                    React.createElement("input", { type: "text", value: filter.search, onChange: (e) => setFilter({ ...filter, search: e.target.value }), placeholder: "Recipient address...", style: {
                            width: '100%',
                            padding: '8px',
                            fontSize: '13px',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff'
                        } })))),
        loading ? (React.createElement("div", { style: { padding: '48px', textAlign: 'center', color: '#6b7280' } }, "Loading history...")) : deliveries.length === 0 ? (React.createElement("div", { style: { padding: '48px', textAlign: 'center', color: '#6b7280' } },
            React.createElement("div", { style: { fontSize: '48px', marginBottom: '16px' } }, "\uD83D\uDCCA"),
            React.createElement("div", { style: { fontSize: '16px', fontWeight: '500', marginBottom: '8px' } }, "No delivery history"),
            React.createElement("div", { style: { fontSize: '14px' } }, "Delivery records will appear here"))) : (React.createElement("div", { style: { border: '1px solid #e5e7eb', borderRadius: '8px', overflow: 'hidden' } },
            React.createElement("table", { style: { width: '100%', borderCollapse: 'collapse', backgroundColor: '#ffffff' } },
                React.createElement("thead", { style: { backgroundColor: '#f9fafb' } },
                    React.createElement("tr", null,
                        React.createElement("th", { style: { padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' } }, "CHANNEL"),
                        React.createElement("th", { style: { padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' } }, "RECIPIENT"),
                        React.createElement("th", { style: { padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' } }, "STATUS"),
                        React.createElement("th", { style: { padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' } }, "ATTEMPTS"),
                        React.createElement("th", { style: { padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' } }, "CREATED"),
                        React.createElement("th", { style: { padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' } }, "DELIVERED"),
                        React.createElement("th", { style: { padding: '12px 16px', textAlign: 'right', fontSize: '11px', fontWeight: '600', color: '#6b7280', borderBottom: '1px solid #e5e7eb' } }, "ACTIONS"))),
                React.createElement("tbody", null, deliveries.map((delivery) => (React.createElement("tr", { key: delivery.id, style: { cursor: 'pointer', transition: 'background-color 0.2s' }, onClick: () => setSelectedDelivery(delivery), onMouseEnter: (e) => {
                        e.currentTarget.style.backgroundColor = '#f9fafb';
                    }, onMouseLeave: (e) => {
                        e.currentTarget.style.backgroundColor = '#ffffff';
                    } },
                    React.createElement("td", { style: { padding: '12px 16px', borderBottom: '1px solid #e5e7eb' } },
                        React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                            React.createElement("span", { style: { fontSize: '20px' } }, getChannelIcon(delivery.channel)),
                            React.createElement("span", { style: { fontSize: '13px', color: '#111827', textTransform: 'capitalize' } }, delivery.channel.replace('_', ' ')))),
                    React.createElement("td", { style: { padding: '12px 16px', fontSize: '13px', color: '#111827', borderBottom: '1px solid #e5e7eb' } }, delivery.recipientAddress),
                    React.createElement("td", { style: { padding: '12px 16px', borderBottom: '1px solid #e5e7eb' } },
                        React.createElement("span", { style: {
                                padding: '4px 8px',
                                fontSize: '11px',
                                fontWeight: '600',
                                borderRadius: '12px',
                                backgroundColor: `${getStatusColor(delivery.status)}20`,
                                color: getStatusColor(delivery.status),
                                textTransform: 'uppercase'
                            } }, delivery.status)),
                    React.createElement("td", { style: { padding: '12px 16px', fontSize: '13px', color: '#6b7280', borderBottom: '1px solid #e5e7eb' } },
                        delivery.attempts,
                        " / ",
                        delivery.maxAttempts),
                    React.createElement("td", { style: { padding: '12px 16px', fontSize: '13px', color: '#6b7280', borderBottom: '1px solid #e5e7eb' } }, formatTimestamp(delivery.createdAt)),
                    React.createElement("td", { style: { padding: '12px 16px', fontSize: '13px', color: '#6b7280', borderBottom: '1px solid #e5e7eb' } }, delivery.deliveredAt ? formatTimestamp(delivery.deliveredAt) : '-'),
                    React.createElement("td", { style: { padding: '12px 16px', borderBottom: '1px solid #e5e7eb', textAlign: 'right' } }, (delivery.status === 'failed' || delivery.status === 'bounced') && delivery.attempts < delivery.maxAttempts && (React.createElement("button", { onClick: (e) => {
                            e.stopPropagation();
                            handleRetry(delivery.id);
                        }, style: {
                            padding: '4px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #3b82f6',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#3b82f6',
                            cursor: 'pointer'
                        } }, "Retry")))))))))),
        selectedDelivery && (React.createElement("div", { style: {
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
            }, onClick: () => setSelectedDelivery(null) },
            React.createElement("div", { onClick: (e) => e.stopPropagation(), style: {
                    backgroundColor: '#ffffff',
                    borderRadius: '8px',
                    padding: '24px',
                    maxWidth: '600px',
                    width: '90%',
                    maxHeight: '80vh',
                    overflowY: 'auto',
                    boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
                } },
                React.createElement("h3", { style: { margin: '0 0 24px 0', fontSize: '20px', fontWeight: '600', color: '#111827' } }, "Delivery Details"),
                React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '16px' } },
                    React.createElement("div", null,
                        React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "DELIVERY ID"),
                        React.createElement("div", { style: { fontSize: '13px', color: '#111827', fontFamily: 'monospace' } }, selectedDelivery.id)),
                    React.createElement("div", null,
                        React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "NOTIFICATION ID"),
                        React.createElement("div", { style: { fontSize: '13px', color: '#111827', fontFamily: 'monospace' } }, selectedDelivery.notificationId)),
                    React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' } },
                        React.createElement("div", null,
                            React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "CHANNEL"),
                            React.createElement("div", { style: { fontSize: '13px', color: '#111827', textTransform: 'capitalize' } }, selectedDelivery.channel.replace('_', ' '))),
                        React.createElement("div", null,
                            React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "STATUS"),
                            React.createElement("span", { style: {
                                    padding: '4px 8px',
                                    fontSize: '11px',
                                    fontWeight: '600',
                                    borderRadius: '12px',
                                    backgroundColor: `${getStatusColor(selectedDelivery.status)}20`,
                                    color: getStatusColor(selectedDelivery.status),
                                    textTransform: 'uppercase'
                                } }, selectedDelivery.status))),
                    React.createElement("div", null,
                        React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "RECIPIENT"),
                        React.createElement("div", { style: { fontSize: '13px', color: '#111827' } }, selectedDelivery.recipientAddress)),
                    React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' } },
                        React.createElement("div", null,
                            React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "ATTEMPTS"),
                            React.createElement("div", { style: { fontSize: '13px', color: '#111827' } },
                                selectedDelivery.attempts,
                                " / ",
                                selectedDelivery.maxAttempts)),
                        React.createElement("div", null,
                            React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "LAST ATTEMPT"),
                            React.createElement("div", { style: { fontSize: '13px', color: '#111827' } }, selectedDelivery.lastAttemptAt ? formatTimestamp(selectedDelivery.lastAttemptAt) : '-'))),
                    React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' } },
                        React.createElement("div", null,
                            React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "CREATED"),
                            React.createElement("div", { style: { fontSize: '13px', color: '#111827' } }, formatTimestamp(selectedDelivery.createdAt))),
                        React.createElement("div", null,
                            React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "DELIVERED"),
                            React.createElement("div", { style: { fontSize: '13px', color: '#111827' } }, selectedDelivery.deliveredAt ? formatTimestamp(selectedDelivery.deliveredAt) : '-'))),
                    selectedDelivery.errorMessage && (React.createElement("div", null,
                        React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "ERROR MESSAGE"),
                        React.createElement("div", { style: {
                                padding: '12px',
                                fontSize: '13px',
                                color: '#dc2626',
                                backgroundColor: '#fee2e2',
                                borderRadius: '4px',
                                fontFamily: 'monospace'
                            } }, selectedDelivery.errorMessage))),
                    selectedDelivery.metadata && Object.keys(selectedDelivery.metadata).length > 0 && (React.createElement("div", null,
                        React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } }, "METADATA"),
                        React.createElement("pre", { style: {
                                padding: '12px',
                                fontSize: '12px',
                                backgroundColor: '#f9fafb',
                                borderRadius: '4px',
                                overflow: 'auto',
                                fontFamily: 'monospace',
                                margin: 0
                            } }, JSON.stringify(selectedDelivery.metadata, null, 2))))),
                React.createElement("div", { style: { display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' } },
                    React.createElement("button", { onClick: () => setSelectedDelivery(null), style: {
                            padding: '10px 20px',
                            fontSize: '14px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '6px',
                            backgroundColor: '#ffffff',
                            color: '#374151',
                            cursor: 'pointer'
                        } }, "Close"),
                    (selectedDelivery.status === 'failed' || selectedDelivery.status === 'bounced') &&
                        selectedDelivery.attempts < selectedDelivery.maxAttempts && (React.createElement("button", { onClick: () => {
                            handleRetry(selectedDelivery.id);
                            setSelectedDelivery(null);
                        }, style: {
                            padding: '10px 20px',
                            fontSize: '14px',
                            fontWeight: '500',
                            border: 'none',
                            borderRadius: '6px',
                            backgroundColor: '#3b82f6',
                            color: '#ffffff',
                            cursor: 'pointer'
                        } }, "Retry Delivery"))))))));
};
//# sourceMappingURL=NotificationHistory.js.map