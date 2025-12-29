export var PluginType;
(function (PluginType) {
    PluginType["Wasm"] = "Wasm";
    PluginType["Native"] = "Native";
})(PluginType || (PluginType = {}));
export var PluginState;
(function (PluginState) {
    PluginState["Loading"] = "Loading";
    PluginState["Loaded"] = "Loaded";
    PluginState["Initializing"] = "Initializing";
    PluginState["Ready"] = "Ready";
    PluginState["Starting"] = "Starting";
    PluginState["Running"] = "Running";
    PluginState["Suspended"] = "Suspended";
    PluginState["Stopping"] = "Stopping";
    PluginState["Stopped"] = "Stopped";
    PluginState["Error"] = "Error";
    PluginState["Unloading"] = "Unloading";
    PluginState["Unloaded"] = "Unloaded";
})(PluginState || (PluginState = {}));
export var Permission;
(function (Permission) {
    Permission["GeometryRead"] = "geometry:read";
    Permission["GeometryWrite"] = "geometry:write";
    Permission["GeometryDelete"] = "geometry:delete";
    Permission["RenderingRead"] = "rendering:read";
    Permission["RenderingWrite"] = "rendering:write";
    Permission["RenderingShaderAccess"] = "rendering:shader";
    Permission["UIRead"] = "ui:read";
    Permission["UIWrite"] = "ui:write";
    Permission["UIMenuAccess"] = "ui:menu";
    Permission["UIToolbarAccess"] = "ui:toolbar";
    Permission["UIDialogAccess"] = "ui:dialog";
    Permission["FileRead"] = "file:read";
    Permission["FileWrite"] = "file:write";
    Permission["FileDelete"] = "file:delete";
    Permission["FileExecute"] = "file:execute";
    Permission["CommandExecute"] = "command:execute";
    Permission["CommandRegister"] = "command:register";
    Permission["LayerRead"] = "layer:read";
    Permission["LayerWrite"] = "layer:write";
    Permission["LayerDelete"] = "layer:delete";
    Permission["NetworkHTTP"] = "network:http";
    Permission["NetworkWebSocket"] = "network:websocket";
    Permission["NetworkUnrestricted"] = "network:unrestricted";
    Permission["SystemClipboard"] = "system:clipboard";
    Permission["SystemNotifications"] = "system:notifications";
    Permission["DatabaseRead"] = "database:read";
    Permission["DatabaseWrite"] = "database:write";
    Permission["EnterpriseAccess"] = "enterprise:access";
})(Permission || (Permission = {}));
export var SortBy;
(function (SortBy) {
    SortBy["Relevance"] = "Relevance";
    SortBy["Downloads"] = "Downloads";
    SortBy["Rating"] = "Rating";
    SortBy["Updated"] = "Updated";
    SortBy["Name"] = "Name";
})(SortBy || (SortBy = {}));
export var PluginEventType;
(function (PluginEventType) {
    PluginEventType["Loaded"] = "loaded";
    PluginEventType["Unloaded"] = "unloaded";
    PluginEventType["Reloaded"] = "reloaded";
    PluginEventType["StateChanged"] = "stateChanged";
    PluginEventType["Error"] = "error";
    PluginEventType["ConfigChanged"] = "configChanged";
})(PluginEventType || (PluginEventType = {}));
export var NotificationLevel;
(function (NotificationLevel) {
    NotificationLevel["Info"] = "Info";
    NotificationLevel["Warning"] = "Warning";
    NotificationLevel["Error"] = "Error";
    NotificationLevel["Success"] = "Success";
})(NotificationLevel || (NotificationLevel = {}));
export var DialogType;
(function (DialogType) {
    DialogType["Info"] = "Info";
    DialogType["Warning"] = "Warning";
    DialogType["Error"] = "Error";
    DialogType["Question"] = "Question";
})(DialogType || (DialogType = {}));
//# sourceMappingURL=types.js.map