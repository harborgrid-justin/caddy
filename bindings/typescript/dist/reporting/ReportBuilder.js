import React, { useState, useCallback, useRef, useEffect } from 'react';
const defaultTheme = {
    name: 'Default',
    colors: {
        primary: '#2563eb',
        secondary: '#64748b',
        background: '#ffffff',
        text: '#1e293b',
        border: '#e2e8f0',
        chart: ['#2563eb', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#06b6d4'],
    },
    fonts: {
        title: { family: 'Inter', size: 24, weight: 700, color: '#1e293b' },
        header: { family: 'Inter', size: 16, weight: 600, color: '#334155' },
        body: { family: 'Inter', size: 14, weight: 400, color: '#475569' },
    },
    spacing: {
        section: 16,
        element: 8,
    },
};
export const ReportBuilder = ({ initialDefinition, dataSources, onSave, onCancel, onValidate, readOnly = false, }) => {
    const [state, setState] = useState({
        definition: initializeDefinition(initialDefinition),
        previewMode: false,
        isDirty: false,
        history: {
            past: [],
            future: [],
        },
    });
    const [selectedSection, setSelectedSection] = useState(null);
    const [draggedItem, setDraggedItem] = useState(null);
    const [validation, setValidation] = useState(null);
    const [saving, setSaving] = useState(false);
    const canvasRef = useRef(null);
    const [zoom, setZoom] = useState(100);
    function initializeDefinition(partial) {
        const now = new Date();
        return {
            id: partial?.id || generateId(),
            name: partial?.name || 'Untitled Report',
            description: partial?.description,
            type: partial?.type || 'table',
            version: partial?.version || 1,
            status: partial?.status || 'draft',
            dataSource: partial?.dataSource || dataSources[0] || {},
            query: partial?.query || {
                select: [],
                from: '',
            },
            layout: partial?.layout || {
                type: 'grid',
                sections: [],
                theme: defaultTheme,
                pageSize: { width: 1000, height: 1400, orientation: 'portrait' },
                margins: { top: 50, right: 50, bottom: 50, left: 50 },
            },
            parameters: partial?.parameters || [],
            permissions: partial?.permissions || [],
            metadata: {
                createdBy: partial?.metadata?.createdBy || 'current-user',
                createdAt: partial?.metadata?.createdAt || now,
                updatedBy: 'current-user',
                updatedAt: now,
                tags: partial?.metadata?.tags || [],
                category: partial?.metadata?.category,
                folder: partial?.metadata?.folder,
            },
        };
    }
    const updateDefinition = useCallback((updater) => {
        setState((prev) => {
            const newDefinition = updater(prev.definition);
            return {
                ...prev,
                definition: newDefinition,
                isDirty: true,
                history: {
                    past: [...prev.history.past, prev.definition],
                    future: [],
                },
            };
        });
    }, []);
    const undo = useCallback(() => {
        setState((prev) => {
            if (prev.history.past.length === 0)
                return prev;
            const previous = prev.history.past[prev.history.past.length - 1];
            const newPast = prev.history.past.slice(0, -1);
            return {
                ...prev,
                definition: previous,
                history: {
                    past: newPast,
                    future: [prev.definition, ...prev.history.future],
                },
            };
        });
    }, []);
    const redo = useCallback(() => {
        setState((prev) => {
            if (prev.history.future.length === 0)
                return prev;
            const next = prev.history.future[0];
            const newFuture = prev.history.future.slice(1);
            return {
                ...prev,
                definition: next,
                history: {
                    past: [...prev.history.past, prev.definition],
                    future: newFuture,
                },
            };
        });
    }, []);
    useEffect(() => {
        const handleKeyDown = (e) => {
            if (readOnly)
                return;
            if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
                e.preventDefault();
                undo();
            }
            else if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
                e.preventDefault();
                redo();
            }
            else if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                e.preventDefault();
                handleSave();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [readOnly, undo, redo]);
    const addSection = useCallback((type, position) => {
        const newSection = {
            id: generateId(),
            type,
            position: position
                ? { ...position, width: 300, height: 200 }
                : {
                    x: 100,
                    y: state.definition.layout.sections.length * 220 + 100,
                    width: 300,
                    height: 200,
                },
            content: getDefaultContent(type),
            style: {},
        };
        updateDefinition((def) => ({
            ...def,
            layout: {
                ...def.layout,
                sections: [...def.layout.sections, newSection],
            },
        }));
        setSelectedSection(newSection.id);
    }, [state.definition.layout.sections.length, updateDefinition]);
    const updateSection = useCallback((sectionId, updates) => {
        updateDefinition((def) => ({
            ...def,
            layout: {
                ...def.layout,
                sections: def.layout.sections.map((section) => section.id === sectionId ? { ...section, ...updates } : section),
            },
        }));
    }, [updateDefinition]);
    const deleteSection = useCallback((sectionId) => {
        updateDefinition((def) => ({
            ...def,
            layout: {
                ...def.layout,
                sections: def.layout.sections.filter((section) => section.id !== sectionId),
            },
        }));
        if (selectedSection === sectionId) {
            setSelectedSection(null);
        }
    }, [selectedSection, updateDefinition]);
    const handleDragStart = useCallback((e, item) => {
        setDraggedItem(item);
        e.dataTransfer.effectAllowed = 'move';
    }, []);
    const handleDragOver = useCallback((e) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = 'move';
    }, []);
    const handleDrop = useCallback((e) => {
        e.preventDefault();
        if (!draggedItem || !canvasRef.current)
            return;
        const rect = canvasRef.current.getBoundingClientRect();
        const x = (e.clientX - rect.left) / (zoom / 100);
        const y = (e.clientY - rect.top) / (zoom / 100);
        if (draggedItem.section) {
            updateSection(draggedItem.section.id, {
                position: { ...draggedItem.section.position, x, y },
            });
        }
        else if (draggedItem.type) {
            addSection(draggedItem.type, { x, y });
        }
        setDraggedItem(null);
    }, [draggedItem, zoom, updateSection, addSection]);
    const validate = useCallback(async () => {
        if (onValidate) {
            const result = await onValidate(state.definition);
            setValidation(result);
            return result;
        }
        const errors = [];
        const warnings = [];
        if (!state.definition.name || state.definition.name.trim() === '') {
            errors.push({ field: 'name', message: 'Report name is required', code: 'REQUIRED' });
        }
        if (!state.definition.dataSource || !state.definition.dataSource.id) {
            errors.push({ field: 'dataSource', message: 'Data source is required', code: 'REQUIRED' });
        }
        if (state.definition.query.select.length === 0) {
            warnings.push({
                field: 'query.select',
                message: 'No fields selected',
                code: 'EMPTY_SELECTION',
            });
        }
        const result = {
            valid: errors.length === 0,
            errors,
            warnings,
        };
        setValidation(result);
        return result;
    }, [state.definition, onValidate]);
    const handleSave = useCallback(async () => {
        if (readOnly || saving)
            return;
        setSaving(true);
        try {
            const validationResult = await validate();
            if (!validationResult.valid) {
                alert('Please fix validation errors before saving');
                return;
            }
            await onSave({
                ...state.definition,
                metadata: {
                    ...state.definition.metadata,
                    updatedBy: 'current-user',
                    updatedAt: new Date(),
                },
            });
            setState((prev) => ({ ...prev, isDirty: false }));
        }
        catch (error) {
            console.error('Failed to save report:', error);
            alert('Failed to save report. Please try again.');
        }
        finally {
            setSaving(false);
        }
    }, [readOnly, saving, validate, onSave, state.definition]);
    const togglePreview = useCallback(() => {
        setState((prev) => ({ ...prev, previewMode: !prev.previewMode }));
    }, []);
    return (React.createElement("div", { style: styles.container },
        React.createElement("div", { style: styles.toolbar },
            React.createElement("div", { style: styles.toolbarLeft },
                React.createElement("input", { type: "text", value: state.definition.name, onChange: (e) => updateDefinition((def) => ({ ...def, name: e.target.value })), style: styles.titleInput, disabled: readOnly, placeholder: "Report Name" }),
                React.createElement("span", { style: styles.versionBadge },
                    "v",
                    state.definition.version),
                React.createElement("span", { style: styles.statusBadge }, state.definition.status)),
            React.createElement("div", { style: styles.toolbarRight },
                React.createElement("button", { onClick: undo, disabled: readOnly || state.history.past.length === 0, style: styles.toolbarButton, title: "Undo (Ctrl+Z)" }, "\u21B6 Undo"),
                React.createElement("button", { onClick: redo, disabled: readOnly || state.history.future.length === 0, style: styles.toolbarButton, title: "Redo (Ctrl+Y)" }, "\u21B7 Redo"),
                React.createElement("div", { style: styles.separator }),
                React.createElement("select", { value: zoom, onChange: (e) => setZoom(Number(e.target.value)), style: styles.zoomSelect },
                    React.createElement("option", { value: 50 }, "50%"),
                    React.createElement("option", { value: 75 }, "75%"),
                    React.createElement("option", { value: 100 }, "100%"),
                    React.createElement("option", { value: 125 }, "125%"),
                    React.createElement("option", { value: 150 }, "150%")),
                React.createElement("button", { onClick: togglePreview, style: styles.toolbarButton }, state.previewMode ? '✏️ Edit' : '👁️ Preview'),
                React.createElement("div", { style: styles.separator }),
                React.createElement("button", { onClick: validate, style: styles.toolbarButton }, "\u2713 Validate"),
                React.createElement("button", { onClick: handleSave, disabled: readOnly || saving || !state.isDirty, style: {
                        ...styles.toolbarButton,
                        ...styles.saveButton,
                    } }, saving ? 'Saving...' : '💾 Save'),
                onCancel && (React.createElement("button", { onClick: onCancel, style: styles.toolbarButton }, "\u2715 Cancel")))),
        validation && (validation.errors.length > 0 || validation.warnings.length > 0) && (React.createElement("div", { style: styles.validationPanel },
            validation.errors.map((error, index) => (React.createElement("div", { key: index, style: styles.errorMessage },
                "\u26A0\uFE0F ",
                error.field,
                ": ",
                error.message))),
            validation.warnings.map((warning, index) => (React.createElement("div", { key: index, style: styles.warningMessage },
                "\u2139\uFE0F ",
                warning.field,
                ": ",
                warning.message))))),
        React.createElement("div", { style: styles.content },
            !state.previewMode && !readOnly && (React.createElement("div", { style: styles.palette },
                React.createElement("h3", { style: styles.paletteTitle }, "Components"),
                React.createElement("div", { style: styles.paletteGrid }, [
                    { type: 'header', icon: '📄', label: 'Header' },
                    { type: 'footer', icon: '📋', label: 'Footer' },
                    { type: 'table', icon: '📊', label: 'Table' },
                    { type: 'chart', icon: '📈', label: 'Chart' },
                    { type: 'text', icon: '📝', label: 'Text' },
                    { type: 'image', icon: '🖼️', label: 'Image' },
                ].map((component) => (React.createElement("div", { key: component.type, draggable: true, onDragStart: (e) => handleDragStart(e, { type: component.type }), style: styles.paletteItem },
                    React.createElement("span", { style: styles.paletteIcon }, component.icon),
                    React.createElement("span", { style: styles.paletteLabel }, component.label))))))),
            React.createElement("div", { style: styles.canvasContainer },
                React.createElement("div", { ref: canvasRef, onDragOver: handleDragOver, onDrop: handleDrop, style: {
                        ...styles.canvas,
                        transform: `scale(${zoom / 100})`,
                        width: state.definition.layout.pageSize?.width || 1000,
                        height: state.definition.layout.pageSize?.height || 1400,
                    } }, state.definition.layout.sections.map((section) => (React.createElement("div", { key: section.id, draggable: !state.previewMode && !readOnly, onDragStart: (e) => handleDragStart(e, { type: 'section', section, index: 0 }), onClick: () => !state.previewMode && setSelectedSection(section.id), style: {
                        ...styles.section,
                        ...section.style,
                        left: section.position.x,
                        top: section.position.y,
                        width: section.position.width,
                        height: section.position.height,
                        border: selectedSection === section.id
                            ? '2px solid #2563eb'
                            : '1px dashed #cbd5e1',
                        cursor: state.previewMode ? 'default' : 'move',
                    } },
                    React.createElement("div", { style: styles.sectionHeader },
                        React.createElement("span", { style: styles.sectionType }, section.type),
                        !state.previewMode && !readOnly && (React.createElement("button", { onClick: (e) => {
                                e.stopPropagation();
                                deleteSection(section.id);
                            }, style: styles.deleteButton }, "\u2715"))),
                    React.createElement("div", { style: styles.sectionContent }, renderSectionContent(section))))))),
            selectedSection && !state.previewMode && (React.createElement("div", { style: styles.propertiesPanel },
                React.createElement("h3", { style: styles.propertiesPanelTitle }, "Properties"),
                renderProperties(state.definition.layout.sections.find((s) => s.id === selectedSection), updateSection))))));
};
function generateId() {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}
function getDefaultContent(type) {
    switch (type) {
        case 'header':
            return { text: 'Report Header', alignment: 'center' };
        case 'footer':
            return { text: 'Page {{pageNumber}} of {{totalPages}}', alignment: 'center' };
        case 'text':
            return { text: 'Text content', format: 'plain' };
        case 'table':
            return { columns: [], groupBy: [] };
        case 'chart':
            return { type: 'bar', dataMapping: { xAxis: [], yAxis: [] } };
        case 'image':
            return { url: '', alt: 'Image' };
        default:
            return {};
    }
}
function renderSectionContent(section) {
    switch (section.type) {
        case 'header':
        case 'footer':
        case 'text':
            return React.createElement("div", null, section.content?.text || 'Empty');
        case 'table':
            return React.createElement("div", null,
                "Table: ",
                section.content?.columns?.length || 0,
                " columns");
        case 'chart':
            return React.createElement("div", null,
                "Chart: ",
                section.content?.type || 'bar');
        case 'image':
            return React.createElement("div", null,
                "Image: ",
                section.content?.url || 'No URL');
        default:
            return React.createElement("div", null, "Unknown section type");
    }
}
function renderProperties(section, updateSection) {
    return (React.createElement("div", { style: styles.propertiesForm },
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Width (px)"),
            React.createElement("input", { type: "number", value: section.position.width, onChange: (e) => updateSection(section.id, {
                    position: { ...section.position, width: Number(e.target.value) },
                }), style: styles.input })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Height (px)"),
            React.createElement("input", { type: "number", value: section.position.height, onChange: (e) => updateSection(section.id, {
                    position: { ...section.position, height: Number(e.target.value) },
                }), style: styles.input })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "X Position (px)"),
            React.createElement("input", { type: "number", value: section.position.x, onChange: (e) => updateSection(section.id, {
                    position: { ...section.position, x: Number(e.target.value) },
                }), style: styles.input })),
        React.createElement("div", { style: styles.formGroup },
            React.createElement("label", { style: styles.label }, "Y Position (px)"),
            React.createElement("input", { type: "number", value: section.position.y, onChange: (e) => updateSection(section.id, {
                    position: { ...section.position, y: Number(e.target.value) },
                }), style: styles.input }))));
}
const styles = {
    container: {
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        backgroundColor: '#f8fafc',
        fontFamily: 'Inter, system-ui, sans-serif',
    },
    toolbar: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '12px 16px',
        backgroundColor: '#ffffff',
        borderBottom: '1px solid #e2e8f0',
        boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
    },
    toolbarLeft: {
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
    },
    toolbarRight: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
    },
    titleInput: {
        fontSize: '18px',
        fontWeight: 600,
        border: 'none',
        outline: 'none',
        padding: '4px 8px',
        borderRadius: '4px',
        backgroundColor: 'transparent',
    },
    versionBadge: {
        fontSize: '12px',
        padding: '2px 8px',
        backgroundColor: '#e0e7ff',
        color: '#3730a3',
        borderRadius: '12px',
        fontWeight: 500,
    },
    statusBadge: {
        fontSize: '12px',
        padding: '2px 8px',
        backgroundColor: '#dbeafe',
        color: '#1e40af',
        borderRadius: '12px',
        fontWeight: 500,
        textTransform: 'capitalize',
    },
    toolbarButton: {
        padding: '6px 12px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '14px',
        fontWeight: 500,
        transition: 'all 0.2s',
    },
    saveButton: {
        backgroundColor: '#2563eb',
        color: '#ffffff',
        border: 'none',
    },
    separator: {
        width: '1px',
        height: '24px',
        backgroundColor: '#e2e8f0',
    },
    zoomSelect: {
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        fontSize: '14px',
        cursor: 'pointer',
    },
    validationPanel: {
        padding: '12px 16px',
        backgroundColor: '#fef3c7',
        borderBottom: '1px solid #fbbf24',
    },
    errorMessage: {
        color: '#991b1b',
        fontSize: '14px',
        marginBottom: '4px',
    },
    warningMessage: {
        color: '#92400e',
        fontSize: '14px',
        marginBottom: '4px',
    },
    content: {
        display: 'flex',
        flex: 1,
        overflow: 'hidden',
    },
    palette: {
        width: '200px',
        backgroundColor: '#ffffff',
        borderRight: '1px solid #e2e8f0',
        padding: '16px',
        overflowY: 'auto',
    },
    paletteTitle: {
        fontSize: '14px',
        fontWeight: 600,
        marginBottom: '12px',
        color: '#1e293b',
    },
    paletteGrid: {
        display: 'grid',
        gap: '8px',
    },
    paletteItem: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        padding: '8px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        cursor: 'grab',
        backgroundColor: '#ffffff',
        transition: 'all 0.2s',
    },
    paletteIcon: {
        fontSize: '20px',
    },
    paletteLabel: {
        fontSize: '13px',
        fontWeight: 500,
    },
    canvasContainer: {
        flex: 1,
        overflow: 'auto',
        padding: '32px',
        backgroundColor: '#f1f5f9',
    },
    canvas: {
        backgroundColor: '#ffffff',
        boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
        position: 'relative',
        transformOrigin: 'top left',
        margin: '0 auto',
    },
    section: {
        position: 'absolute',
        backgroundColor: '#ffffff',
        boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
        borderRadius: '4px',
        overflow: 'hidden',
    },
    sectionHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '4px 8px',
        backgroundColor: '#f8fafc',
        borderBottom: '1px solid #e2e8f0',
    },
    sectionType: {
        fontSize: '11px',
        fontWeight: 600,
        color: '#64748b',
        textTransform: 'uppercase',
    },
    deleteButton: {
        border: 'none',
        background: 'none',
        color: '#ef4444',
        cursor: 'pointer',
        fontSize: '14px',
        padding: '0 4px',
    },
    sectionContent: {
        padding: '12px',
        fontSize: '13px',
        color: '#475569',
    },
    propertiesPanel: {
        width: '280px',
        backgroundColor: '#ffffff',
        borderLeft: '1px solid #e2e8f0',
        padding: '16px',
        overflowY: 'auto',
    },
    propertiesPanelTitle: {
        fontSize: '14px',
        fontWeight: 600,
        marginBottom: '16px',
        color: '#1e293b',
    },
    propertiesForm: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
    },
    formGroup: {
        display: 'flex',
        flexDirection: 'column',
        gap: '4px',
    },
    label: {
        fontSize: '12px',
        fontWeight: 500,
        color: '#475569',
    },
    input: {
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '13px',
    },
};
export default ReportBuilder;
//# sourceMappingURL=ReportBuilder.js.map