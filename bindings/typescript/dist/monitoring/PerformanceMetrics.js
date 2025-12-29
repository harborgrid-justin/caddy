import React, { useEffect, useState, useCallback, useRef } from 'react';
import { MetricType } from './types';
export const PerformanceMetrics = ({ service, metrics = [MetricType.CPU, MetricType.MEMORY, MetricType.DISK, MetricType.NETWORK], timeRange = { from: new Date(Date.now() - 3600000), to: new Date(), quick: '1h' }, refreshInterval = 5000, className = '' }) => {
    const [chartData, setChartData] = useState({});
    const [currentValues, setCurrentValues] = useState({});
    const [loading, setLoading] = useState(true);
    const [selectedMetric, setSelectedMetric] = useState('all');
    const [ws, setWs] = useState(null);
    const canvasRefs = useRef({});
    const animationFrameRef = useRef(undefined);
    useEffect(() => {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/api/monitoring/metrics/stream`;
        const socket = new WebSocket(wsUrl);
        socket.onopen = () => {
            console.log('[PerformanceMetrics] WebSocket connected');
            socket.send(JSON.stringify({
                type: 'subscribe',
                service,
                metrics
            }));
        };
        socket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                if (message.type === 'metric') {
                    handleMetricUpdate(message.data);
                }
            }
            catch (error) {
                console.error('[PerformanceMetrics] Failed to parse WebSocket message:', error);
            }
        };
        socket.onerror = (error) => {
            console.error('[PerformanceMetrics] WebSocket error:', error);
        };
        socket.onclose = () => {
            console.log('[PerformanceMetrics] WebSocket disconnected');
            setTimeout(() => {
            }, 5000);
        };
        setWs(socket);
        return () => {
            socket.close();
            if (animationFrameRef.current) {
                cancelAnimationFrame(animationFrameRef.current);
            }
        };
    }, [service, metrics]);
    useEffect(() => {
        fetchHistoricalData();
    }, [service, timeRange]);
    useEffect(() => {
        renderCharts();
    }, [chartData, selectedMetric]);
    const fetchHistoricalData = async () => {
        try {
            setLoading(true);
            const params = new URLSearchParams({
                from: timeRange.from.toISOString(),
                to: timeRange.to.toISOString(),
                ...(service && { service })
            });
            const response = await fetch(`/api/monitoring/metrics/history?${params}`);
            if (!response.ok)
                throw new Error('Failed to fetch metrics');
            const data = await response.json();
            const grouped = {};
            data.forEach(metric => {
                const key = metric.service ? `${metric.type}_${metric.service}` : metric.type;
                if (!grouped[key]) {
                    grouped[key] = [];
                }
                grouped[key].push({
                    timestamp: new Date(metric.timestamp).getTime(),
                    value: metric.value
                });
            });
            Object.keys(grouped).forEach(key => {
                grouped[key].sort((a, b) => a.timestamp - b.timestamp);
            });
            setChartData(grouped);
        }
        catch (error) {
            console.error('[PerformanceMetrics] Failed to fetch historical data:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const handleMetricUpdate = useCallback((metric) => {
        const key = metric.service ? `${metric.type}_${metric.service}` : metric.type;
        const timestamp = new Date(metric.timestamp).getTime();
        setChartData(prev => {
            const updated = { ...prev };
            if (!updated[key]) {
                updated[key] = [];
            }
            updated[key] = [...updated[key], { timestamp, value: metric.value }];
            const cutoff = Date.now() - (timeRange.to.getTime() - timeRange.from.getTime());
            updated[key] = updated[key].filter(point => point.timestamp >= cutoff);
            return updated;
        });
        setCurrentValues(prev => ({ ...prev, [key]: metric.value }));
    }, [timeRange]);
    const renderCharts = useCallback(() => {
        const metricsToRender = selectedMetric === 'all' ? metrics : [selectedMetric];
        metricsToRender.forEach(metricType => {
            const canvas = canvasRefs.current[metricType];
            if (!canvas)
                return;
            const ctx = canvas.getContext('2d');
            if (!ctx)
                return;
            const dpr = window.devicePixelRatio || 1;
            const rect = canvas.getBoundingClientRect();
            canvas.width = rect.width * dpr;
            canvas.height = rect.height * dpr;
            ctx.scale(dpr, dpr);
            ctx.clearRect(0, 0, rect.width, rect.height);
            const key = service ? `${metricType}_${service}` : metricType;
            const data = chartData[key] || [];
            if (data.length === 0) {
                ctx.fillStyle = '#9ca3af';
                ctx.font = '14px sans-serif';
                ctx.textAlign = 'center';
                ctx.fillText('No data available', rect.width / 2, rect.height / 2);
                return;
            }
            const padding = 40;
            const chartWidth = rect.width - padding * 2;
            const chartHeight = rect.height - padding * 2;
            const minTime = Math.min(...data.map(d => d.timestamp));
            const maxTime = Math.max(...data.map(d => d.timestamp));
            const minValue = 0;
            const maxValue = Math.max(...data.map(d => d.value), 100);
            ctx.strokeStyle = '#e5e7eb';
            ctx.lineWidth = 1;
            for (let i = 0; i <= 5; i++) {
                const y = padding + (chartHeight / 5) * i;
                ctx.beginPath();
                ctx.moveTo(padding, y);
                ctx.lineTo(padding + chartWidth, y);
                ctx.stroke();
                const value = maxValue - (maxValue / 5) * i;
                ctx.fillStyle = '#6b7280';
                ctx.font = '12px sans-serif';
                ctx.textAlign = 'right';
                ctx.fillText(value.toFixed(0), padding - 10, y + 4);
            }
            ctx.strokeStyle = getMetricColor(metricType);
            ctx.lineWidth = 2;
            ctx.beginPath();
            data.forEach((point, index) => {
                const x = padding + ((point.timestamp - minTime) / (maxTime - minTime)) * chartWidth;
                const y = padding + chartHeight - ((point.value - minValue) / (maxValue - minValue)) * chartHeight;
                if (index === 0) {
                    ctx.moveTo(x, y);
                }
                else {
                    ctx.lineTo(x, y);
                }
            });
            ctx.stroke();
            ctx.lineTo(padding + chartWidth, padding + chartHeight);
            ctx.lineTo(padding, padding + chartHeight);
            ctx.closePath();
            const gradient = ctx.createLinearGradient(0, padding, 0, padding + chartHeight);
            gradient.addColorStop(0, `${getMetricColor(metricType)}40`);
            gradient.addColorStop(1, `${getMetricColor(metricType)}00`);
            ctx.fillStyle = gradient;
            ctx.fill();
            ctx.fillStyle = '#6b7280';
            ctx.font = '11px sans-serif';
            ctx.textAlign = 'center';
            for (let i = 0; i <= 4; i++) {
                const timestamp = minTime + ((maxTime - minTime) / 4) * i;
                const x = padding + (chartWidth / 4) * i;
                const time = new Date(timestamp).toLocaleTimeString([], {
                    hour: '2-digit',
                    minute: '2-digit'
                });
                ctx.fillText(time, x, rect.height - 10);
            }
            if (currentValues[key] !== undefined) {
                const latestPoint = data[data.length - 1];
                const x = padding + ((latestPoint.timestamp - minTime) / (maxTime - minTime)) * chartWidth;
                const y = padding + chartHeight - ((latestPoint.value - minValue) / (maxValue - minValue)) * chartHeight;
                ctx.fillStyle = getMetricColor(metricType);
                ctx.beginPath();
                ctx.arc(x, y, 4, 0, Math.PI * 2);
                ctx.fill();
                ctx.fillStyle = '#fff';
                ctx.fillRect(x - 30, y - 30, 60, 20);
                ctx.strokeStyle = getMetricColor(metricType);
                ctx.strokeRect(x - 30, y - 30, 60, 20);
                ctx.fillStyle = '#111827';
                ctx.font = 'bold 12px sans-serif';
                ctx.textAlign = 'center';
                ctx.fillText(`${currentValues[key].toFixed(1)}`, x, y - 16);
            }
        });
        animationFrameRef.current = requestAnimationFrame(renderCharts);
    }, [chartData, currentValues, selectedMetric, service, metrics]);
    const getMetricColor = (metric) => {
        switch (metric) {
            case MetricType.CPU:
                return '#3b82f6';
            case MetricType.MEMORY:
                return '#8b5cf6';
            case MetricType.DISK:
                return '#f59e0b';
            case MetricType.NETWORK:
                return '#10b981';
            case MetricType.LATENCY:
                return '#ec4899';
            case MetricType.THROUGHPUT:
                return '#06b6d4';
            case MetricType.ERROR_RATE:
                return '#ef4444';
            default:
                return '#6b7280';
        }
    };
    const getMetricLabel = (metric) => {
        return metric
            .split('_')
            .map(word => word.charAt(0).toUpperCase() + word.slice(1))
            .join(' ');
    };
    const getMetricUnit = (metric) => {
        switch (metric) {
            case MetricType.CPU:
            case MetricType.MEMORY:
            case MetricType.DISK:
            case MetricType.ERROR_RATE:
                return '%';
            case MetricType.LATENCY:
                return 'ms';
            case MetricType.THROUGHPUT:
                return 'req/s';
            case MetricType.NETWORK:
                return 'MB/s';
            default:
                return '';
        }
    };
    if (loading) {
        return (React.createElement("div", { style: styles.loading },
            React.createElement("div", { style: styles.spinner }),
            React.createElement("p", null, "Loading performance metrics...")));
    }
    return (React.createElement("div", { className: `performance-metrics ${className}`, style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h2", { style: styles.title }, "Performance Metrics"),
            service && React.createElement("div", { style: styles.serviceLabel },
                "Service: ",
                service)),
        React.createElement("div", { style: styles.metricSelector },
            React.createElement("button", { style: {
                    ...styles.metricButton,
                    ...(selectedMetric === 'all' ? styles.metricButtonActive : {})
                }, onClick: () => setSelectedMetric('all') }, "All Metrics"),
            metrics.map(metric => (React.createElement("button", { key: metric, style: {
                    ...styles.metricButton,
                    ...(selectedMetric === metric ? styles.metricButtonActive : {}),
                    borderLeftColor: getMetricColor(metric)
                }, onClick: () => setSelectedMetric(metric) },
                React.createElement("span", { style: {
                        ...styles.metricDot,
                        backgroundColor: getMetricColor(metric)
                    } }),
                getMetricLabel(metric))))),
        React.createElement("div", { style: styles.currentValues }, metrics.map(metric => {
            const key = service ? `${metric}_${service}` : metric;
            const value = currentValues[key];
            return (React.createElement("div", { key: metric, style: styles.currentValue },
                React.createElement("div", { style: {
                        ...styles.valueDot,
                        backgroundColor: getMetricColor(metric)
                    } }),
                React.createElement("div", { style: styles.valueContent },
                    React.createElement("div", { style: styles.valueLabel }, getMetricLabel(metric)),
                    React.createElement("div", { style: styles.valueNumber },
                        value !== undefined ? value.toFixed(1) : '--',
                        React.createElement("span", { style: styles.valueUnit }, getMetricUnit(metric))))));
        })),
        React.createElement("div", { style: styles.charts }, (selectedMetric === 'all' ? metrics : [selectedMetric]).map(metric => (React.createElement("div", { key: metric, style: styles.chartContainer },
            React.createElement("div", { style: styles.chartHeader },
                React.createElement("h3", { style: styles.chartTitle },
                    React.createElement("span", { style: {
                            ...styles.chartDot,
                            backgroundColor: getMetricColor(metric)
                        } }),
                    getMetricLabel(metric)),
                React.createElement("div", { style: styles.chartStats }, (() => {
                    const key = service ? `${metric}_${service}` : metric;
                    const data = chartData[key] || [];
                    if (data.length === 0)
                        return null;
                    const values = data.map(d => d.value);
                    const avg = values.reduce((a, b) => a + b, 0) / values.length;
                    const max = Math.max(...values);
                    const min = Math.min(...values);
                    return (React.createElement(React.Fragment, null,
                        React.createElement("span", null,
                            "Avg: ",
                            avg.toFixed(1),
                            getMetricUnit(metric)),
                        React.createElement("span", null,
                            "Max: ",
                            max.toFixed(1),
                            getMetricUnit(metric)),
                        React.createElement("span", null,
                            "Min: ",
                            min.toFixed(1),
                            getMetricUnit(metric))));
                })())),
            React.createElement("canvas", { ref: (el) => {
                    canvasRefs.current[metric] = el;
                }, style: styles.canvas })))))));
};
const styles = {
    container: {
        padding: '24px',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'
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
        marginBottom: '24px'
    },
    title: {
        fontSize: '24px',
        fontWeight: 700,
        color: '#111827',
        margin: 0
    },
    serviceLabel: {
        fontSize: '14px',
        color: '#6b7280',
        backgroundColor: '#f3f4f6',
        padding: '6px 12px',
        borderRadius: '6px'
    },
    metricSelector: {
        display: 'flex',
        gap: '8px',
        marginBottom: '24px',
        flexWrap: 'wrap'
    },
    metricButton: {
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
        padding: '8px 16px',
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderLeft: '3px solid #e5e7eb',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 500,
        color: '#6b7280',
        cursor: 'pointer',
        transition: 'all 0.2s'
    },
    metricButtonActive: {
        backgroundColor: '#f3f4f6',
        color: '#111827',
        borderColor: 'inherit'
    },
    metricDot: {
        width: '8px',
        height: '8px',
        borderRadius: '50%'
    },
    currentValues: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: '16px',
        marginBottom: '24px'
    },
    currentValue: {
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
        padding: '16px',
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px'
    },
    valueDot: {
        width: '12px',
        height: '12px',
        borderRadius: '50%',
        flexShrink: 0
    },
    valueContent: {
        flex: 1
    },
    valueLabel: {
        fontSize: '13px',
        color: '#6b7280',
        marginBottom: '4px'
    },
    valueNumber: {
        fontSize: '24px',
        fontWeight: 700,
        color: '#111827'
    },
    valueUnit: {
        fontSize: '14px',
        fontWeight: 400,
        color: '#6b7280',
        marginLeft: '4px'
    },
    charts: {
        display: 'flex',
        flexDirection: 'column',
        gap: '24px'
    },
    chartContainer: {
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        padding: '20px'
    },
    chartHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '16px'
    },
    chartTitle: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontSize: '16px',
        fontWeight: 600,
        color: '#111827',
        margin: 0
    },
    chartDot: {
        width: '10px',
        height: '10px',
        borderRadius: '50%'
    },
    chartStats: {
        display: 'flex',
        gap: '16px',
        fontSize: '13px',
        color: '#6b7280'
    },
    canvas: {
        width: '100%',
        height: '300px',
        display: 'block'
    }
};
export default PerformanceMetrics;
//# sourceMappingURL=PerformanceMetrics.js.map