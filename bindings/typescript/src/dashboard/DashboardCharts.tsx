/**
 * CADDY Enterprise Dashboard Charts Component v0.4.0
 *
 * Interactive charts with support for line, bar, pie, area, scatter, radar, and heatmap.
 * Includes zoom, pan, tooltips, and accessibility features.
 */

import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import type { ChartData, ChartType, ChartDataset, DataPoint, ChartOptions } from './types';
import { useDashboard } from './DashboardLayout';

/**
 * Chart component props
 */
export interface ChartProps {
  /** Chart data */
  data: ChartData;
  /** Override chart type */
  type?: ChartType;
  /** Chart height in pixels */
  height?: number;
  /** Chart width (responsive if not specified) */
  width?: string | number;
  /** Enable interactions */
  interactive?: boolean;
  /** Custom click handler */
  onClick?: (dataPoint: DataPoint, datasetIndex: number) => void;
  /** Custom class name */
  className?: string;
  /** Loading state */
  isLoading?: boolean;
}

/**
 * Main chart component
 */
export const Chart: React.FC<ChartProps> = ({
  data,
  type,
  height = 300,
  width = '100%',
  interactive = true,
  onClick,
  className = '',
  isLoading = false,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [hoveredPoint, setHoveredPoint] = useState<{
    datasetIndex: number;
    pointIndex: number;
  } | null>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height });

  const chartType = type || data.type;
  const { theme, accessibility } = useDashboard();

  /**
   * Update dimensions on resize
   */
  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        const rect = containerRef.current.getBoundingClientRect();
        setDimensions({ width: rect.width, height });
      }
    };

    updateDimensions();
    window.addEventListener('resize', updateDimensions);
    return () => window.removeEventListener('resize', updateDimensions);
  }, [height]);

  /**
   * Draw chart on canvas
   */
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas dimensions
    canvas.width = dimensions.width;
    canvas.height = dimensions.height;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw based on chart type
    switch (chartType) {
      case 'line':
        drawLineChart(ctx, data, dimensions, theme, hoveredPoint);
        break;
      case 'bar':
        drawBarChart(ctx, data, dimensions, theme, hoveredPoint);
        break;
      case 'pie':
        drawPieChart(ctx, data, dimensions, theme, hoveredPoint);
        break;
      case 'area':
        drawAreaChart(ctx, data, dimensions, theme, hoveredPoint);
        break;
      case 'scatter':
        drawScatterChart(ctx, data, dimensions, theme, hoveredPoint);
        break;
      case 'radar':
        drawRadarChart(ctx, data, dimensions, theme, hoveredPoint);
        break;
      case 'heatmap':
        drawHeatmapChart(ctx, data, dimensions, theme);
        break;
    }
  }, [data, chartType, dimensions, theme, hoveredPoint]);

  /**
   * Handle mouse move for tooltips
   */
  const handleMouseMove = useCallback(
    (event: React.MouseEvent<HTMLCanvasElement>) => {
      if (!interactive || !canvasRef.current) return;

      const rect = canvasRef.current.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;

      // Find closest data point
      const point = findNearestPoint(x, y, data, dimensions, chartType);
      setHoveredPoint(point);
    },
    [interactive, data, dimensions, chartType]
  );

  /**
   * Handle click
   */
  const handleClick = useCallback(
    (event: React.MouseEvent<HTMLCanvasElement>) => {
      if (!onClick || !hoveredPoint) return;

      const dataset = data.datasets[hoveredPoint.datasetIndex];
      const dataPoint = dataset.data[hoveredPoint.pointIndex];

      if (typeof dataPoint === 'number') {
        onClick(
          { x: hoveredPoint.pointIndex, y: dataPoint },
          hoveredPoint.datasetIndex
        );
      } else {
        onClick(dataPoint as DataPoint, hoveredPoint.datasetIndex);
      }
    },
    [onClick, hoveredPoint, data]
  );

  if (isLoading) {
    return (
      <div
        className={`chart-container skeleton ${className}`}
        style={{ ...styles.container, height }}
      >
        <div style={styles.skeleton} />
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className={`chart-container ${className}`}
      style={{ ...styles.container, height }}
      role="img"
      aria-label={`${chartType} chart: ${data.title}`}
    >
      {/* Title */}
      {data.title && (
        <h3 style={styles.title} id={`chart-title-${data.id}`}>
          {data.title}
        </h3>
      )}

      {/* Canvas */}
      <canvas
        ref={canvasRef}
        onMouseMove={handleMouseMove}
        onMouseLeave={() => setHoveredPoint(null)}
        onClick={handleClick}
        style={styles.canvas}
        aria-labelledby={`chart-title-${data.id}`}
        role="img"
      />

      {/* Tooltip */}
      {hoveredPoint && data.options.showTooltips !== false && (
        <ChartTooltip
          data={data}
          hoveredPoint={hoveredPoint}
          containerRef={containerRef}
        />
      )}

      {/* Legend */}
      {data.options.showLegend !== false && (
        <ChartLegend data={data} accessibility={accessibility} />
      )}

      {/* Last updated */}
      <div style={styles.footer}>
        <span style={styles.lastUpdated}>
          Updated: {new Date(data.lastUpdated).toLocaleString()}
        </span>
      </div>
    </div>
  );
};

/**
 * Chart tooltip component
 */
interface ChartTooltipProps {
  data: ChartData;
  hoveredPoint: { datasetIndex: number; pointIndex: number };
  containerRef: React.RefObject<HTMLDivElement | null>;
}

const ChartTooltip: React.FC<ChartTooltipProps> = ({ data, hoveredPoint, containerRef }) => {
  const dataset = data.datasets[hoveredPoint.datasetIndex];
  const dataPoint = dataset.data[hoveredPoint.pointIndex];
  const label = data.labels?.[hoveredPoint.pointIndex] || `Point ${hoveredPoint.pointIndex}`;

  const value = typeof dataPoint === 'number' ? dataPoint : dataPoint.y;
  const formattedValue = typeof value === 'number' ? value.toFixed(2) : value;

  return (
    <div style={styles.tooltip} role="tooltip" aria-live="polite">
      <div style={styles.tooltipLabel}>{label}</div>
      <div style={styles.tooltipValue}>
        <span
          style={{
            ...styles.tooltipColor,
            backgroundColor: dataset.color || dataset.backgroundColor,
          }}
        />
        <strong>{dataset.label}:</strong> {formattedValue}
      </div>
    </div>
  );
};

/**
 * Chart legend component
 */
interface ChartLegendProps {
  data: ChartData;
  accessibility: any;
}

const ChartLegend: React.FC<ChartLegendProps> = ({ data, accessibility }) => {
  const position = data.options.legendPosition || 'bottom';

  return (
    <div
      style={{
        ...styles.legend,
        ...(position === 'top' && styles.legendTop),
        ...(position === 'bottom' && styles.legendBottom),
      }}
      role="list"
      aria-label="Chart legend"
    >
      {data.datasets.map((dataset, index) => (
        <div
          key={index}
          style={styles.legendItem}
          role="listitem"
          tabIndex={accessibility.keyboardNavigation ? 0 : undefined}
        >
          <span
            style={{
              ...styles.legendColor,
              backgroundColor: dataset.color || dataset.backgroundColor || '#ccc',
            }}
            aria-hidden="true"
          />
          <span style={styles.legendLabel}>{dataset.label}</span>
        </div>
      ))}
    </div>
  );
};

/**
 * Draw line chart
 */
function drawLineChart(
  ctx: CanvasRenderingContext2D,
  data: ChartData,
  dimensions: { width: number; height: number },
  theme: string,
  hoveredPoint: { datasetIndex: number; pointIndex: number } | null
): void {
  const padding = 40;
  const chartWidth = dimensions.width - padding * 2;
  const chartHeight = dimensions.height - padding * 2;

  // Draw grid
  if (data.options.showGrid !== false) {
    drawGrid(ctx, padding, chartWidth, chartHeight, theme);
  }

  // Draw axes
  drawAxes(ctx, padding, chartWidth, chartHeight, data, theme);

  // Draw datasets
  data.datasets.forEach((dataset, datasetIndex) => {
    const points = dataset.data.map((point, index) => {
      const value = typeof point === 'number' ? point : point.y;
      const x = padding + (index / (dataset.data.length - 1)) * chartWidth;
      const y = padding + chartHeight - (value / getMaxValue(data)) * chartHeight;
      return { x, y, value };
    });

    // Draw line
    ctx.beginPath();
    ctx.strokeStyle = dataset.borderColor || dataset.color || '#1976d2';
    ctx.lineWidth = dataset.borderWidth || 2;
    ctx.lineJoin = 'round';
    ctx.lineCap = 'round';

    points.forEach((point, index) => {
      if (index === 0) {
        ctx.moveTo(point.x, point.y);
      } else {
        ctx.lineTo(point.x, point.y);
      }
    });

    ctx.stroke();

    // Draw points
    points.forEach((point, pointIndex) => {
      const isHovered =
        hoveredPoint?.datasetIndex === datasetIndex &&
        hoveredPoint?.pointIndex === pointIndex;

      ctx.beginPath();
      ctx.arc(point.x, point.y, isHovered ? 6 : dataset.pointRadius || 4, 0, Math.PI * 2);
      ctx.fillStyle = dataset.backgroundColor || dataset.color || '#1976d2';
      ctx.fill();

      if (isHovered) {
        ctx.strokeStyle = '#fff';
        ctx.lineWidth = 2;
        ctx.stroke();
      }
    });
  });
}

/**
 * Draw bar chart
 */
function drawBarChart(
  ctx: CanvasRenderingContext2D,
  data: ChartData,
  dimensions: { width: number; height: number },
  theme: string,
  hoveredPoint: { datasetIndex: number; pointIndex: number } | null
): void {
  const padding = 40;
  const chartWidth = dimensions.width - padding * 2;
  const chartHeight = dimensions.height - padding * 2;

  // Draw grid
  if (data.options.showGrid !== false) {
    drawGrid(ctx, padding, chartWidth, chartHeight, theme);
  }

  // Draw axes
  drawAxes(ctx, padding, chartWidth, chartHeight, data, theme);

  const barCount = data.datasets[0].data.length;
  const datasetCount = data.datasets.length;
  const groupWidth = chartWidth / barCount;
  const barWidth = groupWidth / (datasetCount + 1);
  const maxValue = getMaxValue(data);

  // Draw bars
  data.datasets.forEach((dataset, datasetIndex) => {
    dataset.data.forEach((point, pointIndex) => {
      const value = typeof point === 'number' ? point : point.y;
      const x = padding + pointIndex * groupWidth + datasetIndex * barWidth;
      const barHeight = (value / maxValue) * chartHeight;
      const y = padding + chartHeight - barHeight;

      const isHovered =
        hoveredPoint?.datasetIndex === datasetIndex &&
        hoveredPoint?.pointIndex === pointIndex;

      ctx.fillStyle = dataset.backgroundColor || dataset.color || '#1976d2';
      ctx.globalAlpha = isHovered ? 1 : 0.8;
      ctx.fillRect(x, y, barWidth, barHeight);
      ctx.globalAlpha = 1;
    });
  });
}

/**
 * Draw pie chart
 */
function drawPieChart(
  ctx: CanvasRenderingContext2D,
  data: ChartData,
  dimensions: { width: number; height: number },
  theme: string,
  hoveredPoint: { datasetIndex: number; pointIndex: number } | null
): void {
  const centerX = dimensions.width / 2;
  const centerY = dimensions.height / 2;
  const radius = Math.min(centerX, centerY) - 40;

  const dataset = data.datasets[0];
  const total = dataset.data.reduce((sum: number, point) => {
    const value = typeof point === 'number' ? point : point.y;
    return sum + value;
  }, 0);

  let currentAngle = -Math.PI / 2;
  const colors = data.options.colors || generateColors(dataset.data.length);

  dataset.data.forEach((point, index) => {
    const value: number = typeof point === 'number' ? point : point.y;
    const sliceAngle = (value / total) * Math.PI * 2;
    const isHovered =
      hoveredPoint?.datasetIndex === 0 && hoveredPoint?.pointIndex === index;

    const drawRadius = isHovered ? radius + 10 : radius;

    ctx.beginPath();
    ctx.moveTo(centerX, centerY);
    ctx.arc(centerX, centerY, drawRadius, currentAngle, currentAngle + sliceAngle);
    ctx.closePath();

    ctx.fillStyle = colors[index];
    ctx.fill();

    ctx.strokeStyle = theme === 'dark' ? '#333' : '#fff';
    ctx.lineWidth = 2;
    ctx.stroke();

    currentAngle += sliceAngle;
  });
}

/**
 * Draw area chart
 */
function drawAreaChart(
  ctx: CanvasRenderingContext2D,
  data: ChartData,
  dimensions: { width: number; height: number },
  theme: string,
  hoveredPoint: { datasetIndex: number; pointIndex: number } | null
): void {
  const padding = 40;
  const chartWidth = dimensions.width - padding * 2;
  const chartHeight = dimensions.height - padding * 2;

  // Draw grid
  if (data.options.showGrid !== false) {
    drawGrid(ctx, padding, chartWidth, chartHeight, theme);
  }

  // Draw axes
  drawAxes(ctx, padding, chartWidth, chartHeight, data, theme);

  // Draw datasets
  data.datasets.forEach((dataset, datasetIndex) => {
    const points = dataset.data.map((point, index) => {
      const value = typeof point === 'number' ? point : point.y;
      const x = padding + (index / (dataset.data.length - 1)) * chartWidth;
      const y = padding + chartHeight - (value / getMaxValue(data)) * chartHeight;
      return { x, y, value };
    });

    // Draw filled area
    ctx.beginPath();
    ctx.moveTo(points[0].x, padding + chartHeight);

    points.forEach((point, index) => {
      if (index === 0) {
        ctx.lineTo(point.x, point.y);
      } else {
        ctx.lineTo(point.x, point.y);
      }
    });

    ctx.lineTo(points[points.length - 1].x, padding + chartHeight);
    ctx.closePath();

    ctx.fillStyle = dataset.backgroundColor || dataset.color || '#1976d2';
    ctx.globalAlpha = 0.3;
    ctx.fill();
    ctx.globalAlpha = 1;

    // Draw line
    ctx.beginPath();
    points.forEach((point, index) => {
      if (index === 0) {
        ctx.moveTo(point.x, point.y);
      } else {
        ctx.lineTo(point.x, point.y);
      }
    });

    ctx.strokeStyle = dataset.borderColor || dataset.color || '#1976d2';
    ctx.lineWidth = dataset.borderWidth || 2;
    ctx.stroke();
  });
}

/**
 * Draw scatter chart
 */
function drawScatterChart(
  ctx: CanvasRenderingContext2D,
  data: ChartData,
  dimensions: { width: number; height: number },
  theme: string,
  hoveredPoint: { datasetIndex: number; pointIndex: number } | null
): void {
  const padding = 40;
  const chartWidth = dimensions.width - padding * 2;
  const chartHeight = dimensions.height - padding * 2;

  drawGrid(ctx, padding, chartWidth, chartHeight, theme);
  drawAxes(ctx, padding, chartWidth, chartHeight, data, theme);

  const maxValue = getMaxValue(data);

  data.datasets.forEach((dataset, datasetIndex) => {
    dataset.data.forEach((point, pointIndex) => {
      const dataPoint = point as DataPoint;
      const x = padding + (Number(dataPoint.x) / maxValue) * chartWidth;
      const y = padding + chartHeight - (dataPoint.y / maxValue) * chartHeight;

      const isHovered =
        hoveredPoint?.datasetIndex === datasetIndex &&
        hoveredPoint?.pointIndex === pointIndex;

      ctx.beginPath();
      ctx.arc(x, y, isHovered ? 8 : 5, 0, Math.PI * 2);
      ctx.fillStyle = dataset.backgroundColor || dataset.color || '#1976d2';
      ctx.fill();

      if (isHovered) {
        ctx.strokeStyle = '#fff';
        ctx.lineWidth = 2;
        ctx.stroke();
      }
    });
  });
}

/**
 * Draw radar chart
 */
function drawRadarChart(
  ctx: CanvasRenderingContext2D,
  data: ChartData,
  dimensions: { width: number; height: number },
  theme: string,
  hoveredPoint: { datasetIndex: number; pointIndex: number } | null
): void {
  const centerX = dimensions.width / 2;
  const centerY = dimensions.height / 2;
  const radius = Math.min(centerX, centerY) - 60;
  const pointCount = data.datasets[0].data.length;
  const angleStep = (Math.PI * 2) / pointCount;

  // Draw grid circles
  for (let i = 1; i <= 5; i++) {
    ctx.beginPath();
    ctx.arc(centerX, centerY, (radius / 5) * i, 0, Math.PI * 2);
    ctx.strokeStyle = theme === 'dark' ? '#444' : '#e0e0e0';
    ctx.lineWidth = 1;
    ctx.stroke();
  }

  // Draw axes
  for (let i = 0; i < pointCount; i++) {
    const angle = i * angleStep - Math.PI / 2;
    ctx.beginPath();
    ctx.moveTo(centerX, centerY);
    ctx.lineTo(
      centerX + Math.cos(angle) * radius,
      centerY + Math.sin(angle) * radius
    );
    ctx.strokeStyle = theme === 'dark' ? '#444' : '#e0e0e0';
    ctx.stroke();
  }

  // Draw data
  data.datasets.forEach((dataset) => {
    const maxValue = getMaxValue(data);
    ctx.beginPath();

    dataset.data.forEach((point, index) => {
      const value = typeof point === 'number' ? point : point.y;
      const angle = index * angleStep - Math.PI / 2;
      const distance = (value / maxValue) * radius;
      const x = centerX + Math.cos(angle) * distance;
      const y = centerY + Math.sin(angle) * distance;

      if (index === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    });

    ctx.closePath();
    ctx.fillStyle = dataset.backgroundColor || dataset.color || '#1976d2';
    ctx.globalAlpha = 0.3;
    ctx.fill();
    ctx.globalAlpha = 1;

    ctx.strokeStyle = dataset.borderColor || dataset.color || '#1976d2';
    ctx.lineWidth = 2;
    ctx.stroke();
  });
}

/**
 * Draw heatmap chart
 */
function drawHeatmapChart(
  ctx: CanvasRenderingContext2D,
  data: ChartData,
  dimensions: { width: number; height: number },
  theme: string
): void {
  const padding = 40;
  const rows = data.datasets.length;
  const cols = data.datasets[0].data.length;
  const cellWidth = (dimensions.width - padding * 2) / cols;
  const cellHeight = (dimensions.height - padding * 2) / rows;

  const maxValue = getMaxValue(data);

  data.datasets.forEach((dataset, rowIndex) => {
    dataset.data.forEach((point, colIndex) => {
      const value = typeof point === 'number' ? point : point.y;
      const intensity = value / maxValue;

      const x = padding + colIndex * cellWidth;
      const y = padding + rowIndex * cellHeight;

      ctx.fillStyle = `rgba(25, 118, 210, ${intensity})`;
      ctx.fillRect(x, y, cellWidth, cellHeight);

      ctx.strokeStyle = theme === 'dark' ? '#333' : '#fff';
      ctx.lineWidth = 1;
      ctx.strokeRect(x, y, cellWidth, cellHeight);
    });
  });
}

/**
 * Helper: Draw grid
 */
function drawGrid(
  ctx: CanvasRenderingContext2D,
  padding: number,
  width: number,
  height: number,
  theme: string
): void {
  ctx.strokeStyle = theme === 'dark' ? '#444' : '#e0e0e0';
  ctx.lineWidth = 1;

  // Horizontal lines
  for (let i = 0; i <= 5; i++) {
    const y = padding + (height / 5) * i;
    ctx.beginPath();
    ctx.moveTo(padding, y);
    ctx.lineTo(padding + width, y);
    ctx.stroke();
  }

  // Vertical lines
  for (let i = 0; i <= 5; i++) {
    const x = padding + (width / 5) * i;
    ctx.beginPath();
    ctx.moveTo(x, padding);
    ctx.lineTo(x, padding + height);
    ctx.stroke();
  }
}

/**
 * Helper: Draw axes
 */
function drawAxes(
  ctx: CanvasRenderingContext2D,
  padding: number,
  width: number,
  height: number,
  data: ChartData,
  theme: string
): void {
  ctx.strokeStyle = theme === 'dark' ? '#666' : '#333';
  ctx.lineWidth = 2;

  // X-axis
  ctx.beginPath();
  ctx.moveTo(padding, padding + height);
  ctx.lineTo(padding + width, padding + height);
  ctx.stroke();

  // Y-axis
  ctx.beginPath();
  ctx.moveTo(padding, padding);
  ctx.lineTo(padding, padding + height);
  ctx.stroke();
}

/**
 * Helper: Get max value from all datasets
 */
function getMaxValue(data: ChartData): number {
  let max = 0;
  data.datasets.forEach((dataset) => {
    dataset.data.forEach((point) => {
      const value = typeof point === 'number' ? point : point.y;
      max = Math.max(max, value);
    });
  });
  return max || 100;
}

/**
 * Helper: Find nearest point to mouse position
 */
function findNearestPoint(
  x: number,
  y: number,
  data: ChartData,
  dimensions: { width: number; height: number },
  chartType: ChartType
): { datasetIndex: number; pointIndex: number } | null {
  // Simplified nearest point detection
  const padding = 40;
  const chartWidth = dimensions.width - padding * 2;
  const pointCount = data.datasets[0].data.length;
  const pointIndex = Math.floor(((x - padding) / chartWidth) * pointCount);

  if (pointIndex >= 0 && pointIndex < pointCount) {
    return { datasetIndex: 0, pointIndex };
  }

  return null;
}

/**
 * Helper: Generate color palette
 */
function generateColors(count: number): string[] {
  const colors = [
    '#1976d2',
    '#f44336',
    '#4caf50',
    '#ff9800',
    '#9c27b0',
    '#00bcd4',
    '#ffeb3b',
    '#795548',
  ];

  return Array.from({ length: count }, (_, i) => colors[i % colors.length]);
}

/**
 * Component styles
 */
const styles: Record<string, React.CSSProperties> = {
  container: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    border: '1px solid var(--color-border, #e0e0e0)',
    padding: 20,
    position: 'relative',
  },
  title: {
    margin: '0 0 16px 0',
    fontSize: 16,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
  },
  canvas: {
    display: 'block',
    width: '100%',
    cursor: 'crosshair',
  },
  tooltip: {
    position: 'absolute',
    top: 20,
    right: 20,
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    color: '#fff',
    padding: '8px 12px',
    borderRadius: 4,
    fontSize: 12,
    pointerEvents: 'none',
    zIndex: 1000,
  },
  tooltipLabel: {
    fontWeight: 600,
    marginBottom: 4,
  },
  tooltipValue: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
  },
  tooltipColor: {
    width: 12,
    height: 12,
    borderRadius: 2,
  },
  legend: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: 16,
    padding: '12px 0',
  },
  legendTop: {
    borderBottom: '1px solid var(--color-divider, #e0e0e0)',
    marginBottom: 16,
  },
  legendBottom: {
    borderTop: '1px solid var(--color-divider, #e0e0e0)',
    marginTop: 16,
  },
  legendItem: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    fontSize: 12,
    cursor: 'pointer',
  },
  legendColor: {
    width: 16,
    height: 16,
    borderRadius: 2,
  },
  legendLabel: {
    color: 'var(--color-text, #333)',
  },
  footer: {
    marginTop: 12,
    paddingTop: 12,
    borderTop: '1px solid var(--color-divider, #e0e0e0)',
  },
  lastUpdated: {
    fontSize: 11,
    color: 'var(--color-text-secondary, #999)',
  },
  skeleton: {
    width: '100%',
    height: '100%',
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 4,
    animation: 'pulse 1.5s ease-in-out infinite',
  },
};

export default Chart;
