/**
 * Plugin SDK for CADDY Enterprise
 *
 * TypeScript SDK for plugin developers to interact with the CADDY
 * plugin system, including API access, lifecycle management, and
 * communication with the host application.
 */

import EventEmitter from 'eventemitter3';
import {
  PluginApiContext,
  PluginConfig,
  PluginEvent,
  PluginEventType,
  Permission,
  NotificationLevel,
  DialogConfig,
  HttpRequestConfig,
  HttpResponse,
  PluginError,
} from './types';

/**
 * Plugin SDK main class
 */
export class PluginSDK extends EventEmitter {
  private context: PluginApiContext;
  private ws: WebSocket | null = null;
  private connected: boolean = false;
  private messageHandlers: Map<string, (data: any) => void> = new Map();
  private requestId: number = 0;

  constructor(context: PluginApiContext) {
    super();
    this.context = context;
  }

  /**
   * Initialize the SDK connection to the host
   */
  async initialize(wsUrl?: string): Promise<void> {
    const url = wsUrl || this.getDefaultWebSocketUrl();

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(url);

        this.ws.onopen = () => {
          this.connected = true;
          this.emit('connected');

          // Send initialization message
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
      } catch (error) {
        reject(error);
      }
    });
  }

  /**
   * Disconnect from the host
   */
  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
      this.connected = false;
    }
  }

  /**
   * Check if SDK is connected
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Get plugin context
   */
  getContext(): PluginApiContext {
    return this.context;
  }

  /**
   * Update plugin configuration
   */
  async updateConfig(config: PluginConfig): Promise<void> {
    this.context.config = config;
    await this.call('plugin.updateConfig', { config });
    this.emit('configChanged', config);
  }

  /**
   * Check if a permission is granted
   */
  hasPermission(permission: Permission): boolean {
    return this.context.permissions.includes(permission);
  }

  /**
   * Require a specific permission (throws if not granted)
   */
  requirePermission(permission: Permission): void {
    if (!this.hasPermission(permission)) {
      throw this.createError('PermissionDenied', `Permission not granted: ${permission}`);
    }
  }

  // ===================
  // Geometry API
  // ===================

  /**
   * Create a geometry entity
   */
  async createEntity(type: string, params: Record<string, any>): Promise<string> {
    this.requirePermission(Permission.GeometryWrite);
    const result = await this.call('geometry.createEntity', { type, params });
    return result.entityId;
  }

  /**
   * Get entity by ID
   */
  async getEntity(id: string): Promise<any> {
    this.requirePermission(Permission.GeometryRead);
    return await this.call('geometry.getEntity', { id });
  }

  /**
   * Update entity
   */
  async updateEntity(id: string, updates: Record<string, any>): Promise<void> {
    this.requirePermission(Permission.GeometryWrite);
    await this.call('geometry.updateEntity', { id, updates });
  }

  /**
   * Delete entity
   */
  async deleteEntity(id: string): Promise<void> {
    this.requirePermission(Permission.GeometryDelete);
    await this.call('geometry.deleteEntity', { id });
  }

  // ===================
  // Rendering API
  // ===================

  /**
   * Request a render update
   */
  async requestRender(): Promise<void> {
    this.requirePermission(Permission.RenderingWrite);
    await this.call('rendering.requestRender', {});
  }

  /**
   * Set render mode
   */
  async setRenderMode(mode: string): Promise<void> {
    this.requirePermission(Permission.RenderingWrite);
    await this.call('rendering.setRenderMode', { mode });
  }

  /**
   * Get viewport information
   */
  async getViewport(): Promise<{ width: number; height: number; scale: number }> {
    this.requirePermission(Permission.RenderingRead);
    return await this.call('rendering.getViewport', {});
  }

  // ===================
  // UI API
  // ===================

  /**
   * Show notification
   */
  async showNotification(
    title: string,
    message: string,
    level: NotificationLevel = NotificationLevel.Info
  ): Promise<void> {
    this.requirePermission(Permission.UIWrite);
    await this.call('ui.showNotification', { title, message, level });
  }

  /**
   * Show dialog
   */
  async showDialog(config: DialogConfig): Promise<string> {
    this.requirePermission(Permission.UIWrite);
    const result = await this.call('ui.showDialog', config);
    return result.button;
  }

  /**
   * Register menu item
   */
  async registerMenuItem(path: string, label: string, handler: () => void): Promise<void> {
    this.requirePermission(Permission.UIMenuAccess);

    const callbackId = this.registerCallback(handler);

    await this.call('ui.registerMenuItem', {
      path,
      label,
      callbackId,
    });
  }

  /**
   * Register toolbar button
   */
  async registerToolbarButton(config: {
    label: string;
    icon?: string;
    tooltip?: string;
    handler: () => void;
  }): Promise<string> {
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

  // ===================
  // File I/O API
  // ===================

  /**
   * Read file
   */
  async readFile(path: string): Promise<Uint8Array> {
    this.requirePermission(Permission.FileRead);
    const result = await this.call('file.read', { path });
    return new Uint8Array(result.data);
  }

  /**
   * Write file
   */
  async writeFile(path: string, data: Uint8Array): Promise<void> {
    this.requirePermission(Permission.FileWrite);
    await this.call('file.write', { path, data: Array.from(data) });
  }

  /**
   * Read text file
   */
  async readTextFile(path: string): Promise<string> {
    const data = await this.readFile(path);
    return new TextDecoder().decode(data);
  }

  /**
   * Write text file
   */
  async writeTextFile(path: string, text: string): Promise<void> {
    const data = new TextEncoder().encode(text);
    await this.writeFile(path, data);
  }

  /**
   * List directory
   */
  async listDirectory(path: string): Promise<string[]> {
    this.requirePermission(Permission.FileRead);
    const result = await this.call('file.listDirectory', { path });
    return result.entries;
  }

  // ===================
  // Command API
  // ===================

  /**
   * Execute command
   */
  async executeCommand(command: string, params: Record<string, any> = {}): Promise<any> {
    this.requirePermission(Permission.CommandExecute);
    return await this.call('command.execute', { command, params });
  }

  /**
   * Register command
   */
  async registerCommand(
    name: string,
    description: string,
    handler: (params: Record<string, any>) => Promise<any>
  ): Promise<void> {
    this.requirePermission(Permission.CommandRegister);

    const callbackId = this.registerCallback(handler);

    await this.call('command.register', {
      name,
      description,
      callbackId,
    });
  }

  // ===================
  // Network API
  // ===================

  /**
   * Make HTTP request
   */
  async httpRequest(config: HttpRequestConfig): Promise<HttpResponse> {
    this.requirePermission(Permission.NetworkHTTP);
    return await this.call('network.httpRequest', config);
  }

  /**
   * HTTP GET request
   */
  async get(url: string, headers?: Record<string, string>): Promise<HttpResponse> {
    return await this.httpRequest({
      url,
      method: 'GET',
      headers,
    });
  }

  /**
   * HTTP POST request
   */
  async post(
    url: string,
    body: any,
    headers?: Record<string, string>
  ): Promise<HttpResponse> {
    return await this.httpRequest({
      url,
      method: 'POST',
      body,
      headers,
    });
  }

  // ===================
  // System API
  // ===================

  /**
   * Read from clipboard
   */
  async readClipboard(): Promise<string> {
    this.requirePermission(Permission.SystemClipboard);
    const result = await this.call('system.readClipboard', {});
    return result.text;
  }

  /**
   * Write to clipboard
   */
  async writeClipboard(text: string): Promise<void> {
    this.requirePermission(Permission.SystemClipboard);
    await this.call('system.writeClipboard', { text });
  }

  // ===================
  // Storage API
  // ===================

  /**
   * Get plugin storage value
   */
  async getStorageValue(key: string): Promise<any> {
    const result = await this.call('storage.get', { key });
    return result.value;
  }

  /**
   * Set plugin storage value
   */
  async setStorageValue(key: string, value: any): Promise<void> {
    await this.call('storage.set', { key, value });
  }

  /**
   * Delete plugin storage value
   */
  async deleteStorageValue(key: string): Promise<void> {
    await this.call('storage.delete', { key });
  }

  /**
   * Clear all plugin storage
   */
  async clearStorage(): Promise<void> {
    await this.call('storage.clear', {});
  }

  // ===================
  // Internal Methods
  // ===================

  /**
   * Send message to host
   */
  private send(type: string, data: any): void {
    if (!this.ws || !this.connected) {
      throw this.createError('NotConnected', 'Plugin SDK is not connected');
    }

    this.ws.send(
      JSON.stringify({
        type,
        data,
        pluginId: this.context.pluginId,
      })
    );
  }

  /**
   * Call host API method
   */
  private async call(method: string, params: any): Promise<any> {
    return new Promise((resolve, reject) => {
      const id = ++this.requestId;

      const handler = (response: any) => {
        if (response.error) {
          reject(this.createError(response.error.code, response.error.message, response.error.details));
        } else {
          resolve(response.result);
        }
      };

      this.messageHandlers.set(`response:${id}`, handler);

      this.send('call', {
        id,
        method,
        params,
      });

      // Timeout after 30 seconds
      setTimeout(() => {
        this.messageHandlers.delete(`response:${id}`);
        reject(this.createError('Timeout', 'Request timed out'));
      }, 30000);
    });
  }

  /**
   * Handle incoming message from host
   */
  private handleMessage(data: string): void {
    try {
      const message = JSON.parse(data);

      // Handle response to API call
      if (message.type === 'response' && message.id !== undefined) {
        const handler = this.messageHandlers.get(`response:${message.id}`);
        if (handler) {
          handler(message);
          this.messageHandlers.delete(`response:${message.id}`);
        }
        return;
      }

      // Handle callback invocation
      if (message.type === 'callback' && message.callbackId) {
        const handler = this.messageHandlers.get(`callback:${message.callbackId}`);
        if (handler) {
          const result = handler(message.data);
          // Send callback result back
          this.send('callbackResult', {
            callbackId: message.callbackId,
            result,
          });
        }
        return;
      }

      // Handle plugin events
      if (message.type === 'event') {
        this.emit(message.eventType, message.data);
      }
    } catch (error) {
      console.error('Failed to handle message:', error);
    }
  }

  /**
   * Register callback handler
   */
  private registerCallback(handler: (...args: any[]) => any): string {
    const callbackId = `callback_${++this.requestId}`;
    this.messageHandlers.set(`callback:${callbackId}`, handler);
    return callbackId;
  }

  /**
   * Get default WebSocket URL
   */
  private getDefaultWebSocketUrl(): string {
    // In a real implementation, this would be configured based on environment
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    return `${protocol}//${host}/api/plugins/${this.context.pluginId}/ws`;
  }

  /**
   * Create plugin error
   */
  private createError(code: string, message: string, details?: any): PluginError {
    return {
      code,
      message,
      pluginId: this.context.pluginId,
      details,
    };
  }
}

/**
 * Create plugin SDK instance
 */
export function createPluginSDK(context: PluginApiContext): PluginSDK {
  return new PluginSDK(context);
}

/**
 * Plugin base class for easy development
 */
export abstract class Plugin {
  protected sdk: PluginSDK;

  constructor(protected context: PluginApiContext) {
    this.sdk = new PluginSDK(context);
  }

  /**
   * Initialize the plugin
   */
  async initialize(): Promise<void> {
    await this.sdk.initialize();
    await this.onInit();
  }

  /**
   * Shutdown the plugin
   */
  async shutdown(): Promise<void> {
    await this.onShutdown();
    this.sdk.disconnect();
  }

  /**
   * Override to handle plugin initialization
   */
  protected abstract onInit(): Promise<void>;

  /**
   * Override to handle plugin shutdown
   */
  protected abstract onShutdown(): Promise<void>;

  /**
   * Get SDK instance
   */
  getSDK(): PluginSDK {
    return this.sdk;
  }
}

export default PluginSDK;
