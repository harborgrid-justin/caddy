import React, { useState, useCallback } from 'react';
export const ReportScheduler = ({ schedule, onChange, readOnly = false, }) => {
    const [currentSchedule, setCurrentSchedule] = useState(schedule || createDefaultSchedule());
    const updateSchedule = useCallback((updates) => {
        const updated = { ...currentSchedule, ...updates };
        setCurrentSchedule(updated);
        onChange(updated);
    }, [currentSchedule, onChange]);
    const generateCronExpression = (frequency) => {
        switch (frequency) {
            case 'hourly':
                return '0 * * * *';
            case 'daily':
                return '0 0 * * *';
            case 'weekly':
                return '0 0 * * 0';
            case 'monthly':
                return '0 0 1 * *';
            case 'quarterly':
                return '0 0 1 */3 *';
            case 'yearly':
                return '0 0 1 1 *';
            default:
                return '';
        }
    };
    const handleFrequencyChange = (frequency) => {
        const cronExpression = frequency === 'cron' ? currentSchedule.cronExpression || '' : generateCronExpression(frequency);
        updateSchedule({
            frequency,
            cronExpression: frequency === 'cron' ? cronExpression : undefined,
        });
    };
    return (React.createElement("div", { style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h3", { style: styles.title }, "Schedule Configuration"),
            React.createElement("label", { style: styles.toggleLabel },
                React.createElement("input", { type: "checkbox", checked: currentSchedule.enabled, onChange: (e) => updateSchedule({ enabled: e.target.checked }), disabled: readOnly }),
                React.createElement("span", null, "Enabled"))),
        React.createElement("div", { style: styles.content },
            React.createElement("div", { style: styles.section },
                React.createElement("h4", { style: styles.sectionTitle }, "Frequency"),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Schedule Type"),
                    React.createElement("select", { value: currentSchedule.frequency, onChange: (e) => handleFrequencyChange(e.target.value), style: styles.select, disabled: readOnly || !currentSchedule.enabled },
                        React.createElement("option", { value: "once" }, "Once"),
                        React.createElement("option", { value: "hourly" }, "Hourly"),
                        React.createElement("option", { value: "daily" }, "Daily"),
                        React.createElement("option", { value: "weekly" }, "Weekly"),
                        React.createElement("option", { value: "monthly" }, "Monthly"),
                        React.createElement("option", { value: "quarterly" }, "Quarterly"),
                        React.createElement("option", { value: "yearly" }, "Yearly"),
                        React.createElement("option", { value: "cron" }, "Custom (Cron)"))),
                currentSchedule.frequency === 'cron' && (React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Cron Expression"),
                    React.createElement("input", { type: "text", value: currentSchedule.cronExpression || '', onChange: (e) => updateSchedule({ cronExpression: e.target.value }), style: styles.input, placeholder: "0 0 * * *", disabled: readOnly || !currentSchedule.enabled }),
                    React.createElement("div", { style: styles.helpText },
                        "Format: minute hour day month weekday",
                        React.createElement("br", null),
                        "Example: \"0 9 * * 1-5\" = 9 AM on weekdays"))),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Start Date"),
                    React.createElement("input", { type: "datetime-local", value: formatDateTimeLocal(currentSchedule.startDate), onChange: (e) => updateSchedule({ startDate: new Date(e.target.value) }), style: styles.input, disabled: readOnly || !currentSchedule.enabled })),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "End Date (Optional)"),
                    React.createElement("input", { type: "datetime-local", value: currentSchedule.endDate ? formatDateTimeLocal(currentSchedule.endDate) : '', onChange: (e) => updateSchedule({ endDate: e.target.value ? new Date(e.target.value) : undefined }), style: styles.input, disabled: readOnly || !currentSchedule.enabled })),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Timezone"),
                    React.createElement("select", { value: currentSchedule.timezone, onChange: (e) => updateSchedule({ timezone: e.target.value }), style: styles.select, disabled: readOnly || !currentSchedule.enabled },
                        React.createElement("option", { value: "UTC" }, "UTC"),
                        React.createElement("option", { value: "America/New_York" }, "America/New York (EST)"),
                        React.createElement("option", { value: "America/Chicago" }, "America/Chicago (CST)"),
                        React.createElement("option", { value: "America/Denver" }, "America/Denver (MST)"),
                        React.createElement("option", { value: "America/Los_Angeles" }, "America/Los Angeles (PST)"),
                        React.createElement("option", { value: "Europe/London" }, "Europe/London"),
                        React.createElement("option", { value: "Europe/Paris" }, "Europe/Paris"),
                        React.createElement("option", { value: "Asia/Tokyo" }, "Asia/Tokyo"),
                        React.createElement("option", { value: "Asia/Shanghai" }, "Asia/Shanghai"),
                        React.createElement("option", { value: "Australia/Sydney" }, "Australia/Sydney")))),
            React.createElement("div", { style: styles.section },
                React.createElement("h4", { style: styles.sectionTitle }, "Execution Conditions"),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.checkboxLabel },
                        React.createElement("input", { type: "checkbox", checked: currentSchedule.conditions?.dataAvailable ?? false, onChange: (e) => updateSchedule({
                                conditions: {
                                    ...currentSchedule.conditions,
                                    dataAvailable: e.target.checked,
                                },
                            }), disabled: readOnly || !currentSchedule.enabled }),
                        React.createElement("span", null, "Wait for data availability"))),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Minimum Rows Required"),
                    React.createElement("input", { type: "number", value: currentSchedule.conditions?.minimumRows || 0, onChange: (e) => updateSchedule({
                            conditions: {
                                ...currentSchedule.conditions,
                                minimumRows: Number(e.target.value),
                            },
                        }), style: styles.input, min: "0", disabled: readOnly || !currentSchedule.enabled })),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Custom Condition (SQL/Expression)"),
                    React.createElement("textarea", { value: currentSchedule.conditions?.customCondition || '', onChange: (e) => updateSchedule({
                            conditions: {
                                ...currentSchedule.conditions,
                                customCondition: e.target.value,
                            },
                        }), style: styles.textarea, placeholder: "e.g., SELECT COUNT(*) > 0 FROM table WHERE updated_at > NOW() - INTERVAL 1 DAY", rows: 3, disabled: readOnly || !currentSchedule.enabled }))),
            React.createElement("div", { style: styles.section },
                React.createElement("h4", { style: styles.sectionTitle }, "Retry Policy"),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Max Retry Attempts"),
                    React.createElement("input", { type: "number", value: currentSchedule.retryPolicy?.maxAttempts || 3, onChange: (e) => updateSchedule({
                            retryPolicy: {
                                ...currentSchedule.retryPolicy,
                                maxAttempts: Number(e.target.value),
                            },
                        }), style: styles.input, min: "0", max: "10", disabled: readOnly || !currentSchedule.enabled })),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Retry Interval (seconds)"),
                    React.createElement("input", { type: "number", value: currentSchedule.retryPolicy?.retryInterval || 60, onChange: (e) => updateSchedule({
                            retryPolicy: {
                                ...currentSchedule.retryPolicy,
                                retryInterval: Number(e.target.value),
                            },
                        }), style: styles.input, min: "1", disabled: readOnly || !currentSchedule.enabled })),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Backoff Multiplier"),
                    React.createElement("input", { type: "number", value: currentSchedule.retryPolicy?.backoffMultiplier || 2, onChange: (e) => updateSchedule({
                            retryPolicy: {
                                ...currentSchedule.retryPolicy,
                                backoffMultiplier: Number(e.target.value),
                            },
                        }), style: styles.input, min: "1", step: "0.1", disabled: readOnly || !currentSchedule.enabled }),
                    React.createElement("div", { style: styles.helpText }, "Each retry will wait (interval \u00D7 multiplier^attempt) seconds"))),
            React.createElement("div", { style: styles.section },
                React.createElement("h4", { style: styles.sectionTitle }, "Notifications"),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.checkboxLabel },
                        React.createElement("input", { type: "checkbox", checked: currentSchedule.notifications?.onSuccess ?? false, onChange: (e) => updateSchedule({
                                notifications: {
                                    ...currentSchedule.notifications,
                                    onSuccess: e.target.checked,
                                },
                            }), disabled: readOnly || !currentSchedule.enabled }),
                        React.createElement("span", null, "Notify on successful execution"))),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.checkboxLabel },
                        React.createElement("input", { type: "checkbox", checked: currentSchedule.notifications?.onFailure ?? true, onChange: (e) => updateSchedule({
                                notifications: {
                                    ...currentSchedule.notifications,
                                    onFailure: e.target.checked,
                                },
                            }), disabled: readOnly || !currentSchedule.enabled }),
                        React.createElement("span", null, "Notify on execution failure"))),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Recipients (comma-separated emails)"),
                    React.createElement("input", { type: "text", value: (currentSchedule.notifications?.recipients || []).join(', '), onChange: (e) => updateSchedule({
                            notifications: {
                                ...currentSchedule.notifications,
                                recipients: e.target.value.split(',').map((r) => r.trim()).filter(Boolean),
                            },
                        }), style: styles.input, placeholder: "user1@example.com, user2@example.com", disabled: readOnly || !currentSchedule.enabled }))),
            React.createElement("div", { style: styles.section },
                React.createElement("h4", { style: styles.sectionTitle }, "Schedule Summary"),
                React.createElement("div", { style: styles.summary },
                    React.createElement("div", { style: styles.summaryItem },
                        React.createElement("span", { style: styles.summaryLabel }, "Status:"),
                        React.createElement("span", { style: currentSchedule.enabled ? styles.statusEnabled : styles.statusDisabled }, currentSchedule.enabled ? 'Enabled' : 'Disabled')),
                    React.createElement("div", { style: styles.summaryItem },
                        React.createElement("span", { style: styles.summaryLabel }, "Frequency:"),
                        React.createElement("span", null, currentSchedule.frequency)),
                    currentSchedule.cronExpression && (React.createElement("div", { style: styles.summaryItem },
                        React.createElement("span", { style: styles.summaryLabel }, "Cron:"),
                        React.createElement("code", { style: styles.codeText }, currentSchedule.cronExpression))),
                    React.createElement("div", { style: styles.summaryItem },
                        React.createElement("span", { style: styles.summaryLabel }, "Next Run:"),
                        React.createElement("span", null, calculateNextRun(currentSchedule))),
                    React.createElement("div", { style: styles.summaryItem },
                        React.createElement("span", { style: styles.summaryLabel }, "Timezone:"),
                        React.createElement("span", null, currentSchedule.timezone)))))));
};
function createDefaultSchedule() {
    return {
        id: generateId(),
        enabled: false,
        frequency: 'daily',
        startDate: new Date(),
        timezone: 'UTC',
        retryPolicy: {
            maxAttempts: 3,
            retryInterval: 60,
            backoffMultiplier: 2,
        },
        notifications: {
            onSuccess: false,
            onFailure: true,
            recipients: [],
        },
    };
}
function generateId() {
    return `schedule-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}
function formatDateTimeLocal(date) {
    const d = new Date(date);
    const offset = d.getTimezoneOffset();
    const adjusted = new Date(d.getTime() - offset * 60000);
    return adjusted.toISOString().slice(0, 16);
}
function calculateNextRun(schedule) {
    if (!schedule.enabled)
        return 'N/A (Disabled)';
    const now = new Date();
    const start = new Date(schedule.startDate);
    if (start > now) {
        return start.toLocaleString();
    }
    switch (schedule.frequency) {
        case 'once':
            return start.toLocaleString();
        case 'hourly':
            return new Date(now.getFullYear(), now.getMonth(), now.getDate(), now.getHours() + 1).toLocaleString();
        case 'daily':
            return new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1).toLocaleString();
        case 'weekly':
            return new Date(now.getFullYear(), now.getMonth(), now.getDate() + 7).toLocaleString();
        case 'monthly':
            return new Date(now.getFullYear(), now.getMonth() + 1, 1).toLocaleString();
        default:
            return 'Calculating...';
    }
}
const styles = {
    container: {
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        fontFamily: 'Inter, system-ui, sans-serif',
        overflow: 'hidden',
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '12px 16px',
        borderBottom: '1px solid #e2e8f0',
        backgroundColor: '#f8fafc',
    },
    title: {
        fontSize: '14px',
        fontWeight: 600,
        margin: 0,
        color: '#1e293b',
    },
    toggleLabel: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontSize: '13px',
        fontWeight: 500,
        color: '#475569',
        cursor: 'pointer',
    },
    content: {
        flex: 1,
        overflow: 'auto',
        padding: '16px',
    },
    section: {
        marginBottom: '24px',
        paddingBottom: '24px',
        borderBottom: '1px solid #e2e8f0',
    },
    sectionTitle: {
        fontSize: '13px',
        fontWeight: 600,
        margin: '0 0 12px 0',
        color: '#1e293b',
    },
    formGroup: {
        marginBottom: '12px',
    },
    label: {
        display: 'block',
        fontSize: '12px',
        fontWeight: 500,
        color: '#475569',
        marginBottom: '4px',
    },
    input: {
        width: '100%',
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '13px',
    },
    select: {
        width: '100%',
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '13px',
        cursor: 'pointer',
    },
    textarea: {
        width: '100%',
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '12px',
        fontFamily: 'monospace',
        resize: 'vertical',
    },
    checkboxLabel: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontSize: '13px',
        color: '#475569',
        cursor: 'pointer',
    },
    helpText: {
        fontSize: '11px',
        color: '#64748b',
        marginTop: '4px',
    },
    summary: {
        backgroundColor: '#f8fafc',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        padding: '12px',
    },
    summaryItem: {
        display: 'flex',
        justifyContent: 'space-between',
        padding: '6px 0',
        fontSize: '13px',
        borderBottom: '1px solid #e2e8f0',
    },
    summaryLabel: {
        fontWeight: 600,
        color: '#475569',
    },
    statusEnabled: {
        color: '#10b981',
        fontWeight: 600,
    },
    statusDisabled: {
        color: '#ef4444',
        fontWeight: 600,
    },
    codeText: {
        fontFamily: 'monospace',
        fontSize: '12px',
        backgroundColor: '#f1f5f9',
        padding: '2px 6px',
        borderRadius: '3px',
    },
};
export default ReportScheduler;
//# sourceMappingURL=ReportScheduler.js.map