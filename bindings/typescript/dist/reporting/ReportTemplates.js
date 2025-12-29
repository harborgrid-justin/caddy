import React, { useState, useCallback } from 'react';
export const ReportTemplates = ({ templates, onSelectTemplate, onCreateFromTemplate, onSaveAsTemplate, showCreateButton = true, }) => {
    const [searchTerm, setSearchTerm] = useState('');
    const [selectedCategory, setSelectedCategory] = useState('all');
    const [selectedTemplate, setSelectedTemplate] = useState(null);
    const [viewMode, setViewMode] = useState('grid');
    const [sortBy, setSortBy] = useState('popularity');
    const [creating, setCreating] = useState(false);
    const categories = ['all', ...new Set(templates.map((t) => t.category))];
    const filteredTemplates = templates
        .filter((template) => {
        const matchesSearch = template.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
            template.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
            template.tags.some((tag) => tag.toLowerCase().includes(searchTerm.toLowerCase()));
        const matchesCategory = selectedCategory === 'all' || template.category === selectedCategory;
        return matchesSearch && matchesCategory;
    })
        .sort((a, b) => {
        switch (sortBy) {
            case 'name':
                return a.name.localeCompare(b.name);
            case 'popularity':
                return b.popularity - a.popularity;
            case 'recent':
                return new Date(b.metadata.updatedAt).getTime() - new Date(a.metadata.updatedAt).getTime();
            default:
                return 0;
        }
    });
    const handleCreateFromTemplate = useCallback(async (template) => {
        if (!onCreateFromTemplate)
            return;
        setCreating(true);
        try {
            await onCreateFromTemplate(template);
        }
        catch (error) {
            console.error('Failed to create report from template:', error);
            alert('Failed to create report from template. Please try again.');
        }
        finally {
            setCreating(false);
        }
    }, [onCreateFromTemplate]);
    const renderTemplateCard = (template) => (React.createElement("div", { key: template.id, onClick: () => setSelectedTemplate(template), style: {
            ...styles.templateCard,
            ...(selectedTemplate?.id === template.id ? styles.templateCardSelected : {}),
        } },
        template.thumbnail && (React.createElement("div", { style: styles.templateThumbnail },
            React.createElement("img", { src: template.thumbnail, alt: template.name, style: styles.thumbnailImage }))),
        React.createElement("div", { style: styles.templateContent },
            React.createElement("h4", { style: styles.templateName }, template.name),
            React.createElement("p", { style: styles.templateDescription }, template.description),
            React.createElement("div", { style: styles.templateMeta },
                React.createElement("span", { style: styles.templateCategory }, template.category),
                React.createElement("span", { style: styles.templatePopularity },
                    "\u2B50 ",
                    template.popularity)),
            React.createElement("div", { style: styles.templateTags },
                template.tags.slice(0, 3).map((tag, index) => (React.createElement("span", { key: index, style: styles.tag }, tag))),
                template.tags.length > 3 && (React.createElement("span", { style: styles.tagMore },
                    "+",
                    template.tags.length - 3))),
            React.createElement("div", { style: styles.templateFooter },
                React.createElement("span", { style: styles.templateAuthor },
                    "by ",
                    template.metadata.author),
                React.createElement("button", { onClick: (e) => {
                        e.stopPropagation();
                        handleCreateFromTemplate(template);
                    }, style: styles.useTemplateButton, disabled: creating }, "Use Template")))));
    const renderTemplateListItem = (template) => (React.createElement("div", { key: template.id, onClick: () => setSelectedTemplate(template), style: {
            ...styles.templateListItem,
            ...(selectedTemplate?.id === template.id ? styles.templateListItemSelected : {}),
        } },
        React.createElement("div", { style: styles.templateListLeft },
            template.thumbnail && (React.createElement("img", { src: template.thumbnail, alt: template.name, style: styles.listThumbnail })),
            React.createElement("div", { style: styles.templateListInfo },
                React.createElement("h4", { style: styles.templateListName }, template.name),
                React.createElement("p", { style: styles.templateListDescription }, template.description),
                React.createElement("div", { style: styles.templateListMeta },
                    React.createElement("span", { style: styles.templateCategory }, template.category),
                    React.createElement("span", null, "\u2022"),
                    React.createElement("span", null,
                        "by ",
                        template.metadata.author),
                    React.createElement("span", null, "\u2022"),
                    React.createElement("span", null,
                        "\u2B50 ",
                        template.popularity)))),
        React.createElement("button", { onClick: (e) => {
                e.stopPropagation();
                handleCreateFromTemplate(template);
            }, style: styles.useTemplateButton, disabled: creating }, "Use Template")));
    return (React.createElement("div", { style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h2", { style: styles.title }, "Report Templates"),
            React.createElement("div", { style: styles.headerActions },
                React.createElement("div", { style: styles.viewModeToggle },
                    React.createElement("button", { onClick: () => setViewMode('grid'), style: {
                            ...styles.viewModeButton,
                            ...(viewMode === 'grid' ? styles.viewModeButtonActive : {}),
                        } }, "\u229E"),
                    React.createElement("button", { onClick: () => setViewMode('list'), style: {
                            ...styles.viewModeButton,
                            ...(viewMode === 'list' ? styles.viewModeButtonActive : {}),
                        } }, "\u2630")))),
        React.createElement("div", { style: styles.filters },
            React.createElement("input", { type: "text", placeholder: "Search templates...", value: searchTerm, onChange: (e) => setSearchTerm(e.target.value), style: styles.searchInput }),
            React.createElement("select", { value: selectedCategory, onChange: (e) => setSelectedCategory(e.target.value), style: styles.categorySelect }, categories.map((category) => (React.createElement("option", { key: category, value: category }, category === 'all' ? 'All Categories' : category)))),
            React.createElement("select", { value: sortBy, onChange: (e) => setSortBy(e.target.value), style: styles.sortSelect },
                React.createElement("option", { value: "popularity" }, "Most Popular"),
                React.createElement("option", { value: "recent" }, "Most Recent"),
                React.createElement("option", { value: "name" }, "Name (A-Z)"))),
        React.createElement("div", { style: styles.content }, filteredTemplates.length === 0 ? (React.createElement("div", { style: styles.emptyState },
            React.createElement("div", { style: styles.emptyStateIcon }, "\uD83D\uDCCB"),
            React.createElement("div", { style: styles.emptyStateText }, "No templates found"),
            React.createElement("div", { style: styles.emptyStateHint }, "Try adjusting your search or filter criteria"))) : viewMode === 'grid' ? (React.createElement("div", { style: styles.templatesGrid }, filteredTemplates.map(renderTemplateCard))) : (React.createElement("div", { style: styles.templatesList }, filteredTemplates.map(renderTemplateListItem)))),
        selectedTemplate && (React.createElement("div", { style: styles.modalOverlay, onClick: () => setSelectedTemplate(null) },
            React.createElement("div", { style: styles.modal, onClick: (e) => e.stopPropagation() },
                React.createElement("div", { style: styles.modalHeader },
                    React.createElement("h3", { style: styles.modalTitle }, selectedTemplate.name),
                    React.createElement("button", { onClick: () => setSelectedTemplate(null), style: styles.modalCloseButton }, "\u2715")),
                React.createElement("div", { style: styles.modalBody },
                    selectedTemplate.thumbnail && (React.createElement("img", { src: selectedTemplate.thumbnail, alt: selectedTemplate.name, style: styles.modalThumbnail })),
                    React.createElement("div", { style: styles.modalSection },
                        React.createElement("h4", { style: styles.modalSectionTitle }, "Description"),
                        React.createElement("p", { style: styles.modalText }, selectedTemplate.description)),
                    React.createElement("div", { style: styles.modalSection },
                        React.createElement("h4", { style: styles.modalSectionTitle }, "Category"),
                        React.createElement("span", { style: styles.templateCategory }, selectedTemplate.category)),
                    React.createElement("div", { style: styles.modalSection },
                        React.createElement("h4", { style: styles.modalSectionTitle }, "Tags"),
                        React.createElement("div", { style: styles.modalTags }, selectedTemplate.tags.map((tag, index) => (React.createElement("span", { key: index, style: styles.tag }, tag))))),
                    selectedTemplate.requiredDataSources.length > 0 && (React.createElement("div", { style: styles.modalSection },
                        React.createElement("h4", { style: styles.modalSectionTitle }, "Required Data Sources"),
                        React.createElement("ul", { style: styles.modalList }, selectedTemplate.requiredDataSources.map((ds, index) => (React.createElement("li", { key: index }, ds)))))),
                    React.createElement("div", { style: styles.modalSection },
                        React.createElement("h4", { style: styles.modalSectionTitle }, "Template Info"),
                        React.createElement("div", { style: styles.modalInfo },
                            React.createElement("div", null,
                                "Author: ",
                                selectedTemplate.metadata.author),
                            React.createElement("div", null,
                                "Version: ",
                                selectedTemplate.metadata.version),
                            React.createElement("div", null,
                                "Popularity: \u2B50 ",
                                selectedTemplate.popularity),
                            React.createElement("div", null,
                                "Updated: ",
                                new Date(selectedTemplate.metadata.updatedAt).toLocaleDateString())))),
                React.createElement("div", { style: styles.modalFooter },
                    React.createElement("button", { onClick: () => setSelectedTemplate(null), style: styles.cancelButton }, "Cancel"),
                    React.createElement("button", { onClick: () => {
                            handleCreateFromTemplate(selectedTemplate);
                            setSelectedTemplate(null);
                        }, style: styles.createButton, disabled: creating }, creating ? 'Creating...' : 'Create Report')))))));
};
const styles = {
    container: {
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        backgroundColor: '#f8fafc',
        fontFamily: 'Inter, system-ui, sans-serif',
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '16px',
        backgroundColor: '#ffffff',
        borderBottom: '1px solid #e2e8f0',
    },
    title: {
        fontSize: '20px',
        fontWeight: 600,
        margin: 0,
        color: '#1e293b',
    },
    headerActions: {
        display: 'flex',
        gap: '8px',
    },
    viewModeToggle: {
        display: 'flex',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        overflow: 'hidden',
    },
    viewModeButton: {
        padding: '6px 12px',
        border: 'none',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '16px',
    },
    viewModeButtonActive: {
        backgroundColor: '#2563eb',
        color: '#ffffff',
    },
    filters: {
        display: 'flex',
        gap: '12px',
        padding: '16px',
        backgroundColor: '#ffffff',
        borderBottom: '1px solid #e2e8f0',
    },
    searchInput: {
        flex: 1,
        padding: '8px 12px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        fontSize: '14px',
    },
    categorySelect: {
        padding: '8px 12px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        fontSize: '14px',
        cursor: 'pointer',
    },
    sortSelect: {
        padding: '8px 12px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        fontSize: '14px',
        cursor: 'pointer',
    },
    content: {
        flex: 1,
        overflow: 'auto',
        padding: '16px',
    },
    templatesGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
        gap: '16px',
    },
    templateCard: {
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        overflow: 'hidden',
        cursor: 'pointer',
        transition: 'all 0.2s',
    },
    templateCardSelected: {
        borderColor: '#2563eb',
        boxShadow: '0 0 0 3px rgba(37, 99, 235, 0.1)',
    },
    templateThumbnail: {
        width: '100%',
        height: '150px',
        backgroundColor: '#f8fafc',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        overflow: 'hidden',
    },
    thumbnailImage: {
        width: '100%',
        height: '100%',
        objectFit: 'cover',
    },
    templateContent: {
        padding: '16px',
    },
    templateName: {
        fontSize: '16px',
        fontWeight: 600,
        margin: '0 0 8px 0',
        color: '#1e293b',
    },
    templateDescription: {
        fontSize: '13px',
        color: '#64748b',
        margin: '0 0 12px 0',
        lineHeight: 1.5,
    },
    templateMeta: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '12px',
    },
    templateCategory: {
        fontSize: '11px',
        padding: '2px 8px',
        backgroundColor: '#dbeafe',
        color: '#1e40af',
        borderRadius: '12px',
        fontWeight: 500,
    },
    templatePopularity: {
        fontSize: '12px',
        color: '#64748b',
    },
    templateTags: {
        display: 'flex',
        flexWrap: 'wrap',
        gap: '4px',
        marginBottom: '12px',
    },
    tag: {
        fontSize: '10px',
        padding: '2px 6px',
        backgroundColor: '#f1f5f9',
        color: '#475569',
        borderRadius: '4px',
    },
    tagMore: {
        fontSize: '10px',
        padding: '2px 6px',
        color: '#64748b',
    },
    templateFooter: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingTop: '12px',
        borderTop: '1px solid #e2e8f0',
    },
    templateAuthor: {
        fontSize: '11px',
        color: '#64748b',
    },
    useTemplateButton: {
        padding: '6px 12px',
        border: 'none',
        borderRadius: '4px',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        fontSize: '12px',
        fontWeight: 500,
        cursor: 'pointer',
    },
    templatesList: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
    },
    templateListItem: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '16px',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        cursor: 'pointer',
        transition: 'all 0.2s',
    },
    templateListItemSelected: {
        borderColor: '#2563eb',
        boxShadow: '0 0 0 3px rgba(37, 99, 235, 0.1)',
    },
    templateListLeft: {
        display: 'flex',
        gap: '16px',
        flex: 1,
    },
    listThumbnail: {
        width: '80px',
        height: '80px',
        borderRadius: '6px',
        objectFit: 'cover',
    },
    templateListInfo: {
        flex: 1,
    },
    templateListName: {
        fontSize: '16px',
        fontWeight: 600,
        margin: '0 0 4px 0',
        color: '#1e293b',
    },
    templateListDescription: {
        fontSize: '13px',
        color: '#64748b',
        margin: '0 0 8px 0',
    },
    templateListMeta: {
        display: 'flex',
        gap: '8px',
        fontSize: '12px',
        color: '#94a3b8',
    },
    emptyState: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '64px',
        gap: '12px',
    },
    emptyStateIcon: {
        fontSize: '48px',
    },
    emptyStateText: {
        fontSize: '16px',
        color: '#64748b',
        fontWeight: 500,
    },
    emptyStateHint: {
        fontSize: '13px',
        color: '#94a3b8',
    },
    modalOverlay: {
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
    },
    modal: {
        backgroundColor: '#ffffff',
        borderRadius: '8px',
        width: '600px',
        maxHeight: '80vh',
        display: 'flex',
        flexDirection: 'column',
        boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1)',
    },
    modalHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '16px',
        borderBottom: '1px solid #e2e8f0',
    },
    modalTitle: {
        fontSize: '18px',
        fontWeight: 600,
        margin: 0,
        color: '#1e293b',
    },
    modalCloseButton: {
        border: 'none',
        background: 'none',
        fontSize: '20px',
        color: '#64748b',
        cursor: 'pointer',
    },
    modalBody: {
        flex: 1,
        overflow: 'auto',
        padding: '16px',
    },
    modalThumbnail: {
        width: '100%',
        height: '200px',
        objectFit: 'cover',
        borderRadius: '6px',
        marginBottom: '16px',
    },
    modalSection: {
        marginBottom: '16px',
    },
    modalSectionTitle: {
        fontSize: '13px',
        fontWeight: 600,
        margin: '0 0 8px 0',
        color: '#1e293b',
    },
    modalText: {
        fontSize: '13px',
        color: '#64748b',
        margin: 0,
        lineHeight: 1.5,
    },
    modalTags: {
        display: 'flex',
        flexWrap: 'wrap',
        gap: '6px',
    },
    modalList: {
        fontSize: '13px',
        color: '#64748b',
        margin: 0,
        paddingLeft: '20px',
    },
    modalInfo: {
        fontSize: '13px',
        color: '#64748b',
        display: 'flex',
        flexDirection: 'column',
        gap: '4px',
    },
    modalFooter: {
        display: 'flex',
        justifyContent: 'flex-end',
        gap: '8px',
        padding: '16px',
        borderTop: '1px solid #e2e8f0',
    },
    cancelButton: {
        padding: '8px 16px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '14px',
    },
    createButton: {
        padding: '8px 16px',
        border: 'none',
        borderRadius: '6px',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        cursor: 'pointer',
        fontSize: '14px',
        fontWeight: 500,
    },
};
export default ReportTemplates;
//# sourceMappingURL=ReportTemplates.js.map