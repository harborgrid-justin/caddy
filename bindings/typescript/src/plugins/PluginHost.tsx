/**
 * Plugin Host Component
 *
 * Provides a sandboxed environment for rendering plugin UI components
 * with proper isolation and communication channel.
 */

import React, { useRef, useEffect, useState } from 'react';
import { PluginSDK } from './PluginSDK';
import { PluginApiContext, PluginEvent, PluginEventType } from './types';

export interface PluginHostProps {
  /** Plugin ID to host */
  pluginId: string;

  /** Plugin API context */
  context: PluginApiContext;

  /** Plugin UI URL (iframe source) */
  uiUrl?: string;

  /** CSS class name */
  className?: string;

  /** Width of plugin container */
  width?: string | number;

  /** Height of plugin container */
  height?: string | number;

  /** Whether to show loading state */
  showLoading?: boolean;

  /** Callback when plugin is ready */
  onReady?: () => void;

  /** Callback when plugin errors */
  onError?: (error: Error) => void;

  /** Callback for plugin events */
  onEvent?: (event: PluginEvent) => void;
}

export const PluginHost: React.FC<PluginHostProps> = ({
  pluginId,
  context,
  uiUrl,
  className = '',
  width = '100%',
  height = '100%',
  showLoading = true,
  onReady,
  onError,
  onEvent,
}) => {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const sdkRef = useRef<PluginSDK | null>(null);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

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

      // Create SDK instance for this plugin
      const sdk = new PluginSDK(context);
      sdkRef.current = sdk;

      // Set up event listeners
      sdk.on('connected', () => {
        setLoading(false);
        onReady?.();
      });

      sdk.on('error', (err) => {
        setError(err);
        onError?.(err);
      });

      sdk.on('*', (eventType: string, data: any) => {
        onEvent?.({
          type: eventType as PluginEventType,
          pluginId,
          timestamp: new Date().toISOString(),
          data,
        });
      });

      // Initialize SDK connection
      await sdk.initialize();
    } catch (err: any) {
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
    // Inject plugin SDK into iframe
    if (iframeRef.current && iframeRef.current.contentWindow && sdkRef.current) {
      try {
        (iframeRef.current.contentWindow as any).__caddyPluginSDK = sdkRef.current;
      } catch (err) {
        console.error('Failed to inject SDK into plugin iframe:', err);
      }
    }
  };

  const containerStyle: React.CSSProperties = {
    width: typeof width === 'number' ? `${width}px` : width,
    height: typeof height === 'number' ? `${height}px` : height,
    position: 'relative',
    overflow: 'hidden',
  };

  const iframeStyle: React.CSSProperties = {
    width: '100%',
    height: '100%',
    border: 'none',
    display: loading ? 'none' : 'block',
  };

  return (
    <div className={`plugin-host ${className}`} style={containerStyle}>
      {showLoading && loading && (
        <div className="plugin-host-loading">
          <div className="loading-spinner">Loading plugin...</div>
        </div>
      )}

      {error && (
        <div className="plugin-host-error">
          <div className="error-icon">⚠</div>
          <div className="error-message">
            <h3>Plugin Error</h3>
            <p>{error.message}</p>
          </div>
        </div>
      )}

      {uiUrl && (
        <iframe
          ref={iframeRef}
          src={uiUrl}
          style={iframeStyle}
          sandbox="allow-scripts allow-same-origin"
          onLoad={handleIframeLoad}
          title={`Plugin: ${pluginId}`}
        />
      )}

      {!uiUrl && !loading && !error && (
        <PluginHeadlessHost sdk={sdkRef.current} pluginId={pluginId} />
      )}
    </div>
  );
};

interface PluginHeadlessHostProps {
  sdk: PluginSDK | null;
  pluginId: string;
}

/**
 * Headless plugin host (for plugins without UI)
 */
const PluginHeadlessHost: React.FC<PluginHeadlessHostProps> = ({
  sdk,
  pluginId,
}) => {
  const [status, setStatus] = useState<'idle' | 'running' | 'error'>('idle');
  const [messages, setMessages] = useState<string[]>([]);

  useEffect(() => {
    if (!sdk) return;

    setStatus('running');

    // Listen for plugin messages
    const handleMessage = (data: any) => {
      setMessages((prev) => [...prev, JSON.stringify(data)]);
    };

    sdk.on('message', handleMessage);

    return () => {
      sdk.off('message', handleMessage);
    };
  }, [sdk]);

  return (
    <div className="plugin-headless-host">
      <div className="headless-info">
        <h3>Headless Plugin</h3>
        <p>Plugin ID: {pluginId}</p>
        <p>
          Status:{' '}
          <span className={`status-${status}`}>
            {status === 'running' ? '🟢 Running' : '⚪ Idle'}
          </span>
        </p>
      </div>

      {messages.length > 0 && (
        <div className="headless-messages">
          <h4>Messages ({messages.length})</h4>
          <div className="message-list">
            {messages.slice(-10).map((msg, idx) => (
              <div key={idx} className="message-item">
                <pre>{msg}</pre>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

/**
 * Plugin sandbox container with multiple plugins
 */
export interface PluginContainerProps {
  plugins: Array<{
    pluginId: string;
    context: PluginApiContext;
    uiUrl?: string;
  }>;
  layout?: 'grid' | 'tabs' | 'sidebar';
  className?: string;
}

export const PluginContainer: React.FC<PluginContainerProps> = ({
  plugins,
  layout = 'grid',
  className = '',
}) => {
  const [activeTab, setActiveTab] = useState(0);

  if (layout === 'tabs') {
    return (
      <div className={`plugin-container plugin-container-tabs ${className}`}>
        <div className="plugin-tabs">
          {plugins.map((plugin, index) => (
            <button
              key={plugin.pluginId}
              className={`tab-button ${index === activeTab ? 'active' : ''}`}
              onClick={() => setActiveTab(index)}
            >
              {plugin.pluginId}
            </button>
          ))}
        </div>
        <div className="plugin-tab-content">
          {plugins.map((plugin, index) => (
            <div
              key={plugin.pluginId}
              className={`tab-pane ${index === activeTab ? 'active' : ''}`}
              style={{ display: index === activeTab ? 'block' : 'none' }}
            >
              <PluginHost
                pluginId={plugin.pluginId}
                context={plugin.context}
                uiUrl={plugin.uiUrl}
              />
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (layout === 'sidebar') {
    return (
      <div className={`plugin-container plugin-container-sidebar ${className}`}>
        <div className="plugin-sidebar">
          {plugins.map((plugin, index) => (
            <button
              key={plugin.pluginId}
              className={`sidebar-item ${index === activeTab ? 'active' : ''}`}
              onClick={() => setActiveTab(index)}
            >
              {plugin.pluginId}
            </button>
          ))}
        </div>
        <div className="plugin-main">
          {plugins[activeTab] && (
            <PluginHost
              pluginId={plugins[activeTab].pluginId}
              context={plugins[activeTab].context}
              uiUrl={plugins[activeTab].uiUrl}
            />
          )}
        </div>
      </div>
    );
  }

  // Grid layout (default)
  return (
    <div className={`plugin-container plugin-container-grid ${className}`}>
      {plugins.map((plugin) => (
        <div key={plugin.pluginId} className="plugin-grid-item">
          <PluginHost
            pluginId={plugin.pluginId}
            context={plugin.context}
            uiUrl={plugin.uiUrl}
          />
        </div>
      ))}
    </div>
  );
};

export default PluginHost;
