/**
 * Issue Assignment Component
 *
 * Interface for assigning issues with workload balancing and smart assignment.
 */

import React, { useState, useEffect, useMemo } from 'react';
import type {
  Assignment,
  AssignmentPriority,
  AssignmentStatus,
  Member,
  WorkloadBalance,
} from './types';

interface IssueAssignmentProps {
  workspaceId: string;
  issueId?: string;
  currentAssignment?: Assignment;
  onAssign?: (assignment: Assignment) => void;
  onCancel?: () => void;
}

export function IssueAssignment({
  workspaceId,
  issueId,
  currentAssignment,
  onAssign,
  onCancel,
}: IssueAssignmentProps) {
  const [members, setMembers] = useState<Member[]>([]);
  const [workloads, setWorkloads] = useState<Map<string, WorkloadBalance>>(new Map());
  const [selectedMember, setSelectedMember] = useState<string>(
    currentAssignment?.assignee_id || ''
  );
  const [priority, setPriority] = useState<AssignmentPriority>(
    currentAssignment?.priority || 'Medium'
  );
  const [dueDate, setDueDate] = useState<string>(
    currentAssignment?.due_date || ''
  );
  const [estimatedHours, setEstimatedHours] = useState<number>(
    currentAssignment?.estimated_hours || 0
  );
  const [tags, setTags] = useState<string[]>(currentAssignment?.tags || []);
  const [skills, setSkills] = useState<string[]>(
    currentAssignment?.required_skills || []
  );
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, [workspaceId]);

  const loadData = async () => {
    setLoading(true);
    try {
      await Promise.all([loadMembers(), loadWorkloads()]);
    } finally {
      setLoading(false);
    }
  };

  const loadMembers = async () => {
    // Mock implementation
    const mockMembers: Member[] = [
      {
        id: '1',
        workspace_id: workspaceId,
        user_id: 'user1',
        email: 'alice@example.com',
        display_name: 'Alice Johnson',
        role: 'Developer',
        status: 'Active',
        joined_at: new Date().toISOString(),
        metadata: {},
      },
      {
        id: '2',
        workspace_id: workspaceId,
        user_id: 'user2',
        email: 'bob@example.com',
        display_name: 'Bob Smith',
        role: 'Developer',
        status: 'Active',
        joined_at: new Date().toISOString(),
        metadata: {},
      },
      {
        id: '3',
        workspace_id: workspaceId,
        user_id: 'user3',
        email: 'carol@example.com',
        display_name: 'Carol Davis',
        role: 'Designer',
        status: 'Active',
        joined_at: new Date().toISOString(),
        metadata: {},
      },
    ];
    setMembers(mockMembers.filter((m) => m.status === 'Active'));
  };

  const loadWorkloads = async () => {
    // Mock implementation
    const mockWorkloads = new Map<string, WorkloadBalance>([
      [
        'user1',
        {
          user_id: 'user1',
          total_assignments: 5,
          active_assignments: 3,
          workload_score: 6.5,
          available_capacity: 0.68,
          avg_completion_time: 4.2,
          on_time_rate: 0.85,
        },
      ],
      [
        'user2',
        {
          user_id: 'user2',
          total_assignments: 8,
          active_assignments: 5,
          workload_score: 9.0,
          available_capacity: 0.55,
          avg_completion_time: 5.1,
          on_time_rate: 0.75,
        },
      ],
      [
        'user3',
        {
          user_id: 'user3',
          total_assignments: 3,
          active_assignments: 2,
          workload_score: 3.5,
          available_capacity: 0.82,
          avg_completion_time: 3.8,
          on_time_rate: 0.90,
        },
      ],
    ]);
    setWorkloads(mockWorkloads);
  };

  const suggestedAssignee = useMemo(() => {
    // Find member with best combination of capacity and on-time rate
    let best: { userId: string; score: number } | null = null;

    members.forEach((member) => {
      const workload = workloads.get(member.user_id);
      if (!workload) return;

      // Calculate score: capacity (0-1) + on_time_rate (0-1)
      const score =
        workload.available_capacity + (workload.on_time_rate || 0.5);

      if (!best || score > best.score) {
        best = { userId: member.user_id, score };
      }
    });

    return best?.userId;
  }, [members, workloads]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const assignment: Assignment = {
      id: currentAssignment?.id || `assignment-${Date.now()}`,
      issue_id: issueId || 'new-issue',
      workspace_id: workspaceId,
      assignee_id: selectedMember,
      assigner_id: 'current-user',
      priority,
      status: 'Pending',
      due_date: dueDate || undefined,
      created_at: new Date().toISOString(),
      estimated_hours: estimatedHours > 0 ? estimatedHours : undefined,
      tags,
      required_skills: skills,
      metadata: {},
    };

    onAssign?.(assignment);
  };

  if (loading) {
    return (
      <div className="issue-assignment loading">
        <div className="loading-spinner" />
        <p>Loading...</p>
      </div>
    );
  }

  return (
    <div className="issue-assignment">
      <div className="assignment-header">
        <h2>{currentAssignment ? 'Reassign Issue' : 'Assign Issue'}</h2>
        <p className="assignment-subtitle">
          {issueId && `Issue #${issueId}`}
        </p>
      </div>

      <form onSubmit={handleSubmit}>
        {/* Assignee Selection */}
        <div className="form-section">
          <h3>Select Assignee</h3>

          {suggestedAssignee && !selectedMember && (
            <div className="suggestion-banner">
              <span className="suggestion-icon">💡</span>
              <span>
                Suggested:{' '}
                <strong>
                  {members.find((m) => m.user_id === suggestedAssignee)
                    ?.display_name}
                </strong>{' '}
                (based on workload and performance)
              </span>
              <button
                type="button"
                className="btn-small"
                onClick={() => setSelectedMember(suggestedAssignee)}
              >
                Accept
              </button>
            </div>
          )}

          <div className="member-selection">
            {members.map((member) => {
              const workload = workloads.get(member.user_id);
              const isSelected = selectedMember === member.user_id;
              const isSuggested = suggestedAssignee === member.user_id;

              return (
                <div
                  key={member.id}
                  className={`member-card ${isSelected ? 'selected' : ''} ${
                    isSuggested ? 'suggested' : ''
                  }`}
                  onClick={() => setSelectedMember(member.user_id)}
                >
                  <div className="member-header">
                    <div className="member-avatar">
                      {member.display_name?.charAt(0) || '?'}
                    </div>
                    <div className="member-info">
                      <div className="member-name">{member.display_name}</div>
                      <div className="member-role">{member.role}</div>
                    </div>
                    {isSelected && <span className="check-mark">✓</span>}
                  </div>

                  {workload && (
                    <div className="workload-info">
                      <div className="workload-bar">
                        <div
                          className="workload-fill"
                          style={{
                            width: `${(1 - workload.available_capacity) * 100}%`,
                          }}
                        />
                      </div>
                      <div className="workload-stats">
                        <span>
                          {workload.active_assignments} active assignments
                        </span>
                        <span
                          className={
                            workload.available_capacity > 0.6
                              ? 'capacity-good'
                              : workload.available_capacity > 0.3
                              ? 'capacity-medium'
                              : 'capacity-low'
                          }
                        >
                          {Math.round(workload.available_capacity * 100)}%
                          available
                        </span>
                      </div>
                      {workload.on_time_rate !== undefined && (
                        <div className="performance-stat">
                          On-time: {Math.round(workload.on_time_rate * 100)}%
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {/* Assignment Details */}
        <div className="form-section">
          <h3>Assignment Details</h3>

          <div className="form-row">
            <div className="form-group">
              <label htmlFor="priority">Priority</label>
              <select
                id="priority"
                value={priority}
                onChange={(e) => setPriority(e.target.value as AssignmentPriority)}
                required
              >
                <option value="Low">Low</option>
                <option value="Medium">Medium</option>
                <option value="High">High</option>
                <option value="Critical">Critical</option>
              </select>
            </div>

            <div className="form-group">
              <label htmlFor="due-date">Due Date</label>
              <input
                id="due-date"
                type="date"
                value={dueDate}
                onChange={(e) => setDueDate(e.target.value)}
              />
            </div>

            <div className="form-group">
              <label htmlFor="estimated-hours">Estimated Hours</label>
              <input
                id="estimated-hours"
                type="number"
                min="0"
                step="0.5"
                value={estimatedHours || ''}
                onChange={(e) => setEstimatedHours(parseFloat(e.target.value))}
                placeholder="0"
              />
            </div>
          </div>

          {/* Advanced Options */}
          <div className="advanced-section">
            <button
              type="button"
              className="btn-link"
              onClick={() => setShowAdvanced(!showAdvanced)}
            >
              {showAdvanced ? '▼' : '▶'} Advanced Options
            </button>

            {showAdvanced && (
              <div className="advanced-content">
                <div className="form-group">
                  <label htmlFor="tags">Tags</label>
                  <input
                    id="tags"
                    type="text"
                    value={tags.join(', ')}
                    onChange={(e) =>
                      setTags(
                        e.target.value.split(',').map((t) => t.trim()).filter(Boolean)
                      )
                    }
                    placeholder="bug, feature, urgent"
                  />
                  <p className="form-help">Comma-separated tags</p>
                </div>

                <div className="form-group">
                  <label htmlFor="skills">Required Skills</label>
                  <input
                    id="skills"
                    type="text"
                    value={skills.join(', ')}
                    onChange={(e) =>
                      setSkills(
                        e.target.value.split(',').map((s) => s.trim()).filter(Boolean)
                      )
                    }
                    placeholder="react, typescript, design"
                  />
                  <p className="form-help">Comma-separated skills</p>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Actions */}
        <div className="form-actions">
          <button
            type="button"
            className="btn-secondary"
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            type="submit"
            className="btn-primary"
            disabled={!selectedMember}
          >
            {currentAssignment ? 'Reassign' : 'Assign'}
          </button>
        </div>
      </form>

      {/* Workload Summary */}
      <div className="workload-summary">
        <h3>Team Workload Summary</h3>
        <div className="summary-stats">
          <div className="stat">
            <span className="stat-label">Total Active Assignments</span>
            <span className="stat-value">
              {Array.from(workloads.values()).reduce(
                (sum, w) => sum + w.active_assignments,
                0
              )}
            </span>
          </div>
          <div className="stat">
            <span className="stat-label">Average Capacity</span>
            <span className="stat-value">
              {Math.round(
                (Array.from(workloads.values()).reduce(
                  (sum, w) => sum + w.available_capacity,
                  0
                ) /
                  workloads.size) *
                  100
              )}
              %
            </span>
          </div>
          <div className="stat">
            <span className="stat-label">Team On-time Rate</span>
            <span className="stat-value">
              {Math.round(
                (Array.from(workloads.values()).reduce(
                  (sum, w) => sum + (w.on_time_rate || 0),
                  0
                ) /
                  workloads.size) *
                  100
              )}
              %
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Assignment List Component
// ============================================================================

interface AssignmentListProps {
  workspaceId: string;
  userId?: string;
  onAssignmentClick?: (assignment: Assignment) => void;
}

export function AssignmentList({
  workspaceId,
  userId,
  onAssignmentClick,
}: AssignmentListProps) {
  const [assignments, setAssignments] = useState<Assignment[]>([]);
  const [filter, setFilter] = useState<AssignmentStatus | 'all'>('all');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadAssignments();
  }, [workspaceId, userId]);

  const loadAssignments = async () => {
    setLoading(true);
    try {
      // Mock implementation
      const mockAssignments: Assignment[] = [];
      setAssignments(mockAssignments);
    } finally {
      setLoading(false);
    }
  };

  const filteredAssignments = useMemo(() => {
    if (filter === 'all') return assignments;
    return assignments.filter((a) => a.status === filter);
  }, [assignments, filter]);

  if (loading) {
    return <div className="loading">Loading assignments...</div>;
  }

  return (
    <div className="assignment-list">
      <div className="list-header">
        <h2>Assignments</h2>
        <select value={filter} onChange={(e) => setFilter(e.target.value as any)}>
          <option value="all">All</option>
          <option value="Pending">Pending</option>
          <option value="InProgress">In Progress</option>
          <option value="Completed">Completed</option>
        </select>
      </div>

      {filteredAssignments.length === 0 ? (
        <div className="empty-state">
          <p>No assignments found</p>
        </div>
      ) : (
        <div className="assignments">
          {filteredAssignments.map((assignment) => (
            <div
              key={assignment.id}
              className="assignment-item"
              onClick={() => onAssignmentClick?.(assignment)}
            >
              <div className="assignment-header">
                <span className={`priority-badge ${assignment.priority.toLowerCase()}`}>
                  {assignment.priority}
                </span>
                <span className={`status-badge ${assignment.status.toLowerCase()}`}>
                  {assignment.status}
                </span>
              </div>

              <h3>Issue #{assignment.issue_id}</h3>

              {assignment.due_date && (
                <div className="due-date">
                  Due: {new Date(assignment.due_date).toLocaleDateString()}
                </div>
              )}

              {assignment.tags.length > 0 && (
                <div className="tags">
                  {assignment.tags.map((tag) => (
                    <span key={tag} className="tag">
                      {tag}
                    </span>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
