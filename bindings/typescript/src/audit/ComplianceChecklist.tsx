/**
 * Compliance Checklist Component
 * Interactive checklist for compliance requirements across multiple frameworks
 */

import React, { useState, useEffect } from 'react';
import type { ComplianceRequirement, ComplianceFramework } from './types';

interface ComplianceChecklistProps {
  organizationId?: string;
  framework?: ComplianceFramework;
}

export const ComplianceChecklist: React.FC<ComplianceChecklistProps> = ({
  organizationId,
  framework: initialFramework,
}) => {
  const [selectedFramework, setSelectedFramework] = useState<ComplianceFramework>(
    initialFramework || 'SOC2'
  );
  const [requirements, setRequirements] = useState<ComplianceRequirement[]>([]);
  const [loading, setLoading] = useState(true);
  const [filterStatus, setFilterStatus] = useState<'all' | 'compliant' | 'non_compliant' | 'partial'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');

  useEffect(() => {
    loadRequirements();
  }, [selectedFramework, organizationId]);

  const loadRequirements = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams({
        framework: selectedFramework,
        ...(organizationId && { organization_id: organizationId }),
      });

      const response = await fetch(`/api/compliance/requirements?${params}`);
      const data = await response.json();
      setRequirements(data.requirements || []);
    } catch (error) {
      console.error('Failed to load compliance requirements:', error);
    } finally {
      setLoading(false);
    }
  };

  const updateRequirementStatus = async (
    requirementId: string,
    status: 'compliant' | 'non_compliant' | 'partial' | 'not_applicable'
  ) => {
    try {
      await fetch(`/api/compliance/requirements/${requirementId}/status`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status }),
      });

      setRequirements((prev) =>
        prev.map((req) =>
          req.id === requirementId ? { ...req, status } : req
        )
      );
    } catch (error) {
      console.error('Failed to update requirement status:', error);
    }
  };

  const filteredRequirements = requirements.filter((req) => {
    if (filterStatus !== 'all' && req.status !== filterStatus) return false;
    if (selectedCategory !== 'all' && req.category !== selectedCategory) return false;
    if (searchQuery && !req.title.toLowerCase().includes(searchQuery.toLowerCase()) &&
        !req.description.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false;
    }
    return true;
  });

  const categories = Array.from(new Set(requirements.map((r) => r.category)));

  const complianceStats = {
    total: requirements.length,
    compliant: requirements.filter((r) => r.status === 'compliant').length,
    non_compliant: requirements.filter((r) => r.status === 'non_compliant').length,
    partial: requirements.filter((r) => r.status === 'partial').length,
    not_applicable: requirements.filter((r) => r.status === 'not_applicable').length,
  };

  const overallCompliance =
    requirements.length > 0
      ? ((complianceStats.compliant + complianceStats.partial * 0.5) / requirements.length) * 100
      : 0;

  return (
    <div className="compliance-checklist">
      {/* Header */}
      <div className="checklist-header">
        <div>
          <h2>Compliance Checklist</h2>
          <p className="subtitle">
            Track and manage compliance requirements across frameworks
          </p>
        </div>
      </div>

      {/* Framework Selector */}
      <div className="framework-selector">
        {(['SOC2', 'GDPR', 'HIPAA', 'ISO27001', 'PCI_DSS', 'CCPA', 'NIST', 'FedRAMP'] as ComplianceFramework[]).map(
          (framework) => (
            <button
              key={framework}
              className={`framework-tab ${
                selectedFramework === framework ? 'active' : ''
              }`}
              onClick={() => setSelectedFramework(framework)}
            >
              {framework}
            </button>
          )
        )}
      </div>

      {/* Compliance Overview */}
      <div className="compliance-overview">
        <div className="overview-stats">
          <StatCard
            label="Overall Compliance"
            value={`${overallCompliance.toFixed(1)}%`}
            color={overallCompliance >= 80 ? 'green' : overallCompliance >= 60 ? 'yellow' : 'red'}
          />
          <StatCard
            label="Compliant"
            value={complianceStats.compliant.toString()}
            color="green"
          />
          <StatCard
            label="Partial"
            value={complianceStats.partial.toString()}
            color="yellow"
          />
          <StatCard
            label="Non-Compliant"
            value={complianceStats.non_compliant.toString()}
            color="red"
          />
        </div>

        <div className="progress-bar-container">
          <div className="progress-bar">
            <div
              className="progress-segment compliant"
              style={{
                width: `${(complianceStats.compliant / requirements.length) * 100}%`,
              }}
            />
            <div
              className="progress-segment partial"
              style={{
                width: `${(complianceStats.partial / requirements.length) * 100}%`,
              }}
            />
            <div
              className="progress-segment non-compliant"
              style={{
                width: `${(complianceStats.non_compliant / requirements.length) * 100}%`,
              }}
            />
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className="checklist-filters">
        <input
          type="text"
          className="search-input"
          placeholder="Search requirements..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />

        <select
          value={selectedCategory}
          onChange={(e) => setSelectedCategory(e.target.value)}
          className="category-select"
        >
          <option value="all">All Categories</option>
          {categories.map((category) => (
            <option key={category} value={category}>
              {category}
            </option>
          ))}
        </select>

        <div className="status-filters">
          {(['all', 'compliant', 'partial', 'non_compliant'] as const).map((status) => (
            <button
              key={status}
              className={`status-filter ${filterStatus === status ? 'active' : ''}`}
              onClick={() => setFilterStatus(status)}
            >
              {status === 'all' ? 'All' : status.replace('_', ' ')}
            </button>
          ))}
        </div>
      </div>

      {/* Requirements List */}
      {loading ? (
        <div className="loading-state">
          <div className="loading-spinner" />
          <p>Loading compliance requirements...</p>
        </div>
      ) : (
        <div className="requirements-list">
          {filteredRequirements.length === 0 ? (
            <div className="empty-state">
              <p>No requirements match your filters</p>
            </div>
          ) : (
            filteredRequirements.map((requirement) => (
              <RequirementCard
                key={requirement.id}
                requirement={requirement}
                onUpdateStatus={updateRequirementStatus}
              />
            ))
          )}
        </div>
      )}
    </div>
  );
};

// Stat Card Component
function StatCard({
  label,
  value,
  color,
}: {
  label: string;
  value: string;
  color: 'green' | 'yellow' | 'red' | 'gray';
}) {
  return (
    <div className={`stat-card stat-${color}`}>
      <div className="stat-value">{value}</div>
      <div className="stat-label">{label}</div>
    </div>
  );
}

// Requirement Card Component
function RequirementCard({
  requirement,
  onUpdateStatus,
}: {
  requirement: ComplianceRequirement;
  onUpdateStatus: (id: string, status: ComplianceRequirement['status']) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const [showEvidenceModal, setShowEvidenceModal] = useState(false);

  const statusColors: Record<ComplianceRequirement['status'], string> = {
    compliant: 'green',
    partial: 'yellow',
    non_compliant: 'red',
    not_applicable: 'gray',
  };

  const statusIcons: Record<ComplianceRequirement['status'], string> = {
    compliant: '✓',
    partial: '◐',
    non_compliant: '✗',
    not_applicable: '○',
  };

  return (
    <div className={`requirement-card status-${requirement.status}`}>
      <div className="requirement-header" onClick={() => setExpanded(!expanded)}>
        <div className="requirement-title-section">
          <div className="requirement-id">{requirement.requirement_id}</div>
          <div className="requirement-title">{requirement.title}</div>
          <div className="requirement-category">{requirement.category}</div>
        </div>

        <div className="requirement-status-section">
          <div className="compliance-percentage">
            {requirement.compliance_percentage.toFixed(0)}%
          </div>
          <div className={`status-badge badge-${statusColors[requirement.status]}`}>
            {statusIcons[requirement.status]} {requirement.status.replace('_', ' ')}
          </div>
          <button className="expand-button">
            {expanded ? '▼' : '▶'}
          </button>
        </div>
      </div>

      {expanded && (
        <div className="requirement-details">
          <div className="requirement-description">{requirement.description}</div>

          {/* Evidence */}
          <div className="evidence-section">
            <div className="evidence-header">
              <h4>Evidence ({requirement.evidence_collected.length}/{requirement.evidence_required.length})</h4>
              <button
                className="btn btn-sm btn-secondary"
                onClick={() => setShowEvidenceModal(true)}
              >
                Add Evidence
              </button>
            </div>

            {requirement.evidence_required.length > 0 && (
              <div className="evidence-required">
                <strong>Required:</strong>
                <ul>
                  {requirement.evidence_required.map((evidence, index) => (
                    <li key={index}>{evidence}</li>
                  ))}
                </ul>
              </div>
            )}

            {requirement.evidence_collected.length > 0 && (
              <div className="evidence-collected">
                <strong>Collected:</strong>
                <ul>
                  {requirement.evidence_collected.map((evidence, index) => (
                    <li key={index}>
                      <div className="evidence-type">{evidence.type}</div>
                      <div className="evidence-description">{evidence.description}</div>
                      <div className="evidence-meta">
                        Collected by {evidence.collected_by} on{' '}
                        {new Date(evidence.collected_at).toLocaleDateString()}
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>

          {/* Remediation Tasks */}
          {requirement.remediation_required && requirement.remediation_tasks && (
            <div className="remediation-section">
              <h4>Remediation Tasks</h4>
              <ul className="remediation-tasks">
                {requirement.remediation_tasks.map((task) => (
                  <li key={task.id} className={`task-${task.status}`}>
                    <div className="task-description">{task.description}</div>
                    <div className="task-meta">
                      {task.assigned_to && <span>Assigned to: {task.assigned_to}</span>}
                      {task.due_date && (
                        <span>Due: {new Date(task.due_date).toLocaleDateString()}</span>
                      )}
                      <span className={`status-badge badge-${task.status}`}>
                        {task.status}
                      </span>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Assessment Info */}
          <div className="assessment-info">
            <div className="info-item">
              <strong>Last Assessed:</strong>{' '}
              {new Date(requirement.last_assessed).toLocaleDateString()}
            </div>
            <div className="info-item">
              <strong>Assessed By:</strong> {requirement.assessed_by}
            </div>
            <div className="info-item">
              <strong>Next Assessment:</strong>{' '}
              {new Date(requirement.next_assessment).toLocaleDateString()}
            </div>
          </div>

          {/* Status Update */}
          <div className="status-update-section">
            <label>Update Status:</label>
            <div className="status-buttons">
              {(['compliant', 'partial', 'non_compliant', 'not_applicable'] as const).map(
                (status) => (
                  <button
                    key={status}
                    className={`status-button ${
                      requirement.status === status ? 'active' : ''
                    } status-${statusColors[status]}`}
                    onClick={() => onUpdateStatus(requirement.id, status)}
                  >
                    {statusIcons[status]} {status.replace('_', ' ')}
                  </button>
                )
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
