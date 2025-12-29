/**
 * CADDY v0.4.0 - Notification Rules
 * Rule-based notification routing and processing
 */

import React, { useState, useCallback, useEffect } from 'react';
import { NotificationRule } from './types';

interface NotificationRulesProps {
  tenantId: string;
  apiUrl?: string;
}

export const NotificationRules: React.FC<NotificationRulesProps> = ({
  tenantId,
  apiUrl = '/api/notifications/rules'
}) => {
  const [rules, setRules] = useState<NotificationRule[]>([]);
  const [loading, setLoading] = useState(false);
  const [editingRule, setEditingRule] = useState<Partial<NotificationRule> | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const fetchRules = useCallback(async () => {
    setLoading(true);
    try {
      const response = await fetch(`${apiUrl}?tenantId=${tenantId}`, {
        credentials: 'include'
      });
      const data = await response.json();
      setRules(data.rules || []);
    } catch (err) {
      console.error('Error fetching rules:', err);
    } finally {
      setLoading(false);
    }
  }, [apiUrl, tenantId]);

  useEffect(() => {
    fetchRules();
  }, [fetchRules]);

  const handleCreate = useCallback(() => {
    setEditingRule({
      name: '',
      enabled: true,
      priority: 100,
      conditions: [],
      conditionLogic: 'AND',
      actions: []
    });
    setIsModalOpen(true);
  }, []);

  const handleEdit = useCallback((rule: NotificationRule) => {
    setEditingRule(rule);
    setIsModalOpen(true);
  }, []);

  const handleSave = useCallback(async () => {
    if (!editingRule) return;

    try {
      const method = editingRule.id ? 'PUT' : 'POST';
      const url = editingRule.id ? `${apiUrl}/${editingRule.id}` : apiUrl;

      const response = await fetch(url, {
        method,
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ ...editingRule, tenantId })
      });

      if (response.ok) {
        await fetchRules();
        setIsModalOpen(false);
        setEditingRule(null);
      }
    } catch (err) {
      console.error('Error saving rule:', err);
      alert('Failed to save rule');
    }
  }, [editingRule, apiUrl, tenantId, fetchRules]);

  const handleDelete = useCallback(async (id: string) => {
    if (!window.confirm('Are you sure you want to delete this rule?')) return;

    try {
      await fetch(`${apiUrl}/${id}`, {
        method: 'DELETE',
        credentials: 'include'
      });
      await fetchRules();
    } catch (err) {
      console.error('Error deleting rule:', err);
    }
  }, [apiUrl, fetchRules]);

  const handleToggleEnabled = useCallback(async (rule: NotificationRule) => {
    try {
      await fetch(`${apiUrl}/${rule.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ ...rule, enabled: !rule.enabled })
      });
      await fetchRules();
    } catch (err) {
      console.error('Error toggling rule:', err);
    }
  }, [apiUrl, fetchRules]);

  const addCondition = useCallback(() => {
    if (!editingRule) return;
    setEditingRule({
      ...editingRule,
      conditions: [
        ...(editingRule.conditions || []),
        { field: 'type', operator: 'eq' as const, value: '' }
      ]
    });
  }, [editingRule]);

  const removeCondition = useCallback((index: number) => {
    if (!editingRule) return;
    setEditingRule({
      ...editingRule,
      conditions: editingRule.conditions?.filter((_, i) => i !== index) || []
    });
  }, [editingRule]);

  const updateCondition = useCallback((index: number, updates: any) => {
    if (!editingRule) return;
    const newConditions = [...(editingRule.conditions || [])];
    newConditions[index] = { ...newConditions[index], ...updates };
    setEditingRule({ ...editingRule, conditions: newConditions });
  }, [editingRule]);

  const addAction = useCallback(() => {
    if (!editingRule) return;
    setEditingRule({
      ...editingRule,
      actions: [
        ...(editingRule.actions || []),
        { type: 'route' as const, config: {} }
      ]
    });
  }, [editingRule]);

  const removeAction = useCallback((index: number) => {
    if (!editingRule) return;
    setEditingRule({
      ...editingRule,
      actions: editingRule.actions?.filter((_, i) => i !== index) || []
    });
  }, [editingRule]);

  const updateAction = useCallback((index: number, updates: any) => {
    if (!editingRule) return;
    const newActions = [...(editingRule.actions || [])];
    newActions[index] = { ...newActions[index], ...updates };
    setEditingRule({ ...editingRule, actions: newActions });
  }, [editingRule]);

  return (
    <div style={{ padding: '24px', maxWidth: '1200px', margin: '0 auto' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '24px' }}>
        <div>
          <h2 style={{ margin: '0 0 4px 0', fontSize: '20px', fontWeight: '600', color: '#111827' }}>
            Notification Rules
          </h2>
          <p style={{ margin: 0, fontSize: '14px', color: '#6b7280' }}>
            Define rules for routing, escalating, and processing notifications
          </p>
        </div>
        <button
          onClick={handleCreate}
          style={{
            padding: '10px 20px',
            fontSize: '14px',
            fontWeight: '500',
            border: 'none',
            borderRadius: '6px',
            backgroundColor: '#3b82f6',
            color: '#ffffff',
            cursor: 'pointer'
          }}
        >
          + Create Rule
        </button>
      </div>

      {loading ? (
        <div style={{ padding: '48px', textAlign: 'center', color: '#6b7280' }}>
          Loading rules...
        </div>
      ) : rules.length === 0 ? (
        <div style={{ padding: '48px', textAlign: 'center', color: '#6b7280' }}>
          <div style={{ fontSize: '48px', marginBottom: '16px' }}>⚙️</div>
          <div style={{ fontSize: '16px', fontWeight: '500', marginBottom: '8px' }}>
            No rules configured
          </div>
          <div style={{ fontSize: '14px' }}>
            Create rules to automate notification processing
          </div>
        </div>
      ) : (
        <div style={{ display: 'grid', gap: '16px' }}>
          {rules.sort((a, b) => b.priority - a.priority).map((rule) => (
            <div
              key={rule.id}
              style={{
                padding: '16px',
                border: '1px solid #e5e7eb',
                borderRadius: '8px',
                backgroundColor: rule.enabled ? '#ffffff' : '#f9fafb'
              }}
            >
              <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', marginBottom: '12px' }}>
                <div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                    <h3 style={{ margin: 0, fontSize: '16px', fontWeight: '600', color: '#111827' }}>
                      {rule.name}
                    </h3>
                    <span
                      style={{
                        padding: '2px 8px',
                        fontSize: '11px',
                        fontWeight: '500',
                        borderRadius: '12px',
                        backgroundColor: rule.enabled ? '#dcfce7' : '#f3f4f6',
                        color: rule.enabled ? '#166534' : '#6b7280'
                      }}
                    >
                      {rule.enabled ? 'Active' : 'Inactive'}
                    </span>
                    <span
                      style={{
                        padding: '2px 8px',
                        fontSize: '11px',
                        fontWeight: '500',
                        borderRadius: '12px',
                        backgroundColor: '#e0f2fe',
                        color: '#075985'
                      }}
                    >
                      Priority: {rule.priority}
                    </span>
                  </div>
                  {rule.description && (
                    <p style={{ margin: '4px 0 0 0', fontSize: '13px', color: '#6b7280' }}>
                      {rule.description}
                    </p>
                  )}
                </div>
                <div style={{ display: 'flex', gap: '8px' }}>
                  <button
                    onClick={() => handleToggleEnabled(rule)}
                    style={{
                      padding: '6px 12px',
                      fontSize: '12px',
                      fontWeight: '500',
                      border: '1px solid #d1d5db',
                      borderRadius: '4px',
                      backgroundColor: '#ffffff',
                      color: '#374151',
                      cursor: 'pointer'
                    }}
                  >
                    {rule.enabled ? 'Disable' : 'Enable'}
                  </button>
                  <button
                    onClick={() => handleEdit(rule)}
                    style={{
                      padding: '6px 12px',
                      fontSize: '12px',
                      fontWeight: '500',
                      border: '1px solid #d1d5db',
                      borderRadius: '4px',
                      backgroundColor: '#ffffff',
                      color: '#374151',
                      cursor: 'pointer'
                    }}
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => handleDelete(rule.id)}
                    style={{
                      padding: '6px 12px',
                      fontSize: '12px',
                      fontWeight: '500',
                      border: '1px solid #dc2626',
                      borderRadius: '4px',
                      backgroundColor: '#ffffff',
                      color: '#dc2626',
                      cursor: 'pointer'
                    }}
                  >
                    Delete
                  </button>
                </div>
              </div>

              <div style={{ display: 'grid', gap: '8px' }}>
                <div>
                  <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                    CONDITIONS ({rule.conditionLogic})
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    {rule.conditions.map((condition, index) => (
                      <div
                        key={index}
                        style={{
                          padding: '6px 8px',
                          fontSize: '12px',
                          backgroundColor: '#f9fafb',
                          borderRadius: '4px',
                          fontFamily: 'monospace'
                        }}
                      >
                        {condition.field} {condition.operator} {JSON.stringify(condition.value)}
                      </div>
                    ))}
                  </div>
                </div>

                <div>
                  <div style={{ fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' }}>
                    ACTIONS ({rule.actions.length})
                  </div>
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
                    {rule.actions.map((action, index) => (
                      <span
                        key={index}
                        style={{
                          padding: '4px 8px',
                          fontSize: '11px',
                          fontWeight: '500',
                          backgroundColor: '#dbeafe',
                          color: '#1e40af',
                          borderRadius: '4px',
                          textTransform: 'capitalize'
                        }}
                      >
                        {action.type}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Edit Modal */}
      {isModalOpen && editingRule && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 10000
          }}
          onClick={() => setIsModalOpen(false)}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              backgroundColor: '#ffffff',
              borderRadius: '8px',
              padding: '24px',
              maxWidth: '700px',
              width: '90%',
              maxHeight: '80vh',
              overflowY: 'auto',
              boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
            }}
          >
            <h3 style={{ margin: '0 0 24px 0', fontSize: '20px', fontWeight: '600', color: '#111827' }}>
              {editingRule.id ? 'Edit Rule' : 'Create Rule'}
            </h3>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
              <div>
                <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                  Name *
                </label>
                <input
                  type="text"
                  value={editingRule.name}
                  onChange={(e) => setEditingRule({ ...editingRule, name: e.target.value })}
                  style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                />
              </div>

              <div>
                <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                  Description
                </label>
                <textarea
                  value={editingRule.description || ''}
                  onChange={(e) => setEditingRule({ ...editingRule, description: e.target.value })}
                  rows={2}
                  style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px', fontFamily: 'inherit', resize: 'vertical' }}
                />
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
                <div>
                  <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                    Priority *
                  </label>
                  <input
                    type="number"
                    value={editingRule.priority}
                    onChange={(e) => setEditingRule({ ...editingRule, priority: parseInt(e.target.value) })}
                    style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                    Logic *
                  </label>
                  <select
                    value={editingRule.conditionLogic}
                    onChange={(e) => setEditingRule({ ...editingRule, conditionLogic: e.target.value as 'AND' | 'OR' })}
                    style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                  >
                    <option value="AND">AND</option>
                    <option value="OR">OR</option>
                  </select>
                </div>
              </div>

              {/* Conditions */}
              <div>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
                  <label style={{ fontSize: '13px', fontWeight: '600', color: '#374151' }}>
                    Conditions
                  </label>
                  <button
                    onClick={addCondition}
                    style={{
                      padding: '4px 12px',
                      fontSize: '12px',
                      fontWeight: '500',
                      border: '1px solid #3b82f6',
                      borderRadius: '4px',
                      backgroundColor: '#ffffff',
                      color: '#3b82f6',
                      cursor: 'pointer'
                    }}
                  >
                    + Add Condition
                  </button>
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                  {editingRule.conditions?.map((condition, index) => (
                    <div key={index} style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr auto', gap: '8px', alignItems: 'center' }}>
                      <input
                        type="text"
                        value={condition.field}
                        onChange={(e) => updateCondition(index, { field: e.target.value })}
                        placeholder="Field"
                        style={{ padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                      />
                      <select
                        value={condition.operator}
                        onChange={(e) => updateCondition(index, { operator: e.target.value })}
                        style={{ padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                      >
                        <option value="eq">Equals</option>
                        <option value="ne">Not Equals</option>
                        <option value="gt">Greater Than</option>
                        <option value="gte">Greater or Equal</option>
                        <option value="lt">Less Than</option>
                        <option value="lte">Less or Equal</option>
                        <option value="in">In</option>
                        <option value="nin">Not In</option>
                        <option value="contains">Contains</option>
                        <option value="matches">Matches</option>
                      </select>
                      <input
                        type="text"
                        value={typeof condition.value === 'string' ? condition.value : JSON.stringify(condition.value)}
                        onChange={(e) => updateCondition(index, { value: e.target.value })}
                        placeholder="Value"
                        style={{ padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                      />
                      <button
                        onClick={() => removeCondition(index)}
                        style={{
                          padding: '6px 8px',
                          fontSize: '12px',
                          border: 'none',
                          borderRadius: '4px',
                          backgroundColor: '#fee2e2',
                          color: '#dc2626',
                          cursor: 'pointer'
                        }}
                      >
                        ✕
                      </button>
                    </div>
                  ))}
                </div>
              </div>

              {/* Actions */}
              <div>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
                  <label style={{ fontSize: '13px', fontWeight: '600', color: '#374151' }}>
                    Actions
                  </label>
                  <button
                    onClick={addAction}
                    style={{
                      padding: '4px 12px',
                      fontSize: '12px',
                      fontWeight: '500',
                      border: '1px solid #3b82f6',
                      borderRadius: '4px',
                      backgroundColor: '#ffffff',
                      color: '#3b82f6',
                      cursor: 'pointer'
                    }}
                  >
                    + Add Action
                  </button>
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                  {editingRule.actions?.map((action, index) => (
                    <div key={index} style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                      <select
                        value={action.type}
                        onChange={(e) => updateAction(index, { type: e.target.value })}
                        style={{ flex: 1, padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                      >
                        <option value="route">Route</option>
                        <option value="escalate">Escalate</option>
                        <option value="suppress">Suppress</option>
                        <option value="transform">Transform</option>
                        <option value="delay">Delay</option>
                      </select>
                      <button
                        onClick={() => removeAction(index)}
                        style={{
                          padding: '6px 8px',
                          fontSize: '12px',
                          border: 'none',
                          borderRadius: '4px',
                          backgroundColor: '#fee2e2',
                          color: '#dc2626',
                          cursor: 'pointer'
                        }}
                      >
                        ✕
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' }}>
              <button
                onClick={() => setIsModalOpen(false)}
                style={{
                  padding: '10px 20px',
                  fontSize: '14px',
                  fontWeight: '500',
                  border: '1px solid #d1d5db',
                  borderRadius: '6px',
                  backgroundColor: '#ffffff',
                  color: '#374151',
                  cursor: 'pointer'
                }}
              >
                Cancel
              </button>
              <button
                onClick={handleSave}
                style={{
                  padding: '10px 20px',
                  fontSize: '14px',
                  fontWeight: '500',
                  border: 'none',
                  borderRadius: '6px',
                  backgroundColor: '#3b82f6',
                  color: '#ffffff',
                  cursor: 'pointer'
                }}
              >
                {editingRule.id ? 'Update' : 'Create'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
