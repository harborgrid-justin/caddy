import React, { useMemo, useState, useCallback } from 'react';
const DEFAULT_ACTIONS = [
    {
        id: 'email-send',
        type: 'email',
        name: 'Send Email',
        description: 'Send an email message to one or more recipients',
        icon: '📧',
        config: {
            to: '',
            cc: '',
            bcc: '',
            subject: '',
            body: '',
            attachments: [],
        },
        inputSchema: {
            type: 'object',
            properties: {
                data: { type: 'object' },
            },
        },
        outputSchema: {
            type: 'object',
            properties: {
                sent: { type: 'boolean' },
                messageId: { type: 'string' },
                timestamp: { type: 'string' },
            },
        },
    },
    {
        id: 'http-request',
        type: 'http-request',
        name: 'HTTP Request',
        description: 'Make an HTTP API request',
        icon: '🌐',
        config: {
            method: 'GET',
            url: '',
            headers: {},
            body: null,
            timeout: 30000,
            retries: 3,
        },
        inputSchema: {
            type: 'object',
            properties: {
                params: { type: 'object' },
                body: { type: 'object' },
            },
        },
        outputSchema: {
            type: 'object',
            properties: {
                status: { type: 'number' },
                data: { type: 'object' },
                headers: { type: 'object' },
            },
        },
    },
    {
        id: 'db-query',
        type: 'database-query',
        name: 'Database Query',
        description: 'Execute a database query',
        icon: '💾',
        config: {
            connection: '',
            query: '',
            parameters: {},
        },
        inputSchema: {
            type: 'object',
            properties: {
                params: { type: 'object' },
            },
        },
        outputSchema: {
            type: 'object',
            properties: {
                rows: { type: 'array' },
                count: { type: 'number' },
            },
        },
    },
    {
        id: 'transform-data',
        type: 'transform-data',
        name: 'Transform Data',
        description: 'Transform data using JavaScript',
        icon: '🔧',
        config: {
            script: 'return input;',
            language: 'javascript',
        },
        inputSchema: {
            type: 'object',
            properties: {
                input: { type: 'object' },
            },
        },
        outputSchema: {
            type: 'object',
            properties: {
                output: { type: 'object' },
            },
        },
    },
    {
        id: 'notification-send',
        type: 'send-notification',
        name: 'Send Notification',
        description: 'Send push notification or in-app message',
        icon: '🔔',
        config: {
            channel: 'push',
            title: '',
            message: '',
            recipients: [],
            priority: 'normal',
        },
        inputSchema: {
            type: 'object',
            properties: {
                data: { type: 'object' },
            },
        },
        outputSchema: {
            type: 'object',
            properties: {
                sent: { type: 'boolean' },
                recipientCount: { type: 'number' },
            },
        },
    },
    {
        id: 'record-create',
        type: 'create-record',
        name: 'Create Record',
        description: 'Create a new record in database',
        icon: '➕',
        config: {
            table: '',
            data: {},
        },
        inputSchema: {
            type: 'object',
            properties: {
                data: { type: 'object' },
            },
            required: ['data'],
        },
        outputSchema: {
            type: 'object',
            properties: {
                id: { type: 'string' },
                created: { type: 'boolean' },
            },
        },
    },
    {
        id: 'record-update',
        type: 'update-record',
        name: 'Update Record',
        description: 'Update an existing record',
        icon: '✏️',
        config: {
            table: '',
            id: '',
            data: {},
        },
        inputSchema: {
            type: 'object',
            properties: {
                id: { type: 'string' },
                data: { type: 'object' },
            },
            required: ['id', 'data'],
        },
        outputSchema: {
            type: 'object',
            properties: {
                updated: { type: 'boolean' },
            },
        },
    },
    {
        id: 'record-delete',
        type: 'delete-record',
        name: 'Delete Record',
        description: 'Delete a record from database',
        icon: '🗑️',
        config: {
            table: '',
            id: '',
        },
        inputSchema: {
            type: 'object',
            properties: {
                id: { type: 'string' },
            },
            required: ['id'],
        },
        outputSchema: {
            type: 'object',
            properties: {
                deleted: { type: 'boolean' },
            },
        },
    },
];
export const WorkflowActions = ({ actions = DEFAULT_ACTIONS, selectedAction, onActionSelect, onActionCreate, onActionUpdate, onActionDelete, readOnly = false, }) => {
    const [searchTerm, setSearchTerm] = useState('');
    const [filterType, setFilterType] = useState('all');
    const [isCreating, setIsCreating] = useState(false);
    const [newAction, setNewAction] = useState({
        type: 'email',
        name: '',
        description: '',
        config: {},
    });
    const actionTypes = useMemo(() => {
        const types = new Set(actions.map((a) => a.type));
        return ['all', ...Array.from(types)];
    }, [actions]);
    const filteredActions = useMemo(() => {
        return actions.filter((action) => {
            const matchesSearch = searchTerm === '' ||
                action.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                action.description.toLowerCase().includes(searchTerm.toLowerCase());
            const matchesType = filterType === 'all' || action.type === filterType;
            return matchesSearch && matchesType;
        });
    }, [actions, searchTerm, filterType]);
    const handleActionClick = useCallback((action) => {
        if (onActionSelect) {
            onActionSelect(action);
        }
    }, [onActionSelect]);
    const handleCreateAction = useCallback(() => {
        if (onActionCreate && newAction.name && newAction.type) {
            onActionCreate({
                type: newAction.type,
                name: newAction.name,
                description: newAction.description || '',
                config: newAction.config || {},
            });
            setIsCreating(false);
            setNewAction({
                type: 'email',
                name: '',
                description: '',
                config: {},
            });
        }
    }, [onActionCreate, newAction]);
    const handleConfigUpdate = useCallback((key, value) => {
        if (selectedAction && onActionUpdate && !readOnly) {
            onActionUpdate(selectedAction.id, {
                config: {
                    ...selectedAction.config,
                    [key]: value,
                },
            });
        }
    }, [selectedAction, onActionUpdate, readOnly]);
    const renderActionConfig = useCallback((action) => {
        switch (action.type) {
            case 'email':
                return (React.createElement(React.Fragment, null,
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "To"),
                        React.createElement("input", { type: "email", value: action.config.to || '', onChange: (e) => handleConfigUpdate('to', e.target.value), disabled: readOnly, placeholder: "recipient@example.com", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                            } })),
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Subject"),
                        React.createElement("input", { type: "text", value: action.config.subject || '', onChange: (e) => handleConfigUpdate('subject', e.target.value), disabled: readOnly, placeholder: "Email subject", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                            } })),
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Body"),
                        React.createElement("textarea", { value: action.config.body || '', onChange: (e) => handleConfigUpdate('body', e.target.value), disabled: readOnly, placeholder: "Email body", rows: 5, style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                                resize: 'vertical',
                            } }))));
            case 'http-request':
                return (React.createElement(React.Fragment, null,
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Method"),
                        React.createElement("select", { value: action.config.method || 'GET', onChange: (e) => handleConfigUpdate('method', e.target.value), disabled: readOnly, style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                                cursor: readOnly ? 'default' : 'pointer',
                            } },
                            React.createElement("option", { value: "GET" }, "GET"),
                            React.createElement("option", { value: "POST" }, "POST"),
                            React.createElement("option", { value: "PUT" }, "PUT"),
                            React.createElement("option", { value: "PATCH" }, "PATCH"),
                            React.createElement("option", { value: "DELETE" }, "DELETE"))),
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "URL"),
                        React.createElement("input", { type: "url", value: action.config.url || '', onChange: (e) => handleConfigUpdate('url', e.target.value), disabled: readOnly, placeholder: "https://api.example.com/endpoint", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                            } })),
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Headers (JSON)"),
                        React.createElement("textarea", { value: JSON.stringify(action.config.headers || {}, null, 2), onChange: (e) => {
                                try {
                                    handleConfigUpdate('headers', JSON.parse(e.target.value));
                                }
                                catch { }
                            }, disabled: readOnly, rows: 4, style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '12px',
                                fontFamily: 'monospace',
                                resize: 'vertical',
                            } }))));
            case 'database-query':
                return (React.createElement(React.Fragment, null,
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Connection"),
                        React.createElement("input", { type: "text", value: action.config.connection || '', onChange: (e) => handleConfigUpdate('connection', e.target.value), disabled: readOnly, placeholder: "Database connection name", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                            } })),
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Query"),
                        React.createElement("textarea", { value: action.config.query || '', onChange: (e) => handleConfigUpdate('query', e.target.value), disabled: readOnly, placeholder: "SELECT * FROM users WHERE id = ?", rows: 5, style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '12px',
                                fontFamily: 'monospace',
                                resize: 'vertical',
                            } }))));
            case 'transform-data':
                return (React.createElement("div", { style: { marginBottom: '12px' } },
                    React.createElement("label", { style: {
                            display: 'block',
                            fontSize: '12px',
                            fontWeight: 600,
                            color: '#64748b',
                            marginBottom: '6px',
                        } }, "Transformation Script"),
                    React.createElement("textarea", { value: action.config.script || '', onChange: (e) => handleConfigUpdate('script', e.target.value), disabled: readOnly, placeholder: "return input;", rows: 10, style: {
                            width: '100%',
                            padding: '8px 12px',
                            border: '1px solid #e2e8f0',
                            borderRadius: '4px',
                            fontSize: '12px',
                            fontFamily: 'monospace',
                            resize: 'vertical',
                        } })));
            default:
                return (React.createElement("div", { style: { color: '#94a3b8', textAlign: 'center', padding: '20px' } }, "No configuration available for this action type"));
        }
    }, [handleConfigUpdate, readOnly]);
    return (React.createElement("div", { style: {
            display: 'flex',
            height: '100%',
            backgroundColor: '#fff',
            borderRadius: '8px',
            overflow: 'hidden',
            border: '1px solid #e2e8f0',
        } },
        React.createElement("div", { style: {
                width: '300px',
                borderRight: '1px solid #e2e8f0',
                display: 'flex',
                flexDirection: 'column',
            } },
            React.createElement("div", { style: { padding: '16px', borderBottom: '1px solid #e2e8f0' } },
                React.createElement("h3", { style: { fontSize: '16px', fontWeight: 600, marginBottom: '12px' } }, "Actions Library"),
                React.createElement("input", { type: "text", placeholder: "Search actions...", value: searchTerm, onChange: (e) => setSearchTerm(e.target.value), style: {
                        width: '100%',
                        padding: '8px 12px',
                        border: '1px solid #e2e8f0',
                        borderRadius: '6px',
                        fontSize: '13px',
                        marginBottom: '8px',
                    } }),
                !readOnly && (React.createElement("button", { onClick: () => setIsCreating(true), style: {
                        width: '100%',
                        padding: '8px 12px',
                        backgroundColor: '#3b82f6',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '6px',
                        cursor: 'pointer',
                        fontSize: '13px',
                        fontWeight: 500,
                    } }, "+ Create Action"))),
            React.createElement("div", { style: { flex: 1, overflow: 'auto', padding: '12px' } }, filteredActions.map((action) => (React.createElement("div", { key: action.id, onClick: () => handleActionClick(action), style: {
                    padding: '12px',
                    backgroundColor: selectedAction?.id === action.id ? '#eff6ff' : '#fff',
                    border: `1px solid ${selectedAction?.id === action.id ? '#3b82f6' : '#e2e8f0'}`,
                    borderRadius: '6px',
                    marginBottom: '8px',
                    cursor: 'pointer',
                    transition: 'all 0.2s ease',
                } },
                React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                    React.createElement("span", { style: { fontSize: '20px' } }, action.icon),
                    React.createElement("div", { style: { flex: 1 } },
                        React.createElement("div", { style: { fontSize: '14px', fontWeight: 600, color: '#1e293b' } }, action.name),
                        React.createElement("div", { style: { fontSize: '12px', color: '#64748b', marginTop: '2px' } }, action.description)))))))),
        React.createElement("div", { style: { flex: 1, display: 'flex', flexDirection: 'column' } }, selectedAction ? (React.createElement(React.Fragment, null,
            React.createElement("div", { style: {
                    padding: '16px',
                    borderBottom: '1px solid #e2e8f0',
                    backgroundColor: '#f8fafc',
                } },
                React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '12px' } },
                    React.createElement("span", { style: { fontSize: '32px' } }, selectedAction.icon),
                    React.createElement("div", null,
                        React.createElement("h3", { style: { fontSize: '18px', fontWeight: 600, color: '#1e293b' } }, selectedAction.name),
                        React.createElement("p", { style: { fontSize: '13px', color: '#64748b', marginTop: '4px' } }, selectedAction.description)))),
            React.createElement("div", { style: { flex: 1, overflow: 'auto', padding: '16px' } }, renderActionConfig(selectedAction)))) : (React.createElement("div", { style: {
                flex: 1,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: '#94a3b8',
            } }, "Select an action to view details")))));
};
export default WorkflowActions;
//# sourceMappingURL=WorkflowActions.js.map