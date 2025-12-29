import React, { useState, useCallback, useMemo } from 'react';
import { useDrag } from 'react-dnd';
const NODE_PALETTE = [
    {
        type: 'trigger',
        label: 'Trigger',
        description: 'Start workflow on event',
        icon: '⚡',
        color: '#10b981',
        category: 'Triggers',
    },
    {
        type: 'webhook',
        label: 'Webhook',
        description: 'Receive HTTP webhook',
        icon: '🔗',
        color: '#14b8a6',
        category: 'Triggers',
    },
    {
        type: 'action',
        label: 'Action',
        description: 'Perform an action',
        icon: '🎯',
        color: '#3b82f6',
        category: 'Actions',
    },
    {
        type: 'email',
        label: 'Send Email',
        description: 'Send email message',
        icon: '📧',
        color: '#ef4444',
        category: 'Actions',
    },
    {
        type: 'api',
        label: 'API Request',
        description: 'Make HTTP request',
        icon: '🌐',
        color: '#ec4899',
        category: 'Actions',
    },
    {
        type: 'database',
        label: 'Database',
        description: 'Query database',
        icon: '💾',
        color: '#84cc16',
        category: 'Actions',
    },
    {
        type: 'condition',
        label: 'Condition',
        description: 'Branch based on logic',
        icon: '🔀',
        color: '#f59e0b',
        category: 'Logic',
    },
    {
        type: 'loop',
        label: 'Loop',
        description: 'Iterate over items',
        icon: '🔄',
        color: '#8b5cf6',
        category: 'Logic',
    },
    {
        type: 'delay',
        label: 'Delay',
        description: 'Wait for duration',
        icon: '⏱',
        color: '#6366f1',
        category: 'Logic',
    },
    {
        type: 'transform',
        label: 'Transform',
        description: 'Transform data',
        icon: '🔧',
        color: '#06b6d4',
        category: 'Data',
    },
    {
        type: 'script',
        label: 'Script',
        description: 'Run custom code',
        icon: '📝',
        color: '#64748b',
        category: 'Advanced',
    },
];
const PaletteNode = ({ item }) => {
    const [{ isDragging }, dragRef] = useDrag({
        type: 'workflow-node-palette',
        item: {
            nodeData: {
                type: item.type,
                label: item.label,
                data: {
                    description: item.description,
                    icon: item.icon,
                    color: item.color,
                },
            },
        },
        collect: (monitor) => ({
            isDragging: monitor.isDragging(),
        }),
    });
    return (React.createElement("div", { ref: dragRef, style: {
            padding: '12px',
            backgroundColor: '#fff',
            borderRadius: '6px',
            border: '1px solid #e2e8f0',
            cursor: 'grab',
            opacity: isDragging ? 0.5 : 1,
            transition: 'all 0.2s ease',
            display: 'flex',
            alignItems: 'center',
            gap: '12px',
        }, className: "palette-node" },
        React.createElement("div", { style: {
                width: '40px',
                height: '40px',
                borderRadius: '8px',
                backgroundColor: `${item.color}20`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontSize: '20px',
            } }, item.icon),
        React.createElement("div", { style: { flex: 1 } },
            React.createElement("div", { style: { fontWeight: 600, fontSize: '14px', color: '#1e293b' } }, item.label),
            React.createElement("div", { style: { fontSize: '12px', color: '#64748b', marginTop: '2px' } }, item.description))));
};
export const WorkflowSidebar = ({ selectedNode, variables = [], onNodeUpdate, onVariableCreate, onVariableUpdate, onVariableDelete, readOnly = false, }) => {
    const [activeTab, setActiveTab] = useState('palette');
    const [searchTerm, setSearchTerm] = useState('');
    const [selectedCategory, setSelectedCategory] = useState('all');
    const categories = useMemo(() => {
        const cats = new Set(NODE_PALETTE.map((item) => item.category));
        return ['all', ...Array.from(cats)];
    }, []);
    const filteredPalette = useMemo(() => {
        return NODE_PALETTE.filter((item) => {
            const matchesSearch = item.label.toLowerCase().includes(searchTerm.toLowerCase()) ||
                item.description.toLowerCase().includes(searchTerm.toLowerCase());
            const matchesCategory = selectedCategory === 'all' || item.category === selectedCategory;
            return matchesSearch && matchesCategory;
        });
    }, [searchTerm, selectedCategory]);
    const groupedPalette = useMemo(() => {
        const grouped = {};
        filteredPalette.forEach((item) => {
            if (!grouped[item.category]) {
                grouped[item.category] = [];
            }
            grouped[item.category].push(item);
        });
        return grouped;
    }, [filteredPalette]);
    const handlePropertyChange = useCallback((key, value) => {
        if (selectedNode && onNodeUpdate && !readOnly) {
            onNodeUpdate(selectedNode.id, {
                config: {
                    ...selectedNode.config,
                    [key]: value,
                },
            });
        }
    }, [selectedNode, onNodeUpdate, readOnly]);
    const handleVariableAdd = useCallback(() => {
        if (onVariableCreate && !readOnly) {
            onVariableCreate({
                name: 'newVariable',
                type: 'string',
                value: '',
                scope: 'global',
            });
        }
    }, [onVariableCreate, readOnly]);
    const sidebarStyle = {
        width: '320px',
        height: '100%',
        backgroundColor: '#fff',
        borderLeft: '1px solid #e2e8f0',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
    };
    const tabBarStyle = {
        display: 'flex',
        borderBottom: '1px solid #e2e8f0',
        backgroundColor: '#f8fafc',
    };
    const tabStyle = (isActive) => ({
        flex: 1,
        padding: '12px',
        backgroundColor: isActive ? '#fff' : 'transparent',
        border: 'none',
        borderBottom: isActive ? '2px solid #3b82f6' : '2px solid transparent',
        cursor: 'pointer',
        fontSize: '14px',
        fontWeight: isActive ? 600 : 400,
        color: isActive ? '#3b82f6' : '#64748b',
        transition: 'all 0.2s ease',
    });
    const contentStyle = {
        flex: 1,
        overflow: 'auto',
        padding: '16px',
    };
    return (React.createElement("div", { style: sidebarStyle, className: "workflow-sidebar" },
        React.createElement("div", { style: tabBarStyle },
            React.createElement("button", { onClick: () => setActiveTab('palette'), style: tabStyle(activeTab === 'palette') }, "Nodes"),
            React.createElement("button", { onClick: () => setActiveTab('properties'), style: tabStyle(activeTab === 'properties') }, "Properties"),
            React.createElement("button", { onClick: () => setActiveTab('variables'), style: tabStyle(activeTab === 'variables') }, "Variables")),
        React.createElement("div", { style: contentStyle },
            activeTab === 'palette' && (React.createElement("div", null,
                React.createElement("input", { type: "text", placeholder: "Search nodes...", value: searchTerm, onChange: (e) => setSearchTerm(e.target.value), style: {
                        width: '100%',
                        padding: '8px 12px',
                        border: '1px solid #e2e8f0',
                        borderRadius: '6px',
                        fontSize: '14px',
                        marginBottom: '12px',
                    } }),
                React.createElement("div", { style: {
                        display: 'flex',
                        gap: '8px',
                        marginBottom: '16px',
                        flexWrap: 'wrap',
                    } }, categories.map((category) => (React.createElement("button", { key: category, onClick: () => setSelectedCategory(category), style: {
                        padding: '6px 12px',
                        backgroundColor: selectedCategory === category ? '#3b82f6' : '#f1f5f9',
                        color: selectedCategory === category ? '#fff' : '#64748b',
                        border: 'none',
                        borderRadius: '4px',
                        fontSize: '12px',
                        cursor: 'pointer',
                        fontWeight: 500,
                    } }, category.charAt(0).toUpperCase() + category.slice(1))))),
                Object.entries(groupedPalette).map(([category, items]) => (React.createElement("div", { key: category, style: { marginBottom: '24px' } },
                    React.createElement("h3", { style: {
                            fontSize: '12px',
                            fontWeight: 600,
                            color: '#64748b',
                            textTransform: 'uppercase',
                            marginBottom: '8px',
                            letterSpacing: '0.5px',
                        } }, category),
                    React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '8px' } }, items.map((item) => (React.createElement(PaletteNode, { key: item.type, item: item }))))))))),
            activeTab === 'properties' && (React.createElement("div", null, selectedNode ? (React.createElement("div", null,
                React.createElement("h3", { style: { fontSize: '16px', fontWeight: 600, marginBottom: '16px' } }, selectedNode.label),
                React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '16px' } },
                    React.createElement("div", null,
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Label"),
                        React.createElement("input", { type: "text", value: selectedNode.label, onChange: (e) => onNodeUpdate &&
                                !readOnly &&
                                onNodeUpdate(selectedNode.id, { label: e.target.value }), disabled: readOnly, style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '6px',
                                fontSize: '14px',
                            } })),
                    React.createElement("div", null,
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Description"),
                        React.createElement("textarea", { value: selectedNode.data.description || '', onChange: (e) => onNodeUpdate &&
                                !readOnly &&
                                onNodeUpdate(selectedNode.id, {
                                    data: { ...selectedNode.data, description: e.target.value },
                                }), disabled: readOnly, rows: 3, style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '6px',
                                fontSize: '14px',
                                resize: 'vertical',
                            } })),
                    selectedNode.type === 'email' && (React.createElement(React.Fragment, null,
                        React.createElement("div", null,
                            React.createElement("label", { style: {
                                    display: 'block',
                                    fontSize: '12px',
                                    fontWeight: 600,
                                    color: '#64748b',
                                    marginBottom: '6px',
                                } }, "To Email"),
                            React.createElement("input", { type: "email", value: selectedNode.config.to || '', onChange: (e) => handlePropertyChange('to', e.target.value), disabled: readOnly, style: {
                                    width: '100%',
                                    padding: '8px 12px',
                                    border: '1px solid #e2e8f0',
                                    borderRadius: '6px',
                                    fontSize: '14px',
                                } })),
                        React.createElement("div", null,
                            React.createElement("label", { style: {
                                    display: 'block',
                                    fontSize: '12px',
                                    fontWeight: 600,
                                    color: '#64748b',
                                    marginBottom: '6px',
                                } }, "Subject"),
                            React.createElement("input", { type: "text", value: selectedNode.config.subject || '', onChange: (e) => handlePropertyChange('subject', e.target.value), disabled: readOnly, style: {
                                    width: '100%',
                                    padding: '8px 12px',
                                    border: '1px solid #e2e8f0',
                                    borderRadius: '6px',
                                    fontSize: '14px',
                                } })))),
                    selectedNode.type === 'delay' && (React.createElement("div", null,
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Delay (seconds)"),
                        React.createElement("input", { type: "number", value: selectedNode.config.delay || 0, onChange: (e) => handlePropertyChange('delay', parseInt(e.target.value)), disabled: readOnly, min: "0", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '6px',
                                fontSize: '14px',
                            } })))))) : (React.createElement("div", { style: {
                    textAlign: 'center',
                    color: '#94a3b8',
                    padding: '40px 20px',
                } }, "Select a node to view properties")))),
            activeTab === 'variables' && (React.createElement("div", null,
                React.createElement("div", { style: {
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        marginBottom: '16px',
                    } },
                    React.createElement("h3", { style: { fontSize: '16px', fontWeight: 600 } }, "Variables"),
                    !readOnly && (React.createElement("button", { onClick: handleVariableAdd, style: {
                            padding: '6px 12px',
                            backgroundColor: '#3b82f6',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '14px',
                            fontWeight: 500,
                        } }, "+ Add"))),
                React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '12px' } },
                    variables.map((variable) => (React.createElement("div", { key: variable.id, style: {
                            padding: '12px',
                            backgroundColor: '#f8fafc',
                            borderRadius: '6px',
                            border: '1px solid #e2e8f0',
                        } },
                        React.createElement("div", { style: {
                                display: 'flex',
                                justifyContent: 'space-between',
                                alignItems: 'center',
                                marginBottom: '8px',
                            } },
                            React.createElement("input", { type: "text", value: variable.name, onChange: (e) => onVariableUpdate &&
                                    !readOnly &&
                                    onVariableUpdate(variable.id, { name: e.target.value }), disabled: readOnly, style: {
                                    flex: 1,
                                    padding: '6px 8px',
                                    border: '1px solid #e2e8f0',
                                    borderRadius: '4px',
                                    fontSize: '13px',
                                    fontWeight: 600,
                                } }),
                            !readOnly && (React.createElement("button", { onClick: () => onVariableDelete && onVariableDelete(variable.id), style: {
                                    marginLeft: '8px',
                                    padding: '4px 8px',
                                    backgroundColor: '#ef4444',
                                    color: '#fff',
                                    border: 'none',
                                    borderRadius: '4px',
                                    cursor: 'pointer',
                                    fontSize: '12px',
                                } }, "\u00D7"))),
                        React.createElement("select", { value: variable.type, onChange: (e) => onVariableUpdate &&
                                !readOnly &&
                                onVariableUpdate(variable.id, {
                                    type: e.target.value,
                                }), disabled: readOnly, style: {
                                width: '100%',
                                padding: '6px 8px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '12px',
                                marginBottom: '8px',
                            } },
                            React.createElement("option", { value: "string" }, "String"),
                            React.createElement("option", { value: "number" }, "Number"),
                            React.createElement("option", { value: "boolean" }, "Boolean"),
                            React.createElement("option", { value: "object" }, "Object"),
                            React.createElement("option", { value: "array" }, "Array")),
                        React.createElement("input", { type: "text", value: String(variable.value), onChange: (e) => onVariableUpdate &&
                                !readOnly &&
                                onVariableUpdate(variable.id, { value: e.target.value }), disabled: readOnly, placeholder: "Value", style: {
                                width: '100%',
                                padding: '6px 8px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '12px',
                            } })))),
                    variables.length === 0 && (React.createElement("div", { style: {
                            textAlign: 'center',
                            color: '#94a3b8',
                            padding: '40px 20px',
                        } }, "No variables defined"))))))));
};
export default WorkflowSidebar;
//# sourceMappingURL=WorkflowSidebar.js.map