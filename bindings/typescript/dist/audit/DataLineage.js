import React, { useState, useEffect, useRef } from 'react';
export const DataLineage = ({ organizationId, resourceId, }) => {
    const [nodes, setNodes] = useState([]);
    const [edges, setEdges] = useState([]);
    const [selectedNode, setSelectedNode] = useState(null);
    const [loading, setLoading] = useState(true);
    const [viewMode, setViewMode] = useState('graph');
    const canvasRef = useRef(null);
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
        }
        catch (error) {
            console.error('Failed to load lineage data:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const renderGraph = () => {
        const canvas = canvasRef.current;
        if (!canvas)
            return;
        const ctx = canvas.getContext('2d');
        if (!ctx)
            return;
        canvas.width = canvas.offsetWidth;
        canvas.height = canvas.offsetHeight;
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        const positions = calculateNodePositions(nodes, edges, canvas.width, canvas.height);
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
                drawArrow(ctx, sourcePos.x, sourcePos.y, targetPos.x, targetPos.y);
            }
        });
        nodes.forEach((node) => {
            const pos = positions.get(node.id);
            if (!pos)
                return;
            ctx.beginPath();
            ctx.arc(pos.x, pos.y, 30, 0, 2 * Math.PI);
            ctx.fillStyle = getNodeColor(node);
            ctx.fill();
            ctx.strokeStyle = '#334155';
            ctx.lineWidth = 2;
            ctx.stroke();
            ctx.fillStyle = '#1e293b';
            ctx.font = '12px sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText(node.name.substring(0, 15), pos.x, pos.y + 50);
        });
    };
    const calculateNodePositions = (nodes, edges, width, height) => {
        const positions = new Map();
        const layers = {
            source: nodes.filter((n) => n.type === 'source'),
            transformation: nodes.filter((n) => n.type === 'transformation'),
            process: nodes.filter((n) => n.type === 'process'),
            destination: nodes.filter((n) => n.type === 'destination'),
        };
        const layerKeys = Object.keys(layers);
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
    const getNodeColor = (node) => {
        const classificationColors = {
            public: '#10b981',
            internal: '#3b82f6',
            confidential: '#f59e0b',
            restricted: '#ef4444',
        };
        return classificationColors[node.classification];
    };
    const drawArrow = (ctx, fromX, fromY, toX, toY) => {
        const headlen = 10;
        const angle = Math.atan2(toY - fromY, toX - fromX);
        ctx.beginPath();
        ctx.moveTo(toX - headlen * Math.cos(angle - Math.PI / 6), toY - headlen * Math.sin(angle - Math.PI / 6));
        ctx.lineTo(toX, toY);
        ctx.lineTo(toX - headlen * Math.cos(angle + Math.PI / 6), toY - headlen * Math.sin(angle + Math.PI / 6));
        ctx.strokeStyle = '#64748b';
        ctx.lineWidth = 2;
        ctx.stroke();
    };
    if (loading) {
        return (React.createElement("div", { className: "data-lineage loading" },
            React.createElement("div", { className: "loading-spinner" }),
            React.createElement("p", null, "Loading data lineage...")));
    }
    return (React.createElement("div", { className: "data-lineage" },
        React.createElement("div", { className: "lineage-header" },
            React.createElement("div", null,
                React.createElement("h2", null, "Data Lineage"),
                React.createElement("p", { className: "subtitle" }, "Track data flow and transformations across your system")),
            React.createElement("div", { className: "view-controls" },
                React.createElement("button", { className: `view-button ${viewMode === 'graph' ? 'active' : ''}`, onClick: () => setViewMode('graph') }, "Graph View"),
                React.createElement("button", { className: `view-button ${viewMode === 'tree' ? 'active' : ''}`, onClick: () => setViewMode('tree') }, "Tree View"),
                React.createElement("button", { className: `view-button ${viewMode === 'table' ? 'active' : ''}`, onClick: () => setViewMode('table') }, "Table View"))),
        React.createElement("div", { className: "lineage-legend" },
            React.createElement("h4", null, "Data Classification"),
            React.createElement("div", { className: "legend-items" },
                React.createElement("div", { className: "legend-item" },
                    React.createElement("div", { className: "legend-color", style: { backgroundColor: '#10b981' } }),
                    React.createElement("span", null, "Public")),
                React.createElement("div", { className: "legend-item" },
                    React.createElement("div", { className: "legend-color", style: { backgroundColor: '#3b82f6' } }),
                    React.createElement("span", null, "Internal")),
                React.createElement("div", { className: "legend-item" },
                    React.createElement("div", { className: "legend-color", style: { backgroundColor: '#f59e0b' } }),
                    React.createElement("span", null, "Confidential")),
                React.createElement("div", { className: "legend-item" },
                    React.createElement("div", { className: "legend-color", style: { backgroundColor: '#ef4444' } }),
                    React.createElement("span", null, "Restricted")))),
        React.createElement("div", { className: "lineage-content" },
            viewMode === 'graph' && (React.createElement("div", { className: "graph-view" },
                React.createElement("canvas", { ref: canvasRef, className: "lineage-canvas", width: 800, height: 600 }))),
            viewMode === 'tree' && (React.createElement("div", { className: "tree-view" },
                React.createElement(TreeView, { nodes: nodes, edges: edges, onNodeSelect: setSelectedNode }))),
            viewMode === 'table' && (React.createElement("div", { className: "table-view" },
                React.createElement(NodesTable, { nodes: nodes, onNodeSelect: setSelectedNode })))),
        selectedNode && (React.createElement("div", { className: "node-details-panel" },
            React.createElement("div", { className: "panel-header" },
                React.createElement("h3", null, "Node Details"),
                React.createElement("button", { onClick: () => setSelectedNode(null) }, "\u00D7")),
            React.createElement(NodeDetails, { node: selectedNode, edges: edges.filter((e) => e.source_id === selectedNode.id || e.target_id === selectedNode.id) })))));
};
function TreeView({ nodes, edges, onNodeSelect, }) {
    const rootNodes = nodes.filter((n) => n.parent_ids.length === 0);
    const renderNode = (node, level = 0) => {
        const children = nodes.filter((n) => n.parent_ids.includes(node.id));
        return (React.createElement("div", { key: node.id, className: "tree-node", style: { marginLeft: `${level * 30}px` } },
            React.createElement("div", { className: "tree-node-content", onClick: () => onNodeSelect(node) },
                React.createElement("span", { className: `node-type-icon type-${node.type}` },
                    node.type === 'source' && '📁',
                    node.type === 'transformation' && '⚙️',
                    node.type === 'process' && '🔄',
                    node.type === 'destination' && '📦'),
                React.createElement("span", { className: "node-name" }, node.name),
                React.createElement("span", { className: `classification-badge classification-${node.classification}` }, node.classification)),
            children.length > 0 && (React.createElement("div", { className: "tree-children" }, children.map((child) => renderNode(child, level + 1))))));
    };
    return (React.createElement("div", { className: "tree-container" }, rootNodes.map((node) => renderNode(node))));
}
function NodesTable({ nodes, onNodeSelect, }) {
    return (React.createElement("table", { className: "nodes-table" },
        React.createElement("thead", null,
            React.createElement("tr", null,
                React.createElement("th", null, "Name"),
                React.createElement("th", null, "Type"),
                React.createElement("th", null, "Classification"),
                React.createElement("th", null, "Owner"),
                React.createElement("th", null, "Compliance"),
                React.createElement("th", null, "Last Accessed"),
                React.createElement("th", null, "Actions"))),
        React.createElement("tbody", null, nodes.map((node) => (React.createElement("tr", { key: node.id },
            React.createElement("td", null, node.name),
            React.createElement("td", null,
                React.createElement("span", { className: `type-badge type-${node.type}` }, node.type)),
            React.createElement("td", null,
                React.createElement("span", { className: `classification-badge classification-${node.classification}` }, node.classification)),
            React.createElement("td", null, node.owner),
            React.createElement("td", null,
                React.createElement("div", { className: "compliance-tags" }, node.compliance_frameworks.map((fw) => (React.createElement("span", { key: fw, className: "compliance-tag" }, fw))))),
            React.createElement("td", null, node.last_accessed
                ? new Date(node.last_accessed).toLocaleDateString()
                : 'Never'),
            React.createElement("td", null,
                React.createElement("button", { className: "btn-icon", onClick: () => onNodeSelect(node), title: "View Details" }, "\uD83D\uDC41\uFE0F"))))))));
}
function NodeDetails({ node, edges, }) {
    const incomingEdges = edges.filter((e) => e.target_id === node.id);
    const outgoingEdges = edges.filter((e) => e.source_id === node.id);
    return (React.createElement("div", { className: "node-details" },
        React.createElement("div", { className: "details-section" },
            React.createElement("h4", null, "Basic Information"),
            React.createElement("div", { className: "details-grid" },
                React.createElement("div", { className: "detail-item" },
                    React.createElement("label", null, "Name:"),
                    React.createElement("span", null, node.name)),
                React.createElement("div", { className: "detail-item" },
                    React.createElement("label", null, "Type:"),
                    React.createElement("span", null, node.type)),
                React.createElement("div", { className: "detail-item" },
                    React.createElement("label", null, "Classification:"),
                    React.createElement("span", { className: `classification-badge classification-${node.classification}` }, node.classification)),
                React.createElement("div", { className: "detail-item" },
                    React.createElement("label", null, "Owner:"),
                    React.createElement("span", null, node.owner)),
                React.createElement("div", { className: "detail-item full-width" },
                    React.createElement("label", null, "Description:"),
                    React.createElement("span", null, node.description)))),
        node.data_type && (React.createElement("div", { className: "details-section" },
            React.createElement("h4", null, "Data Information"),
            React.createElement("div", { className: "details-grid" },
                React.createElement("div", { className: "detail-item" },
                    React.createElement("label", null, "Data Type:"),
                    React.createElement("span", null, node.data_type)),
                node.schema && (React.createElement("div", { className: "detail-item full-width" },
                    React.createElement("label", null, "Schema:"),
                    React.createElement("pre", { className: "schema-preview" }, JSON.stringify(node.schema, null, 2))))))),
        React.createElement("div", { className: "details-section" },
            React.createElement("h4", null, "Compliance & Governance"),
            React.createElement("div", { className: "details-grid" },
                React.createElement("div", { className: "detail-item" },
                    React.createElement("label", null, "Retention Policy:"),
                    React.createElement("span", null, node.retention_policy)),
                React.createElement("div", { className: "detail-item full-width" },
                    React.createElement("label", null, "Compliance Frameworks:"),
                    React.createElement("div", { className: "compliance-tags" }, node.compliance_frameworks.map((fw) => (React.createElement("span", { key: fw, className: "compliance-tag" }, fw))))))),
        React.createElement("div", { className: "details-section" },
            React.createElement("h4", null, "Connections"),
            React.createElement("div", { className: "connections" },
                React.createElement("div", { className: "connection-group" },
                    React.createElement("h5", null,
                        "Incoming (",
                        incomingEdges.length,
                        ")"),
                    incomingEdges.length === 0 ? (React.createElement("p", { className: "empty-message" }, "No incoming connections")) : (React.createElement("ul", null, incomingEdges.map((edge) => (React.createElement("li", { key: edge.id },
                        edge.transformation_type && (React.createElement("span", { className: "transformation-type" }, edge.transformation_type)),
                        edge.access_control?.requires_approval && (React.createElement("span", { className: "access-badge" }, "Requires Approval")))))))),
                React.createElement("div", { className: "connection-group" },
                    React.createElement("h5", null,
                        "Outgoing (",
                        outgoingEdges.length,
                        ")"),
                    outgoingEdges.length === 0 ? (React.createElement("p", { className: "empty-message" }, "No outgoing connections")) : (React.createElement("ul", null, outgoingEdges.map((edge) => (React.createElement("li", { key: edge.id },
                        edge.transformation_type && (React.createElement("span", { className: "transformation-type" }, edge.transformation_type)),
                        edge.access_control?.encryption_required && (React.createElement("span", { className: "access-badge" }, "Encrypted")))))))))),
        React.createElement("div", { className: "details-section" },
            React.createElement("h4", null, "Timestamps"),
            React.createElement("div", { className: "details-grid" },
                React.createElement("div", { className: "detail-item" },
                    React.createElement("label", null, "Created:"),
                    React.createElement("span", null, new Date(node.created_at).toLocaleString())),
                React.createElement("div", { className: "detail-item" },
                    React.createElement("label", null, "Updated:"),
                    React.createElement("span", null, new Date(node.updated_at).toLocaleString())),
                node.last_accessed && (React.createElement("div", { className: "detail-item" },
                    React.createElement("label", null, "Last Accessed:"),
                    React.createElement("span", null, new Date(node.last_accessed).toLocaleString())))))));
}
//# sourceMappingURL=DataLineage.js.map