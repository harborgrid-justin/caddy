/**
 * CADDY v0.4.0 - Service Dependency Map
 * Visual representation of service dependencies and health
 * @module monitoring/ServiceMap
 */

import React, { useEffect, useState, useRef } from 'react';
import { ServiceDependency, ServiceStatus } from './types';

interface ServiceMapProps {
  rootService?: string;
  className?: string;
}

interface Node {
  id: string;
  name: string;
  type: ServiceDependency['type'];
  status: ServiceStatus;
  x: number;
  y: number;
  level: number;
}

interface Edge {
  from: string;
  to: string;
  critical: boolean;
}

export const ServiceMap: React.FC<ServiceMapProps> = ({
  rootService,
  className = ''
}) => {
  const [dependencies, setDependencies] = useState<ServiceDependency[]>([]);
  const [selectedNode, setSelectedNode] = useState<ServiceDependency | null>(null);
  const [loading, setLoading] = useState(true);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    fetchDependencies();
  }, [rootService]);

  useEffect(() => {
    if (dependencies.length > 0) {
      renderMap();
    }
  }, [dependencies, selectedNode]);

  const fetchDependencies = async () => {
    try {
      setLoading(true);
      const params = rootService ? `?root=${rootService}` : '';
      const response = await fetch(`/api/monitoring/dependencies${params}`);
      if (!response.ok) throw new Error('Failed to fetch dependencies');

      const data = await response.json();
      setDependencies(data);
    } catch (error) {
      console.error('[ServiceMap] Failed to fetch dependencies:', error);
    } finally {
      setLoading(false);
    }
  };

  const buildGraph = (): { nodes: Node[]; edges: Edge[] } => {
    const nodes: Node[] = [];
    const edges: Edge[] = [];
    const visited = new Set<string>();

    const traverse = (dep: ServiceDependency, level: number, parentX: number = 0) => {
      if (visited.has(dep.id)) return;
      visited.add(dep.id);

      const siblingsAtLevel = nodes.filter(n => n.level === level).length;
      const x = parentX + (siblingsAtLevel * 150);
      const y = level * 120;

      nodes.push({
        id: dep.id,
        name: dep.name,
        type: dep.type,
        status: dep.status,
        x,
        y,
        level
      });

      dep.dependencies.forEach((child, index) => {
        edges.push({
          from: dep.id,
          to: child.id,
          critical: child.criticalPath
        });
        traverse(child, level + 1, x + (index * 100));
      });
    };

    dependencies.forEach((dep, index) => {
      traverse(dep, 0, index * 200);
    });

    return { nodes, edges };
  };

  const renderMap = () => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = container.getBoundingClientRect();

    canvas.width = rect.width * dpr;
    canvas.height = Math.max(rect.height, 600) * dpr;

    ctx.scale(dpr, dpr);

    // Clear canvas
    ctx.clearRect(0, 0, rect.width, rect.height);

    const { nodes, edges } = buildGraph();

    // Draw edges
    edges.forEach(edge => {
      const fromNode = nodes.find(n => n.id === edge.from);
      const toNode = nodes.find(n => n.id === edge.to);

      if (!fromNode || !toNode) return;

      ctx.beginPath();
      ctx.moveTo(fromNode.x + 60, fromNode.y + 40);
      ctx.lineTo(toNode.x + 60, toNode.y + 20);

      if (edge.critical) {
        ctx.strokeStyle = '#ef4444';
        ctx.lineWidth = 3;
        ctx.setLineDash([]);
      } else {
        ctx.strokeStyle = '#9ca3af';
        ctx.lineWidth = 2;
        ctx.setLineDash([5, 5]);
      }

      ctx.stroke();
      ctx.setLineDash([]);

      // Arrow head
      const angle = Math.atan2(toNode.y + 20 - (fromNode.y + 40), toNode.x + 60 - (fromNode.x + 60));
      ctx.beginPath();
      ctx.moveTo(toNode.x + 60, toNode.y + 20);
      ctx.lineTo(
        toNode.x + 60 - 10 * Math.cos(angle - Math.PI / 6),
        toNode.y + 20 - 10 * Math.sin(angle - Math.PI / 6)
      );
      ctx.lineTo(
        toNode.x + 60 - 10 * Math.cos(angle + Math.PI / 6),
        toNode.y + 20 - 10 * Math.sin(angle + Math.PI / 6)
      );
      ctx.closePath();
      ctx.fillStyle = edge.critical ? '#ef4444' : '#9ca3af';
      ctx.fill();
    });

    // Draw nodes
    nodes.forEach(node => {
      const isSelected = selectedNode?.id === node.id;
      const statusColor = getStatusColor(node.status);
      const typeIcon = getTypeIcon(node.type);

      // Node background
      ctx.fillStyle = isSelected ? '#f3f4f6' : '#fff';
      ctx.strokeStyle = isSelected ? '#3b82f6' : '#e5e7eb';
      ctx.lineWidth = isSelected ? 3 : 2;

      ctx.beginPath();
      ctx.roundRect(node.x, node.y, 120, 60, 8);
      ctx.fill();
      ctx.stroke();

      // Status indicator
      ctx.fillStyle = statusColor;
      ctx.beginPath();
      ctx.arc(node.x + 15, node.y + 15, 6, 0, Math.PI * 2);
      ctx.fill();

      // Type icon
      ctx.font = '16px sans-serif';
      ctx.fillText(typeIcon, node.x + 30, node.y + 20);

      // Node name
      ctx.fillStyle = '#111827';
      ctx.font = 'bold 12px sans-serif';
      ctx.textAlign = 'left';

      const maxWidth = 100;
      const text = node.name.length > 15 ? node.name.substring(0, 12) + '...' : node.name;
      ctx.fillText(text, node.x + 10, node.y + 40);

      // Node type
      ctx.fillStyle = '#6b7280';
      ctx.font = '10px sans-serif';
      ctx.fillText(node.type, node.x + 10, node.y + 52);
    });
  };

  const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const { nodes } = buildGraph();

    // Check if click is within any node
    for (const node of nodes) {
      if (x >= node.x && x <= node.x + 120 && y >= node.y && y <= node.y + 60) {
        const dep = dependencies.find(d => d.id === node.id) || findInDependencies(dependencies, node.id);
        setSelectedNode(dep || null);
        return;
      }
    }

    setSelectedNode(null);
  };

  const findInDependencies = (deps: ServiceDependency[], id: string): ServiceDependency | null => {
    for (const dep of deps) {
      if (dep.id === id) return dep;
      const found = findInDependencies(dep.dependencies, id);
      if (found) return found;
    }
    return null;
  };

  const getStatusColor = (status: ServiceStatus): string => {
    switch (status) {
      case ServiceStatus.HEALTHY:
        return '#10b981';
      case ServiceStatus.DEGRADED:
        return '#f59e0b';
      case ServiceStatus.DOWN:
        return '#ef4444';
      case ServiceStatus.MAINTENANCE:
        return '#3b82f6';
      default:
        return '#6b7280';
    }
  };

  const getTypeIcon = (type: ServiceDependency['type']): string => {
    switch (type) {
      case 'internal':
        return '🔧';
      case 'external':
        return '🌐';
      case 'database':
        return '🗄️';
      case 'cache':
        return '⚡';
      case 'queue':
        return '📬';
      default:
        return '📦';
    }
  };

  const getStatusLabel = (status: ServiceStatus): string => {
    return status.charAt(0).toUpperCase() + status.slice(1);
  };

  if (loading) {
    return (
      <div style={styles.loading}>
        <div style={styles.spinner} />
        <p>Loading service map...</p>
      </div>
    );
  }

  return (
    <div className={`service-map ${className}`} style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <h2 style={styles.title}>Service Dependency Map</h2>
        <button style={styles.refreshButton} onClick={fetchDependencies}>
          Refresh
        </button>
      </div>

      {/* Legend */}
      <div style={styles.legend}>
        <div style={styles.legendTitle}>Legend:</div>
        <div style={styles.legendItems}>
          <div style={styles.legendItem}>
            <div style={{ ...styles.legendDot, backgroundColor: '#10b981' }} />
            <span>Healthy</span>
          </div>
          <div style={styles.legendItem}>
            <div style={{ ...styles.legendDot, backgroundColor: '#f59e0b' }} />
            <span>Degraded</span>
          </div>
          <div style={styles.legendItem}>
            <div style={{ ...styles.legendDot, backgroundColor: '#ef4444' }} />
            <span>Down</span>
          </div>
          <div style={styles.legendItem}>
            <div style={{ ...styles.legendLine, borderColor: '#ef4444', borderWidth: '2px' }} />
            <span>Critical Path</span>
          </div>
          <div style={styles.legendItem}>
            <div style={{ ...styles.legendLine, borderStyle: 'dashed' }} />
            <span>Dependency</span>
          </div>
        </div>
      </div>

      {/* Map Container */}
      <div style={styles.mapContainer} ref={containerRef}>
        <canvas
          ref={canvasRef}
          style={styles.canvas}
          onClick={handleCanvasClick}
        />
      </div>

      {/* Selected Node Details */}
      {selectedNode && (
        <div style={styles.detailsPanel}>
          <div style={styles.detailsHeader}>
            <h3 style={styles.detailsTitle}>{selectedNode.name}</h3>
            <button
              style={styles.closeButton}
              onClick={() => setSelectedNode(null)}
            >
              ×
            </button>
          </div>
          <div style={styles.detailsBody}>
            <div style={styles.detailRow}>
              <strong>Type:</strong>
              <span>{getTypeIcon(selectedNode.type)} {selectedNode.type}</span>
            </div>
            <div style={styles.detailRow}>
              <strong>Status:</strong>
              <span style={{ color: getStatusColor(selectedNode.status) }}>
                {getStatusLabel(selectedNode.status)}
              </span>
            </div>
            {selectedNode.healthEndpoint && (
              <div style={styles.detailRow}>
                <strong>Health Endpoint:</strong>
                <code style={styles.code}>{selectedNode.healthEndpoint}</code>
              </div>
            )}
            <div style={styles.detailRow}>
              <strong>Critical Path:</strong>
              <span>{selectedNode.criticalPath ? 'Yes' : 'No'}</span>
            </div>
            {selectedNode.dependencies.length > 0 && (
              <div style={styles.detailRow}>
                <strong>Dependencies:</strong>
                <div style={styles.dependenciesList}>
                  {selectedNode.dependencies.map((dep, idx) => (
                    <span key={idx} style={styles.dependencyTag}>
                      {dep.name}
                    </span>
                  ))}
                </div>
              </div>
            )}
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
  loading: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '48px',
    color: '#6b7280'
  },
  spinner: {
    width: '40px',
    height: '40px',
    border: '4px solid #e5e7eb',
    borderTopColor: '#3b82f6',
    borderRadius: '50%',
    animation: 'spin 1s linear infinite'
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px'
  },
  title: {
    fontSize: '24px',
    fontWeight: 700,
    color: '#111827',
    margin: 0
  },
  refreshButton: {
    padding: '8px 16px',
    backgroundColor: '#3b82f6',
    color: '#fff',
    border: 'none',
    borderRadius: '6px',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer'
  },
  legend: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '12px 16px',
    backgroundColor: '#f9fafb',
    borderRadius: '8px',
    marginBottom: '16px',
    flexWrap: 'wrap'
  },
  legendTitle: {
    fontSize: '13px',
    fontWeight: 600,
    color: '#374151'
  },
  legendItems: {
    display: 'flex',
    gap: '16px',
    flexWrap: 'wrap'
  },
  legendItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '12px',
    color: '#6b7280'
  },
  legendDot: {
    width: '12px',
    height: '12px',
    borderRadius: '50%'
  },
  legendLine: {
    width: '24px',
    height: '0',
    borderTop: '2px solid #9ca3af'
  },
  mapContainer: {
    flex: 1,
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    overflow: 'auto',
    position: 'relative',
    minHeight: '400px'
  },
  canvas: {
    width: '100%',
    height: '100%',
    cursor: 'pointer'
  },
  detailsPanel: {
    marginTop: '16px',
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    overflow: 'hidden'
  },
  detailsHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '16px',
    borderBottom: '1px solid #e5e7eb',
    backgroundColor: '#f9fafb'
  },
  detailsTitle: {
    fontSize: '16px',
    fontWeight: 600,
    color: '#111827',
    margin: 0
  },
  closeButton: {
    background: 'none',
    border: 'none',
    fontSize: '24px',
    cursor: 'pointer',
    color: '#6b7280',
    lineHeight: 1
  },
  detailsBody: {
    padding: '16px'
  },
  detailRow: {
    padding: '8px 0',
    fontSize: '14px',
    display: 'flex',
    flexDirection: 'column',
    gap: '4px'
  },
  code: {
    backgroundColor: '#f3f4f6',
    padding: '4px 8px',
    borderRadius: '4px',
    fontSize: '12px',
    fontFamily: 'Monaco, "Courier New", monospace'
  },
  dependenciesList: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '6px',
    marginTop: '4px'
  },
  dependencyTag: {
    fontSize: '12px',
    padding: '4px 8px',
    backgroundColor: '#f3f4f6',
    borderRadius: '4px',
    color: '#374151'
  }
};

export default ServiceMap;
