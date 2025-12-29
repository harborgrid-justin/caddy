/**
 * CADDY v0.4.0 - Workflow Conditions Component
 * Conditional logic builder with visual interface
 */

import React, { useCallback, useState } from 'react';
import type { Condition, ConditionGroup, ConditionOperator } from './types';

export interface WorkflowConditionsProps {
  conditions?: ConditionGroup;
  onChange?: (conditions: ConditionGroup) => void;
  readOnly?: boolean;
}

const OPERATORS: { value: ConditionOperator; label: string }[] = [
  { value: 'equals', label: 'Equals' },
  { value: 'not-equals', label: 'Not Equals' },
  { value: 'greater-than', label: 'Greater Than' },
  { value: 'less-than', label: 'Less Than' },
  { value: 'contains', label: 'Contains' },
  { value: 'not-contains', label: 'Not Contains' },
  { value: 'starts-with', label: 'Starts With' },
  { value: 'ends-with', label: 'Ends With' },
  { value: 'matches-regex', label: 'Matches Regex' },
  { value: 'is-empty', label: 'Is Empty' },
  { value: 'is-not-empty', label: 'Is Not Empty' },
];

const generateId = () => `cond_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

export const WorkflowConditions: React.FC<WorkflowConditionsProps> = ({
  conditions = { id: 'root', conditions: [], logic: 'AND' },
  onChange,
  readOnly = false,
}) => {
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set(['root']));

  const handleAddCondition = useCallback(
    (groupId: string) => {
      const newCondition: Condition = {
        id: generateId(),
        field: '',
        operator: 'equals',
        value: '',
        logic: 'AND',
      };

      const addToGroup = (group: ConditionGroup): ConditionGroup => {
        if (group.id === groupId) {
          return {
            ...group,
            conditions: [...group.conditions, newCondition],
          };
        }
        return {
          ...group,
          conditions: group.conditions.map((item) =>
            'logic' in item && 'conditions' in item ? addToGroup(item) : item
          ),
        };
      };

      if (onChange) {
        onChange(addToGroup(conditions));
      }
    },
    [conditions, onChange]
  );

  const handleAddGroup = useCallback(
    (parentGroupId: string) => {
      const newGroup: ConditionGroup = {
        id: generateId(),
        conditions: [],
        logic: 'AND',
      };

      const addToGroup = (group: ConditionGroup): ConditionGroup => {
        if (group.id === parentGroupId) {
          return {
            ...group,
            conditions: [...group.conditions, newGroup],
          };
        }
        return {
          ...group,
          conditions: group.conditions.map((item) =>
            'logic' in item && 'conditions' in item ? addToGroup(item) : item
          ),
        };
      };

      if (onChange) {
        onChange(addToGroup(conditions));
        setExpandedGroups((prev) => new Set([...prev, newGroup.id]));
      }
    },
    [conditions, onChange]
  );

  const handleUpdateCondition = useCallback(
    (conditionId: string, updates: Partial<Condition>) => {
      const updateInGroup = (group: ConditionGroup): ConditionGroup => {
        return {
          ...group,
          conditions: group.conditions.map((item) => {
            if ('field' in item && item.id === conditionId) {
              return { ...item, ...updates };
            }
            if ('conditions' in item) {
              return updateInGroup(item);
            }
            return item;
          }),
        };
      };

      if (onChange) {
        onChange(updateInGroup(conditions));
      }
    },
    [conditions, onChange]
  );

  const handleUpdateGroupLogic = useCallback(
    (groupId: string, logic: 'AND' | 'OR') => {
      const updateInGroup = (group: ConditionGroup): ConditionGroup => {
        if (group.id === groupId) {
          return { ...group, logic };
        }
        return {
          ...group,
          conditions: group.conditions.map((item) =>
            'conditions' in item ? updateInGroup(item) : item
          ),
        };
      };

      if (onChange) {
        onChange(updateInGroup(conditions));
      }
    },
    [conditions, onChange]
  );

  const handleDeleteCondition = useCallback(
    (conditionId: string) => {
      const deleteFromGroup = (group: ConditionGroup): ConditionGroup => {
        return {
          ...group,
          conditions: group.conditions
            .filter((item) => item.id !== conditionId)
            .map((item) => ('conditions' in item ? deleteFromGroup(item) : item)),
        };
      };

      if (onChange) {
        onChange(deleteFromGroup(conditions));
      }
    },
    [conditions, onChange]
  );

  const toggleGroupExpanded = useCallback((groupId: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(groupId)) {
        next.delete(groupId);
      } else {
        next.add(groupId);
      }
      return next;
    });
  }, []);

  const renderCondition = useCallback(
    (condition: Condition, depth: number = 0) => {
      return (
        <div
          key={condition.id}
          style={{
            display: 'flex',
            gap: '8px',
            alignItems: 'center',
            padding: '12px',
            backgroundColor: '#fff',
            border: '1px solid #e2e8f0',
            borderRadius: '6px',
            marginLeft: `${depth * 24}px`,
          }}
        >
          {/* Field */}
          <input
            type="text"
            placeholder="Field name"
            value={condition.field}
            onChange={(e) =>
              handleUpdateCondition(condition.id, { field: e.target.value })
            }
            disabled={readOnly}
            style={{
              flex: 1,
              padding: '6px 10px',
              border: '1px solid #e2e8f0',
              borderRadius: '4px',
              fontSize: '13px',
            }}
          />

          {/* Operator */}
          <select
            value={condition.operator}
            onChange={(e) =>
              handleUpdateCondition(condition.id, {
                operator: e.target.value as ConditionOperator,
              })
            }
            disabled={readOnly}
            style={{
              flex: 1,
              padding: '6px 10px',
              border: '1px solid #e2e8f0',
              borderRadius: '4px',
              fontSize: '13px',
              cursor: readOnly ? 'default' : 'pointer',
            }}
          >
            {OPERATORS.map((op) => (
              <option key={op.value} value={op.value}>
                {op.label}
              </option>
            ))}
          </select>

          {/* Value */}
          {!['is-empty', 'is-not-empty'].includes(condition.operator) && (
            <input
              type="text"
              placeholder="Value"
              value={String(condition.value)}
              onChange={(e) =>
                handleUpdateCondition(condition.id, { value: e.target.value })
              }
              disabled={readOnly}
              style={{
                flex: 1,
                padding: '6px 10px',
                border: '1px solid #e2e8f0',
                borderRadius: '4px',
                fontSize: '13px',
              }}
            />
          )}

          {/* Delete Button */}
          {!readOnly && (
            <button
              onClick={() => handleDeleteCondition(condition.id)}
              style={{
                padding: '6px 10px',
                backgroundColor: '#ef4444',
                color: '#fff',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '13px',
              }}
              title="Delete condition"
            >
              ×
            </button>
          )}
        </div>
      );
    },
    [handleUpdateCondition, handleDeleteCondition, readOnly]
  );

  const renderGroup = useCallback(
    (group: ConditionGroup, depth: number = 0): React.ReactNode => {
      const isExpanded = expandedGroups.has(group.id);
      const isRoot = group.id === 'root';

      return (
        <div
          key={group.id}
          style={{
            marginBottom: '12px',
            marginLeft: depth > 0 ? '24px' : '0',
          }}
        >
          {/* Group Header */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              padding: '10px 12px',
              backgroundColor: isRoot ? '#f8fafc' : '#fef3c7',
              border: `1px solid ${isRoot ? '#e2e8f0' : '#fcd34d'}`,
              borderRadius: '6px',
              marginBottom: '8px',
            }}
          >
            {/* Expand/Collapse */}
            {!isRoot && (
              <button
                onClick={() => toggleGroupExpanded(group.id)}
                style={{
                  background: 'none',
                  border: 'none',
                  cursor: 'pointer',
                  fontSize: '14px',
                  color: '#64748b',
                }}
              >
                {isExpanded ? '▼' : '▶'}
              </button>
            )}

            {/* Logic Selector */}
            <select
              value={group.logic}
              onChange={(e) =>
                handleUpdateGroupLogic(group.id, e.target.value as 'AND' | 'OR')
              }
              disabled={readOnly}
              style={{
                padding: '4px 8px',
                backgroundColor: '#fff',
                border: '1px solid #e2e8f0',
                borderRadius: '4px',
                fontSize: '12px',
                fontWeight: 600,
                cursor: readOnly ? 'default' : 'pointer',
              }}
            >
              <option value="AND">AND</option>
              <option value="OR">OR</option>
            </select>

            <span style={{ fontSize: '13px', color: '#64748b' }}>
              {group.conditions.length} condition(s)
            </span>

            {/* Action Buttons */}
            {!readOnly && (
              <div style={{ marginLeft: 'auto', display: 'flex', gap: '8px' }}>
                <button
                  onClick={() => handleAddCondition(group.id)}
                  style={{
                    padding: '4px 10px',
                    backgroundColor: '#3b82f6',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '12px',
                  }}
                >
                  + Condition
                </button>
                <button
                  onClick={() => handleAddGroup(group.id)}
                  style={{
                    padding: '4px 10px',
                    backgroundColor: '#8b5cf6',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '12px',
                  }}
                >
                  + Group
                </button>
                {!isRoot && (
                  <button
                    onClick={() => handleDeleteCondition(group.id)}
                    style={{
                      padding: '4px 10px',
                      backgroundColor: '#ef4444',
                      color: '#fff',
                      border: 'none',
                      borderRadius: '4px',
                      cursor: 'pointer',
                      fontSize: '12px',
                    }}
                  >
                    Delete
                  </button>
                )}
              </div>
            )}
          </div>

          {/* Group Content */}
          {(isRoot || isExpanded) && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {group.conditions.map((item, index) => {
                if ('conditions' in item) {
                  return renderGroup(item, depth + 1);
                }
                return (
                  <React.Fragment key={item.id}>
                    {index > 0 && (
                      <div
                        style={{
                          padding: '4px 12px',
                          textAlign: 'center',
                          fontSize: '11px',
                          fontWeight: 600,
                          color: '#64748b',
                          marginLeft: `${depth * 24}px`,
                        }}
                      >
                        {group.logic}
                      </div>
                    )}
                    {renderCondition(item, depth)}
                  </React.Fragment>
                );
              })}

              {group.conditions.length === 0 && (
                <div
                  style={{
                    padding: '20px',
                    textAlign: 'center',
                    color: '#94a3b8',
                    fontSize: '13px',
                    marginLeft: `${depth * 24}px`,
                  }}
                >
                  No conditions defined. Click "+ Condition" to add one.
                </div>
              )}
            </div>
          )}
        </div>
      );
    },
    [
      expandedGroups,
      handleUpdateGroupLogic,
      handleAddCondition,
      handleAddGroup,
      handleDeleteCondition,
      toggleGroupExpanded,
      renderCondition,
      readOnly,
    ]
  );

  return (
    <div
      style={{
        padding: '16px',
        backgroundColor: '#f8fafc',
        borderRadius: '8px',
        border: '1px solid #e2e8f0',
      }}
    >
      <h3
        style={{
          fontSize: '16px',
          fontWeight: 600,
          marginBottom: '16px',
          color: '#1e293b',
        }}
      >
        Conditions
      </h3>
      {renderGroup(conditions)}
    </div>
  );
};

export default WorkflowConditions;
