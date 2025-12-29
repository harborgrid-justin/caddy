export interface PluginManifest {
    id: string;
    name: string;
    version: string;
    description: string;
    author: string;
    apiVersion: string;
    entryPoint: string;
    pluginType: PluginType;
    permissions: string[];
    dependencies?: PluginDependency[];
    capabilities?: string[];
    icon?: string;
    website?: string;
    repository?: string;
    license?: string;
    minCaddyVersion?: string;
    maxCaddyVersion?: string;
}
export declare enum PluginType {
    Wasm = "Wasm",
    Native = "Native"
}
export interface PluginDependency {
    id: string;
    versionRequirement: string;
    optional?: boolean;
}
export declare enum PluginState {
    Loading = "Loading",
    Loaded = "Loaded",
    Initializing = "Initializing",
    Ready = "Ready",
    Starting = "Starting",
    Running = "Running",
    Suspended = "Suspended",
    Stopping = "Stopping",
    Stopped = "Stopped",
    Error = "Error",
    Unloading = "Unloading",
    Unloaded = "Unloaded"
}
export declare enum Permission {
    GeometryRead = "geometry:read",
    GeometryWrite = "geometry:write",
    GeometryDelete = "geometry:delete",
    RenderingRead = "rendering:read",
    RenderingWrite = "rendering:write",
    RenderingShaderAccess = "rendering:shader",
    UIRead = "ui:read",
    UIWrite = "ui:write",
    UIMenuAccess = "ui:menu",
    UIToolbarAccess = "ui:toolbar",
    UIDialogAccess = "ui:dialog",
    FileRead = "file:read",
    FileWrite = "file:write",
    FileDelete = "file:delete",
    FileExecute = "file:execute",
    CommandExecute = "command:execute",
    CommandRegister = "command:register",
    LayerRead = "layer:read",
    LayerWrite = "layer:write",
    LayerDelete = "layer:delete",
    NetworkHTTP = "network:http",
    NetworkWebSocket = "network:websocket",
    NetworkUnrestricted = "network:unrestricted",
    SystemClipboard = "system:clipboard",
    SystemNotifications = "system:notifications",
    DatabaseRead = "database:read",
    DatabaseWrite = "database:write",
    EnterpriseAccess = "enterprise:access"
}
export interface PluginInfo {
    manifest: PluginManifest;
    state: PluginState;
    loadedAt: string;
    enabled: boolean;
    source: InstallationSource;
    resourceUsage?: ResourceUsageStats;
}
export type InstallationSource = {
    type: 'marketplace';
    url: string;
} | {
    type: 'local';
} | {
    type: 'git';
    repo: string;
    commit: string;
} | {
    type: 'url';
    url: string;
} | {
    type: 'builtin';
};
export interface ResourceUsageStats {
    memoryUsedBytes: number;
    memoryLimitBytes: number;
    executionTimeMs: number;
    executionLimitMs: number;
    fileOpsCount: number;
    networkRequestsCount: number;
}
export interface MarketplacePlugin {
    id: string;
    name: string;
    description: string;
    version: string;
    author: MarketplaceAuthor;
    iconUrl?: string;
    downloads: number;
    rating: number;
    ratingCount: number;
    categories: string[];
    updatedAt: string;
    sizeBytes: number;
    downloadUrl: string;
    manifestUrl: string;
    verified: boolean;
    license: string;
    minCaddyVersion: string;
}
export interface MarketplaceAuthor {
    id: string;
    name: string;
    email?: string;
    website?: string;
    verified: boolean;
}
export interface SearchFilters {
    query?: string;
    category?: string;
    minRating?: number;
    verifiedOnly?: boolean;
    sortBy?: SortBy;
    page?: number;
    perPage?: number;
}
export declare enum SortBy {
    Relevance = "Relevance",
    Downloads = "Downloads",
    Rating = "Rating",
    Updated = "Updated",
    Name = "Name"
}
export interface SearchResults {
    plugins: MarketplacePlugin[];
    totalCount: number;
    page: number;
    perPage: number;
    totalPages: number;
}
export interface PluginUpdate {
    pluginId: string;
    currentVersion: string;
    latestVersion: string;
    changelogUrl?: string;
}
export interface Category {
    id: string;
    name: string;
    description: string;
    icon?: string;
    pluginCount: number;
}
export interface PluginConfig {
    [key: string]: any;
}
export interface PluginSettings {
    pluginId: string;
    enabled: boolean;
    config: PluginConfig;
    autoStart: boolean;
    resourceLimits?: ResourceLimits;
}
export interface ResourceLimits {
    maxMemoryBytes: number;
    maxExecutionTimeMs: number;
    maxFileSizeBytes: number;
    maxFileOpsPerSecond: number;
    maxNetworkRequestsPerSecond: number;
    maxCpuPercent: number;
}
export declare enum PluginEventType {
    Loaded = "loaded",
    Unloaded = "unloaded",
    Reloaded = "reloaded",
    StateChanged = "stateChanged",
    Error = "error",
    ConfigChanged = "configChanged"
}
export interface PluginEvent {
    type: PluginEventType;
    pluginId: string;
    timestamp: string;
    data?: any;
}
export interface PluginApiContext {
    pluginId: string;
    apiVersion: string;
    permissions: string[];
    config: PluginConfig;
}
export interface PluginSystemStats {
    loadedPlugins: number;
    registeredPlugins: number;
    enabledPlugins: number;
    runningPlugins: number;
    totalDownloads: number;
}
export interface PluginError {
    code: string;
    message: string;
    pluginId?: string;
    details?: any;
}
export declare enum NotificationLevel {
    Info = "Info",
    Warning = "Warning",
    Error = "Error",
    Success = "Success"
}
export declare enum DialogType {
    Info = "Info",
    Warning = "Warning",
    Error = "Error",
    Question = "Question"
}
export interface DialogConfig {
    title: string;
    message: string;
    dialogType: DialogType;
    buttons: string[];
}
export interface HttpRequestConfig {
    url: string;
    method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
    headers?: Record<string, string>;
    body?: any;
    timeoutMs?: number;
}
export interface HttpResponse {
    status: number;
    headers: Record<string, string>;
    body: any;
}
//# sourceMappingURL=types.d.ts.map