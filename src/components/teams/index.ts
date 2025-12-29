/**
 * Team Collaboration Components
 *
 * Export all team collaboration components and types.
 */

// Component exports
export { TeamDashboard } from './TeamDashboard';
export { WorkspaceSettings } from './WorkspaceSettings';
export { MemberManagement } from './MemberManagement';
export { ActivityFeed } from './ActivityFeed';
export { IssueAssignment, AssignmentList } from './IssueAssignment';

// Type exports
export type {
  // Workspace types
  Workspace,
  WorkspaceSettings,
  WorkspaceTemplate,
  WorkspaceVisibility,
  WorkspaceStatus,
  WorkingHours,
  ArchiveInfo,

  // Member types
  Member,
  MemberRole,
  MemberStatus,
  MemberInvitation,
  MemberPermissions,
  MemberActivity,
  InvitationStatus,

  // Assignment types
  Assignment,
  AssignmentPriority,
  AssignmentStatus,
  WorkloadBalance,

  // Comment types
  Comment,
  CommentThread,
  ThreadStatus,
  RichContent,
  Mention,
  Attachment,

  // Activity types
  Activity,
  ActivityType,
  ActivityFilter,
  ActivityChanges,
  UserActivitySummary,
  TeamMetrics,

  // UI state types
  TeamDashboardState,
  WorkspaceFormData,
  MemberInviteFormData,
  AssignmentFormData,

  // API types
  ApiResponse,
  PaginatedResponse,

  // WebSocket types
  WebSocketEvent,
  WebSocketMessage,

  // Notification types
  Notification,
} from './types';
