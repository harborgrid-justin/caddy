/**
 * CADDY v0.4.0 - Report Templates Component
 * $650M Platform - Production Ready
 *
 * Comprehensive template library with categories, search, preview,
 * and one-click report creation from templates.
 */

import React, { useState, useCallback } from 'react';
import {
  ReportTemplate,
  ReportDefinition,
} from './types';

export interface ReportTemplatesProps {
  templates: ReportTemplate[];
  onSelectTemplate: (template: ReportTemplate) => void;
  onCreateFromTemplate?: (template: ReportTemplate) => Promise<ReportDefinition>;
  onSaveAsTemplate?: (definition: ReportDefinition, metadata: {
    name: string;
    description: string;
    category: string;
    tags: string[];
  }) => Promise<void>;
  showCreateButton?: boolean;
}

export const ReportTemplates: React.FC<ReportTemplatesProps> = ({
  templates,
  onSelectTemplate,
  onCreateFromTemplate,
  onSaveAsTemplate,
  showCreateButton = true,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedTemplate, setSelectedTemplate] = useState<ReportTemplate | null>(null);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [sortBy, setSortBy] = useState<'name' | 'popularity' | 'recent'>('popularity');
  const [creating, setCreating] = useState(false);

  // Get unique categories
  const categories = ['all', ...new Set(templates.map((t) => t.category))];

  // Filter and sort templates
  const filteredTemplates = templates
    .filter((template) => {
      const matchesSearch =
        template.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        template.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
        template.tags.some((tag) => tag.toLowerCase().includes(searchTerm.toLowerCase()));

      const matchesCategory =
        selectedCategory === 'all' || template.category === selectedCategory;

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

  // Handle template creation
  const handleCreateFromTemplate = useCallback(
    async (template: ReportTemplate) => {
      if (!onCreateFromTemplate) return;

      setCreating(true);
      try {
        await onCreateFromTemplate(template);
      } catch (error) {
        console.error('Failed to create report from template:', error);
        alert('Failed to create report from template. Please try again.');
      } finally {
        setCreating(false);
      }
    },
    [onCreateFromTemplate]
  );

  // Render template card
  const renderTemplateCard = (template: ReportTemplate) => (
    <div
      key={template.id}
      onClick={() => setSelectedTemplate(template)}
      style={{
        ...styles.templateCard,
        ...(selectedTemplate?.id === template.id ? styles.templateCardSelected : {}),
      }}
    >
      {template.thumbnail && (
        <div style={styles.templateThumbnail}>
          <img src={template.thumbnail} alt={template.name} style={styles.thumbnailImage} />
        </div>
      )}
      <div style={styles.templateContent}>
        <h4 style={styles.templateName}>{template.name}</h4>
        <p style={styles.templateDescription}>{template.description}</p>

        <div style={styles.templateMeta}>
          <span style={styles.templateCategory}>{template.category}</span>
          <span style={styles.templatePopularity}>⭐ {template.popularity}</span>
        </div>

        <div style={styles.templateTags}>
          {template.tags.slice(0, 3).map((tag, index) => (
            <span key={index} style={styles.tag}>
              {tag}
            </span>
          ))}
          {template.tags.length > 3 && (
            <span style={styles.tagMore}>+{template.tags.length - 3}</span>
          )}
        </div>

        <div style={styles.templateFooter}>
          <span style={styles.templateAuthor}>by {template.metadata.author}</span>
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleCreateFromTemplate(template);
            }}
            style={styles.useTemplateButton}
            disabled={creating}
          >
            Use Template
          </button>
        </div>
      </div>
    </div>
  );

  // Render template list item
  const renderTemplateListItem = (template: ReportTemplate) => (
    <div
      key={template.id}
      onClick={() => setSelectedTemplate(template)}
      style={{
        ...styles.templateListItem,
        ...(selectedTemplate?.id === template.id ? styles.templateListItemSelected : {}),
      }}
    >
      <div style={styles.templateListLeft}>
        {template.thumbnail && (
          <img src={template.thumbnail} alt={template.name} style={styles.listThumbnail} />
        )}
        <div style={styles.templateListInfo}>
          <h4 style={styles.templateListName}>{template.name}</h4>
          <p style={styles.templateListDescription}>{template.description}</p>
          <div style={styles.templateListMeta}>
            <span style={styles.templateCategory}>{template.category}</span>
            <span>•</span>
            <span>by {template.metadata.author}</span>
            <span>•</span>
            <span>⭐ {template.popularity}</span>
          </div>
        </div>
      </div>
      <button
        onClick={(e) => {
          e.stopPropagation();
          handleCreateFromTemplate(template);
        }}
        style={styles.useTemplateButton}
        disabled={creating}
      >
        Use Template
      </button>
    </div>
  );

  return (
    <div style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <h2 style={styles.title}>Report Templates</h2>
        <div style={styles.headerActions}>
          <div style={styles.viewModeToggle}>
            <button
              onClick={() => setViewMode('grid')}
              style={{
                ...styles.viewModeButton,
                ...(viewMode === 'grid' ? styles.viewModeButtonActive : {}),
              }}
            >
              ⊞
            </button>
            <button
              onClick={() => setViewMode('list')}
              style={{
                ...styles.viewModeButton,
                ...(viewMode === 'list' ? styles.viewModeButtonActive : {}),
              }}
            >
              ☰
            </button>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div style={styles.filters}>
        <input
          type="text"
          placeholder="Search templates..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          style={styles.searchInput}
        />

        <select
          value={selectedCategory}
          onChange={(e) => setSelectedCategory(e.target.value)}
          style={styles.categorySelect}
        >
          {categories.map((category) => (
            <option key={category} value={category}>
              {category === 'all' ? 'All Categories' : category}
            </option>
          ))}
        </select>

        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as 'name' | 'popularity' | 'recent')}
          style={styles.sortSelect}
        >
          <option value="popularity">Most Popular</option>
          <option value="recent">Most Recent</option>
          <option value="name">Name (A-Z)</option>
        </select>
      </div>

      {/* Content */}
      <div style={styles.content}>
        {filteredTemplates.length === 0 ? (
          <div style={styles.emptyState}>
            <div style={styles.emptyStateIcon}>📋</div>
            <div style={styles.emptyStateText}>No templates found</div>
            <div style={styles.emptyStateHint}>
              Try adjusting your search or filter criteria
            </div>
          </div>
        ) : viewMode === 'grid' ? (
          <div style={styles.templatesGrid}>
            {filteredTemplates.map(renderTemplateCard)}
          </div>
        ) : (
          <div style={styles.templatesList}>
            {filteredTemplates.map(renderTemplateListItem)}
          </div>
        )}
      </div>

      {/* Template Preview Modal */}
      {selectedTemplate && (
        <div style={styles.modalOverlay} onClick={() => setSelectedTemplate(null)}>
          <div style={styles.modal} onClick={(e) => e.stopPropagation()}>
            <div style={styles.modalHeader}>
              <h3 style={styles.modalTitle}>{selectedTemplate.name}</h3>
              <button
                onClick={() => setSelectedTemplate(null)}
                style={styles.modalCloseButton}
              >
                ✕
              </button>
            </div>

            <div style={styles.modalBody}>
              {selectedTemplate.thumbnail && (
                <img
                  src={selectedTemplate.thumbnail}
                  alt={selectedTemplate.name}
                  style={styles.modalThumbnail}
                />
              )}

              <div style={styles.modalSection}>
                <h4 style={styles.modalSectionTitle}>Description</h4>
                <p style={styles.modalText}>{selectedTemplate.description}</p>
              </div>

              <div style={styles.modalSection}>
                <h4 style={styles.modalSectionTitle}>Category</h4>
                <span style={styles.templateCategory}>{selectedTemplate.category}</span>
              </div>

              <div style={styles.modalSection}>
                <h4 style={styles.modalSectionTitle}>Tags</h4>
                <div style={styles.modalTags}>
                  {selectedTemplate.tags.map((tag, index) => (
                    <span key={index} style={styles.tag}>
                      {tag}
                    </span>
                  ))}
                </div>
              </div>

              {selectedTemplate.requiredDataSources.length > 0 && (
                <div style={styles.modalSection}>
                  <h4 style={styles.modalSectionTitle}>Required Data Sources</h4>
                  <ul style={styles.modalList}>
                    {selectedTemplate.requiredDataSources.map((ds, index) => (
                      <li key={index}>{ds}</li>
                    ))}
                  </ul>
                </div>
              )}

              <div style={styles.modalSection}>
                <h4 style={styles.modalSectionTitle}>Template Info</h4>
                <div style={styles.modalInfo}>
                  <div>Author: {selectedTemplate.metadata.author}</div>
                  <div>Version: {selectedTemplate.metadata.version}</div>
                  <div>Popularity: ⭐ {selectedTemplate.popularity}</div>
                  <div>
                    Updated: {new Date(selectedTemplate.metadata.updatedAt).toLocaleDateString()}
                  </div>
                </div>
              </div>
            </div>

            <div style={styles.modalFooter}>
              <button
                onClick={() => setSelectedTemplate(null)}
                style={styles.cancelButton}
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  handleCreateFromTemplate(selectedTemplate);
                  setSelectedTemplate(null);
                }}
                style={styles.createButton}
                disabled={creating}
              >
                {creating ? 'Creating...' : 'Create Report'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

// Styles
const styles: Record<string, React.CSSProperties> = {
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
