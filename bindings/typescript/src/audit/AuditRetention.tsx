/**
 * Audit Retention Component
 * Data retention policy management for audit logs
 */

import React, { useState, useEffect } from 'react';
import type { RetentionRule, AuditEventType, AuditSeverity, DataClassification, ComplianceFramework } from './types';

interface AuditRetentionProps {
  organizationId?: string;
}

export const AuditRetention: React.FC<AuditRetentionProps> = ({ organizationId }) => {
  const [rules, setRules] = useState<RetentionRule[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingRule, setEditingRule] = useState<RetentionRule | null>(null);

  useEffect(() => {
    loadRetentionRules();
  }, [organizationId]);

  const loadRetentionRules = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams(
        organizationId ? { organization_id: organizationId } : {}
      );
      const response = await fetch(`/api/audit/retention?${params}`);
      const data = await response.json();
      setRules(data.rules || []);
    } catch (error) {
      console.error('Failed to load retention rules:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateRule = () => {
    setEditingRule(null);
    setShowCreateModal(true);
  };

  const handleEditRule = (rule: RetentionRule) => {
    setEditingRule(rule);
    setShowCreateModal(true);
  };

  const handleDeleteRule = async (ruleId: string) => {
    if (!confirm('Are you sure you want to delete this retention rule?')) return;

    try {
      await fetch(`/api/audit/retention/${ruleId}`, {
        method: 'DELETE',
      });
      setRules((prev) => prev.filter((r) => r.id !== ruleId));
    } catch (error) {
      console.error('Failed to delete retention rule:', error);
    }
  };

  const handleToggleRule = async (rule: RetentionRule) => {
    try {
      const response = await fetch(`/api/audit/retention/${rule.id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled: !rule.enabled }),
      });
      const updated = await response.json();
      setRules((prev) => prev.map((r) => (r.id === rule.id ? updated : r)));
    } catch (error) {
      console.error('Failed to toggle retention rule:', error);
    }
  };

  if (loading) {
    return (
      <div className="audit-retention loading">
        <div className="loading-spinner" />
        <p>Loading retention rules...</p>
      </div>
    );
  }

  return (
    <div className="audit-retention">
      {/* Header */}
      <div className="retention-header">
        <div>
          <h2>Audit Log Retention</h2>
          <p className="subtitle">
            Manage data retention policies for compliance and storage optimization
          </p>
        </div>
        <button className="btn btn-primary" onClick={handleCreateRule}>
          Create Retention Rule
        </button>
      </div>

      {/* Retention Overview */}
      <div className="retention-overview">
        <OverviewCard
          title="Total Rules"
          value={rules.length.toString()}
          icon="📋"
        />
        <OverviewCard
          title="Active Rules"
          value={rules.filter((r) => r.enabled).length.toString()}
          icon="✓"
        />
        <OverviewCard
          title="Legal Holds"
          value={rules.filter((r) => r.legal_hold).length.toString()}
          icon="⚖️"
        />
      </div>

      {/* Retention Rules Table */}
      <div className="retention-rules">
        {rules.length === 0 ? (
          <div className="empty-state">
            <h3>No Retention Rules</h3>
            <p>Create your first retention rule to manage audit log lifecycle.</p>
            <button className="btn btn-primary" onClick={handleCreateRule}>
              Create Retention Rule
            </button>
          </div>
        ) : (
          <table className="rules-table">
            <thead>
              <tr>
                <th>Rule Name</th>
                <th>Scope</th>
                <th>Retention Period</th>
                <th>Archive</th>
                <th>Delete</th>
                <th>Status</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {rules.map((rule) => (
                <tr key={rule.id} className={!rule.enabled ? 'disabled' : ''}>
                  <td>
                    <div className="rule-name">
                      {rule.name}
                      {rule.legal_hold && (
                        <span className="badge badge-warning" title="Legal Hold">
                          ⚖️
                        </span>
                      )}
                    </div>
                    <div className="rule-description">{rule.description}</div>
                  </td>
                  <td>
                    <RuleScope rule={rule} />
                  </td>
                  <td>{rule.retention_days} days</td>
                  <td>{rule.archive_after_days ? `${rule.archive_after_days} days` : '-'}</td>
                  <td>{rule.delete_after_days ? `${rule.delete_after_days} days` : 'Never'}</td>
                  <td>
                    <label className="toggle-switch">
                      <input
                        type="checkbox"
                        checked={rule.enabled}
                        onChange={() => handleToggleRule(rule)}
                      />
                      <span className="toggle-slider"></span>
                    </label>
                  </td>
                  <td className="actions-cell">
                    <button
                      className="btn-icon"
                      onClick={() => handleEditRule(rule)}
                      title="Edit"
                    >
                      ✏️
                    </button>
                    <button
                      className="btn-icon"
                      onClick={() => handleDeleteRule(rule.id)}
                      title="Delete"
                      disabled={rule.legal_hold}
                    >
                      🗑️
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Best Practices */}
      <div className="best-practices">
        <h3>Retention Best Practices</h3>
        <div className="practices-grid">
          <div className="practice-card">
            <h4>SOC 2 Compliance</h4>
            <p>Retain audit logs for at least 1 year, with 90-day retention for active logs.</p>
            <span className="recommended">Recommended: 365 days</span>
          </div>
          <div className="practice-card">
            <h4>GDPR Compliance</h4>
            <p>Delete personal data when no longer necessary, typically within 2 years.</p>
            <span className="recommended">Recommended: 730 days</span>
          </div>
          <div className="practice-card">
            <h4>HIPAA Compliance</h4>
            <p>Retain audit logs for at least 6 years from date of creation or last use.</p>
            <span className="recommended">Recommended: 2190 days</span>
          </div>
          <div className="practice-card">
            <h4>PCI DSS Compliance</h4>
            <p>Retain audit trail history for at least one year with 90 days immediately available.</p>
            <span className="recommended">Recommended: 365 days</span>
          </div>
        </div>
      </div>

      {/* Create/Edit Modal */}
      {showCreateModal && (
        <RetentionRuleModal
          rule={editingRule}
          onSave={(savedRule) => {
            if (editingRule) {
              setRules((prev) => prev.map((r) => (r.id === savedRule.id ? savedRule : r)));
            } else {
              setRules((prev) => [savedRule, ...prev]);
            }
            setShowCreateModal(false);
            setEditingRule(null);
          }}
          onClose={() => {
            setShowCreateModal(false);
            setEditingRule(null);
          }}
        />
      )}
    </div>
  );
};

// Overview Card Component
function OverviewCard({
  title,
  value,
  icon,
}: {
  title: string;
  value: string;
  icon: string;
}) {
  return (
    <div className="overview-card">
      <div className="card-icon">{icon}</div>
      <div className="card-content">
        <div className="card-value">{value}</div>
        <div className="card-title">{title}</div>
      </div>
    </div>
  );
}

// Rule Scope Component
function RuleScope({ rule }: { rule: RetentionRule }) {
  const scopes: string[] = [];

  if (rule.event_types && rule.event_types.length > 0) {
    scopes.push(`${rule.event_types.length} event types`);
  }
  if (rule.severities && rule.severities.length > 0) {
    scopes.push(`${rule.severities.length} severities`);
  }
  if (rule.data_classifications && rule.data_classifications.length > 0) {
    scopes.push(`${rule.data_classifications.length} classifications`);
  }
  if (rule.compliance_frameworks && rule.compliance_frameworks.length > 0) {
    scopes.push(`${rule.compliance_frameworks.length} frameworks`);
  }

  return (
    <div className="rule-scope">
      {scopes.length > 0 ? scopes.join(', ') : 'All events'}
    </div>
  );
}

// Retention Rule Modal
function RetentionRuleModal({
  rule,
  onSave,
  onClose,
}: {
  rule: RetentionRule | null;
  onSave: (rule: RetentionRule) => void;
  onClose: () => void;
}) {
  const [name, setName] = useState(rule?.name || '');
  const [description, setDescription] = useState(rule?.description || '');
  const [retentionDays, setRetentionDays] = useState(rule?.retention_days || 365);
  const [archiveDays, setArchiveDays] = useState(rule?.archive_after_days || undefined);
  const [deleteDays, setDeleteDays] = useState(rule?.delete_after_days || undefined);
  const [legalHold, setLegalHold] = useState(rule?.legal_hold || false);
  const [legalHoldReason, setLegalHoldReason] = useState(rule?.legal_hold_reason || '');
  const [selectedFrameworks, setSelectedFrameworks] = useState<ComplianceFramework[]>(
    rule?.compliance_frameworks || []
  );
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!name.trim()) {
      newErrors.name = 'Rule name is required';
    }
    if (retentionDays < 1) {
      newErrors.retentionDays = 'Retention period must be at least 1 day';
    }
    if (archiveDays && archiveDays >= retentionDays) {
      newErrors.archiveDays = 'Archive period must be less than retention period';
    }
    if (deleteDays && deleteDays <= retentionDays) {
      newErrors.deleteDays = 'Delete period must be greater than retention period';
    }
    if (legalHold && !legalHoldReason.trim()) {
      newErrors.legalHoldReason = 'Legal hold reason is required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSave = async () => {
    if (!validate()) return;

    const ruleData: RetentionRule = {
      id: rule?.id || crypto.randomUUID(),
      name,
      description,
      enabled: rule?.enabled ?? true,
      retention_days: retentionDays,
      archive_after_days: archiveDays,
      delete_after_days: deleteDays,
      legal_hold: legalHold,
      legal_hold_reason: legalHold ? legalHoldReason : undefined,
      compliance_frameworks: selectedFrameworks.length > 0 ? selectedFrameworks : undefined,
      created_by: rule?.created_by || 'current_user',
      created_at: rule?.created_at || new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    try {
      const response = await fetch(
        rule ? `/api/audit/retention/${rule.id}` : '/api/audit/retention',
        {
          method: rule ? 'PUT' : 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(ruleData),
        }
      );

      if (response.ok) {
        const savedRule = await response.json();
        onSave(savedRule);
      }
    } catch (error) {
      console.error('Failed to save retention rule:', error);
    }
  };

  const toggleFramework = (framework: ComplianceFramework) => {
    setSelectedFrameworks((prev) =>
      prev.includes(framework)
        ? prev.filter((f) => f !== framework)
        : [...prev, framework]
    );
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal retention-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{rule ? 'Edit Retention Rule' : 'Create Retention Rule'}</h2>
          <button className="modal-close" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="modal-content">
          <div className="form-section">
            <label>Rule Name *</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Standard Retention Policy"
              className={errors.name ? 'error' : ''}
            />
            {errors.name && <span className="error-message">{errors.name}</span>}
          </div>

          <div className="form-section">
            <label>Description</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Describe the purpose of this retention rule"
              rows={3}
            />
          </div>

          <div className="form-section">
            <label>Retention Period (days) *</label>
            <input
              type="number"
              value={retentionDays}
              onChange={(e) => setRetentionDays(parseInt(e.target.value) || 0)}
              min="1"
              className={errors.retentionDays ? 'error' : ''}
            />
            {errors.retentionDays && (
              <span className="error-message">{errors.retentionDays}</span>
            )}
          </div>

          <div className="form-row">
            <div className="form-section">
              <label>Archive After (days)</label>
              <input
                type="number"
                value={archiveDays || ''}
                onChange={(e) =>
                  setArchiveDays(e.target.value ? parseInt(e.target.value) : undefined)
                }
                placeholder="Optional"
                min="1"
                className={errors.archiveDays ? 'error' : ''}
              />
              {errors.archiveDays && (
                <span className="error-message">{errors.archiveDays}</span>
              )}
            </div>

            <div className="form-section">
              <label>Delete After (days)</label>
              <input
                type="number"
                value={deleteDays || ''}
                onChange={(e) =>
                  setDeleteDays(e.target.value ? parseInt(e.target.value) : undefined)
                }
                placeholder="Never"
                min="1"
                className={errors.deleteDays ? 'error' : ''}
              />
              {errors.deleteDays && (
                <span className="error-message">{errors.deleteDays}</span>
              )}
            </div>
          </div>

          <div className="form-section">
            <label>Compliance Frameworks</label>
            <div className="framework-chips">
              {(['SOC2', 'GDPR', 'HIPAA', 'ISO27001', 'PCI_DSS', 'CCPA', 'NIST', 'FedRAMP'] as ComplianceFramework[]).map(
                (framework) => (
                  <button
                    key={framework}
                    className={`chip ${
                      selectedFrameworks.includes(framework) ? 'active' : ''
                    }`}
                    onClick={() => toggleFramework(framework)}
                  >
                    {framework}
                  </button>
                )
              )}
            </div>
          </div>

          <div className="form-section">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={legalHold}
                onChange={(e) => setLegalHold(e.target.checked)}
              />
              <span>Legal Hold</span>
            </label>
            {legalHold && (
              <div className="legal-hold-reason">
                <input
                  type="text"
                  value={legalHoldReason}
                  onChange={(e) => setLegalHoldReason(e.target.value)}
                  placeholder="Enter legal hold reason"
                  className={errors.legalHoldReason ? 'error' : ''}
                />
                {errors.legalHoldReason && (
                  <span className="error-message">{errors.legalHoldReason}</span>
                )}
              </div>
            )}
          </div>
        </div>

        <div className="modal-footer">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button className="btn btn-primary" onClick={handleSave}>
            {rule ? 'Save Changes' : 'Create Rule'}
          </button>
        </div>
      </div>
    </div>
  );
}
