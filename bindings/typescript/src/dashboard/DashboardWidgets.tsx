/**
 * CADDY Enterprise Dashboard Widgets Component v0.4.0
 *
 * Customizable widget system with drag-and-drop, resize, and configuration.
 * Supports various widget types and real-time data binding.
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import type { WidgetConfig, WidgetSize, DataSourceConfig, LoadingState, ErrorState } from './types';
import { useDashboard } from './DashboardLayout';

/**
 * Widget props
 */
export interface WidgetProps {
  /** Widget configuration */
  config: WidgetConfig;
  /** Widget content */
  children?: React.ReactNode;
  /** On configuration change */
  onConfigChange?: (config: WidgetConfig) => void;
  /** On remove */
  onRemove?: (widgetId: string) => void;
  /** On resize */
  onResize?: (widgetId: string, size: WidgetSize) => void;
  /** Enable editing mode */
  editable?: boolean;
  /** Custom class name */
  className?: string;
}

/**
 * Base widget component
 */
export const Widget: React.FC<WidgetProps> = ({
  config,
  children,
  onConfigChange,
  onRemove,
  onResize,
  editable = false,
  className = '',
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [showMenu, setShowMenu] = useState(false);
  const [loading, setLoading] = useState<LoadingState>({ isLoading: false });
  const [error, setError] = useState<ErrorState>({ hasError: false });
  const [data, setData] = useState<any>(null);
  const widgetRef = useRef<HTMLDivElement>(null);
  const { theme, accessibility, refreshData } = useDashboard();

  /**
   * Fetch widget data
   */
  const fetchData = useCallback(async () => {
    if (!config.dataSource) return;

    setLoading({ isLoading: true, message: 'Loading widget data...' });
    setError({ hasError: false });

    try {
      const result = await fetchDataSource(config.dataSource);
      setData(result);
    } catch (err: any) {
      setError({
        hasError: true,
        message: err.message || 'Failed to load widget data',
        retry: fetchData,
      });
    } finally {
      setLoading({ isLoading: false });
    }
  }, [config.dataSource]);

  /**
   * Initial data fetch
   */
  useEffect(() => {
    fetchData();
  }, [fetchData]);

  /**
   * Auto-refresh setup
   */
  useEffect(() => {
    if (!config.autoRefresh || !config.refreshInterval) return;

    const interval = setInterval(fetchData, config.refreshInterval * 1000);
    return () => clearInterval(interval);
  }, [config.autoRefresh, config.refreshInterval, fetchData]);

  /**
   * Handle refresh
   */
  const handleRefresh = useCallback(() => {
    fetchData();
  }, [fetchData]);

  /**
   * Handle expand/collapse
   */
  const handleToggleExpand = useCallback(() => {
    setIsExpanded((prev) => !prev);
  }, []);

  /**
   * Handle remove
   */
  const handleRemove = useCallback(() => {
    if (onRemove) {
      onRemove(config.id);
    }
  }, [config.id, onRemove]);

  /**
   * Handle resize
   */
  const handleResize = useCallback(
    (newSize: WidgetSize) => {
      if (onResize) {
        onResize(config.id, newSize);
      }
    },
    [config.id, onResize]
  );

  /**
   * Close menu on outside click
   */
  useEffect(() => {
    if (!showMenu) return;

    const handleClickOutside = (event: MouseEvent) => {
      if (widgetRef.current && !widgetRef.current.contains(event.target as Node)) {
        setShowMenu(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [showMenu]);

  return (
    <div
      ref={widgetRef}
      className={`widget ${className} ${config.size} ${isExpanded ? 'expanded' : ''}`}
      style={{
        ...styles.widget,
        ...(isExpanded && styles.widgetExpanded),
        ...config.style,
      }}
      role="region"
      aria-label={config.title}
      aria-describedby={`widget-desc-${config.id}`}
    >
      {/* Widget Header */}
      <div style={styles.header}>
        <div style={styles.headerLeft}>
          <h4 style={styles.title} id={`widget-title-${config.id}`}>
            {config.title}
          </h4>
          {config.description && (
            <p style={styles.description} id={`widget-desc-${config.id}`}>
              {config.description}
            </p>
          )}
        </div>

        <div style={styles.headerRight}>
          {/* Refresh button */}
          <button
            onClick={handleRefresh}
            style={styles.iconButton}
            aria-label="Refresh widget"
            title="Refresh"
            disabled={loading.isLoading}
          >
            ↻
          </button>

          {/* Expand button */}
          <button
            onClick={handleToggleExpand}
            style={styles.iconButton}
            aria-label={isExpanded ? 'Collapse widget' : 'Expand widget'}
            title={isExpanded ? 'Collapse' : 'Expand'}
          >
            {isExpanded ? '⊖' : '⊕'}
          </button>

          {/* Menu button */}
          {editable && (
            <div style={styles.menuContainer}>
              <button
                onClick={() => setShowMenu(!showMenu)}
                style={styles.iconButton}
                aria-label="Widget menu"
                aria-expanded={showMenu}
                aria-haspopup="true"
              >
                ⋮
              </button>

              {/* Dropdown menu */}
              {showMenu && (
                <div style={styles.menu} role="menu">
                  <button
                    onClick={() => handleResize('small')}
                    style={styles.menuItem}
                    role="menuitem"
                  >
                    Small
                  </button>
                  <button
                    onClick={() => handleResize('medium')}
                    style={styles.menuItem}
                    role="menuitem"
                  >
                    Medium
                  </button>
                  <button
                    onClick={() => handleResize('large')}
                    style={styles.menuItem}
                    role="menuitem"
                  >
                    Large
                  </button>
                  <div style={styles.menuDivider} role="separator" />
                  <button
                    onClick={handleRemove}
                    style={{ ...styles.menuItem, ...styles.menuItemDanger }}
                    role="menuitem"
                  >
                    Remove
                  </button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Widget Content */}
      <div style={styles.content}>
        {loading.isLoading && (
          <div style={styles.loading} role="status" aria-live="polite">
            <div style={styles.spinner} />
            <p>{loading.message || 'Loading...'}</p>
          </div>
        )}

        {error.hasError && (
          <div style={styles.error} role="alert">
            <p style={styles.errorMessage}>{error.message}</p>
            {error.retry && (
              <button onClick={error.retry} style={styles.retryButton}>
                Retry
              </button>
            )}
          </div>
        )}

        {!loading.isLoading && !error.hasError && (
          <div className="widget-body" aria-live="polite">
            {children || <WidgetContent config={config} data={data} />}
          </div>
        )}
      </div>
    </div>
  );
};

/**
 * Widget content renderer based on type
 */
interface WidgetContentProps {
  config: WidgetConfig;
  data: any;
}

const WidgetContent: React.FC<WidgetContentProps> = ({ config, data }) => {
  switch (config.type) {
    case 'metric':
      return <MetricWidgetContent data={data} options={config.options} />;
    case 'chart':
      return <ChartWidgetContent data={data} options={config.options} />;
    case 'table':
      return <TableWidgetContent data={data} options={config.options} />;
    case 'feed':
      return <FeedWidgetContent data={data} options={config.options} />;
    case 'custom':
      return <CustomWidgetContent data={data} options={config.options} />;
    default:
      return <div>Widget type: {config.type}</div>;
  }
};

/**
 * Metric widget content
 */
const MetricWidgetContent: React.FC<{ data: any; options?: any }> = ({ data, options }) => {
  if (!data) return null;

  return (
    <div style={styles.metricContent}>
      <div style={styles.metricValue}>{data.value || 0}</div>
      {data.label && <div style={styles.metricLabel}>{data.label}</div>}
      {data.change && (
        <div
          style={{
            ...styles.metricChange,
            color: data.change >= 0 ? 'var(--color-success, #4caf50)' : 'var(--color-error, #f44336)',
          }}
        >
          {data.change >= 0 ? '↑' : '↓'} {Math.abs(data.change)}%
        </div>
      )}
    </div>
  );
};

/**
 * Chart widget content
 */
const ChartWidgetContent: React.FC<{ data: any; options?: any }> = ({ data, options }) => {
  if (!data || !data.datasets) return null;

  return (
    <div style={styles.chartContent}>
      <canvas id={`chart-${Math.random()}`} width="100%" height="200" />
    </div>
  );
};

/**
 * Table widget content
 */
const TableWidgetContent: React.FC<{ data: any; options?: any }> = ({ data, options }) => {
  if (!data || !Array.isArray(data.rows)) return null;

  return (
    <div style={styles.tableContent}>
      <table style={styles.table}>
        <thead>
          <tr>
            {data.columns?.map((col: string, index: number) => (
              <th key={index} style={styles.tableHeader}>
                {col}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.rows.map((row: any[], rowIndex: number) => (
            <tr key={rowIndex}>
              {row.map((cell, cellIndex) => (
                <td key={cellIndex} style={styles.tableCell}>
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

/**
 * Feed widget content
 */
const FeedWidgetContent: React.FC<{ data: any; options?: any }> = ({ data, options }) => {
  if (!data || !Array.isArray(data.items)) return null;

  return (
    <div style={styles.feedContent}>
      {data.items.map((item: any, index: number) => (
        <div key={index} style={styles.feedItem}>
          <div style={styles.feedItemTitle}>{item.title}</div>
          <div style={styles.feedItemDescription}>{item.description}</div>
          <div style={styles.feedItemTime}>{item.timestamp}</div>
        </div>
      ))}
    </div>
  );
};

/**
 * Custom widget content
 */
const CustomWidgetContent: React.FC<{ data: any; options?: any }> = ({ data, options }) => {
  return (
    <div style={styles.customContent}>
      <pre>{JSON.stringify(data, null, 2)}</pre>
    </div>
  );
};

/**
 * Fetch data from data source
 */
async function fetchDataSource(config: DataSourceConfig): Promise<any> {
  switch (config.type) {
    case 'api':
      return fetchApiData(config);
    case 'websocket':
      return fetchWebSocketData(config);
    case 'static':
      return config.data || null;
    default:
      throw new Error(`Unsupported data source type: ${config.type}`);
  }
}

/**
 * Fetch data from API
 */
async function fetchApiData(config: DataSourceConfig): Promise<any> {
  if (!config.url) {
    throw new Error('API URL is required');
  }

  const response = await fetch(config.url, {
    method: config.method || 'GET',
    headers: {
      'Content-Type': 'application/json',
      ...config.headers,
    },
    body: config.body ? JSON.stringify(config.body) : undefined,
  });

  if (!response.ok) {
    throw new Error(`API request failed: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Fetch data from WebSocket
 */
async function fetchWebSocketData(config: DataSourceConfig): Promise<any> {
  // WebSocket implementation would go here
  // For now, return placeholder
  return { message: 'WebSocket data not implemented' };
}

/**
 * Widget grid container
 */
export interface WidgetGridProps {
  /** Widgets to display */
  widgets: WidgetConfig[];
  /** Grid columns */
  columns?: number;
  /** Enable editing */
  editable?: boolean;
  /** On widget change */
  onWidgetChange?: (widgets: WidgetConfig[]) => void;
  /** Custom class name */
  className?: string;
}

export const WidgetGrid: React.FC<WidgetGridProps> = ({
  widgets,
  columns = 3,
  editable = false,
  onWidgetChange,
  className = '',
}) => {
  const [widgetList, setWidgetList] = useState<WidgetConfig[]>(widgets);

  /**
   * Handle widget removal
   */
  const handleRemoveWidget = useCallback(
    (widgetId: string) => {
      const updated = widgetList.filter((w) => w.id !== widgetId);
      setWidgetList(updated);
      if (onWidgetChange) {
        onWidgetChange(updated);
      }
    },
    [widgetList, onWidgetChange]
  );

  /**
   * Handle widget resize
   */
  const handleResizeWidget = useCallback(
    (widgetId: string, size: WidgetSize) => {
      const updated = widgetList.map((w) =>
        w.id === widgetId ? { ...w, size } : w
      );
      setWidgetList(updated);
      if (onWidgetChange) {
        onWidgetChange(updated);
      }
    },
    [widgetList, onWidgetChange]
  );

  useEffect(() => {
    setWidgetList(widgets);
  }, [widgets]);

  return (
    <div
      className={`widget-grid ${className}`}
      style={{
        ...styles.grid,
        gridTemplateColumns: `repeat(${columns}, 1fr)`,
      }}
    >
      {widgetList.map((widget) => (
        <div
          key={widget.id}
          style={{
            gridColumn: `span ${widget.span.cols}`,
            gridRow: `span ${widget.span.rows}`,
          }}
        >
          <Widget
            config={widget}
            editable={editable}
            onRemove={handleRemoveWidget}
            onResize={handleResizeWidget}
          />
        </div>
      ))}
    </div>
  );
};

/**
 * Component styles
 */
const styles: Record<string, React.CSSProperties> = {
  widget: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    border: '1px solid var(--color-border, #e0e0e0)',
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    overflow: 'hidden',
    transition: 'box-shadow var(--animation-duration, 200ms)',
  },
  widgetExpanded: {
    position: 'fixed',
    top: '5%',
    left: '5%',
    right: '5%',
    bottom: '5%',
    zIndex: 1000,
    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.2)',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    padding: '16px 20px',
    borderBottom: '1px solid var(--color-divider, #e0e0e0)',
  },
  headerLeft: {
    flex: 1,
  },
  headerRight: {
    display: 'flex',
    gap: 8,
  },
  title: {
    margin: 0,
    fontSize: 16,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
  },
  description: {
    margin: '4px 0 0 0',
    fontSize: 12,
    color: 'var(--color-text-secondary, #666)',
  },
  iconButton: {
    width: 32,
    height: 32,
    border: 'none',
    backgroundColor: 'transparent',
    color: 'var(--color-text-secondary, #666)',
    cursor: 'pointer',
    borderRadius: 4,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: 18,
    transition: 'background-color var(--animation-duration, 200ms)',
  },
  menuContainer: {
    position: 'relative',
  },
  menu: {
    position: 'absolute',
    top: '100%',
    right: 0,
    marginTop: 4,
    backgroundColor: 'var(--color-surface, #fff)',
    border: '1px solid var(--color-border, #e0e0e0)',
    borderRadius: 4,
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.1)',
    minWidth: 120,
    zIndex: 1001,
  },
  menuItem: {
    width: '100%',
    padding: '8px 16px',
    border: 'none',
    backgroundColor: 'transparent',
    color: 'var(--color-text, #333)',
    cursor: 'pointer',
    fontSize: 14,
    textAlign: 'left',
    transition: 'background-color var(--animation-duration, 200ms)',
  },
  menuItemDanger: {
    color: 'var(--color-error, #f44336)',
  },
  menuDivider: {
    height: 1,
    backgroundColor: 'var(--color-divider, #e0e0e0)',
    margin: '4px 0',
  },
  content: {
    flex: 1,
    padding: 20,
    overflow: 'auto',
    position: 'relative',
  },
  loading: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    gap: 12,
  },
  spinner: {
    width: 40,
    height: 40,
    border: '4px solid var(--color-border, #e0e0e0)',
    borderTop: '4px solid var(--color-primary, #1976d2)',
    borderRadius: '50%',
    animation: 'spin 1s linear infinite',
  },
  error: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    gap: 12,
    padding: 20,
    textAlign: 'center',
  },
  errorMessage: {
    color: 'var(--color-error, #f44336)',
    margin: 0,
  },
  retryButton: {
    padding: '8px 16px',
    backgroundColor: 'var(--color-primary, #1976d2)',
    color: '#fff',
    border: 'none',
    borderRadius: 4,
    cursor: 'pointer',
    fontSize: 14,
    fontWeight: 500,
  },
  metricContent: {
    textAlign: 'center',
  },
  metricValue: {
    fontSize: 48,
    fontWeight: 700,
    color: 'var(--color-text, #333)',
    marginBottom: 8,
  },
  metricLabel: {
    fontSize: 16,
    color: 'var(--color-text-secondary, #666)',
    marginBottom: 8,
  },
  metricChange: {
    fontSize: 18,
    fontWeight: 600,
  },
  chartContent: {
    width: '100%',
    height: '100%',
  },
  tableContent: {
    overflowX: 'auto',
  },
  table: {
    width: '100%',
    borderCollapse: 'collapse',
    fontSize: 14,
  },
  tableHeader: {
    padding: '8px 12px',
    textAlign: 'left',
    fontWeight: 600,
    borderBottom: '2px solid var(--color-divider, #e0e0e0)',
    color: 'var(--color-text, #333)',
  },
  tableCell: {
    padding: '8px 12px',
    borderBottom: '1px solid var(--color-divider, #e0e0e0)',
    color: 'var(--color-text, #333)',
  },
  feedContent: {
    display: 'flex',
    flexDirection: 'column',
    gap: 12,
  },
  feedItem: {
    padding: 12,
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 4,
  },
  feedItemTitle: {
    fontWeight: 600,
    marginBottom: 4,
    color: 'var(--color-text, #333)',
  },
  feedItemDescription: {
    fontSize: 13,
    color: 'var(--color-text-secondary, #666)',
    marginBottom: 4,
  },
  feedItemTime: {
    fontSize: 11,
    color: 'var(--color-text-secondary, #999)',
  },
  customContent: {
    fontSize: 12,
    fontFamily: 'monospace',
    overflow: 'auto',
  },
  grid: {
    display: 'grid',
    gap: 16,
    width: '100%',
  },
};

export default Widget;
