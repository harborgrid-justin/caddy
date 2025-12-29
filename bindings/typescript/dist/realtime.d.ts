import { EventEmitter } from 'eventemitter3';
export interface RealtimeConfig {
    wsUrl: string;
    token?: string;
    autoReconnect?: boolean;
    reconnectInterval?: number;
    maxReconnectAttempts?: number;
}
export interface CollaborationSession {
    id: string;
    documentId: string;
    participants: Participant[];
    createdAt: string;
    expiresAt?: string;
}
export interface Participant {
    userId: string;
    userName: string;
    color: string;
    cursor?: CursorPosition;
    selection?: Selection;
    online: boolean;
    joinedAt: string;
}
export interface CursorPosition {
    x: number;
    y: number;
    layer?: string;
}
export interface Selection {
    start: CursorPosition;
    end: CursorPosition;
}
export interface DocumentUpdate {
    id: string;
    documentId: string;
    userId: string;
    type: 'insert' | 'delete' | 'update' | 'move';
    data: any;
    vectorClock: Record<string, number>;
    timestamp: string;
}
export interface PresenceUpdate {
    userId: string;
    cursor?: CursorPosition;
    selection?: Selection;
    data?: Record<string, any>;
}
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
export declare class RealtimeClient extends EventEmitter<RealtimeEvents> {
    private config;
    private ws;
    private reconnectAttempts;
    private reconnectTimer;
    private currentSession;
    private isConnecting;
    constructor(config: RealtimeConfig);
    connect(): Promise<void>;
    disconnect(): void;
    joinSession(documentId: string, userName: string): Promise<CollaborationSession>;
    leaveSession(): void;
    sendUpdate(update: Omit<DocumentUpdate, 'id' | 'timestamp'>): void;
    updatePresence(presence: Omit<PresenceUpdate, 'userId'>): void;
    requestSync(): void;
    getCurrentSession(): CollaborationSession | null;
    isConnected(): boolean;
    private send;
    private handleMessage;
    private scheduleReconnect;
}
export {};
//# sourceMappingURL=realtime.d.ts.map