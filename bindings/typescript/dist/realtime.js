import WebSocket from 'ws';
import { EventEmitter } from 'eventemitter3';
export class RealtimeClient extends EventEmitter {
    constructor(config) {
        super();
        this.ws = null;
        this.reconnectAttempts = 0;
        this.reconnectTimer = null;
        this.currentSession = null;
        this.isConnecting = false;
        this.config = {
            wsUrl: config.wsUrl,
            token: config.token || '',
            autoReconnect: config.autoReconnect ?? true,
            reconnectInterval: config.reconnectInterval || 5000,
            maxReconnectAttempts: config.maxReconnectAttempts || 10,
        };
    }
    async connect() {
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
            this.ws.on('message', (data) => {
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
    disconnect() {
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
    async joinSession(documentId, userName) {
        this.send({
            type: 'session:join',
            documentId,
            userName,
        });
        return new Promise((resolve) => {
            this.once('session:joined', resolve);
        });
    }
    leaveSession() {
        this.send({ type: 'session:leave' });
        this.currentSession = null;
        this.emit('session:left');
    }
    sendUpdate(update) {
        this.send({
            type: 'document:update',
            update,
        });
    }
    updatePresence(presence) {
        this.send({
            type: 'presence:update',
            presence,
        });
    }
    requestSync() {
        this.send({ type: 'sync:request' });
    }
    getCurrentSession() {
        return this.currentSession;
    }
    isConnected() {
        return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
    }
    send(message) {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            throw new Error('WebSocket is not connected');
        }
        this.ws.send(JSON.stringify(message));
    }
    handleMessage(data) {
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
        }
        catch (error) {
            this.emit('error', new Error(`Failed to parse message: ${error.message}`));
        }
    }
    scheduleReconnect() {
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
//# sourceMappingURL=realtime.js.map