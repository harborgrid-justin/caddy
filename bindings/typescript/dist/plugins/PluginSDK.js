import EventEmitter from 'eventemitter3';
import { Permission, NotificationLevel, } from './types';
export class PluginSDK extends EventEmitter {
    constructor(context) {
        super();
        this.ws = null;
        this.connected = false;
        this.messageHandlers = new Map();
        this.requestId = 0;
        this.context = context;
    }
    async initialize(wsUrl) {
        const url = wsUrl || this.getDefaultWebSocketUrl();
        return new Promise((resolve, reject) => {
            try {
                this.ws = new WebSocket(url);
                this.ws.onopen = () => {
                    this.connected = true;
                    this.emit('connected');
                    this.send('plugin.init', {
                        pluginId: this.context.pluginId,
                        apiVersion: this.context.apiVersion,
                    });
                    resolve();
                };
                this.ws.onerror = (error) => {
                    this.connected = false;
                    this.emit('error', error);
                    reject(new Error('WebSocket connection failed'));
                };
                this.ws.onclose = () => {
                    this.connected = false;
                    this.emit('disconnected');
                };
                this.ws.onmessage = (event) => {
                    this.handleMessage(event.data);
                };
            }
            catch (error) {
                reject(error);
            }
        });
    }
    disconnect() {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
            this.connected = false;
        }
    }
    isConnected() {
        return this.connected;
    }
    getContext() {
        return this.context;
    }
    async updateConfig(config) {
        this.context.config = config;
        await this.call('plugin.updateConfig', { config });
        this.emit('configChanged', config);
    }
    hasPermission(permission) {
        return this.context.permissions.includes(permission);
    }
    requirePermission(permission) {
        if (!this.hasPermission(permission)) {
            throw this.createError('PermissionDenied', `Permission not granted: ${permission}`);
        }
    }
    async createEntity(type, params) {
        this.requirePermission(Permission.GeometryWrite);
        const result = await this.call('geometry.createEntity', { type, params });
        return result.entityId;
    }
    async getEntity(id) {
        this.requirePermission(Permission.GeometryRead);
        return await this.call('geometry.getEntity', { id });
    }
    async updateEntity(id, updates) {
        this.requirePermission(Permission.GeometryWrite);
        await this.call('geometry.updateEntity', { id, updates });
    }
    async deleteEntity(id) {
        this.requirePermission(Permission.GeometryDelete);
        await this.call('geometry.deleteEntity', { id });
    }
    async requestRender() {
        this.requirePermission(Permission.RenderingWrite);
        await this.call('rendering.requestRender', {});
    }
    async setRenderMode(mode) {
        this.requirePermission(Permission.RenderingWrite);
        await this.call('rendering.setRenderMode', { mode });
    }
    async getViewport() {
        this.requirePermission(Permission.RenderingRead);
        return await this.call('rendering.getViewport', {});
    }
    async showNotification(title, message, level = NotificationLevel.Info) {
        this.requirePermission(Permission.UIWrite);
        await this.call('ui.showNotification', { title, message, level });
    }
    async showDialog(config) {
        this.requirePermission(Permission.UIWrite);
        const result = await this.call('ui.showDialog', config);
        return result.button;
    }
    async registerMenuItem(path, label, handler) {
        this.requirePermission(Permission.UIMenuAccess);
        const callbackId = this.registerCallback(handler);
        await this.call('ui.registerMenuItem', {
            path,
            label,
            callbackId,
        });
    }
    async registerToolbarButton(config) {
        this.requirePermission(Permission.UIToolbarAccess);
        const callbackId = this.registerCallback(config.handler);
        const result = await this.call('ui.registerToolbarButton', {
            label: config.label,
            icon: config.icon,
            tooltip: config.tooltip,
            callbackId,
        });
        return result.buttonId;
    }
    async readFile(path) {
        this.requirePermission(Permission.FileRead);
        const result = await this.call('file.read', { path });
        return new Uint8Array(result.data);
    }
    async writeFile(path, data) {
        this.requirePermission(Permission.FileWrite);
        await this.call('file.write', { path, data: Array.from(data) });
    }
    async readTextFile(path) {
        const data = await this.readFile(path);
        return new TextDecoder().decode(data);
    }
    async writeTextFile(path, text) {
        const data = new TextEncoder().encode(text);
        await this.writeFile(path, data);
    }
    async listDirectory(path) {
        this.requirePermission(Permission.FileRead);
        const result = await this.call('file.listDirectory', { path });
        return result.entries;
    }
    async executeCommand(command, params = {}) {
        this.requirePermission(Permission.CommandExecute);
        return await this.call('command.execute', { command, params });
    }
    async registerCommand(name, description, handler) {
        this.requirePermission(Permission.CommandRegister);
        const callbackId = this.registerCallback(handler);
        await this.call('command.register', {
            name,
            description,
            callbackId,
        });
    }
    async httpRequest(config) {
        this.requirePermission(Permission.NetworkHTTP);
        return await this.call('network.httpRequest', config);
    }
    async get(url, headers) {
        return await this.httpRequest({
            url,
            method: 'GET',
            headers,
        });
    }
    async post(url, body, headers) {
        return await this.httpRequest({
            url,
            method: 'POST',
            body,
            headers,
        });
    }
    async readClipboard() {
        this.requirePermission(Permission.SystemClipboard);
        const result = await this.call('system.readClipboard', {});
        return result.text;
    }
    async writeClipboard(text) {
        this.requirePermission(Permission.SystemClipboard);
        await this.call('system.writeClipboard', { text });
    }
    async getStorageValue(key) {
        const result = await this.call('storage.get', { key });
        return result.value;
    }
    async setStorageValue(key, value) {
        await this.call('storage.set', { key, value });
    }
    async deleteStorageValue(key) {
        await this.call('storage.delete', { key });
    }
    async clearStorage() {
        await this.call('storage.clear', {});
    }
    send(type, data) {
        if (!this.ws || !this.connected) {
            throw this.createError('NotConnected', 'Plugin SDK is not connected');
        }
        this.ws.send(JSON.stringify({
            type,
            data,
            pluginId: this.context.pluginId,
        }));
    }
    async call(method, params) {
        return new Promise((resolve, reject) => {
            const id = ++this.requestId;
            const handler = (response) => {
                if (response.error) {
                    reject(this.createError(response.error.code, response.error.message, response.error.details));
                }
                else {
                    resolve(response.result);
                }
            };
            this.messageHandlers.set(`response:${id}`, handler);
            this.send('call', {
                id,
                method,
                params,
            });
            setTimeout(() => {
                this.messageHandlers.delete(`response:${id}`);
                reject(this.createError('Timeout', 'Request timed out'));
            }, 30000);
        });
    }
    handleMessage(data) {
        try {
            const message = JSON.parse(data);
            if (message.type === 'response' && message.id !== undefined) {
                const handler = this.messageHandlers.get(`response:${message.id}`);
                if (handler) {
                    handler(message);
                    this.messageHandlers.delete(`response:${message.id}`);
                }
                return;
            }
            if (message.type === 'callback' && message.callbackId) {
                const handler = this.messageHandlers.get(`callback:${message.callbackId}`);
                if (handler) {
                    const result = handler(message.data);
                    this.send('callbackResult', {
                        callbackId: message.callbackId,
                        result,
                    });
                }
                return;
            }
            if (message.type === 'event') {
                this.emit(message.eventType, message.data);
            }
        }
        catch (error) {
            console.error('Failed to handle message:', error);
        }
    }
    registerCallback(handler) {
        const callbackId = `callback_${++this.requestId}`;
        this.messageHandlers.set(`callback:${callbackId}`, handler);
        return callbackId;
    }
    getDefaultWebSocketUrl() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const host = window.location.host;
        return `${protocol}//${host}/api/plugins/${this.context.pluginId}/ws`;
    }
    createError(code, message, details) {
        return {
            code,
            message,
            pluginId: this.context.pluginId,
            details,
        };
    }
}
export function createPluginSDK(context) {
    return new PluginSDK(context);
}
export class Plugin {
    constructor(context) {
        this.context = context;
        this.sdk = new PluginSDK(context);
    }
    async initialize() {
        await this.sdk.initialize();
        await this.onInit();
    }
    async shutdown() {
        await this.onShutdown();
        this.sdk.disconnect();
    }
    getSDK() {
        return this.sdk;
    }
}
export default PluginSDK;
//# sourceMappingURL=PluginSDK.js.map