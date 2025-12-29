import React, { useState, useCallback, useEffect } from 'react';
import { NotificationType } from './types';
import { useNotifications, useDesktopNotifications } from './useNotifications';
export const NotificationPreferences = () => {
    const { preferences, updatePreferences } = useNotifications();
    const { permission, requestPermission, enabled: desktopEnabled } = useDesktopNotifications();
    const [localPreferences, setLocalPreferences] = useState(null);
    const [isSaving, setIsSaving] = useState(false);
    const [hasChanges, setHasChanges] = useState(false);
    useEffect(() => {
        if (preferences) {
            setLocalPreferences(preferences);
        }
    }, [preferences]);
    const handleSave = useCallback(async () => {
        if (!localPreferences)
            return;
        setIsSaving(true);
        try {
            await updatePreferences(localPreferences);
            setHasChanges(false);
        }
        catch (err) {
            console.error('Error saving preferences:', err);
            alert('Failed to save preferences. Please try again.');
        }
        finally {
            setIsSaving(false);
        }
    }, [localPreferences, updatePreferences]);
    const handleReset = useCallback(() => {
        setLocalPreferences(preferences);
        setHasChanges(false);
    }, [preferences]);
    const updateLocal = useCallback((updates) => {
        setLocalPreferences(prev => prev ? { ...prev, ...updates } : null);
        setHasChanges(true);
    }, []);
    const toggleChannel = useCallback((channel) => {
        if (!localPreferences)
            return;
        updateLocal({
            channels: {
                ...localPreferences.channels,
                [channel]: !localPreferences.channels?.[channel]
            }
        });
    }, [localPreferences, updateLocal]);
    const toggleTypeChannel = useCallback((type, channel) => {
        if (!localPreferences)
            return;
        const typeConfig = localPreferences.types?.[type] || { enabled: true, channels: [] };
        const channels = typeConfig.channels.includes(channel)
            ? typeConfig.channels.filter(c => c !== channel)
            : [...typeConfig.channels, channel];
        updateLocal({
            types: {
                ...localPreferences.types,
                [type]: { ...typeConfig, channels }
            }
        });
    }, [localPreferences, updateLocal]);
    const toggleType = useCallback((type) => {
        if (!localPreferences)
            return;
        const typeConfig = localPreferences.types?.[type] || { enabled: true, channels: [] };
        updateLocal({
            types: {
                ...localPreferences.types,
                [type]: { ...typeConfig, enabled: !typeConfig.enabled }
            }
        });
    }, [localPreferences, updateLocal]);
    const handleRequestDesktopPermission = useCallback(async () => {
        await requestPermission();
    }, [requestPermission]);
    if (!localPreferences) {
        return (React.createElement("div", { style: { padding: '48px 24px', textAlign: 'center', color: '#6b7280' } }, "Loading preferences..."));
    }
    return (React.createElement("div", { style: { padding: '24px', maxWidth: '900px', margin: '0 auto' } },
        React.createElement("div", { style: { marginBottom: '32px' } },
            React.createElement("h2", { style: { margin: '0 0 16px 0', fontSize: '18px', fontWeight: '600', color: '#111827' } }, "Global Settings"),
            React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '16px' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '12px', padding: '12px', border: '1px solid #e5e7eb', borderRadius: '8px' } },
                    React.createElement("input", { type: "checkbox", checked: localPreferences.enabled, onChange: (e) => updateLocal({ enabled: e.target.checked }), style: { width: '20px', height: '20px', cursor: 'pointer' } }),
                    React.createElement("div", { style: { flex: 1 } },
                        React.createElement("div", { style: { fontSize: '14px', fontWeight: '500', color: '#111827' } }, "Enable all notifications"),
                        React.createElement("div", { style: { fontSize: '12px', color: '#6b7280', marginTop: '2px' } }, "Master switch for all notification types"))),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '12px', padding: '12px', border: '1px solid #e5e7eb', borderRadius: '8px' } },
                    React.createElement("input", { type: "checkbox", checked: localPreferences.soundEnabled, onChange: (e) => updateLocal({ soundEnabled: e.target.checked }), style: { width: '20px', height: '20px', cursor: 'pointer' } }),
                    React.createElement("div", { style: { flex: 1 } },
                        React.createElement("div", { style: { fontSize: '14px', fontWeight: '500', color: '#111827' } }, "Sound notifications"),
                        React.createElement("div", { style: { fontSize: '12px', color: '#6b7280', marginTop: '2px' } }, "Play sound when new notifications arrive"))),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '12px', padding: '12px', border: '1px solid #e5e7eb', borderRadius: '8px' } },
                    React.createElement("input", { type: "checkbox", checked: localPreferences.desktopEnabled, onChange: (e) => updateLocal({ desktopEnabled: e.target.checked }), style: { width: '20px', height: '20px', cursor: 'pointer' } }),
                    React.createElement("div", { style: { flex: 1 } },
                        React.createElement("div", { style: { fontSize: '14px', fontWeight: '500', color: '#111827' } }, "Desktop notifications"),
                        React.createElement("div", { style: { fontSize: '12px', color: '#6b7280', marginTop: '2px' } }, "Show desktop notifications (requires browser permission)"),
                        permission === 'denied' && (React.createElement("div", { style: { fontSize: '11px', color: '#dc2626', marginTop: '4px' } }, "Permission denied. Please enable in browser settings.")),
                        permission === 'default' && (React.createElement("button", { onClick: handleRequestDesktopPermission, style: {
                                marginTop: '8px',
                                padding: '4px 12px',
                                fontSize: '12px',
                                fontWeight: '500',
                                border: '1px solid #3b82f6',
                                borderRadius: '4px',
                                backgroundColor: '#ffffff',
                                color: '#3b82f6',
                                cursor: 'pointer'
                            } }, "Request Permission")))),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '12px', padding: '12px', border: '1px solid #e5e7eb', borderRadius: '8px' } },
                    React.createElement("input", { type: "checkbox", checked: localPreferences.mobileEnabled, onChange: (e) => updateLocal({ mobileEnabled: e.target.checked }), style: { width: '20px', height: '20px', cursor: 'pointer' } }),
                    React.createElement("div", { style: { flex: 1 } },
                        React.createElement("div", { style: { fontSize: '14px', fontWeight: '500', color: '#111827' } }, "Mobile push notifications"),
                        React.createElement("div", { style: { fontSize: '12px', color: '#6b7280', marginTop: '2px' } }, "Receive push notifications on mobile devices"))))),
        React.createElement("div", { style: { marginBottom: '32px' } },
            React.createElement("h2", { style: { margin: '0 0 16px 0', fontSize: '18px', fontWeight: '600', color: '#111827' } }, "Do Not Disturb"),
            React.createElement("div", { style: { padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#f9fafb' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' } },
                    React.createElement("input", { type: "checkbox", checked: localPreferences.doNotDisturb.enabled, onChange: (e) => updateLocal({
                            doNotDisturb: { ...localPreferences.doNotDisturb, enabled: e.target.checked }
                        }), style: { width: '20px', height: '20px', cursor: 'pointer' } }),
                    React.createElement("div", { style: { fontSize: '14px', fontWeight: '500', color: '#111827' } }, "Enable Do Not Disturb mode")),
                localPreferences.doNotDisturb.enabled && (React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '12px', marginTop: '12px' } },
                    React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' } },
                        React.createElement("div", null,
                            React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Start Time"),
                            React.createElement("input", { type: "time", value: localPreferences.doNotDisturb.startTime || '22:00', onChange: (e) => updateLocal({
                                    doNotDisturb: { ...localPreferences.doNotDisturb, startTime: e.target.value }
                                }), style: {
                                    width: '100%',
                                    padding: '8px',
                                    fontSize: '14px',
                                    border: '1px solid #d1d5db',
                                    borderRadius: '4px'
                                } })),
                        React.createElement("div", null,
                            React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "End Time"),
                            React.createElement("input", { type: "time", value: localPreferences.doNotDisturb.endTime || '08:00', onChange: (e) => updateLocal({
                                    doNotDisturb: { ...localPreferences.doNotDisturb, endTime: e.target.value }
                                }), style: {
                                    width: '100%',
                                    padding: '8px',
                                    fontSize: '14px',
                                    border: '1px solid #d1d5db',
                                    borderRadius: '4px'
                                } }))),
                    React.createElement("div", { style: { display: 'flex', gap: '8px', flexWrap: 'wrap' } }, ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'].map((day, index) => {
                        const isActive = localPreferences.doNotDisturb.days?.includes(index) ?? true;
                        return (React.createElement("button", { key: day, onClick: () => {
                                const days = localPreferences.doNotDisturb.days || [0, 1, 2, 3, 4, 5, 6];
                                const newDays = isActive
                                    ? days.filter(d => d !== index)
                                    : [...days, index].sort();
                                updateLocal({
                                    doNotDisturb: { ...localPreferences.doNotDisturb, days: newDays }
                                });
                            }, style: {
                                padding: '8px 12px',
                                fontSize: '12px',
                                fontWeight: '500',
                                border: '1px solid #d1d5db',
                                borderRadius: '4px',
                                backgroundColor: isActive ? '#3b82f6' : '#ffffff',
                                color: isActive ? '#ffffff' : '#374151',
                                cursor: 'pointer'
                            } }, day));
                    })),
                    React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                        React.createElement("input", { type: "checkbox", checked: localPreferences.doNotDisturb.allowUrgent, onChange: (e) => updateLocal({
                                doNotDisturb: { ...localPreferences.doNotDisturb, allowUrgent: e.target.checked }
                            }), style: { width: '16px', height: '16px', cursor: 'pointer' } }),
                        React.createElement("span", { style: { fontSize: '13px', color: '#374151' } }, "Allow urgent notifications")),
                    React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                        React.createElement("input", { type: "checkbox", checked: localPreferences.doNotDisturb.allowCritical, onChange: (e) => updateLocal({
                                doNotDisturb: { ...localPreferences.doNotDisturb, allowCritical: e.target.checked }
                            }), style: { width: '16px', height: '16px', cursor: 'pointer' } }),
                        React.createElement("span", { style: { fontSize: '13px', color: '#374151' } }, "Allow critical notifications")))))),
        React.createElement("div", { style: { marginBottom: '32px' } },
            React.createElement("h2", { style: { margin: '0 0 16px 0', fontSize: '18px', fontWeight: '600', color: '#111827' } }, "Email Digest"),
            React.createElement("div", { style: { padding: '16px', border: '1px solid #e5e7eb', borderRadius: '8px', backgroundColor: '#f9fafb' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' } },
                    React.createElement("input", { type: "checkbox", checked: localPreferences.emailDigest.enabled, onChange: (e) => updateLocal({
                            emailDigest: { ...localPreferences.emailDigest, enabled: e.target.checked }
                        }), style: { width: '20px', height: '20px', cursor: 'pointer' } }),
                    React.createElement("div", { style: { fontSize: '14px', fontWeight: '500', color: '#111827' } }, "Enable email digest")),
                localPreferences.emailDigest.enabled && (React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '12px' } },
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Frequency"),
                        React.createElement("select", { value: localPreferences.emailDigest.frequency, onChange: (e) => updateLocal({
                                emailDigest: { ...localPreferences.emailDigest, frequency: e.target.value }
                            }), style: {
                                width: '100%',
                                padding: '8px',
                                fontSize: '14px',
                                border: '1px solid #d1d5db',
                                borderRadius: '4px',
                                cursor: 'pointer'
                            } },
                            React.createElement("option", { value: "realtime" }, "Real-time"),
                            React.createElement("option", { value: "hourly" }, "Hourly"),
                            React.createElement("option", { value: "daily" }, "Daily"),
                            React.createElement("option", { value: "weekly" }, "Weekly"))),
                    (localPreferences.emailDigest.frequency === 'daily' || localPreferences.emailDigest.frequency === 'weekly') && (React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Delivery Time"),
                        React.createElement("input", { type: "time", value: localPreferences.emailDigest.time || '09:00', onChange: (e) => updateLocal({
                                emailDigest: { ...localPreferences.emailDigest, time: e.target.value }
                            }), style: {
                                width: '100%',
                                padding: '8px',
                                fontSize: '14px',
                                border: '1px solid #d1d5db',
                                borderRadius: '4px'
                            } }))))))),
        React.createElement("div", { style: { marginBottom: '32px' } },
            React.createElement("h2", { style: { margin: '0 0 16px 0', fontSize: '18px', fontWeight: '600', color: '#111827' } }, "Notification Types"),
            React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '8px' } }, Object.values(NotificationType).map((type) => {
                const typeConfig = localPreferences.types?.[type] || { enabled: true, channels: [] };
                return (React.createElement("div", { key: type, style: {
                        padding: '12px',
                        border: '1px solid #e5e7eb',
                        borderRadius: '8px',
                        backgroundColor: typeConfig.enabled ? '#ffffff' : '#f9fafb'
                    } },
                    React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '12px', cursor: 'pointer' } },
                        React.createElement("input", { type: "checkbox", checked: typeConfig.enabled, onChange: () => toggleType(type), style: { width: '20px', height: '20px', cursor: 'pointer' } }),
                        React.createElement("span", { style: { fontSize: '14px', fontWeight: '500', color: '#111827', textTransform: 'capitalize' } }, type))));
            }))),
        hasChanges && (React.createElement("div", { style: {
                position: 'sticky',
                bottom: 0,
                padding: '16px',
                backgroundColor: '#ffffff',
                borderTop: '1px solid #e5e7eb',
                display: 'flex',
                gap: '12px',
                justifyContent: 'flex-end',
                marginTop: '24px'
            } },
            React.createElement("button", { onClick: handleReset, disabled: isSaving, style: {
                    padding: '10px 20px',
                    fontSize: '14px',
                    fontWeight: '500',
                    border: '1px solid #d1d5db',
                    borderRadius: '6px',
                    backgroundColor: '#ffffff',
                    color: '#374151',
                    cursor: isSaving ? 'not-allowed' : 'pointer',
                    opacity: isSaving ? 0.6 : 1
                } }, "Reset"),
            React.createElement("button", { onClick: handleSave, disabled: isSaving, style: {
                    padding: '10px 20px',
                    fontSize: '14px',
                    fontWeight: '500',
                    border: 'none',
                    borderRadius: '6px',
                    backgroundColor: '#3b82f6',
                    color: '#ffffff',
                    cursor: isSaving ? 'not-allowed' : 'pointer',
                    opacity: isSaving ? 0.6 : 1
                } }, isSaving ? 'Saving...' : 'Save Changes')))));
};
//# sourceMappingURL=NotificationPreferences.js.map