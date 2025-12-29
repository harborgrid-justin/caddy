import React, { useState, useCallback } from 'react';
export const ReportDistributionComponent = ({ distribution, onChange, readOnly = false, }) => {
    const [currentDistribution, setCurrentDistribution] = useState(distribution || createDefaultDistribution());
    const [selectedChannel, setSelectedChannel] = useState(null);
    const updateDistribution = useCallback((updates) => {
        const updated = { ...currentDistribution, ...updates };
        setCurrentDistribution(updated);
        onChange(updated);
    }, [currentDistribution, onChange]);
    const addChannel = useCallback((type) => {
        const newChannel = {
            type,
            enabled: true,
            config: getDefaultChannelConfig(type),
        };
        updateDistribution({
            channels: [...currentDistribution.channels, newChannel],
        });
        setSelectedChannel(type);
    }, [currentDistribution.channels, updateDistribution]);
    const updateChannel = useCallback((index, updates) => {
        const updatedChannels = currentDistribution.channels.map((channel, i) => i === index ? { ...channel, ...updates } : channel);
        updateDistribution({ channels: updatedChannels });
    }, [currentDistribution.channels, updateDistribution]);
    const removeChannel = useCallback((index) => {
        const updatedChannels = currentDistribution.channels.filter((_, i) => i !== index);
        updateDistribution({ channels: updatedChannels });
        setSelectedChannel(null);
    }, [currentDistribution.channels, updateDistribution]);
    const renderChannelConfig = (channel, index) => {
        switch (channel.type) {
            case 'email':
                return renderEmailConfig(channel.config, index);
            case 'slack':
                return renderSlackConfig(channel.config, index);
            case 'teams':
                return renderTeamsConfig(channel.config, index);
            case 'webhook':
                return renderWebhookConfig(channel.config, index);
            case 's3':
            case 'ftp':
            case 'sftp':
                return renderStorageConfig(channel.config, index);
            default:
                return React.createElement("div", null, "Unknown channel type");
        }
    };
    const renderEmailConfig = (config, index) => (React.createElement("div", { style: styles.configPanel },
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "From Address"),
            React.createElement("input", { type: "email", value: config.from, onChange: (e) => updateChannel(index, {
                    config: { ...config, from: e.target.value },
                }), style: styles.input, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "To Addresses (comma-separated)"),
            React.createElement("input", { type: "text", value: config.to.join(', '), onChange: (e) => updateChannel(index, {
                    config: {
                        ...config,
                        to: e.target.value.split(',').map((addr) => addr.trim()).filter(Boolean),
                    },
                }), style: styles.input, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "CC (optional)"),
            React.createElement("input", { type: "text", value: (config.cc || []).join(', '), onChange: (e) => updateChannel(index, {
                    config: {
                        ...config,
                        cc: e.target.value.split(',').map((addr) => addr.trim()).filter(Boolean),
                    },
                }), style: styles.input, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Subject"),
            React.createElement("input", { type: "text", value: config.subject, onChange: (e) => updateChannel(index, {
                    config: { ...config, subject: e.target.value },
                }), style: styles.input, placeholder: "Report: {{reportName}} - {{date}}", disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Body"),
            React.createElement("textarea", { value: config.body, onChange: (e) => updateChannel(index, {
                    config: { ...config, body: e.target.value },
                }), style: styles.textarea, rows: 4, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Body Format"),
            React.createElement("select", { value: config.bodyFormat, onChange: (e) => updateChannel(index, {
                    config: { ...config, bodyFormat: e.target.value },
                }), style: styles.select, disabled: readOnly },
                React.createElement("option", { value: "text" }, "Plain Text"),
                React.createElement("option", { value: "html" }, "HTML"))),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.checkboxLabel },
                React.createElement("input", { type: "checkbox", checked: config.attachReport, onChange: (e) => updateChannel(index, {
                        config: { ...config, attachReport: e.target.checked },
                    }), disabled: readOnly }),
                React.createElement("span", null, "Attach report file")))));
    const renderSlackConfig = (config, index) => (React.createElement("div", { style: styles.configPanel },
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Webhook URL"),
            React.createElement("input", { type: "url", value: config.webhookUrl, onChange: (e) => updateChannel(index, {
                    config: { ...config, webhookUrl: e.target.value },
                }), style: styles.input, placeholder: "https://hooks.slack.com/services/...", disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Channel"),
            React.createElement("input", { type: "text", value: config.channel, onChange: (e) => updateChannel(index, {
                    config: { ...config, channel: e.target.value },
                }), style: styles.input, placeholder: "#reports", disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Username (optional)"),
            React.createElement("input", { type: "text", value: config.username || '', onChange: (e) => updateChannel(index, {
                    config: { ...config, username: e.target.value },
                }), style: styles.input, placeholder: "Report Bot", disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Message"),
            React.createElement("textarea", { value: config.message, onChange: (e) => updateChannel(index, {
                    config: { ...config, message: e.target.value },
                }), style: styles.textarea, rows: 4, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.checkboxLabel },
                React.createElement("input", { type: "checkbox", checked: config.attachReport, onChange: (e) => updateChannel(index, {
                        config: { ...config, attachReport: e.target.checked },
                    }), disabled: readOnly }),
                React.createElement("span", null, "Attach report file")))));
    const renderTeamsConfig = (config, index) => (React.createElement("div", { style: styles.configPanel },
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Webhook URL"),
            React.createElement("input", { type: "url", value: config.webhookUrl, onChange: (e) => updateChannel(index, {
                    config: { ...config, webhookUrl: e.target.value },
                }), style: styles.input, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Title"),
            React.createElement("input", { type: "text", value: config.title, onChange: (e) => updateChannel(index, {
                    config: { ...config, title: e.target.value },
                }), style: styles.input, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Message"),
            React.createElement("textarea", { value: config.message, onChange: (e) => updateChannel(index, {
                    config: { ...config, message: e.target.value },
                }), style: styles.textarea, rows: 4, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Theme Color"),
            React.createElement("input", { type: "color", value: config.themeColor || '#0078D4', onChange: (e) => updateChannel(index, {
                    config: { ...config, themeColor: e.target.value },
                }), style: styles.colorInput, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.checkboxLabel },
                React.createElement("input", { type: "checkbox", checked: config.attachReport, onChange: (e) => updateChannel(index, {
                        config: { ...config, attachReport: e.target.checked },
                    }), disabled: readOnly }),
                React.createElement("span", null, "Attach report file")))));
    const renderWebhookConfig = (config, index) => (React.createElement("div", { style: styles.configPanel },
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "URL"),
            React.createElement("input", { type: "url", value: config.url, onChange: (e) => updateChannel(index, {
                    config: { ...config, url: e.target.value },
                }), style: styles.input, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "HTTP Method"),
            React.createElement("select", { value: config.method, onChange: (e) => updateChannel(index, {
                    config: { ...config, method: e.target.value },
                }), style: styles.select, disabled: readOnly },
                React.createElement("option", { value: "GET" }, "GET"),
                React.createElement("option", { value: "POST" }, "POST"),
                React.createElement("option", { value: "PUT" }, "PUT"))),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Headers (JSON)"),
            React.createElement("textarea", { value: JSON.stringify(config.headers || {}, null, 2), onChange: (e) => {
                    try {
                        const headers = JSON.parse(e.target.value);
                        updateChannel(index, {
                            config: { ...config, headers },
                        });
                    }
                    catch (err) {
                    }
                }, style: styles.textarea, rows: 4, disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.checkboxLabel },
                React.createElement("input", { type: "checkbox", checked: config.includeReportData, onChange: (e) => updateChannel(index, {
                        config: { ...config, includeReportData: e.target.checked },
                    }), disabled: readOnly }),
                React.createElement("span", null, "Include report data in payload")))));
    const renderStorageConfig = (config, index) => (React.createElement("div", { style: styles.configPanel },
        config.type === 's3' && (React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "S3 Bucket"),
            React.createElement("input", { type: "text", value: config.bucket || '', onChange: (e) => updateChannel(index, {
                    config: { ...config, bucket: e.target.value },
                }), style: styles.input, disabled: readOnly }))),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Path"),
            React.createElement("input", { type: "text", value: config.path, onChange: (e) => updateChannel(index, {
                    config: { ...config, path: e.target.value },
                }), style: styles.input, placeholder: "/reports/{{reportName}}/{{date}}.pdf", disabled: readOnly })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Credentials (JSON)"),
            React.createElement("textarea", { value: JSON.stringify(config.credentials || {}, null, 2), onChange: (e) => {
                    try {
                        const credentials = JSON.parse(e.target.value);
                        updateChannel(index, {
                            config: { ...config, credentials },
                        });
                    }
                    catch (err) {
                    }
                }, style: styles.textarea, rows: 4, disabled: readOnly })),
        config.type === 's3' && (React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.checkboxLabel },
                React.createElement("input", { type: "checkbox", checked: config.publicAccess ?? false, onChange: (e) => updateChannel(index, {
                        config: { ...config, publicAccess: e.target.checked },
                    }), disabled: readOnly }),
                React.createElement("span", null, "Public access"))))));
    return (React.createElement("div", { style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h3", { style: styles.title }, "Distribution Settings")),
        React.createElement("div", { style: styles.content },
            React.createElement("div", { style: styles.section },
                React.createElement("h4", { style: styles.sectionTitle }, "Global Settings"),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Attachment Format"),
                    React.createElement("select", { value: currentDistribution.attachmentFormat || 'pdf', onChange: (e) => updateDistribution({ attachmentFormat: e.target.value }), style: styles.select, disabled: readOnly },
                        React.createElement("option", { value: "pdf" }, "PDF"),
                        React.createElement("option", { value: "excel" }, "Excel"),
                        React.createElement("option", { value: "csv" }, "CSV"),
                        React.createElement("option", { value: "powerpoint" }, "PowerPoint"))),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.checkboxLabel },
                        React.createElement("input", { type: "checkbox", checked: currentDistribution.compression ?? false, onChange: (e) => updateDistribution({ compression: e.target.checked }), disabled: readOnly }),
                        React.createElement("span", null, "Compress attachments (ZIP)"))),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.checkboxLabel },
                        React.createElement("input", { type: "checkbox", checked: currentDistribution.encryption?.enabled ?? false, onChange: (e) => updateDistribution({
                                encryption: {
                                    ...currentDistribution.encryption,
                                    enabled: e.target.checked,
                                    algorithm: 'aes-256',
                                },
                            }), disabled: readOnly }),
                        React.createElement("span", null, "Encrypt attachments"))),
                currentDistribution.encryption?.enabled && (React.createElement(React.Fragment, null,
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Encryption Algorithm"),
                        React.createElement("select", { value: currentDistribution.encryption.algorithm, onChange: (e) => updateDistribution({
                                encryption: {
                                    ...currentDistribution.encryption,
                                    algorithm: e.target.value,
                                },
                            }), style: styles.select, disabled: readOnly },
                            React.createElement("option", { value: "aes-256" }, "AES-256"),
                            React.createElement("option", { value: "rsa-2048" }, "RSA-2048"))),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Encryption Password"),
                        React.createElement("input", { type: "password", value: currentDistribution.encryption.password || '', onChange: (e) => updateDistribution({
                                encryption: {
                                    ...currentDistribution.encryption,
                                    password: e.target.value,
                                },
                            }), style: styles.input, disabled: readOnly }))))),
            React.createElement("div", { style: styles.section },
                React.createElement("div", { style: styles.sectionHeader },
                    React.createElement("h4", { style: styles.sectionTitle }, "Distribution Channels"),
                    !readOnly && (React.createElement("div", { style: styles.addChannelDropdown },
                        React.createElement("button", { style: styles.addButton }, "+ Add Channel"),
                        React.createElement("div", { style: styles.channelMenu },
                            React.createElement("button", { onClick: () => addChannel('email') }, "\uD83D\uDCE7 Email"),
                            React.createElement("button", { onClick: () => addChannel('slack') }, "\uD83D\uDCAC Slack"),
                            React.createElement("button", { onClick: () => addChannel('teams') }, "\uD83D\uDC65 Teams"),
                            React.createElement("button", { onClick: () => addChannel('webhook') }, "\uD83D\uDD17 Webhook"),
                            React.createElement("button", { onClick: () => addChannel('s3') }, "\u2601\uFE0F S3"),
                            React.createElement("button", { onClick: () => addChannel('ftp') }, "\uD83D\uDCC1 FTP"),
                            React.createElement("button", { onClick: () => addChannel('sftp') }, "\uD83D\uDD10 SFTP"))))),
                React.createElement("div", { style: styles.channelsList },
                    currentDistribution.channels.map((channel, index) => (React.createElement("div", { key: index, style: styles.channelItem },
                        React.createElement("div", { style: styles.channelHeader },
                            React.createElement("div", { style: styles.channelHeaderLeft },
                                React.createElement("span", { style: styles.channelIcon }, getChannelIcon(channel.type)),
                                React.createElement("span", { style: styles.channelName }, channel.type.toUpperCase()),
                                React.createElement("label", { style: styles.channelToggle },
                                    React.createElement("input", { type: "checkbox", checked: channel.enabled, onChange: (e) => updateChannel(index, { enabled: e.target.checked }), disabled: readOnly }),
                                    React.createElement("span", null, "Enabled"))),
                            !readOnly && (React.createElement("button", { onClick: () => removeChannel(index), style: styles.removeButton }, "\u2715"))),
                        channel.enabled && renderChannelConfig(channel, index)))),
                    currentDistribution.channels.length === 0 && (React.createElement("div", { style: styles.emptyState },
                        React.createElement("div", { style: styles.emptyStateIcon }, "\uD83D\uDCEC"),
                        React.createElement("div", { style: styles.emptyStateText }, "No distribution channels configured"))))))));
};
function createDefaultDistribution() {
    return {
        channels: [],
        attachmentFormat: 'pdf',
        compression: false,
        encryption: {
            enabled: false,
            algorithm: 'aes-256',
        },
    };
}
function getDefaultChannelConfig(type) {
    switch (type) {
        case 'email':
            return {
                from: '',
                to: [],
                subject: 'Report: {{reportName}} - {{date}}',
                body: 'Please find the attached report.',
                bodyFormat: 'text',
                attachReport: true,
            };
        case 'slack':
            return {
                webhookUrl: '',
                channel: '#reports',
                message: 'New report available: {{reportName}}',
                attachReport: true,
            };
        case 'teams':
            return {
                webhookUrl: '',
                title: 'Report: {{reportName}}',
                message: 'A new report is ready for review.',
                themeColor: '#0078D4',
                attachReport: true,
            };
        case 'webhook':
            return {
                url: '',
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                includeReportData: true,
            };
        case 's3':
        case 'ftp':
        case 'sftp':
            return {
                type,
                path: '/reports/{{reportName}}/{{date}}',
                credentials: {},
            };
        default:
            return {};
    }
}
function getChannelIcon(type) {
    const icons = {
        email: '📧',
        slack: '💬',
        teams: '👥',
        webhook: '🔗',
        s3: '☁️',
        ftp: '📁',
        sftp: '🔐',
    };
    return icons[type] || '📤';
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
    sectionHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '12px',
    },
    sectionTitle: {
        fontSize: '13px',
        fontWeight: 600,
        margin: 0,
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
    colorInput: {
        width: '60px',
        height: '40px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        cursor: 'pointer',
    },
    addChannelDropdown: {
        position: 'relative',
    },
    addButton: {
        padding: '6px 12px',
        border: 'none',
        borderRadius: '6px',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        cursor: 'pointer',
        fontSize: '13px',
        fontWeight: 500,
    },
    channelMenu: {
        display: 'none',
        position: 'absolute',
        top: '100%',
        right: 0,
        marginTop: '4px',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
        zIndex: 10,
    },
    channelsList: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
    },
    channelItem: {
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        overflow: 'hidden',
    },
    channelHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '12px',
        backgroundColor: '#f8fafc',
        borderBottom: '1px solid #e2e8f0',
    },
    channelHeaderLeft: {
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
    },
    channelIcon: {
        fontSize: '20px',
    },
    channelName: {
        fontSize: '13px',
        fontWeight: 600,
        color: '#1e293b',
    },
    channelToggle: {
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
        fontSize: '12px',
        color: '#64748b',
        cursor: 'pointer',
    },
    removeButton: {
        border: 'none',
        background: 'none',
        color: '#ef4444',
        cursor: 'pointer',
        fontSize: '18px',
    },
    configPanel: {
        padding: '16px',
    },
    emptyState: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '48px',
        gap: '12px',
    },
    emptyStateIcon: {
        fontSize: '32px',
    },
    emptyStateText: {
        fontSize: '13px',
        color: '#64748b',
    },
};
export default ReportDistributionComponent;
//# sourceMappingURL=ReportDistribution.js.map