/**
 * CADDY v0.4.0 - Workflow Templates Component
 * Pre-built workflow templates library
 */

import React, { useMemo, useState, useCallback } from 'react';
import type { WorkflowTemplate, Workflow } from './types';

export interface WorkflowTemplatesProps {
  templates?: WorkflowTemplate[];
  onTemplateSelect?: (template: WorkflowTemplate) => void;
  onTemplateCreate?: (workflow: Workflow) => void;
  onTemplateDelete?: (templateId: string) => void;
}

const DEFAULT_TEMPLATES: WorkflowTemplate[] = [
  {
    id: 'welcome-email',
    name: 'Welcome Email Automation',
    description: 'Send welcome email to new users automatically',
    category: 'Email',
    icon: '📧',
    popularity: 95,
    tags: ['email', 'onboarding', 'automation'],
    workflow: {
      name: 'Welcome Email',
      description: 'Automated welcome email workflow',
      version: '1.0.0',
      nodes: [
        {
          id: 'trigger-1',
          type: 'trigger',
          label: 'New User Signup',
          position: { x: 100, y: 100 },
          data: { description: 'Triggered when user signs up', icon: '👤' },
          inputs: [],
          outputs: [{ id: 'out-1', nodeId: 'trigger-1', type: 'output', label: 'User Data' }],
          config: { event: 'user.created' },
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
        {
          id: 'email-1',
          type: 'email',
          label: 'Send Welcome Email',
          position: { x: 400, y: 100 },
          data: { description: 'Send personalized welcome email', icon: '📧' },
          inputs: [{ id: 'in-1', nodeId: 'email-1', type: 'input', label: 'User Data' }],
          outputs: [{ id: 'out-1', nodeId: 'email-1', type: 'output', label: 'Email Status' }],
          config: {
            to: '{{user.email}}',
            subject: 'Welcome to CADDY!',
            template: 'welcome',
          },
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
      ],
      connections: [
        {
          id: 'conn-1',
          sourceNodeId: 'trigger-1',
          sourcePortId: 'out-1',
          targetNodeId: 'email-1',
          targetPortId: 'in-1',
        },
      ],
      variables: [],
      triggers: [],
      settings: {},
      metadata: {
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'system',
        lastModifiedBy: 'system',
        version: 1,
        status: 'published',
        isTemplate: true,
      },
    },
  },
  {
    id: 'data-sync',
    name: 'Database Sync',
    description: 'Sync data between databases on schedule',
    category: 'Data',
    icon: '🔄',
    popularity: 88,
    tags: ['database', 'sync', 'automation'],
    workflow: {
      name: 'Database Sync',
      description: 'Scheduled database synchronization',
      version: '1.0.0',
      nodes: [
        {
          id: 'trigger-1',
          type: 'trigger',
          label: 'Schedule Trigger',
          position: { x: 100, y: 100 },
          data: { description: 'Run every hour', icon: '⏰' },
          inputs: [],
          outputs: [{ id: 'out-1', nodeId: 'trigger-1', type: 'output', label: 'Trigger' }],
          config: { schedule: '0 * * * *' },
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
        {
          id: 'db-1',
          type: 'database',
          label: 'Fetch Source Data',
          position: { x: 400, y: 100 },
          data: { description: 'Query source database', icon: '💾' },
          inputs: [{ id: 'in-1', nodeId: 'db-1', type: 'input', label: 'Trigger' }],
          outputs: [{ id: 'out-1', nodeId: 'db-1', type: 'output', label: 'Data' }],
          config: { query: 'SELECT * FROM users WHERE updated_at > NOW() - INTERVAL 1 HOUR' },
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
        {
          id: 'db-2',
          type: 'database',
          label: 'Update Target',
          position: { x: 700, y: 100 },
          data: { description: 'Update target database', icon: '💾' },
          inputs: [{ id: 'in-1', nodeId: 'db-2', type: 'input', label: 'Data' }],
          outputs: [{ id: 'out-1', nodeId: 'db-2', type: 'output', label: 'Result' }],
          config: { operation: 'upsert' },
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
      ],
      connections: [
        {
          id: 'conn-1',
          sourceNodeId: 'trigger-1',
          sourcePortId: 'out-1',
          targetNodeId: 'db-1',
          targetPortId: 'in-1',
        },
        {
          id: 'conn-2',
          sourceNodeId: 'db-1',
          sourcePortId: 'out-1',
          targetNodeId: 'db-2',
          targetPortId: 'in-1',
        },
      ],
      variables: [],
      triggers: [],
      settings: {},
      metadata: {
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'system',
        lastModifiedBy: 'system',
        version: 1,
        status: 'published',
        isTemplate: true,
      },
    },
  },
  {
    id: 'approval-flow',
    name: 'Approval Workflow',
    description: 'Multi-step approval process with notifications',
    category: 'Business Process',
    icon: '✅',
    popularity: 92,
    tags: ['approval', 'notification', 'workflow'],
    workflow: {
      name: 'Approval Workflow',
      description: 'Request approval with notifications',
      version: '1.0.0',
      nodes: [
        {
          id: 'trigger-1',
          type: 'webhook',
          label: 'Approval Request',
          position: { x: 100, y: 200 },
          data: { description: 'Receive approval request', icon: '🔗' },
          inputs: [],
          outputs: [{ id: 'out-1', nodeId: 'trigger-1', type: 'output', label: 'Request' }],
          config: {},
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
        {
          id: 'email-1',
          type: 'email',
          label: 'Notify Approver',
          position: { x: 400, y: 200 },
          data: { description: 'Send notification to approver', icon: '📧' },
          inputs: [{ id: 'in-1', nodeId: 'email-1', type: 'input', label: 'Request' }],
          outputs: [{ id: 'out-1', nodeId: 'email-1', type: 'output', label: 'Sent' }],
          config: {
            to: '{{approver.email}}',
            subject: 'Approval Required',
          },
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
        {
          id: 'condition-1',
          type: 'condition',
          label: 'Check Approval',
          position: { x: 700, y: 200 },
          data: { description: 'Check approval status', icon: '🔀' },
          inputs: [{ id: 'in-1', nodeId: 'condition-1', type: 'input', label: 'Response' }],
          outputs: [
            { id: 'out-1', nodeId: 'condition-1', type: 'output', label: 'Approved' },
            { id: 'out-2', nodeId: 'condition-1', type: 'output', label: 'Rejected' },
          ],
          config: { field: 'status', operator: 'equals', value: 'approved' },
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
      ],
      connections: [
        {
          id: 'conn-1',
          sourceNodeId: 'trigger-1',
          sourcePortId: 'out-1',
          targetNodeId: 'email-1',
          targetPortId: 'in-1',
        },
        {
          id: 'conn-2',
          sourceNodeId: 'email-1',
          sourcePortId: 'out-1',
          targetNodeId: 'condition-1',
          targetPortId: 'in-1',
        },
      ],
      variables: [],
      triggers: [],
      settings: {},
      metadata: {
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'system',
        lastModifiedBy: 'system',
        version: 1,
        status: 'published',
        isTemplate: true,
      },
    },
  },
  {
    id: 'api-integration',
    name: 'API Integration',
    description: 'Connect and sync with external APIs',
    category: 'Integration',
    icon: '🌐',
    popularity: 85,
    tags: ['api', 'integration', 'webhook'],
    workflow: {
      name: 'API Integration',
      description: 'External API integration workflow',
      version: '1.0.0',
      nodes: [
        {
          id: 'trigger-1',
          type: 'webhook',
          label: 'Webhook Trigger',
          position: { x: 100, y: 100 },
          data: { description: 'Receive webhook data', icon: '🔗' },
          inputs: [],
          outputs: [{ id: 'out-1', nodeId: 'trigger-1', type: 'output', label: 'Data' }],
          config: {},
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
        {
          id: 'transform-1',
          type: 'transform',
          label: 'Transform Data',
          position: { x: 400, y: 100 },
          data: { description: 'Transform incoming data', icon: '🔧' },
          inputs: [{ id: 'in-1', nodeId: 'transform-1', type: 'input', label: 'Raw Data' }],
          outputs: [{ id: 'out-1', nodeId: 'transform-1', type: 'output', label: 'Transformed' }],
          config: {},
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
        {
          id: 'api-1',
          type: 'api',
          label: 'API Request',
          position: { x: 700, y: 100 },
          data: { description: 'Send to external API', icon: '🌐' },
          inputs: [{ id: 'in-1', nodeId: 'api-1', type: 'input', label: 'Data' }],
          outputs: [{ id: 'out-1', nodeId: 'api-1', type: 'output', label: 'Response' }],
          config: { method: 'POST', url: 'https://api.example.com/data' },
          metadata: {
            createdAt: new Date(),
            updatedAt: new Date(),
            createdBy: 'system',
            version: 1,
          },
        },
      ],
      connections: [
        {
          id: 'conn-1',
          sourceNodeId: 'trigger-1',
          sourcePortId: 'out-1',
          targetNodeId: 'transform-1',
          targetPortId: 'in-1',
        },
        {
          id: 'conn-2',
          sourceNodeId: 'transform-1',
          sourcePortId: 'out-1',
          targetNodeId: 'api-1',
          targetPortId: 'in-1',
        },
      ],
      variables: [],
      triggers: [],
      settings: {},
      metadata: {
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'system',
        lastModifiedBy: 'system',
        version: 1,
        status: 'published',
        isTemplate: true,
      },
    },
  },
];

export const WorkflowTemplates: React.FC<WorkflowTemplatesProps> = ({
  templates = DEFAULT_TEMPLATES,
  onTemplateSelect,
  onTemplateCreate,
  onTemplateDelete,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedTemplate, setSelectedTemplate] = useState<WorkflowTemplate | null>(null);

  const categories = useMemo(() => {
    const cats = new Set(templates.map((t) => t.category));
    return ['all', ...Array.from(cats)];
  }, [templates]);

  const filteredTemplates = useMemo(() => {
    return templates
      .filter((template) => {
        const matchesSearch =
          searchTerm === '' ||
          template.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
          template.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
          template.tags?.some((tag) => tag.toLowerCase().includes(searchTerm.toLowerCase()));
        const matchesCategory =
          selectedCategory === 'all' || template.category === selectedCategory;
        return matchesSearch && matchesCategory;
      })
      .sort((a, b) => (b.popularity || 0) - (a.popularity || 0));
  }, [templates, searchTerm, selectedCategory]);

  const handleTemplateClick = useCallback((template: WorkflowTemplate) => {
    setSelectedTemplate(template);
  }, []);

  const handleUseTemplate = useCallback(() => {
    if (selectedTemplate && onTemplateSelect) {
      onTemplateSelect(selectedTemplate);
      setSelectedTemplate(null);
    }
  }, [selectedTemplate, onTemplateSelect]);

  const containerStyle: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#fff',
    borderRadius: '8px',
    overflow: 'hidden',
  };

  const headerStyle: React.CSSProperties = {
    padding: '16px',
    borderBottom: '1px solid #e2e8f0',
    backgroundColor: '#f8fafc',
  };

  return (
    <div style={containerStyle}>
      {/* Header */}
      <div style={headerStyle}>
        <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '12px' }}>
          Workflow Templates
        </h2>

        {/* Search and Filter */}
        <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
          <input
            type="text"
            placeholder="Search templates..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            style={{
              flex: 1,
              minWidth: '200px',
              padding: '8px 12px',
              border: '1px solid #e2e8f0',
              borderRadius: '6px',
              fontSize: '14px',
            }}
          />

          <select
            value={selectedCategory}
            onChange={(e) => setSelectedCategory(e.target.value)}
            style={{
              padding: '8px 12px',
              border: '1px solid #e2e8f0',
              borderRadius: '6px',
              fontSize: '14px',
              cursor: 'pointer',
            }}
          >
            {categories.map((cat) => (
              <option key={cat} value={cat}>
                {cat.charAt(0).toUpperCase() + cat.slice(1)}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Templates Grid */}
      <div style={{ flex: 1, overflow: 'auto', padding: '16px' }}>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
            gap: '16px',
          }}
        >
          {filteredTemplates.map((template) => (
            <div
              key={template.id}
              onClick={() => handleTemplateClick(template)}
              style={{
                backgroundColor: '#fff',
                border: `2px solid ${
                  selectedTemplate?.id === template.id ? '#3b82f6' : '#e2e8f0'
                }`,
                borderRadius: '8px',
                padding: '16px',
                cursor: 'pointer',
                transition: 'all 0.2s ease',
                boxShadow:
                  selectedTemplate?.id === template.id
                    ? '0 4px 12px rgba(59, 130, 246, 0.2)'
                    : '0 1px 3px rgba(0, 0, 0, 0.1)',
              }}
            >
              {/* Icon and Popularity */}
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'start',
                  marginBottom: '12px',
                }}
              >
                <div
                  style={{
                    width: '48px',
                    height: '48px',
                    borderRadius: '8px',
                    backgroundColor: '#f1f5f9',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    fontSize: '24px',
                  }}
                >
                  {template.icon}
                </div>
                {template.popularity && (
                  <div
                    style={{
                      padding: '4px 8px',
                      backgroundColor: '#ecfdf5',
                      color: '#10b981',
                      borderRadius: '4px',
                      fontSize: '11px',
                      fontWeight: 600,
                    }}
                  >
                    {template.popularity}% Popular
                  </div>
                )}
              </div>

              {/* Name and Description */}
              <h3
                style={{
                  fontSize: '16px',
                  fontWeight: 600,
                  color: '#1e293b',
                  marginBottom: '8px',
                }}
              >
                {template.name}
              </h3>
              <p
                style={{
                  fontSize: '13px',
                  color: '#64748b',
                  marginBottom: '12px',
                  lineHeight: '1.5',
                }}
              >
                {template.description}
              </p>

              {/* Category */}
              <div
                style={{
                  display: 'inline-block',
                  padding: '4px 8px',
                  backgroundColor: '#f1f5f9',
                  color: '#64748b',
                  borderRadius: '4px',
                  fontSize: '11px',
                  fontWeight: 500,
                  marginBottom: '12px',
                }}
              >
                {template.category}
              </div>

              {/* Tags */}
              {template.tags && template.tags.length > 0 && (
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px' }}>
                  {template.tags.map((tag) => (
                    <span
                      key={tag}
                      style={{
                        padding: '2px 6px',
                        backgroundColor: '#eff6ff',
                        color: '#3b82f6',
                        borderRadius: '3px',
                        fontSize: '10px',
                        fontWeight: 500,
                      }}
                    >
                      #{tag}
                    </span>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>

        {filteredTemplates.length === 0 && (
          <div
            style={{
              textAlign: 'center',
              color: '#94a3b8',
              padding: '40px 20px',
            }}
          >
            No templates found
          </div>
        )}
      </div>

      {/* Action Bar */}
      {selectedTemplate && (
        <div
          style={{
            padding: '16px',
            borderTop: '1px solid #e2e8f0',
            backgroundColor: '#f8fafc',
            display: 'flex',
            gap: '12px',
            justifyContent: 'flex-end',
          }}
        >
          <button
            onClick={() => setSelectedTemplate(null)}
            style={{
              padding: '10px 20px',
              backgroundColor: '#fff',
              color: '#64748b',
              border: '1px solid #e2e8f0',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: 500,
            }}
          >
            Cancel
          </button>
          <button
            onClick={handleUseTemplate}
            style={{
              padding: '10px 20px',
              backgroundColor: '#3b82f6',
              color: '#fff',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: 500,
            }}
          >
            Use Template
          </button>
        </div>
      )}
    </div>
  );
};

export default WorkflowTemplates;
