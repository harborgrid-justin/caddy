import React, { useRef, useEffect, useState } from 'react';
import { PluginSDK } from './PluginSDK';
export const PluginHost = ({ pluginId, context, uiUrl, className = '', width = '100%', height = '100%', showLoading = true, onReady, onError, onEvent, }) => {
    const iframeRef = useRef(null);
    const sdkRef = useRef(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    useEffect(() => {
        initializePlugin();
        return () => {
            cleanup();
        };
    }, [pluginId]);
    const initializePlugin = async () => {
        try {
            setLoading(true);
            setError(null);
            const sdk = new PluginSDK(context);
            sdkRef.current = sdk;
            sdk.on('connected', () => {
                setLoading(false);
                onReady?.();
            });
            sdk.on('error', (err) => {
                setError(err);
                onError?.(err);
            });
            sdk.on('*', (eventType, data) => {
                onEvent?.({
                    type: eventType,
                    pluginId,
                    timestamp: new Date().toISOString(),
                    data,
                });
            });
            await sdk.initialize();
        }
        catch (err) {
            setError(err);
            onError?.(err);
            setLoading(false);
        }
    };
    const cleanup = () => {
        if (sdkRef.current) {
            sdkRef.current.disconnect();
            sdkRef.current = null;
        }
    };
    const handleIframeLoad = () => {
        if (iframeRef.current && iframeRef.current.contentWindow && sdkRef.current) {
            try {
                iframeRef.current.contentWindow.__caddyPluginSDK = sdkRef.current;
            }
            catch (err) {
                console.error('Failed to inject SDK into plugin iframe:', err);
            }
        }
    };
    const containerStyle = {
        width: typeof width === 'number' ? `${width}px` : width,
        height: typeof height === 'number' ? `${height}px` : height,
        position: 'relative',
        overflow: 'hidden',
    };
    const iframeStyle = {
        width: '100%',
        height: '100%',
        border: 'none',
        display: loading ? 'none' : 'block',
    };
    return (React.createElement("div", { className: `plugin-host ${className}`, style: containerStyle },
        showLoading && loading && (React.createElement("div", { className: "plugin-host-loading" },
            React.createElement("div", { className: "loading-spinner" }, "Loading plugin..."))),
        error && (React.createElement("div", { className: "plugin-host-error" },
            React.createElement("div", { className: "error-icon" }, "\u26A0"),
            React.createElement("div", { className: "error-message" },
                React.createElement("h3", null, "Plugin Error"),
                React.createElement("p", null, error.message)))),
        uiUrl && (React.createElement("iframe", { ref: iframeRef, src: uiUrl, style: iframeStyle, sandbox: "allow-scripts allow-same-origin", onLoad: handleIframeLoad, title: `Plugin: ${pluginId}` })),
        !uiUrl && !loading && !error && (React.createElement(PluginHeadlessHost, { sdk: sdkRef.current, pluginId: pluginId }))));
};
const PluginHeadlessHost = ({ sdk, pluginId, }) => {
    const [status, setStatus] = useState('idle');
    const [messages, setMessages] = useState([]);
    useEffect(() => {
        if (!sdk)
            return;
        setStatus('running');
        const handleMessage = (data) => {
            setMessages((prev) => [...prev, JSON.stringify(data)]);
        };
        sdk.on('message', handleMessage);
        return () => {
            sdk.off('message', handleMessage);
        };
    }, [sdk]);
    return (React.createElement("div", { className: "plugin-headless-host" },
        React.createElement("div", { className: "headless-info" },
            React.createElement("h3", null, "Headless Plugin"),
            React.createElement("p", null,
                "Plugin ID: ",
                pluginId),
            React.createElement("p", null,
                "Status:",
                ' ',
                React.createElement("span", { className: `status-${status}` }, status === 'running' ? '🟢 Running' : '⚪ Idle'))),
        messages.length > 0 && (React.createElement("div", { className: "headless-messages" },
            React.createElement("h4", null,
                "Messages (",
                messages.length,
                ")"),
            React.createElement("div", { className: "message-list" }, messages.slice(-10).map((msg, idx) => (React.createElement("div", { key: idx, className: "message-item" },
                React.createElement("pre", null, msg)))))))));
};
export const PluginContainer = ({ plugins, layout = 'grid', className = '', }) => {
    const [activeTab, setActiveTab] = useState(0);
    if (layout === 'tabs') {
        return (React.createElement("div", { className: `plugin-container plugin-container-tabs ${className}` },
            React.createElement("div", { className: "plugin-tabs" }, plugins.map((plugin, index) => (React.createElement("button", { key: plugin.pluginId, className: `tab-button ${index === activeTab ? 'active' : ''}`, onClick: () => setActiveTab(index) }, plugin.pluginId)))),
            React.createElement("div", { className: "plugin-tab-content" }, plugins.map((plugin, index) => (React.createElement("div", { key: plugin.pluginId, className: `tab-pane ${index === activeTab ? 'active' : ''}`, style: { display: index === activeTab ? 'block' : 'none' } },
                React.createElement(PluginHost, { pluginId: plugin.pluginId, context: plugin.context, uiUrl: plugin.uiUrl })))))));
    }
    if (layout === 'sidebar') {
        return (React.createElement("div", { className: `plugin-container plugin-container-sidebar ${className}` },
            React.createElement("div", { className: "plugin-sidebar" }, plugins.map((plugin, index) => (React.createElement("button", { key: plugin.pluginId, className: `sidebar-item ${index === activeTab ? 'active' : ''}`, onClick: () => setActiveTab(index) }, plugin.pluginId)))),
            React.createElement("div", { className: "plugin-main" }, plugins[activeTab] && (React.createElement(PluginHost, { pluginId: plugins[activeTab].pluginId, context: plugins[activeTab].context, uiUrl: plugins[activeTab].uiUrl })))));
    }
    return (React.createElement("div", { className: `plugin-container plugin-container-grid ${className}` }, plugins.map((plugin) => (React.createElement("div", { key: plugin.pluginId, className: "plugin-grid-item" },
        React.createElement(PluginHost, { pluginId: plugin.pluginId, context: plugin.context, uiUrl: plugin.uiUrl }))))));
};
export default PluginHost;
//# sourceMappingURL=PluginHost.js.map