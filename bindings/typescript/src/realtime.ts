/**
 * Real-Time Collaboration Client
 *
 * Provides TypeScript bindings for CADDY's real-time collaboration system
 * with CRDTs, operational transformation, presence tracking, and document sync.
 */

import WebSocket from 'ws';
import { EventEmitter } from 'eventemitter3';

/**
 * Realtime configuration
 */
export interface RealtimeConfig {
  /** WebSocket URL */
  wsUrl: string;
  /** Authentication token */
  token?: string;
  /** Reconnect automatically */
  autoReconnect?: boolean;
  /** Reconnect interval in ms */
  reconnectInterval?: number;
  /** Max reconnect attempts */
  maxReconnectAttempts?: number;
}

/**
 * Collaboration session
 */
export interface CollaborationSession {
  /** Session ID */
  id: string;
  /** Document ID */
  documentId: string;
  /** Session participants */
  participants: Participant[];
  /** Session created at */
  createdAt: string;
  /** Session expires at */
  expiresAt?: string;
}

/**
 * Participant information
 */
export interface Participant {
  /** User ID */
  userId: string;
  /** User name */
  userName: string;
  /** User color (for cursor/selection) */
  color: string;
  /** Current cursor position */
  cursor?: CursorPosition;
  /** Current selection */
  selection?: Selection;
  /** Online status */
  online: boolean;
  /** Joined at timestamp */
  joinedAt: string;
}

/**
 * Cursor position
 */
export interface CursorPosition {
  x: number;
  y: number;
  /** Optional layer/view identifier */
  layer?: string;
}

/**
 * Selection range
 */
export interface Selection {
  start: CursorPosition;
  end: CursorPosition;
}

/**
 * Document update operation
 */
export interface DocumentUpdate {
  /** Operation ID */
  id: string;
  /** Document ID */
  documentId: string;
  /** User ID who made the update */
  userId: string;
  /** Operation type */
  type: 'insert' | 'delete' | 'update' | 'move';
  /** Operation data */
  data: any;
  /** Vector clock for ordering */
  vectorClock: Record<string, number>;
  /** Timestamp */
  timestamp: string;
}

/**
 * Presence update
 */
export interface PresenceUpdate {
  /** User ID */
  userId: string;
  /** Cursor position */
  cursor?: CursorPosition;
  /** Selection */
  selection?: Selection;
  /** Custom presence data */
  data?: Record<string, any>;
}

/**
 * Real-time collaboration events
 */
interface RealtimeEvents {
  connected: () => void;
  disconnected: () => void;
  error: (error: Error) => void;
  'session:joined': (session: CollaborationSession) => void;
  'session:left': () => void;
  'participant:joined': (participant: Participant) => void;
  'participant:left': (userId: string) => void;
  'document:update': (update: DocumentUpdate) => void;
  'presence:update': (presence: PresenceUpdate) => void;
  'sync:complete': () => void;
}

/**
 * Real-time collaboration client
 */
export class RealtimeClient extends EventEmitter<RealtimeEvents> {
  private config: Required<RealtimeConfig>;
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private currentSession: CollaborationSession | null = null;
  private isConnecting = false;

  constructor(config: RealtimeConfig) {
    super();

    this.config = {
      wsUrl: config.wsUrl,
      token: config.token || '',
      autoReconnect: config.autoReconnect ?? true,
      reconnectInterval: config.reconnectInterval || 5000,
      maxReconnectAttempts: config.maxReconnectAttempts || 10,
    };
  }

  /**
   * Connect to the real-time server
   */
  async connect(): Promise<void> {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      return;
    }

    if (this.isConnecting) {
      return;
    }

    this.isConnecting = true;

    return new Promise((resolve, reject) => {
      const wsUrl = `${this.config.wsUrl}?token=${this.config.token}`;
      this.ws = new WebSocket(wsUrl);

      this.ws.on('open', () => {
        this.isConnecting = false;
        this.reconnectAttempts = 0;
        this.emit('connected');
        resolve();
      });

      this.ws.on('message', (data: WebSocket.Data) => {
        this.handleMessage(data.toString());
      });

      this.ws.on('close', () => {
        this.emit('disconnected');
        if (this.config.autoReconnect) {
          this.scheduleReconnect();
        }
      });

      this.ws.on('error', (error) => {
        this.isConnecting = false;
        this.emit('error', new Error(error.message));
        reject(error);
      });
    });
  }

  /**
   * Disconnect from the real-time server
   */
  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.currentSession = null;
  }

  /**
   * Join a collaboration session
   */
  async joinSession(
    documentId: string,
    userName: string
  ): Promise<CollaborationSession> {
    this.send({
      type: 'session:join',
      documentId,
      userName,
    });

    // Wait for session joined event
    return new Promise((resolve) => {
      this.once('session:joined', resolve);
    });
  }

  /**
   * Leave the current collaboration session
   */
  leaveSession(): void {
    this.send({ type: 'session:leave' });
    this.currentSession = null;
    this.emit('session:left');
  }

  /**
   * Send a document update
   */
  sendUpdate(update: Omit<DocumentUpdate, 'id' | 'timestamp'>): void {
    this.send({
      type: 'document:update',
      update,
    });
  }

  /**
   * Update presence (cursor, selection)
   */
  updatePresence(presence: Omit<PresenceUpdate, 'userId'>): void {
    this.send({
      type: 'presence:update',
      presence,
    });
  }

  /**
   * Request full document synchronization
   */
  requestSync(): void {
    this.send({ type: 'sync:request' });
  }

  /**
   * Get current session
   */
  getCurrentSession(): CollaborationSession | null {
    return this.currentSession;
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }

  /**
   * Send a message to the server
   */
  private send(message: any): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket is not connected');
    }

    this.ws.send(JSON.stringify(message));
  }

  /**
   * Handle incoming messages
   */
  private handleMessage(data: string): void {
    try {
      const message = JSON.parse(data);

      switch (message.type) {
        case 'session:joined':
          this.currentSession = message.session;
          this.emit('session:joined', message.session);
          break;

        case 'participant:joined':
          this.emit('participant:joined', message.participant);
          break;

        case 'participant:left':
          this.emit('participant:left', message.userId);
          break;

        case 'document:update':
          this.emit('document:update', message.update);
          break;

        case 'presence:update':
          this.emit('presence:update', message.presence);
          break;

        case 'sync:complete':
          this.emit('sync:complete');
          break;

        case 'error':
          this.emit('error', new Error(message.error));
          break;
      }
    } catch (error: any) {
      this.emit('error', new Error(`Failed to parse message: ${error.message}`));
    }
  }

  /**
   * Schedule a reconnection attempt
   */
  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      this.emit('error', new Error('Max reconnection attempts reached'));
      return;
    }

    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++;
      this.connect().catch((error) => {
        this.emit('error', error);
      });
    }, this.config.reconnectInterval);
  }
}
