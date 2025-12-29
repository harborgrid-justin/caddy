/**
 * CADDY v0.4.0 - Log Viewer
 * Real-time log streaming and analysis
 * @module monitoring/LogViewer
 */

import React, { useEffect, useState, useRef, useCallback } from 'react';
import { LogEntry, LogFilter, WebSocketMessage } from './types';

interface LogViewerProps {
  service?: string;
  autoScroll?: boolean;
  maxLines?: number;
  className?: string;
}

export const LogViewer: React.FC<LogViewerProps> = ({
  service,
  autoScroll = true,
  maxLines = 1000,
  className = ''
}) => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filter, setFilter] = useState<LogFilter>({
    services: service ? [service] : undefined,
    limit: maxLines
  });
  const [isPaused, setIsPaused] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedLog, setSelectedLog] = useState<LogEntry | null>(null);
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const logsContainerRef = useRef<HTMLDivElement>(null);

  // WebSocket connection for real-time logs
  useEffect(() => {
    if (isPaused) return;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/monitoring/logs/stream`;
    const socket = new WebSocket(wsUrl);

    socket.onopen = () => {
      console.log('[LogViewer] WebSocket connected');
      setIsConnected(true);
      socket.send(JSON.stringify({
        type: 'subscribe',
        filter
      }));
    };

    socket.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        if (message.type === 'log') {
          handleNewLog(message.data);
        }
      } catch (error) {
        console.error('[LogViewer] Failed to parse WebSocket message:', error);
      }
    };

    socket.onerror = (error) => {
      console.error('[LogViewer] WebSocket error:', error);
      setIsConnected(false);
    };

    socket.onclose = () => {
      console.log('[LogViewer] WebSocket disconnected');
      setIsConnected(false);
      if (!isPaused) {
        setTimeout(() => {
          // Reconnect
        }, 5000);
      }
    };

    setWs(socket);

    return () => {
      socket.close();
    };
  }, [isPaused, filter]);

  // Auto-scroll to bottom
  useEffect(() => {
    if (autoScroll && !isPaused && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll, isPaused]);

  // Fetch historical logs
  useEffect(() => {
    fetchHistoricalLogs();
  }, [filter]);

  const fetchHistoricalLogs = async () => {
    try {
      const params = new URLSearchParams();
      if (filter.services) params.set('services', filter.services.join(','));
      if (filter.levels) params.set('levels', filter.levels.join(','));
      if (filter.search) params.set('search', filter.search);
      if (filter.startTime) params.set('startTime', filter.startTime.toISOString());
      if (filter.endTime) params.set('endTime', filter.endTime.toISOString());
      if (filter.traceId) params.set('traceId', filter.traceId);
      if (filter.limit) params.set('limit', filter.limit.toString());

      const response = await fetch(`/api/monitoring/logs?${params}`);
      if (!response.ok) throw new Error('Failed to fetch logs');

      const data = await response.json();
      setLogs(data);
    } catch (error) {
      console.error('[LogViewer] Failed to fetch logs:', error);
    }
  };

  const handleNewLog = useCallback((log: LogEntry) => {
    setLogs(prev => {
      const updated = [...prev, log];
      // Keep only the last maxLines
      if (updated.length > maxLines) {
        return updated.slice(-maxLines);
      }
      return updated;
    });
  }, [maxLines]);

  const clearLogs = () => {
    setLogs([]);
  };

  const exportLogs = () => {
    const logsText = filteredLogs.map(log =>
      `[${new Date(log.timestamp).toISOString()}] [${log.level.toUpperCase()}] [${log.service}] ${log.message}`
    ).join('\n');

    const blob = new Blob([logsText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `logs-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const getLevelColor = (level: LogEntry['level']): string => {
    switch (level) {
      case 'debug':
        return '#6b7280';
      case 'info':
        return '#3b82f6';
      case 'warn':
        return '#f59e0b';
      case 'error':
        return '#ef4444';
      case 'fatal':
        return '#dc2626';
      default:
        return '#6b7280';
    }
  };

  const getLevelBgColor = (level: LogEntry['level']): string => {
    return `${getLevelColor(level)}20`;
  };

  const filteredLogs = logs.filter(log => {
    if (searchTerm && !log.message.toLowerCase().includes(searchTerm.toLowerCase())) {
      return false;
    }
    if (filter.levels && !filter.levels.includes(log.level)) {
      return false;
    }
    if (filter.services && !filter.services.includes(log.service)) {
      return false;
    }
    return true;
  });

  const logCounts = {
    debug: logs.filter(l => l.level === 'debug').length,
    info: logs.filter(l => l.level === 'info').length,
    warn: logs.filter(l => l.level === 'warn').length,
    error: logs.filter(l => l.level === 'error').length,
    fatal: logs.filter(l => l.level === 'fatal').length
  };

  return (
    <div className={`log-viewer ${className}`} style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <div style={styles.headerLeft}>
          <h2 style={styles.title}>Log Viewer</h2>
          <div style={styles.connectionStatus}>
            <div
              style={{
                ...styles.statusDot,
                backgroundColor: isConnected ? '#10b981' : '#ef4444'
              }}
            />
            <span style={styles.statusText}>
              {isConnected ? 'Live' : 'Disconnected'}
            </span>
            <span style={styles.logCount}>{filteredLogs.length} logs</span>
          </div>
        </div>

        <div style={styles.headerRight}>
          <button
            style={{
              ...styles.button,
              ...(isPaused ? styles.buttonPaused : {})
            }}
            onClick={() => setIsPaused(!isPaused)}
          >
            {isPaused ? 'Resume' : 'Pause'}
          </button>
          <button style={styles.button} onClick={clearLogs}>
            Clear
          </button>
          <button style={styles.button} onClick={exportLogs}>
            Export
          </button>
        </div>
      </div>

      {/* Filters */}
      <div style={styles.filters}>
        <input
          type="text"
          placeholder="Search logs..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          style={styles.searchInput}
        />

        <div style={styles.levelFilters}>
          {(['debug', 'info', 'warn', 'error', 'fatal'] as const).map((level) => (
            <button
              key={level}
              style={{
                ...styles.levelButton,
                backgroundColor: getLevelBgColor(level),
                color: getLevelColor(level),
                ...(filter.levels?.includes(level) ? {
                  borderColor: getLevelColor(level),
                  borderWidth: '2px'
                } : {})
              }}
              onClick={() => {
                const currentLevels = filter.levels || [];
                setFilter({
                  ...filter,
                  levels: currentLevels.includes(level)
                    ? currentLevels.filter(l => l !== level)
                    : [...currentLevels, level]
                });
              }}
            >
              {level.toUpperCase()}
              <span style={styles.levelCount}>{logCounts[level]}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Log List */}
      <div style={styles.logsContainer} ref={logsContainerRef}>
        {filteredLogs.length === 0 ? (
          <div style={styles.emptyState}>
            <p>No logs to display</p>
          </div>
        ) : (
          <div style={styles.logsList}>
            {filteredLogs.map((log, index) => (
              <div
                key={log.id || index}
                style={styles.logEntry}
                onClick={() => setSelectedLog(log)}
              >
                <div style={styles.logTimestamp}>
                  {new Date(log.timestamp).toLocaleTimeString([], {
                    hour: '2-digit',
                    minute: '2-digit',
                    second: '2-digit',
                  })}
                </div>
                <div
                  style={{
                    ...styles.logLevel,
                    backgroundColor: getLevelBgColor(log.level),
                    color: getLevelColor(log.level)
                  }}
                >
                  {log.level.toUpperCase()}
                </div>
                <div style={styles.logService}>{log.service}</div>
                <div style={styles.logMessage}>{log.message}</div>
                {log.traceId && (
                  <div style={styles.logTraceId} title={`Trace: ${log.traceId}`}>
                    🔗
                  </div>
                )}
              </div>
            ))}
            <div ref={logsEndRef} />
          </div>
        )}
      </div>

      {/* Log Details Modal */}
      {selectedLog && (
        <div style={styles.modal} onClick={() => setSelectedLog(null)}>
          <div style={styles.modalContent} onClick={(e) => e.stopPropagation()}>
            <div style={styles.modalHeader}>
              <h3>Log Details</h3>
              <button
                style={styles.modalClose}
                onClick={() => setSelectedLog(null)}
              >
                ×
              </button>
            </div>

            <div style={styles.modalBody}>
              <div style={styles.detailRow}>
                <strong>Timestamp:</strong>
                <span>{new Date(selectedLog.timestamp).toISOString()}</span>
              </div>
              <div style={styles.detailRow}>
                <strong>Level:</strong>
                <span
                  style={{
                    ...styles.logLevel,
                    backgroundColor: getLevelBgColor(selectedLog.level),
                    color: getLevelColor(selectedLog.level)
                  }}
                >
                  {selectedLog.level.toUpperCase()}
                </span>
              </div>
              <div style={styles.detailRow}>
                <strong>Service:</strong>
                <span>{selectedLog.service}</span>
              </div>
              <div style={styles.detailRow}>
                <strong>Message:</strong>
                <pre style={styles.messageContent}>{selectedLog.message}</pre>
              </div>
              {selectedLog.traceId && (
                <div style={styles.detailRow}>
                  <strong>Trace ID:</strong>
                  <code style={styles.code}>{selectedLog.traceId}</code>
                </div>
              )}
              {selectedLog.spanId && (
                <div style={styles.detailRow}>
                  <strong>Span ID:</strong>
                  <code style={styles.code}>{selectedLog.spanId}</code>
                </div>
              )}
              {selectedLog.context && Object.keys(selectedLog.context).length > 0 && (
                <div style={styles.detailRow}>
                  <strong>Context:</strong>
                  <pre style={styles.jsonContent}>
                    {JSON.stringify(selectedLog.context, null, 2)}
                  </pre>
                </div>
              )}
              {selectedLog.stack && (
                <div style={styles.detailRow}>
                  <strong>Stack Trace:</strong>
                  <pre style={styles.stackTrace}>{selectedLog.stack}</pre>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: '24px',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    height: '100%',
    display: 'flex',
    flexDirection: 'column'
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px',
    flexWrap: 'wrap',
    gap: '12px'
  },
  headerLeft: {
    flex: 1
  },
  headerRight: {
    display: 'flex',
    gap: '8px'
  },
  title: {
    fontSize: '24px',
    fontWeight: 700,
    color: '#111827',
    margin: 0,
    marginBottom: '8px'
  },
  connectionStatus: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px'
  },
  statusDot: {
    width: '8px',
    height: '8px',
    borderRadius: '50%',
    animation: 'pulse 2s infinite'
  },
  statusText: {
    fontSize: '13px',
    color: '#6b7280',
    fontWeight: 500
  },
  logCount: {
    fontSize: '12px',
    color: '#9ca3af',
    marginLeft: '4px'
  },
  button: {
    padding: '6px 12px',
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '6px',
    fontSize: '13px',
    fontWeight: 500,
    color: '#374151',
    cursor: 'pointer',
    transition: 'all 0.2s'
  },
  buttonPaused: {
    backgroundColor: '#fef3c7',
    borderColor: '#f59e0b',
    color: '#92400e'
  },
  filters: {
    display: 'flex',
    gap: '12px',
    marginBottom: '16px',
    flexWrap: 'wrap'
  },
  searchInput: {
    flex: 1,
    minWidth: '200px',
    padding: '8px 12px',
    border: '1px solid #e5e7eb',
    borderRadius: '6px',
    fontSize: '14px',
    outline: 'none'
  },
  levelFilters: {
    display: 'flex',
    gap: '6px'
  },
  levelButton: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    padding: '6px 12px',
    border: '1px solid transparent',
    borderRadius: '6px',
    fontSize: '11px',
    fontWeight: 600,
    cursor: 'pointer',
    transition: 'all 0.2s',
    textTransform: 'uppercase'
  },
  levelCount: {
    fontSize: '10px',
    opacity: 0.8
  },
  logsContainer: {
    flex: 1,
    backgroundColor: '#1f2937',
    borderRadius: '8px',
    overflow: 'auto',
    border: '1px solid #374151'
  },
  emptyState: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    color: '#9ca3af',
    fontSize: '14px'
  },
  logsList: {
    fontFamily: 'Monaco, "Courier New", monospace',
    fontSize: '12px'
  },
  logEntry: {
    display: 'grid',
    gridTemplateColumns: '100px 60px 120px 1fr 30px',
    gap: '12px',
    padding: '6px 12px',
    borderBottom: '1px solid #374151',
    color: '#e5e7eb',
    cursor: 'pointer',
    transition: 'background-color 0.1s',
    alignItems: 'center'
  },
  logTimestamp: {
    color: '#9ca3af',
    fontSize: '11px',
    fontFamily: 'Monaco, "Courier New", monospace'
  },
  logLevel: {
    fontSize: '10px',
    fontWeight: 700,
    padding: '2px 6px',
    borderRadius: '4px',
    textAlign: 'center'
  },
  logService: {
    color: '#60a5fa',
    fontSize: '11px',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap'
  },
  logMessage: {
    color: '#e5e7eb',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap'
  },
  logTraceId: {
    fontSize: '14px',
    cursor: 'pointer',
    textAlign: 'center'
  },
  modal: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000
  },
  modalContent: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    maxWidth: '800px',
    width: '90%',
    maxHeight: '90vh',
    overflow: 'auto'
  },
  modalHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '20px',
    borderBottom: '1px solid #e5e7eb'
  },
  modalClose: {
    background: 'none',
    border: 'none',
    fontSize: '32px',
    cursor: 'pointer',
    color: '#6b7280',
    lineHeight: 1
  },
  modalBody: {
    padding: '20px'
  },
  detailRow: {
    padding: '12px 0',
    borderBottom: '1px solid #f3f4f6',
    fontSize: '14px',
    display: 'flex',
    flexDirection: 'column',
    gap: '8px'
  },
  messageContent: {
    backgroundColor: '#f9fafb',
    padding: '12px',
    borderRadius: '6px',
    fontSize: '13px',
    fontFamily: 'Monaco, "Courier New", monospace',
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
    margin: 0
  },
  code: {
    backgroundColor: '#f3f4f6',
    padding: '4px 8px',
    borderRadius: '4px',
    fontSize: '12px',
    fontFamily: 'Monaco, "Courier New", monospace'
  },
  jsonContent: {
    backgroundColor: '#1f2937',
    color: '#e5e7eb',
    padding: '12px',
    borderRadius: '6px',
    fontSize: '12px',
    fontFamily: 'Monaco, "Courier New", monospace',
    overflow: 'auto',
    margin: 0
  },
  stackTrace: {
    backgroundColor: '#fef2f2',
    color: '#991b1b',
    padding: '12px',
    borderRadius: '6px',
    fontSize: '11px',
    fontFamily: 'Monaco, "Courier New", monospace',
    overflow: 'auto',
    margin: 0
  }
};

export default LogViewer;
