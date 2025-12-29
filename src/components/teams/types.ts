/**
 * Team Collaboration System Types
 *
 * TypeScript type definitions for the team collaboration features.
 */

// ============================================================================
// Workspace Types
// ============================================================================

export enum WorkspaceVisibility {
  Private = 'Private',
  Internal = 'Internal',
  Public = 'Public',
}

export enum WorkspaceStatus {
  Active = 'Active',
  Archived = 'Archived',
  Provisioning = 'Provisioning',
  Suspended = 'Suspended',
}

export interface WorkingHours {
  start_hour: number;
  end_hour: number;
  working_days: number[];
}

export interface WorkspaceSettings {
  auto_assignment_enabled: boolean;
  require_member_approval: boolean;
  allow_guest_access: boolean;
  default_member_role: string;
  activity_tracking_enabled: boolean;
  email_notifications_enabled: boolean;
  slack_integration_enabled: boolean;
  max_members?: number;
  notification_settings: Record<string, boolean>;
  timezone: string;
  working_hours: WorkingHours;
}

export interface ArchiveInfo {
  archived_at: string;
  archived_by: string;
  reason?: string;
  auto_delete_at?: string;
}

export interface Workspace {
  id: string;
  name: string;
  slug: string;
  description?: string;
  owner_id: string;
  status: WorkspaceStatus;
  visibility: WorkspaceVisibility;
  settings: WorkspaceSettings;
  created_at: string;
  updated_at: string;
  archive_info?: ArchiveInfo;
  metadata: Record<string, string>;
}

export interface WorkspaceTemplate {
  id: string;
  name: string;
  description: string;
  settings: WorkspaceSettings;
  default_roles: string[];
  metadata: Record<string, string>;
  is_builtin: boolean;
}

// ============================================================================
// Member Types
// ============================================================================

export enum MemberRole {
  Owner = 'Owner',
  Admin = 'Admin',
  Manager = 'Manager',
  Developer = 'Developer',
  Reviewer = 'Reviewer',
  Designer = 'Designer',
  Viewer = 'Viewer',
  Guest = 'Guest',
}

export enum MemberStatus {
  Active = 'Active',
  Pending = 'Pending',
  Inactive = 'Inactive',
  Suspended = 'Suspended',
}

export enum InvitationStatus {
  Pending = 'Pending',
  Accepted = 'Accepted',
  Declined = 'Declined',
  Expired = 'Expired',
  Cancelled = 'Cancelled',
}

export interface MemberPermissions {
  can_read: boolean;
  can_create: boolean;
  can_edit: boolean;
  can_delete: boolean;
  can_manage_members: boolean;
  can_manage_settings: boolean;
  can_assign_issues: boolean;
  can_comment: boolean;
  can_approve: boolean;
  can_export: boolean;
}

export interface Member {
  id: string;
  workspace_id: string;
  user_id: string;
  email: string;
  display_name?: string;
  role: MemberRole;
  status: MemberStatus;
  joined_at: string;
  last_active_at?: string;
  invitation_id?: string;
  manager_id?: string;
  team?: string;
  metadata: Record<string, string>;
}

export interface MemberInvitation {
  id: string;
  workspace_id: string;
  user_id: string;
  email: string;
  role: MemberRole;
  token: string;
  invited_by: string;
  created_at: string;
  expires_at: string;
  status: InvitationStatus;
  message?: string;
}

export interface MemberActivity {
  member_id: string;
  last_login?: string;
  total_actions: number;
  weekly_actions: number;
  daily_actions: number;
  last_action_at?: string;
  last_action_type?: string;
}

// ============================================================================
// Assignment Types
// ============================================================================

export enum AssignmentPriority {
  Critical = 'Critical',
  High = 'High',
  Medium = 'Medium',
  Low = 'Low',
}

export enum AssignmentStatus {
  Pending = 'Pending',
  Accepted = 'Accepted',
  InProgress = 'InProgress',
  Blocked = 'Blocked',
  InReview = 'InReview',
  Completed = 'Completed',
  Cancelled = 'Cancelled',
  Escalated = 'Escalated',
}

export interface Assignment {
  id: string;
  issue_id: string;
  workspace_id: string;
  assignee_id: string;
  assigner_id: string;
  priority: AssignmentPriority;
  status: AssignmentStatus;
  due_date?: string;
  sla_deadline?: string;
  created_at: string;
  accepted_at?: string;
  started_at?: string;
  completed_at?: string;
  estimated_hours?: number;
  actual_hours?: number;
  tags: string[];
  required_skills: string[];
  metadata: Record<string, string>;
}

export interface WorkloadBalance {
  user_id: string;
  total_assignments: number;
  active_assignments: number;
  workload_score: number;
  available_capacity: number;
  avg_completion_time?: number;
  on_time_rate?: number;
}

// ============================================================================
// Comment Types
// ============================================================================

export enum ThreadStatus {
  Open = 'Open',
  Resolved = 'Resolved',
  Locked = 'Locked',
  Archived = 'Archived',
}

export interface RichContent {
  markdown: string;
  html?: string;
  plain_text: string;
  format_version: string;
}

export interface Mention {
  user_id: string;
  display_name: string;
  position: number;
  mention_type: 'User' | 'Team' | 'Role' | 'Everyone';
}

export interface Attachment {
  id: string;
  filename: string;
  content_type: string;
  size: number;
  url: string;
  thumbnail_url?: string;
  uploaded_at: string;
  uploaded_by: string;
  checksum: string;
}

export interface Comment {
  id: string;
  resource_id: string;
  resource_type: string;
  thread_id?: string;
  parent_id?: string;
  author_id: string;
  content: RichContent;
  created_at: string;
  edited_at?: string;
  deleted: boolean;
  mentions: Mention[];
  attachments: Attachment[];
  reactions: Record<string, string[]>;
  metadata: Record<string, string>;
}

export interface CommentThread {
  id: string;
  resource_id: string;
  resource_type: string;
  title?: string;
  status: ThreadStatus;
  created_at: string;
  created_by: string;
  updated_at: string;
  resolved: boolean;
  resolved_by?: string;
  resolved_at?: string;
  locked: boolean;
  participants: string[];
  comment_count: number;
}

// ============================================================================
// Activity Types
// ============================================================================

export type ActivityType =
  | 'WorkspaceCreated'
  | 'WorkspaceUpdated'
  | 'WorkspaceArchived'
  | 'MemberInvited'
  | 'MemberJoined'
  | 'IssueCreated'
  | 'IssueAssigned'
  | 'IssueCompleted'
  | 'CommentCreated'
  | 'LoginSuccess'
  | { Custom: string };

export interface ActivityChanges {
  before: Record<string, string>;
  after: Record<string, string>;
}

export interface Activity {
  id: string;
  workspace_id: string;
  user_id: string;
  activity_type: ActivityType;
  description: string;
  timestamp: string;
  resource_id?: string;
  resource_type?: string;
  ip_address?: string;
  user_agent?: string;
  metadata: Record<string, string>;
  changes?: ActivityChanges;
}

export interface ActivityFilter {
  workspace_id?: string;
  user_id?: string;
  activity_types?: ActivityType[];
  categories?: string[];
  resource_id?: string;
  start_date?: string;
  end_date?: string;
  security_only?: boolean;
  limit?: number;
  offset?: number;
}

export interface UserActivitySummary {
  user_id: string;
  workspace_id: string;
  period_start: string;
  period_end: string;
  total_activities: number;
  activities_by_type: Record<string, number>;
  most_active_day?: string;
  avg_activities_per_day: number;
  last_activity?: string;
}

export interface TeamMetrics {
  workspace_id: string;
  period_start: string;
  period_end: string;
  total_activities: number;
  active_members: number;
  issues_created: number;
  issues_completed: number;
  comments_created: number;
  avg_response_time?: number;
  completion_rate?: number;
  top_users: [string, number][];
  activity_trend: [string, number][];
}

// ============================================================================
// UI State Types
// ============================================================================

export interface TeamDashboardState {
  selectedWorkspace?: Workspace;
  members: Member[];
  activities: Activity[];
  metrics: TeamMetrics | null;
  loading: boolean;
  error?: string;
}

export interface WorkspaceFormData {
  name: string;
  slug: string;
  description?: string;
  visibility: WorkspaceVisibility;
  template?: string;
}

export interface MemberInviteFormData {
  email: string;
  role: MemberRole;
  message?: string;
}

export interface AssignmentFormData {
  issue_id: string;
  assignee_id: string;
  priority: AssignmentPriority;
  due_date?: string;
  estimated_hours?: number;
  tags: string[];
  required_skills: string[];
}

// ============================================================================
// API Response Types
// ============================================================================

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  has_more: boolean;
}

// ============================================================================
// WebSocket Event Types
// ============================================================================

export type WebSocketEvent =
  | { type: 'activity:new'; data: Activity }
  | { type: 'member:joined'; data: Member }
  | { type: 'member:updated'; data: Member }
  | { type: 'assignment:created'; data: Assignment }
  | { type: 'assignment:updated'; data: Assignment }
  | { type: 'comment:created'; data: Comment }
  | { type: 'workspace:updated'; data: Workspace };

export interface WebSocketMessage {
  event: string;
  data: unknown;
  timestamp: string;
}

// ============================================================================
// Notification Types
// ============================================================================

export interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message: string;
  timestamp: string;
  read: boolean;
  action_url?: string;
}
