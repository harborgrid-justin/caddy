/**
 * CADDY v0.2.5 - Enterprise Real-Time Collaboration Module
 *
 * Complete collaboration system with CRDT-based synchronization, presence tracking,
 * version control, and conflict resolution.
 *
 * @module collaboration
 */

// Provider and Context
export { CollaborationProvider, CollaborationContext } from './CollaborationProvider';
export type { CollaborationProviderProps, CollaborationContextValue } from './CollaborationProvider';

// Hooks
export {
  useCollaboration,
  usePresence,
  useSync,
  useConflicts,
  useConnection,
  useVersioning,
  useActivity,
  useThrottledCursor,
  useOfflineQueue,
  useCollaborationNotifications,
} from './useCollaboration';

export type {
  User,
  UserPresence,
  CursorPosition,
  DocumentVersion,
  Conflict,
  SyncState,
  CollaborationState,
} from './useCollaboration';

// Components
export { CollaborationPanel } from './CollaborationPanel';
export type { CollaborationPanelProps } from './CollaborationPanel';

export { CursorOverlay, useLocalCursor, CursorMiniMap } from './CursorOverlay';
export type { CursorOverlayProps } from './CursorOverlay';

export { VersionHistory } from './VersionHistory';
export type { VersionHistoryProps } from './VersionHistory';

export { ConflictDialog, ConflictBadge } from './ConflictDialog';
export type { ConflictDialogProps } from './ConflictDialog';
