/**
 * Metrics Chart Component
 *
 * Real-time metrics visualization with drill-down capabilities.
 */

import React, { useMemo, useState } from 'react';
import { useTimeSeriesData } from './useAnalytics';
import type { TimeRange, AggregationWindow } from './types';

interface MetricsChartProps {
  metricName: string;
  timeRange: TimeRange;
  title?: string;
  yAxisLabel?: string;
  height?: number;
  aggregationWindow?: AggregationWindow;
  showLegend?: boolean;
  interactive?: boolean;
}

export function MetricsChart({
  metricName,
  timeRange,
  title,
  yAxisLabel,
  height = 400,
  aggregationWindow,
  showLegend = true,
  interactive = true,
}: MetricsChartProps) {
  const { data, loading, error, refetch } = useTimeSeriesData(metricName, timeRange, 5000);
  const [selectedPoint, setSelectedPoint] = useState<number | null>(null);
  const [zoomedRange, setZoomedRange] = useState<TimeRange | null>(null);

  // Process data for chart
  const chartData = useMemo(() => {
    if (!data || data.length === 0) return [];

    return data.map((point) => ({
      timestamp: new Date(point.timestamp),
      value: point.value,
      labels: point.labels,
    }));
  }, [data]);

  // Calculate statistics
  const stats = useMemo(() => {
    if (chartData.length === 0) {
      return { min: 0, max: 0, avg: 0, current: 0 };
    }

    const values = chartData.map((d) => d.value);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const avg = values.reduce((sum, v) => sum + v, 0) / values.length;
    const current = values[values.length - 1];

    return { min, max, avg, current };
  }, [chartData]);

  // SVG dimensions
  const svgWidth = 800;
  const svgHeight = height;
  const margin = { top: 20, right: 30, bottom: 50, left: 60 };
  const chartWidth = svgWidth - margin.left - margin.right;
  const chartHeight = svgHeight - margin.top - margin.bottom;

  // Scales
  const { xScale, yScale } = useMemo(() => {
    if (chartData.length === 0) {
      return {
        xScale: (t: Date) => 0,
        yScale: (v: number) => chartHeight,
      };
    }

    const timeExtent = zoomedRange
      ? [zoomedRange.start, zoomedRange.end]
      : [chartData[0].timestamp, chartData[chartData.length - 1].timestamp];

    const valueExtent = [stats.min, stats.max];
    const padding = (valueExtent[1] - valueExtent[0]) * 0.1;

    const xScale = (t: Date) => {
      const range = timeExtent[1].getTime() - timeExtent[0].getTime();
      const offset = t.getTime() - timeExtent[0].getTime();
      return (offset / range) * chartWidth;
    };

    const yScale = (v: number) => {
      const range = valueExtent[1] + padding - (valueExtent[0] - padding);
      const offset = v - (valueExtent[0] - padding);
      return chartHeight - (offset / range) * chartHeight;
    };

    return { xScale, yScale };
  }, [chartData, chartHeight, chartWidth, stats, zoomedRange]);

  // Generate line path
  const linePath = useMemo(() => {
    if (chartData.length === 0) return '';

    const path = chartData.map((d, i) => {
      const x = xScale(d.timestamp);
      const y = yScale(d.value);
      return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
    }).join(' ');

    return path;
  }, [chartData, xScale, yScale]);

  // Generate area path
  const areaPath = useMemo(() => {
    if (chartData.length === 0) return '';

    const topPath = chartData.map((d, i) => {
      const x = xScale(d.timestamp);
      const y = yScale(d.value);
      return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
    }).join(' ');

    const lastX = xScale(chartData[chartData.length - 1].timestamp);
    const firstX = xScale(chartData[0].timestamp);

    return `${topPath} L ${lastX} ${chartHeight} L ${firstX} ${chartHeight} Z`;
  }, [chartData, xScale, yScale, chartHeight]);

  // Handle point click for drill-down
  const handlePointClick = (index: number) => {
    if (!interactive) return;
    setSelectedPoint(index === selectedPoint ? null : index);
  };

  // Handle zoom
  const handleZoom = (start: Date, end: Date) => {
    if (!interactive) return;
    setZoomedRange({ start, end });
  };

  // Reset zoom
  const resetZoom = () => {
    setZoomedRange(null);
  };

  if (loading && chartData.length === 0) {
    return (
      <div className="metrics-chart loading" style={{ height }}>
        <div className="loading-spinner" />
        <p>Loading metric data...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="metrics-chart error" style={{ height }}>
        <p>Error: {error.message}</p>
        <button onClick={refetch}>Retry</button>
      </div>
    );
  }

  if (chartData.length === 0) {
    return (
      <div className="metrics-chart empty" style={{ height }}>
        <p>No data available for this time range</p>
      </div>
    );
  }

  return (
    <div className="metrics-chart">
      {/* Header */}
      <div className="chart-header">
        {title && <h3>{title}</h3>}
        <div className="chart-controls">
          {zoomedRange && (
            <button className="reset-zoom-btn" onClick={resetZoom}>
              Reset Zoom
            </button>
          )}
          <button className="refresh-btn" onClick={refetch}>
            🔄
          </button>
        </div>
      </div>

      {/* Statistics */}
      <div className="chart-stats">
        <Stat label="Current" value={stats.current.toFixed(2)} />
        <Stat label="Average" value={stats.avg.toFixed(2)} />
        <Stat label="Min" value={stats.min.toFixed(2)} />
        <Stat label="Max" value={stats.max.toFixed(2)} />
      </div>

      {/* SVG Chart */}
      <svg
        width={svgWidth}
        height={svgHeight}
        className="chart-svg"
        style={{ width: '100%', height }}
      >
        <g transform={`translate(${margin.left}, ${margin.top})`}>
          {/* Grid lines */}
          <g className="grid">
            {[0, 0.25, 0.5, 0.75, 1].map((ratio) => {
              const y = chartHeight * ratio;
              const value = stats.max - (stats.max - stats.min) * ratio;
              return (
                <g key={ratio}>
                  <line
                    x1={0}
                    y1={y}
                    x2={chartWidth}
                    y2={y}
                    stroke="#e0e0e0"
                    strokeDasharray="2,2"
                  />
                  <text
                    x={-10}
                    y={y}
                    textAnchor="end"
                    alignmentBaseline="middle"
                    fontSize="10"
                    fill="#666"
                  >
                    {value.toFixed(0)}
                  </text>
                </g>
              );
            })}
          </g>

          {/* Area fill */}
          <path
            d={areaPath}
            fill="rgba(59, 130, 246, 0.1)"
            className="chart-area"
          />

          {/* Line */}
          <path
            d={linePath}
            fill="none"
            stroke="rgb(59, 130, 246)"
            strokeWidth={2}
            className="chart-line"
          />

          {/* Data points */}
          {interactive && chartData.map((d, i) => (
            <circle
              key={i}
              cx={xScale(d.timestamp)}
              cy={yScale(d.value)}
              r={selectedPoint === i ? 6 : 3}
              fill={selectedPoint === i ? 'rgb(239, 68, 68)' : 'rgb(59, 130, 246)'}
              className="chart-point"
              onClick={() => handlePointClick(i)}
              style={{ cursor: 'pointer' }}
            />
          ))}

          {/* X-axis */}
          <g transform={`translate(0, ${chartHeight})`}>
            <line x1={0} y1={0} x2={chartWidth} y2={0} stroke="#666" />
            {chartData.filter((_, i) => i % Math.ceil(chartData.length / 6) === 0).map((d, i) => (
              <g key={i}>
                <text
                  x={xScale(d.timestamp)}
                  y={20}
                  textAnchor="middle"
                  fontSize="10"
                  fill="#666"
                >
                  {d.timestamp.toLocaleTimeString()}
                </text>
              </g>
            ))}
          </g>

          {/* Y-axis label */}
          {yAxisLabel && (
            <text
              transform={`translate(-40, ${chartHeight / 2}) rotate(-90)`}
              textAnchor="middle"
              fontSize="12"
              fill="#666"
            >
              {yAxisLabel}
            </text>
          )}
        </g>
      </svg>

      {/* Selected point details */}
      {selectedPoint !== null && chartData[selectedPoint] && (
        <div className="selected-point-details">
          <h4>Point Details</h4>
          <p>Time: {chartData[selectedPoint].timestamp.toLocaleString()}</p>
          <p>Value: {chartData[selectedPoint].value.toFixed(4)}</p>
          {Object.keys(chartData[selectedPoint].labels).length > 0 && (
            <div>
              <p>Labels:</p>
              <ul>
                {Object.entries(chartData[selectedPoint].labels).map(([key, value]) => (
                  <li key={key}>
                    {key}: {value}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}

      {/* Legend */}
      {showLegend && (
        <div className="chart-legend">
          <div className="legend-item">
            <span className="legend-color" style={{ backgroundColor: 'rgb(59, 130, 246)' }} />
            <span>{metricName}</span>
          </div>
        </div>
      )}
    </div>
  );
}

// Helper component for statistics
function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="stat">
      <span className="stat-label">{label}:</span>
      <span className="stat-value">{value}</span>
    </div>
  );
}

/**
 * Multi-series chart for comparing multiple metrics
 */
export function MultiSeriesChart({
  metricNames,
  timeRange,
  title,
  height = 400,
}: {
  metricNames: string[];
  timeRange: TimeRange;
  title?: string;
  height?: number;
}) {
  const colors = [
    'rgb(59, 130, 246)',   // Blue
    'rgb(239, 68, 68)',    // Red
    'rgb(34, 197, 94)',    // Green
    'rgb(251, 191, 36)',   // Yellow
    'rgb(168, 85, 247)',   // Purple
  ];

  return (
    <div className="multi-series-chart">
      {title && <h3>{title}</h3>}
      <div className="series-container">
        {metricNames.map((name, index) => (
          <div key={name} className="series-item" style={{ borderLeftColor: colors[index % colors.length] }}>
            <MetricsChart
              metricName={name}
              timeRange={timeRange}
              title={name}
              height={height}
              showLegend={false}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
