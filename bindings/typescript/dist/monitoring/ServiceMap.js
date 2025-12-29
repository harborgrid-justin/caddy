import React, { useEffect, useState, useRef } from 'react';
import { ServiceStatus } from './types';
export const ServiceMap = ({ rootService, className = '' }) => {
    const [dependencies, setDependencies] = useState([]);
    const [selectedNode, setSelectedNode] = useState(null);
    const [loading, setLoading] = useState(true);
    const canvasRef = useRef(null);
    const containerRef = useRef(null);
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
            if (!response.ok)
                throw new Error('Failed to fetch dependencies');
            const data = await response.json();
            setDependencies(data);
        }
        catch (error) {
            console.error('[ServiceMap] Failed to fetch dependencies:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const buildGraph = () => {
        const nodes = [];
        const edges = [];
        const visited = new Set();
        const traverse = (dep, level, parentX = 0) => {
            if (visited.has(dep.id))
                return;
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
        if (!canvas || !container)
            return;
        const ctx = canvas.getContext('2d');
        if (!ctx)
            return;
        const dpr = window.devicePixelRatio || 1;
        const rect = container.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = Math.max(rect.height, 600) * dpr;
        ctx.scale(dpr, dpr);
        ctx.clearRect(0, 0, rect.width, rect.height);
        const { nodes, edges } = buildGraph();
        edges.forEach(edge => {
            const fromNode = nodes.find(n => n.id === edge.from);
            const toNode = nodes.find(n => n.id === edge.to);
            if (!fromNode || !toNode)
                return;
            ctx.beginPath();
            ctx.moveTo(fromNode.x + 60, fromNode.y + 40);
            ctx.lineTo(toNode.x + 60, toNode.y + 20);
            if (edge.critical) {
                ctx.strokeStyle = '#ef4444';
                ctx.lineWidth = 3;
                ctx.setLineDash([]);
            }
            else {
                ctx.strokeStyle = '#9ca3af';
                ctx.lineWidth = 2;
                ctx.setLineDash([5, 5]);
            }
            ctx.stroke();
            ctx.setLineDash([]);
            const angle = Math.atan2(toNode.y + 20 - (fromNode.y + 40), toNode.x + 60 - (fromNode.x + 60));
            ctx.beginPath();
            ctx.moveTo(toNode.x + 60, toNode.y + 20);
            ctx.lineTo(toNode.x + 60 - 10 * Math.cos(angle - Math.PI / 6), toNode.y + 20 - 10 * Math.sin(angle - Math.PI / 6));
            ctx.lineTo(toNode.x + 60 - 10 * Math.cos(angle + Math.PI / 6), toNode.y + 20 - 10 * Math.sin(angle + Math.PI / 6));
            ctx.closePath();
            ctx.fillStyle = edge.critical ? '#ef4444' : '#9ca3af';
            ctx.fill();
        });
        nodes.forEach(node => {
            const isSelected = selectedNode?.id === node.id;
            const statusColor = getStatusColor(node.status);
            const typeIcon = getTypeIcon(node.type);
            ctx.fillStyle = isSelected ? '#f3f4f6' : '#fff';
            ctx.strokeStyle = isSelected ? '#3b82f6' : '#e5e7eb';
            ctx.lineWidth = isSelected ? 3 : 2;
            ctx.beginPath();
            ctx.roundRect(node.x, node.y, 120, 60, 8);
            ctx.fill();
            ctx.stroke();
            ctx.fillStyle = statusColor;
            ctx.beginPath();
            ctx.arc(node.x + 15, node.y + 15, 6, 0, Math.PI * 2);
            ctx.fill();
            ctx.font = '16px sans-serif';
            ctx.fillText(typeIcon, node.x + 30, node.y + 20);
            ctx.fillStyle = '#111827';
            ctx.font = 'bold 12px sans-serif';
            ctx.textAlign = 'left';
            const maxWidth = 100;
            const text = node.name.length > 15 ? node.name.substring(0, 12) + '...' : node.name;
            ctx.fillText(text, node.x + 10, node.y + 40);
            ctx.fillStyle = '#6b7280';
            ctx.font = '10px sans-serif';
            ctx.fillText(node.type, node.x + 10, node.y + 52);
        });
    };
    const handleCanvasClick = (e) => {
        const canvas = canvasRef.current;
        if (!canvas)
            return;
        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        const { nodes } = buildGraph();
        for (const node of nodes) {
            if (x >= node.x && x <= node.x + 120 && y >= node.y && y <= node.y + 60) {
                const dep = dependencies.find(d => d.id === node.id) || findInDependencies(dependencies, node.id);
                setSelectedNode(dep || null);
                return;
            }
        }
        setSelectedNode(null);
    };
    const findInDependencies = (deps, id) => {
        for (const dep of deps) {
            if (dep.id === id)
                return dep;
            const found = findInDependencies(dep.dependencies, id);
            if (found)
                return found;
        }
        return null;
    };
    const getStatusColor = (status) => {
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
    const getTypeIcon = (type) => {
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
    const getStatusLabel = (status) => {
        return status.charAt(0).toUpperCase() + status.slice(1);
    };
    if (loading) {
        return (React.createElement("div", { style: styles.loading },
            React.createElement("div", { style: styles.spinner }),
            React.createElement("p", null, "Loading service map...")));
    }
    return (React.createElement("div", { className: `service-map ${className}`, style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h2", { style: styles.title }, "Service Dependency Map"),
            React.createElement("button", { style: styles.refreshButton, onClick: fetchDependencies }, "Refresh")),
        React.createElement("div", { style: styles.legend },
            React.createElement("div", { style: styles.legendTitle }, "Legend:"),
            React.createElement("div", { style: styles.legendItems },
                React.createElement("div", { style: styles.legendItem },
                    React.createElement("div", { style: { ...styles.legendDot, backgroundColor: '#10b981' } }),
                    React.createElement("span", null, "Healthy")),
                React.createElement("div", { style: styles.legendItem },
                    React.createElement("div", { style: { ...styles.legendDot, backgroundColor: '#f59e0b' } }),
                    React.createElement("span", null, "Degraded")),
                React.createElement("div", { style: styles.legendItem },
                    React.createElement("div", { style: { ...styles.legendDot, backgroundColor: '#ef4444' } }),
                    React.createElement("span", null, "Down")),
                React.createElement("div", { style: styles.legendItem },
                    React.createElement("div", { style: { ...styles.legendLine, borderColor: '#ef4444', borderWidth: '2px' } }),
                    React.createElement("span", null, "Critical Path")),
                React.createElement("div", { style: styles.legendItem },
                    React.createElement("div", { style: { ...styles.legendLine, borderStyle: 'dashed' } }),
                    React.createElement("span", null, "Dependency")))),
        React.createElement("div", { style: styles.mapContainer, ref: containerRef },
            React.createElement("canvas", { ref: canvasRef, style: styles.canvas, onClick: handleCanvasClick })),
        selectedNode && (React.createElement("div", { style: styles.detailsPanel },
            React.createElement("div", { style: styles.detailsHeader },
                React.createElement("h3", { style: styles.detailsTitle }, selectedNode.name),
                React.createElement("button", { style: styles.closeButton, onClick: () => setSelectedNode(null) }, "\u00D7")),
            React.createElement("div", { style: styles.detailsBody },
                React.createElement("div", { style: styles.detailRow },
                    React.createElement("strong", null, "Type:"),
                    React.createElement("span", null,
                        getTypeIcon(selectedNode.type),
                        " ",
                        selectedNode.type)),
                React.createElement("div", { style: styles.detailRow },
                    React.createElement("strong", null, "Status:"),
                    React.createElement("span", { style: { color: getStatusColor(selectedNode.status) } }, getStatusLabel(selectedNode.status))),
                selectedNode.healthEndpoint && (React.createElement("div", { style: styles.detailRow },
                    React.createElement("strong", null, "Health Endpoint:"),
                    React.createElement("code", { style: styles.code }, selectedNode.healthEndpoint))),
                React.createElement("div", { style: styles.detailRow },
                    React.createElement("strong", null, "Critical Path:"),
                    React.createElement("span", null, selectedNode.criticalPath ? 'Yes' : 'No')),
                selectedNode.dependencies.length > 0 && (React.createElement("div", { style: styles.detailRow },
                    React.createElement("strong", null, "Dependencies:"),
                    React.createElement("div", { style: styles.dependenciesList }, selectedNode.dependencies.map((dep, idx) => (React.createElement("span", { key: idx, style: styles.dependencyTag }, dep.name)))))))))));
};
const styles = {
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
//# sourceMappingURL=ServiceMap.js.map