/**
 * React Hooks for Collaboration Features
 *
 * Provides hooks for accessing and managing collaboration state, including
 * real-time presence, document syncing, and conflict resolution.
 */

import { useContext, useEffect, useState, useCallback, useRef } from 'react';
import { CollaborationContext } from './CollaborationProvider';

/**
 * User information
 */
export interface User {
  id: string;
  name: string;
  email: string;
  avatar?: string;
  color: string;
}

/**
 * Cursor position
 */
export interface CursorPosition {
  x: number;
  y: number;
  z?: number;
  viewportId?: string;
}

/**
 * User presence information
 */
export interface UserPresence {
  userId: string;
  user: User;
  cursor?: CursorPosition;
  selection?: string[];
  lastActive: Date;
  isActive: boolean;
}

/**
 * Document version
 */
export interface DocumentVersion {
  id: string;
  message: string;
  author: User;
  timestamp: Date;
  operations: any[];
  tags: string[];
}

/**
 * Conflict information
 */
export interface Conflict {
  id: string;
  type: 'property' | 'delete-modify' | 'layer' | 'transform' | 'constraint' | 'structural';
  severity: 'low' | 'medium' | 'high';
  entityIds: string[];
  description: string;
  operations: any[];
  autoResolvable: boolean;
}

/**
 * Sync state
 */
export type SyncState = 'offline' | 'connecting' | 'synchronized' | 'syncing' | 'conflicted' | 'error';

/**
 * Collaboration state
 */
export interface CollaborationState {
  sessionId: string | null;
  documentId: string | null;
  currentUser: User | null;
  users: UserPresence[];
  syncState: SyncState;
  version: number;
  conflicts: Conflict[];
  isConnected: boolean;
}

/**
 * Main collaboration hook
 * Provides access to all collaboration features
 */
export function useCollaboration() {
  const context = useContext(CollaborationContext);

  if (!context) {
    throw new Error('useCollaboration must be used within CollaborationProvider');
  }

  return context;
}

/**
 * Hook for managing user presence
 */
export function usePresence() {
  const { state, updateCursor, updateSelection } = useCollaboration();

  const setCursor = useCallback((position: CursorPosition | null) => {
    updateCursor(position);
  }, [updateCursor]);

  const setSelection = useCallback((entityIds: string[]) => {
    updateSelection(entityIds);
  }, [updateSelection]);

  const otherUsers = state.users.filter(u => u.userId !== state.currentUser?.id);

  return {
    users: state.users,
    otherUsers,
    currentUser: state.currentUser,
    setCursor,
    setSelection,
  };
}

/**
 * Hook for document synchronization
 */
export function useSync() {
  const { state, applyOperation, getVersionHistory } = useCollaboration();
  const [history, setHistory] = useState<DocumentVersion[]>([]);

  const loadHistory = useCallback(async () => {
    const versions = await getVersionHistory();
    setHistory(versions);
  }, [getVersionHistory]);

  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  return {
    syncState: state.syncState,
    version: state.version,
    isConnected: state.isConnected,
    history,
    applyOperation,
    refreshHistory: loadHistory,
  };
}

/**
 * Hook for conflict management
 */
export function useConflicts() {
  const { state, resolveConflict } = useCollaboration();

  const resolve = useCallback(async (
    conflictId: string,
    strategy?: 'last-write-wins' | 'first-write-wins' | 'manual' | 'merge'
  ) => {
    return resolveConflict(conflictId, strategy);
  }, [resolveConflict]);

  const pendingConflicts = state.conflicts.filter(c => c.autoResolvable === false);
  const autoResolvableConflicts = state.conflicts.filter(c => c.autoResolvable === true);

  return {
    conflicts: state.conflicts,
    pendingConflicts,
    autoResolvableConflicts,
    hasConflicts: state.conflicts.length > 0,
    resolve,
  };
}

/**
 * Hook for WebSocket connection management
 */
export function useConnection() {
  const { state, connect, disconnect } = useCollaboration();
  const [reconnectAttempts, setReconnectAttempts] = useState(0);
  const reconnectTimerRef = useRef<NodeJS.Timeout | null>(null);

  const reconnect = useCallback(() => {
    if (reconnectTimerRef.current) {
      clearTimeout(reconnectTimerRef.current);
    }

    const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000);

    reconnectTimerRef.current = setTimeout(() => {
      connect();
      setReconnectAttempts(prev => prev + 1);
    }, delay);
  }, [connect, reconnectAttempts]);

  useEffect(() => {
    if (!state.isConnected && state.syncState !== 'offline') {
      reconnect();
    } else if (state.isConnected) {
      setReconnectAttempts(0);
      if (reconnectTimerRef.current) {
        clearTimeout(reconnectTimerRef.current);
      }
    }

    return () => {
      if (reconnectTimerRef.current) {
        clearTimeout(reconnectTimerRef.current);
      }
    };
  }, [state.isConnected, state.syncState, reconnect]);

  return {
    isConnected: state.isConnected,
    syncState: state.syncState,
    reconnectAttempts,
    connect,
    disconnect,
    reconnect,
  };
}

/**
 * Hook for document versioning
 */
export function useVersioning(documentId: string) {
  const { createBranch, switchBranch, mergeBranch, createTag, getVersionHistory } = useCollaboration();
  const [branches, setBranches] = useState<string[]>(['main']);
  const [currentBranch, setCurrentBranch] = useState('main');
  const [versions, setVersions] = useState<DocumentVersion[]>([]);

  const loadVersions = useCallback(async () => {
    const history = await getVersionHistory();
    setVersions(history);
  }, [getVersionHistory]);

  useEffect(() => {
    loadVersions();
  }, [loadVersions]);

  const createNewBranch = useCallback(async (name: string) => {
    await createBranch(name);
    setBranches(prev => [...prev, name]);
  }, [createBranch]);

  const switchToBranch = useCallback(async (name: string) => {
    await switchBranch(name);
    setCurrentBranch(name);
  }, [switchBranch]);

  const merge = useCallback(async (sourceBranch: string, strategy?: string) => {
    return mergeBranch(sourceBranch, strategy);
  }, [mergeBranch]);

  const tag = useCallback(async (tagName: string, versionId?: string) => {
    return createTag(tagName, versionId);
  }, [createTag]);

  return {
    branches,
    currentBranch,
    versions,
    createBranch: createNewBranch,
    switchBranch: switchToBranch,
    mergeBranch: merge,
    createTag: tag,
    refreshVersions: loadVersions,
  };
}

/**
 * Hook for activity tracking
 */
export function useActivity() {
  const { state } = useCollaboration();
  const [recentActivities, setRecentActivities] = useState<any[]>([]);

  useEffect(() => {
    // Track recent activities
    const activities = state.users
      .filter(u => u.isActive)
      .map(u => ({
        userId: u.userId,
        user: u.user,
        lastActive: u.lastActive,
      }))
      .sort((a, b) => b.lastActive.getTime() - a.lastActive.getTime());

    setRecentActivities(activities);
  }, [state.users]);

  return {
    recentActivities,
    activeUsers: state.users.filter(u => u.isActive),
    totalUsers: state.users.length,
  };
}

/**
 * Hook for cursor throttling
 */
export function useThrottledCursor(delay: number = 50) {
  const { updateCursor } = useCollaboration();
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const lastPositionRef = useRef<CursorPosition | null>(null);

  const throttledSetCursor = useCallback((position: CursorPosition | null) => {
    lastPositionRef.current = position;

    if (timeoutRef.current) {
      return;
    }

    timeoutRef.current = setTimeout(() => {
      if (lastPositionRef.current) {
        updateCursor(lastPositionRef.current);
      }
      timeoutRef.current = null;
    }, delay);
  }, [updateCursor, delay]);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return throttledSetCursor;
}

/**
 * Hook for offline operation queue
 */
export function useOfflineQueue() {
  const { state } = useCollaboration();
  const [queuedOperations, setQueuedOperations] = useState<any[]>([]);

  useEffect(() => {
    if (state.syncState === 'offline') {
      // Operations will be queued
    } else if (state.syncState === 'synchronized' && queuedOperations.length > 0) {
      // Clear queue when synchronized
      setQueuedOperations([]);
    }
  }, [state.syncState, queuedOperations]);

  return {
    isOffline: state.syncState === 'offline',
    queuedCount: queuedOperations.length,
    queuedOperations,
  };
}

/**
 * Hook for collaborative notifications
 */
export function useCollaborationNotifications() {
  const { state } = useCollaboration();
  const [notifications, setNotifications] = useState<any[]>([]);

  useEffect(() => {
    // Add notification when users join/leave
    const newNotifications: any[] = [];

    state.users.forEach(user => {
      if (user.isActive && user.userId !== state.currentUser?.id) {
        // User is active
      }
    });

    // Add conflict notifications
    state.conflicts.forEach(conflict => {
      if (conflict.severity === 'high') {
        newNotifications.push({
          type: 'conflict',
          severity: 'high',
          message: conflict.description,
          conflictId: conflict.id,
        });
      }
    });

    setNotifications(newNotifications);
  }, [state.users, state.conflicts, state.currentUser]);

  const dismissNotification = useCallback((id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id));
  }, []);

  return {
    notifications,
    hasNotifications: notifications.length > 0,
    dismiss: dismissNotification,
  };
}
