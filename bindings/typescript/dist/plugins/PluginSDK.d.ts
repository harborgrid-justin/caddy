import EventEmitter from 'eventemitter3';
import { PluginApiContext, PluginConfig, Permission, NotificationLevel, DialogConfig, HttpRequestConfig, HttpResponse } from './types';
export declare class PluginSDK extends EventEmitter {
    private context;
    private ws;
    private connected;
    private messageHandlers;
    private requestId;
    constructor(context: PluginApiContext);
    initialize(wsUrl?: string): Promise<void>;
    disconnect(): void;
    isConnected(): boolean;
    getContext(): PluginApiContext;
    updateConfig(config: PluginConfig): Promise<void>;
    hasPermission(permission: Permission): boolean;
    requirePermission(permission: Permission): void;
    createEntity(type: string, params: Record<string, any>): Promise<string>;
    getEntity(id: string): Promise<any>;
    updateEntity(id: string, updates: Record<string, any>): Promise<void>;
    deleteEntity(id: string): Promise<void>;
    requestRender(): Promise<void>;
    setRenderMode(mode: string): Promise<void>;
    getViewport(): Promise<{
        width: number;
        height: number;
        scale: number;
    }>;
    showNotification(title: string, message: string, level?: NotificationLevel): Promise<void>;
    showDialog(config: DialogConfig): Promise<string>;
    registerMenuItem(path: string, label: string, handler: () => void): Promise<void>;
    registerToolbarButton(config: {
        label: string;
        icon?: string;
        tooltip?: string;
        handler: () => void;
    }): Promise<string>;
    readFile(path: string): Promise<Uint8Array>;
    writeFile(path: string, data: Uint8Array): Promise<void>;
    readTextFile(path: string): Promise<string>;
    writeTextFile(path: string, text: string): Promise<void>;
    listDirectory(path: string): Promise<string[]>;
    executeCommand(command: string, params?: Record<string, any>): Promise<any>;
    registerCommand(name: string, description: string, handler: (params: Record<string, any>) => Promise<any>): Promise<void>;
    httpRequest(config: HttpRequestConfig): Promise<HttpResponse>;
    get(url: string, headers?: Record<string, string>): Promise<HttpResponse>;
    post(url: string, body: any, headers?: Record<string, string>): Promise<HttpResponse>;
    readClipboard(): Promise<string>;
    writeClipboard(text: string): Promise<void>;
    getStorageValue(key: string): Promise<any>;
    setStorageValue(key: string, value: any): Promise<void>;
    deleteStorageValue(key: string): Promise<void>;
    clearStorage(): Promise<void>;
    private send;
    private call;
    private handleMessage;
    private registerCallback;
    private getDefaultWebSocketUrl;
    private createError;
}
export declare function createPluginSDK(context: PluginApiContext): PluginSDK;
export declare abstract class Plugin {
    protected context: PluginApiContext;
    protected sdk: PluginSDK;
    constructor(context: PluginApiContext);
    initialize(): Promise<void>;
    shutdown(): Promise<void>;
    protected abstract onInit(): Promise<void>;
    protected abstract onShutdown(): Promise<void>;
    getSDK(): PluginSDK;
}
export default PluginSDK;
//# sourceMappingURL=PluginSDK.d.ts.map