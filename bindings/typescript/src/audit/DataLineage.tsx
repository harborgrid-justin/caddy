/**
 * Data Lineage Component
 * Data flow visualization for compliance and governance
 */

import React, { useState, useEffect, useRef } from 'react';
import type { DataLineageNode, DataLineageEdge, DataClassification } from './types';

interface DataLineageProps {
  organizationId?: string;
  resourceId?: string;
}

export const DataLineage: React.FC<DataLineageProps> = ({
  organizationId,
  resourceId,
}) => {
  const [nodes, setNodes] = useState<DataLineageNode[]>([]);
  const [edges, setEdges] = useState<DataLineageEdge[]>([]);
  const [selectedNode, setSelectedNode] = useState<DataLineageNode | null>(null);
  const [loading, setLoading] = useState(true);
  const [viewMode, setViewMode] = useState<'graph' | 'tree' | 'table'>('graph');
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    loadLineageData();
  }, [organizationId, resourceId]);

  useEffect(() => {
    if (viewMode === 'graph' && nodes.length > 0) {
      renderGraph();
    }
  }, [nodes, edges, viewMode]);

  const loadLineageData = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams({
        ...(organizationId && { organization_id: organizationId }),
        ...(resourceId && { resource_id: resourceId }),
      });

      const response = await fetch(`/api/audit/lineage?${params}`);
      const data = await response.json();
      setNodes(data.nodes || []);
      setEdges(data.edges || []);
    } catch (error) {
      console.error('Failed to load lineage data:', error);
    } finally {
      setLoading(false);
    }
  };

  const renderGraph = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size
    canvas.width = canvas.offsetWidth;
    canvas.height = canvas.offsetHeight;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Simple force-directed layout
    const positions = calculateNodePositions(nodes, edges, canvas.width, canvas.height);

    // Draw edges
    edges.forEach((edge) => {
      const sourcePos = positions.get(edge.source_id);
      const targetPos = positions.get(edge.target_id);

      if (sourcePos && targetPos) {
        ctx.beginPath();
        ctx.moveTo(sourcePos.x, sourcePos.y);
        ctx.lineTo(targetPos.x, targetPos.y);
        ctx.strokeStyle = '#cbd5e1';
        ctx.lineWidth = 2;
        ctx.stroke();

        // Draw arrow
        drawArrow(ctx, sourcePos.x, sourcePos.y, targetPos.x, targetPos.y);
      }
    });

    // Draw nodes
    nodes.forEach((node) => {
      const pos = positions.get(node.id);
      if (!pos) return;

      // Node circle
      ctx.beginPath();
      ctx.arc(pos.x, pos.y, 30, 0, 2 * Math.PI);
      ctx.fillStyle = getNodeColor(node);
      ctx.fill();
      ctx.strokeStyle = '#334155';
      ctx.lineWidth = 2;
      ctx.stroke();

      // Node label
      ctx.fillStyle = '#1e293b';
      ctx.font = '12px sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText(node.name.substring(0, 15), pos.x, pos.y + 50);
    });
  };

  const calculateNodePositions = (
    nodes: DataLineageNode[],
    edges: DataLineageEdge[],
    width: number,
    height: number
  ): Map<string, { x: number; y: number }> => {
    const positions = new Map<string, { x: number; y: number }>();

    // Simple hierarchical layout based on node type
    const layers = {
      source: nodes.filter((n) => n.type === 'source'),
      transformation: nodes.filter((n) => n.type === 'transformation'),
      process: nodes.filter((n) => n.type === 'process'),
      destination: nodes.filter((n) => n.type === 'destination'),
    };

    const layerKeys = Object.keys(layers) as Array<keyof typeof layers>;
    const layerHeight = height / (layerKeys.length + 1);

    layerKeys.forEach((key, layerIndex) => {
      const layerNodes = layers[key];
      const nodeSpacing = width / (layerNodes.length + 1);

      layerNodes.forEach((node, nodeIndex) => {
        positions.set(node.id, {
          x: nodeSpacing * (nodeIndex + 1),
          y: layerHeight * (layerIndex + 1),
        });
      });
    });

    return positions;
  };

  const getNodeColor = (node: DataLineageNode): string => {
    const classificationColors: Record<DataClassification, string> = {
      public: '#10b981',
      internal: '#3b82f6',
      confidential: '#f59e0b',
      restricted: '#ef4444',
    };

    return classificationColors[node.classification];
  };

  const drawArrow = (
    ctx: CanvasRenderingContext2D,
    fromX: number,
    fromY: number,
    toX: number,
    toY: number
  ) => {
    const headlen = 10;
    const angle = Math.atan2(toY - fromY, toX - fromX);

    ctx.beginPath();
    ctx.moveTo(
      toX - headlen * Math.cos(angle - Math.PI / 6),
      toY - headlen * Math.sin(angle - Math.PI / 6)
    );
    ctx.lineTo(toX, toY);
    ctx.lineTo(
      toX - headlen * Math.cos(angle + Math.PI / 6),
      toY - headlen * Math.sin(angle + Math.PI / 6)
    );
    ctx.strokeStyle = '#64748b';
    ctx.lineWidth = 2;
    ctx.stroke();
  };

  if (loading) {
    return (
      <div className="data-lineage loading">
        <div className="loading-spinner" />
        <p>Loading data lineage...</p>
      </div>
    );
  }

  return (
    <div className="data-lineage">
      {/* Header */}
      <div className="lineage-header">
        <div>
          <h2>Data Lineage</h2>
          <p className="subtitle">
            Track data flow and transformations across your system
          </p>
        </div>
        <div className="view-controls">
          <button
            className={`view-button ${viewMode === 'graph' ? 'active' : ''}`}
            onClick={() => setViewMode('graph')}
          >
            Graph View
          </button>
          <button
            className={`view-button ${viewMode === 'tree' ? 'active' : ''}`}
            onClick={() => setViewMode('tree')}
          >
            Tree View
          </button>
          <button
            className={`view-button ${viewMode === 'table' ? 'active' : ''}`}
            onClick={() => setViewMode('table')}
          >
            Table View
          </button>
        </div>
      </div>

      {/* Legend */}
      <div className="lineage-legend">
        <h4>Data Classification</h4>
        <div className="legend-items">
          <div className="legend-item">
            <div className="legend-color" style={{ backgroundColor: '#10b981' }} />
            <span>Public</span>
          </div>
          <div className="legend-item">
            <div className="legend-color" style={{ backgroundColor: '#3b82f6' }} />
            <span>Internal</span>
          </div>
          <div className="legend-item">
            <div className="legend-color" style={{ backgroundColor: '#f59e0b' }} />
            <span>Confidential</span>
          </div>
          <div className="legend-item">
            <div className="legend-color" style={{ backgroundColor: '#ef4444' }} />
            <span>Restricted</span>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="lineage-content">
        {viewMode === 'graph' && (
          <div className="graph-view">
            <canvas
              ref={canvasRef}
              className="lineage-canvas"
              width={800}
              height={600}
            />
          </div>
        )}

        {viewMode === 'tree' && (
          <div className="tree-view">
            <TreeView nodes={nodes} edges={edges} onNodeSelect={setSelectedNode} />
          </div>
        )}

        {viewMode === 'table' && (
          <div className="table-view">
            <NodesTable nodes={nodes} onNodeSelect={setSelectedNode} />
          </div>
        )}
      </div>

      {/* Node Details Panel */}
      {selectedNode && (
        <div className="node-details-panel">
          <div className="panel-header">
            <h3>Node Details</h3>
            <button onClick={() => setSelectedNode(null)}>×</button>
          </div>
          <NodeDetails
            node={selectedNode}
            edges={edges.filter(
              (e) => e.source_id === selectedNode.id || e.target_id === selectedNode.id
            )}
          />
        </div>
      )}
    </div>
  );
};

// Tree View Component
function TreeView({
  nodes,
  edges,
  onNodeSelect,
}: {
  nodes: DataLineageNode[];
  edges: DataLineageEdge[];
  onNodeSelect: (node: DataLineageNode) => void;
}) {
  const rootNodes = nodes.filter((n) => n.parent_ids.length === 0);

  const renderNode = (node: DataLineageNode, level: number = 0) => {
    const children = nodes.filter((n) => n.parent_ids.includes(node.id));

    return (
      <div key={node.id} className="tree-node" style={{ marginLeft: `${level * 30}px` }}>
        <div className="tree-node-content" onClick={() => onNodeSelect(node)}>
          <span className={`node-type-icon type-${node.type}`}>
            {node.type === 'source' && '📁'}
            {node.type === 'transformation' && '⚙️'}
            {node.type === 'process' && '🔄'}
            {node.type === 'destination' && '📦'}
          </span>
          <span className="node-name">{node.name}</span>
          <span className={`classification-badge classification-${node.classification}`}>
            {node.classification}
          </span>
        </div>
        {children.length > 0 && (
          <div className="tree-children">
            {children.map((child) => renderNode(child, level + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="tree-container">
      {rootNodes.map((node) => renderNode(node))}
    </div>
  );
}

// Nodes Table Component
function NodesTable({
  nodes,
  onNodeSelect,
}: {
  nodes: DataLineageNode[];
  onNodeSelect: (node: DataLineageNode) => void;
}) {
  return (
    <table className="nodes-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Type</th>
          <th>Classification</th>
          <th>Owner</th>
          <th>Compliance</th>
          <th>Last Accessed</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {nodes.map((node) => (
          <tr key={node.id}>
            <td>{node.name}</td>
            <td>
              <span className={`type-badge type-${node.type}`}>
                {node.type}
              </span>
            </td>
            <td>
              <span className={`classification-badge classification-${node.classification}`}>
                {node.classification}
              </span>
            </td>
            <td>{node.owner}</td>
            <td>
              <div className="compliance-tags">
                {node.compliance_frameworks.map((fw) => (
                  <span key={fw} className="compliance-tag">
                    {fw}
                  </span>
                ))}
              </div>
            </td>
            <td>
              {node.last_accessed
                ? new Date(node.last_accessed).toLocaleDateString()
                : 'Never'}
            </td>
            <td>
              <button
                className="btn-icon"
                onClick={() => onNodeSelect(node)}
                title="View Details"
              >
                👁️
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

// Node Details Component
function NodeDetails({
  node,
  edges,
}: {
  node: DataLineageNode;
  edges: DataLineageEdge[];
}) {
  const incomingEdges = edges.filter((e) => e.target_id === node.id);
  const outgoingEdges = edges.filter((e) => e.source_id === node.id);

  return (
    <div className="node-details">
      <div className="details-section">
        <h4>Basic Information</h4>
        <div className="details-grid">
          <div className="detail-item">
            <label>Name:</label>
            <span>{node.name}</span>
          </div>
          <div className="detail-item">
            <label>Type:</label>
            <span>{node.type}</span>
          </div>
          <div className="detail-item">
            <label>Classification:</label>
            <span className={`classification-badge classification-${node.classification}`}>
              {node.classification}
            </span>
          </div>
          <div className="detail-item">
            <label>Owner:</label>
            <span>{node.owner}</span>
          </div>
          <div className="detail-item full-width">
            <label>Description:</label>
            <span>{node.description}</span>
          </div>
        </div>
      </div>

      {node.data_type && (
        <div className="details-section">
          <h4>Data Information</h4>
          <div className="details-grid">
            <div className="detail-item">
              <label>Data Type:</label>
              <span>{node.data_type}</span>
            </div>
            {node.schema && (
              <div className="detail-item full-width">
                <label>Schema:</label>
                <pre className="schema-preview">
                  {JSON.stringify(node.schema, null, 2)}
                </pre>
              </div>
            )}
          </div>
        </div>
      )}

      <div className="details-section">
        <h4>Compliance & Governance</h4>
        <div className="details-grid">
          <div className="detail-item">
            <label>Retention Policy:</label>
            <span>{node.retention_policy}</span>
          </div>
          <div className="detail-item full-width">
            <label>Compliance Frameworks:</label>
            <div className="compliance-tags">
              {node.compliance_frameworks.map((fw) => (
                <span key={fw} className="compliance-tag">
                  {fw}
                </span>
              ))}
            </div>
          </div>
        </div>
      </div>

      <div className="details-section">
        <h4>Connections</h4>
        <div className="connections">
          <div className="connection-group">
            <h5>Incoming ({incomingEdges.length})</h5>
            {incomingEdges.length === 0 ? (
              <p className="empty-message">No incoming connections</p>
            ) : (
              <ul>
                {incomingEdges.map((edge) => (
                  <li key={edge.id}>
                    {edge.transformation_type && (
                      <span className="transformation-type">
                        {edge.transformation_type}
                      </span>
                    )}
                    {edge.access_control?.requires_approval && (
                      <span className="access-badge">Requires Approval</span>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </div>
          <div className="connection-group">
            <h5>Outgoing ({outgoingEdges.length})</h5>
            {outgoingEdges.length === 0 ? (
              <p className="empty-message">No outgoing connections</p>
            ) : (
              <ul>
                {outgoingEdges.map((edge) => (
                  <li key={edge.id}>
                    {edge.transformation_type && (
                      <span className="transformation-type">
                        {edge.transformation_type}
                      </span>
                    )}
                    {edge.access_control?.encryption_required && (
                      <span className="access-badge">Encrypted</span>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </div>

      <div className="details-section">
        <h4>Timestamps</h4>
        <div className="details-grid">
          <div className="detail-item">
            <label>Created:</label>
            <span>{new Date(node.created_at).toLocaleString()}</span>
          </div>
          <div className="detail-item">
            <label>Updated:</label>
            <span>{new Date(node.updated_at).toLocaleString()}</span>
          </div>
          {node.last_accessed && (
            <div className="detail-item">
              <label>Last Accessed:</label>
              <span>{new Date(node.last_accessed).toLocaleString()}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
