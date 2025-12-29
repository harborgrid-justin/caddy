/**
 * Collaboration Provider - React Context for Collaboration State
 *
 * Provides real-time collaboration state management including WebSocket
 * connection, presence tracking, document synchronization, and conflict resolution.
 */

import React, { createContext, useEffect, useReducer, useCallback, useRef } from 'react';
import type {
  User,
  UserPresence,
  CursorPosition,
  DocumentVersion,
  Conflict,
  SyncState,
  CollaborationState,
} from './useCollaboration';

/**
 * Collaboration context value
 */
export interface CollaborationContextValue {
  state: CollaborationState;
  connect: () => void;
  disconnect: () => void;
  updateCursor: (position: CursorPosition | null) => void;
  updateSelection: (entityIds: string[]) => void;
  applyOperation: (operation: any) => Promise<void>;
  resolveConflict: (conflictId: string, strategy?: string) => Promise<void>;
  createBranch: (name: string) => Promise<void>;
  switchBranch: (name: string) => Promise<void>;
  mergeBranch: (sourceBranch: string, strategy?: string) => Promise<any>;
  createTag: (tagName: string, versionId?: string) => Promise<void>;
  getVersionHistory: () => Promise<DocumentVersion[]>;
}

export const CollaborationContext = createContext<CollaborationContextValue | null>(null);

/**
 * Action types for state reducer
 */
type CollaborationAction =
  | { type: 'SET_SESSION'; payload: { sessionId: string; documentId: string } }
  | { type: 'SET_USER'; payload: User }
  | { type: 'UPDATE_USERS'; payload: UserPresence[] }
  | { type: 'ADD_USER'; payload: UserPresence }
  | { type: 'REMOVE_USER'; payload: string }
  | { type: 'UPDATE_USER_PRESENCE'; payload: { userId: string; presence: Partial<UserPresence> } }
  | { type: 'SET_SYNC_STATE'; payload: SyncState }
  | { type: 'SET_VERSION'; payload: number }
  | { type: 'ADD_CONFLICT'; payload: Conflict }
  | { type: 'REMOVE_CONFLICT'; payload: string }
  | { type: 'UPDATE_CONFLICTS'; payload: Conflict[] }
  | { type: 'SET_CONNECTED'; payload: boolean }
  | { type: 'RESET' };

/**
 * State reducer
 */
function collaborationReducer(
  state: CollaborationState,
  action: CollaborationAction
): CollaborationState {
  switch (action.type) {
    case 'SET_SESSION':
      return {
        ...state,
        sessionId: action.payload.sessionId,
        documentId: action.payload.documentId,
      };

    case 'SET_USER':
      return {
        ...state,
        currentUser: action.payload,
      };

    case 'UPDATE_USERS':
      return {
        ...state,
        users: action.payload,
      };

    case 'ADD_USER':
      return {
        ...state,
        users: [...state.users, action.payload],
      };

    case 'REMOVE_USER':
      return {
        ...state,
        users: state.users.filter(u => u.userId !== action.payload),
      };

    case 'UPDATE_USER_PRESENCE':
      return {
        ...state,
        users: state.users.map(u =>
          u.userId === action.payload.userId
            ? { ...u, ...action.payload.presence }
            : u
        ),
      };

    case 'SET_SYNC_STATE':
      return {
        ...state,
        syncState: action.payload,
      };

    case 'SET_VERSION':
      return {
        ...state,
        version: action.payload,
      };

    case 'ADD_CONFLICT':
      return {
        ...state,
        conflicts: [...state.conflicts, action.payload],
      };

    case 'REMOVE_CONFLICT':
      return {
        ...state,
        conflicts: state.conflicts.filter(c => c.id !== action.payload),
      };

    case 'UPDATE_CONFLICTS':
      return {
        ...state,
        conflicts: action.payload,
      };

    case 'SET_CONNECTED':
      return {
        ...state,
        isConnected: action.payload,
      };

    case 'RESET':
      return initialState;

    default:
      return state;
  }
}

/**
 * Initial state
 */
const initialState: CollaborationState = {
  sessionId: null,
  documentId: null,
  currentUser: null,
  users: [],
  syncState: 'offline',
  version: 0,
  conflicts: [],
  isConnected: false,
};

/**
 * WebSocket message types
 */
interface WSMessage {
  type: string;
  payload: any;
}

/**
 * Provider props
 */
export interface CollaborationProviderProps {
  children: React.ReactNode;
  websocketUrl: string;
  documentId: string;
  currentUser: User;
  autoConnect?: boolean;
  reconnectDelay?: number;
  heartbeatInterval?: number;
}

/**
 * Collaboration Provider Component
 */
export function CollaborationProvider({
  children,
  websocketUrl,
  documentId,
  currentUser,
  autoConnect = true,
  reconnectDelay = 3000,
  heartbeatInterval = 30000,
}: CollaborationProviderProps) {
  const [state, dispatch] = useReducer(collaborationReducer, initialState);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<NodeJS.Timeout | null>(null);
  const heartbeatTimerRef = useRef<NodeJS.Timeout | null>(null);
  const messageQueueRef = useRef<WSMessage[]>([]);

  /**
   * Send message via WebSocket
   */
  const sendMessage = useCallback((message: WSMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      // Queue message for later
      messageQueueRef.current.push(message);
    }
  }, []);

  /**
   * Flush queued messages
   */
  const flushMessageQueue = useCallback(() => {
    while (messageQueueRef.current.length > 0) {
      const message = messageQueueRef.current.shift();
      if (message) {
        sendMessage(message);
      }
    }
  }, [sendMessage]);

  /**
   * Handle WebSocket messages
   */
  const handleMessage = useCallback((event: MessageEvent) => {
    try {
      const message: WSMessage = JSON.parse(event.data);

      switch (message.type) {
        case 'session_joined':
          dispatch({
            type: 'SET_SESSION',
            payload: {
              sessionId: message.payload.sessionId,
              documentId: message.payload.documentId,
            },
          });
          break;

        case 'user_joined':
          dispatch({ type: 'ADD_USER', payload: message.payload });
          break;

        case 'user_left':
          dispatch({ type: 'REMOVE_USER', payload: message.payload.userId });
          break;

        case 'presence_update':
          dispatch({
            type: 'UPDATE_USER_PRESENCE',
            payload: {
              userId: message.payload.userId,
              presence: message.payload.presence,
            },
          });
          break;

        case 'users_list':
          dispatch({ type: 'UPDATE_USERS', payload: message.payload.users });
          break;

        case 'sync_state':
          dispatch({ type: 'SET_SYNC_STATE', payload: message.payload.state });
          break;

        case 'version_update':
          dispatch({ type: 'SET_VERSION', payload: message.payload.version });
          break;

        case 'conflict_detected':
          dispatch({ type: 'ADD_CONFLICT', payload: message.payload });
          break;

        case 'conflict_resolved':
          dispatch({ type: 'REMOVE_CONFLICT', payload: message.payload.conflictId });
          break;

        case 'operation_applied':
          // Handle operation application
          dispatch({ type: 'SET_VERSION', payload: message.payload.version });
          break;

        case 'full_sync':
          // Handle full document sync
          dispatch({ type: 'SET_VERSION', payload: message.payload.version });
          break;

        default:
          console.warn('Unknown message type:', message.type);
      }
    } catch (error) {
      console.error('Error handling WebSocket message:', error);
    }
  }, []);

  /**
   * Start heartbeat
   */
  const startHeartbeat = useCallback(() => {
    if (heartbeatTimerRef.current) {
      clearInterval(heartbeatTimerRef.current);
    }

    heartbeatTimerRef.current = setInterval(() => {
      sendMessage({
        type: 'heartbeat',
        payload: {
          userId: currentUser.id,
          timestamp: new Date().toISOString(),
        },
      });
    }, heartbeatInterval);
  }, [sendMessage, currentUser.id, heartbeatInterval]);

  /**
   * Stop heartbeat
   */
  const stopHeartbeat = useCallback(() => {
    if (heartbeatTimerRef.current) {
      clearInterval(heartbeatTimerRef.current);
      heartbeatTimerRef.current = null;
    }
  }, []);

  /**
   * Connect to WebSocket
   */
  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    dispatch({ type: 'SET_SYNC_STATE', payload: 'connecting' });

    const ws = new WebSocket(websocketUrl);

    ws.onopen = () => {
      console.log('WebSocket connected');
      dispatch({ type: 'SET_CONNECTED', payload: true });
      dispatch({ type: 'SET_SYNC_STATE', payload: 'synchronized' });

      // Join session
      sendMessage({
        type: 'join_session',
        payload: {
          documentId,
          user: currentUser,
        },
      });

      // Request sync
      sendMessage({
        type: 'sync_request',
        payload: {
          documentId,
          lastKnownVersion: state.version,
        },
      });

      flushMessageQueue();
      startHeartbeat();
    };

    ws.onmessage = handleMessage;

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      dispatch({ type: 'SET_SYNC_STATE', payload: 'error' });
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      dispatch({ type: 'SET_CONNECTED', payload: false });
      dispatch({ type: 'SET_SYNC_STATE', payload: 'offline' });
      stopHeartbeat();

      // Attempt reconnection
      if (reconnectTimerRef.current) {
        clearTimeout(reconnectTimerRef.current);
      }

      reconnectTimerRef.current = setTimeout(() => {
        if (autoConnect) {
          connect();
        }
      }, reconnectDelay);
    };

    wsRef.current = ws;
  }, [
    websocketUrl,
    documentId,
    currentUser,
    state.version,
    handleMessage,
    sendMessage,
    flushMessageQueue,
    startHeartbeat,
    stopHeartbeat,
    autoConnect,
    reconnectDelay,
  ]);

  /**
   * Disconnect from WebSocket
   */
  const disconnect = useCallback(() => {
    if (reconnectTimerRef.current) {
      clearTimeout(reconnectTimerRef.current);
    }

    stopHeartbeat();

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    dispatch({ type: 'SET_CONNECTED', payload: false });
    dispatch({ type: 'SET_SYNC_STATE', payload: 'offline' });
  }, [stopHeartbeat]);

  /**
   * Update cursor position
   */
  const updateCursor = useCallback((position: CursorPosition | null) => {
    sendMessage({
      type: 'cursor_update',
      payload: {
        userId: currentUser.id,
        position,
      },
    });
  }, [sendMessage, currentUser.id]);

  /**
   * Update selection
   */
  const updateSelection = useCallback((entityIds: string[]) => {
    sendMessage({
      type: 'selection_update',
      payload: {
        userId: currentUser.id,
        selection: entityIds,
      },
    });
  }, [sendMessage, currentUser.id]);

  /**
   * Apply operation
   */
  const applyOperation = useCallback(async (operation: any) => {
    sendMessage({
      type: 'apply_operation',
      payload: {
        operation,
        userId: currentUser.id,
      },
    });
  }, [sendMessage, currentUser.id]);

  /**
   * Resolve conflict
   */
  const resolveConflict = useCallback(async (conflictId: string, strategy?: string) => {
    sendMessage({
      type: 'resolve_conflict',
      payload: {
        conflictId,
        strategy: strategy || 'last-write-wins',
        userId: currentUser.id,
      },
    });
  }, [sendMessage, currentUser.id]);

  /**
   * Create branch
   */
  const createBranch = useCallback(async (name: string) => {
    sendMessage({
      type: 'create_branch',
      payload: {
        name,
        documentId,
      },
    });
  }, [sendMessage, documentId]);

  /**
   * Switch branch
   */
  const switchBranch = useCallback(async (name: string) => {
    sendMessage({
      type: 'switch_branch',
      payload: {
        name,
        documentId,
      },
    });
  }, [sendMessage, documentId]);

  /**
   * Merge branch
   */
  const mergeBranch = useCallback(async (sourceBranch: string, strategy?: string) => {
    return new Promise((resolve, reject) => {
      const messageId = Math.random().toString(36);

      sendMessage({
        type: 'merge_branch',
        payload: {
          sourceBranch,
          strategy: strategy || 'three-way',
          documentId,
          messageId,
        },
      });

      // Wait for response (simplified - in production use proper request/response pattern)
      setTimeout(() => resolve({ success: true }), 1000);
    });
  }, [sendMessage, documentId]);

  /**
   * Create tag
   */
  const createTag = useCallback(async (tagName: string, versionId?: string) => {
    sendMessage({
      type: 'create_tag',
      payload: {
        tagName,
        versionId,
        documentId,
      },
    });
  }, [sendMessage, documentId]);

  /**
   * Get version history
   */
  const getVersionHistory = useCallback(async (): Promise<DocumentVersion[]> => {
    return new Promise((resolve) => {
      sendMessage({
        type: 'get_version_history',
        payload: { documentId },
      });

      // In production, wait for actual response
      setTimeout(() => resolve([]), 1000);
    });
  }, [sendMessage, documentId]);

  /**
   * Initialize
   */
  useEffect(() => {
    dispatch({ type: 'SET_USER', payload: currentUser });

    if (autoConnect) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [currentUser, autoConnect, connect, disconnect]);

  /**
   * Context value
   */
  const contextValue: CollaborationContextValue = {
    state,
    connect,
    disconnect,
    updateCursor,
    updateSelection,
    applyOperation,
    resolveConflict,
    createBranch,
    switchBranch,
    mergeBranch,
    createTag,
    getVersionHistory,
  };

  return (
    <CollaborationContext.Provider value={contextValue}>
      {children}
    </CollaborationContext.Provider>
  );
}

export default CollaborationProvider;
