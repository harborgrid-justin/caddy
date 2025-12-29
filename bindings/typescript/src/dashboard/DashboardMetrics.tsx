/**
 * CADDY Enterprise Dashboard Metrics Component v0.4.0
 *
 * Real-time KPI cards with trends, sparklines, and comparison data.
 * Supports accessibility, theming, and automatic refresh.
 */

import React, { useState, useEffect, useCallback, useMemo } from 'react';
import type { MetricData, TrendDirection, LoadingState } from './types';
import { useDashboard } from './DashboardLayout';

/**
 * Metric card props
 */
export interface MetricCardProps {
  /** Metric data */
  metric: MetricData;
  /** Show trend sparkline */
  showSparkline?: boolean;
  /** Show comparison */
  showComparison?: boolean;
  /** Show target progress */
  showProgress?: boolean;
  /** Card size */
  size?: 'small' | 'medium' | 'large';
  /** Custom click handler */
  onClick?: (metric: MetricData) => void;
  /** Custom class name */
  className?: string;
  /** Loading state */
  isLoading?: boolean;
}

/**
 * Single metric card component
 */
export const MetricCard: React.FC<MetricCardProps> = ({
  metric,
  showSparkline = true,
  showComparison = true,
  showProgress = false,
  size = 'medium',
  onClick,
  className = '',
  isLoading = false,
}) => {
  const { theme, accessibility } = useDashboard();
  const [isHovered, setIsHovered] = useState(false);

  /**
   * Get trend icon
   */
  const getTrendIcon = (trend: TrendDirection): string => {
    switch (trend) {
      case 'up':
        return '↑';
      case 'down':
        return '↓';
      default:
        return '→';
    }
  };

  /**
   * Get trend color
   */
  const getTrendColor = (trend: TrendDirection): string => {
    switch (trend) {
      case 'up':
        return 'var(--color-success, #4caf50)';
      case 'down':
        return 'var(--color-error, #f44336)';
      default:
        return 'var(--color-text-secondary, #666)';
    }
  };

  /**
   * Format change percentage
   */
  const formatChange = (): string => {
    if (!metric.changePercent) return '';
    const sign = metric.changePercent >= 0 ? '+' : '';
    return `${sign}${metric.changePercent.toFixed(1)}%`;
  };

  /**
   * Render sparkline
   */
  const renderSparkline = (): React.ReactNode => {
    if (!showSparkline || !metric.history || metric.history.length === 0) {
      return null;
    }

    const width = 100;
    const height = 30;
    const values = metric.history.map((p) => p.y);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const range = max - min || 1;

    const points = metric.history
      .map((point, i) => {
        const x = (i / (metric.history!.length - 1)) * width;
        const y = height - ((point.y - min) / range) * height;
        return `${x},${y}`;
      })
      .join(' ');

    return (
      <svg
        width={width}
        height={height}
        style={styles.sparkline}
        role="img"
        aria-label={`Trend chart for ${metric.name}`}
      >
        <polyline
          points={points}
          fill="none"
          stroke={getTrendColor(metric.trend)}
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    );
  };

  /**
   * Render progress bar
   */
  const renderProgress = (): React.ReactNode => {
    if (!showProgress || metric.progress === undefined) {
      return null;
    }

    return (
      <div style={styles.progressContainer}>
        <div style={styles.progressBar}>
          <div
            style={{
              ...styles.progressFill,
              width: `${Math.min(100, Math.max(0, metric.progress))}%`,
              backgroundColor: getTrendColor(metric.trend),
            }}
            role="progressbar"
            aria-valuenow={metric.progress}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-label={`Progress: ${metric.progress}%`}
          />
        </div>
        <span style={styles.progressText}>{metric.progress.toFixed(0)}%</span>
      </div>
    );
  };

  if (isLoading) {
    return (
      <div
        className={`metric-card skeleton ${className}`}
        style={{ ...styles.card, ...styles[`card${capitalize(size)}`] }}
      >
        <div style={styles.skeleton} />
      </div>
    );
  }

  return (
    <div
      className={`metric-card ${className} ${size}`}
      style={{
        ...styles.card,
        ...styles[`card${capitalize(size)}`],
        ...(isHovered && styles.cardHover),
        ...(onClick && { cursor: 'pointer' }),
        ...(metric.color && { borderLeftColor: metric.color, borderLeftWidth: 4 }),
      }}
      onClick={() => onClick?.(metric)}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      onFocus={() => setIsHovered(true)}
      onBlur={() => setIsHovered(false)}
      role={onClick ? 'button' : 'article'}
      tabIndex={accessibility.keyboardNavigation && onClick ? 0 : undefined}
      aria-label={`${metric.name}: ${metric.formattedValue}`}
    >
      {/* Header */}
      <div style={styles.header}>
        <div style={styles.headerLeft}>
          {metric.icon && <span style={styles.icon}>{metric.icon}</span>}
          <h3 style={styles.title}>{metric.name}</h3>
        </div>
        {metric.category && (
          <span style={styles.category} aria-label={`Category: ${metric.category}`}>
            {metric.category}
          </span>
        )}
      </div>

      {/* Value */}
      <div style={styles.valueContainer}>
        <div style={styles.value} aria-label={`Value: ${metric.formattedValue}`}>
          {metric.formattedValue}
        </div>
        {metric.unit && (
          <span style={styles.unit} aria-label={`Unit: ${metric.unit}`}>
            {metric.unit}
          </span>
        )}
      </div>

      {/* Comparison */}
      {showComparison && metric.changePercent !== undefined && (
        <div
          style={{
            ...styles.comparison,
            color: getTrendColor(metric.trend),
          }}
          aria-label={`Change: ${formatChange()}`}
        >
          <span style={styles.trendIcon}>{getTrendIcon(metric.trend)}</span>
          <span style={styles.changeText}>{formatChange()}</span>
          {metric.previousValue !== undefined && (
            <span style={styles.previousValue}>from {metric.previousValue}</span>
          )}
        </div>
      )}

      {/* Progress bar */}
      {renderProgress()}

      {/* Sparkline */}
      {renderSparkline()}

      {/* Target */}
      {metric.target !== undefined && (
        <div style={styles.target} aria-label={`Target: ${metric.target}`}>
          Target: {metric.target} {metric.unit}
        </div>
      )}

      {/* Last updated */}
      <div style={styles.footer}>
        <span style={styles.lastUpdated} aria-label={`Last updated: ${metric.lastUpdated}`}>
          Updated: {new Date(metric.lastUpdated).toLocaleString()}
        </span>
      </div>
    </div>
  );
};

/**
 * Metrics grid props
 */
export interface MetricsGridProps {
  /** Array of metrics to display */
  metrics: MetricData[];
  /** Grid columns */
  columns?: { xs: number; sm: number; md: number; lg: number; xl: number };
  /** Show sparklines */
  showSparklines?: boolean;
  /** Show comparisons */
  showComparisons?: boolean;
  /** Show progress bars */
  showProgress?: boolean;
  /** Card size */
  cardSize?: 'small' | 'medium' | 'large';
  /** On metric click */
  onMetricClick?: (metric: MetricData) => void;
  /** Auto refresh interval in ms */
  refreshInterval?: number;
  /** Data fetch function */
  onRefresh?: () => Promise<MetricData[]>;
  /** Custom class name */
  className?: string;
}

/**
 * Metrics grid component
 */
export const MetricsGrid: React.FC<MetricsGridProps> = ({
  metrics: initialMetrics,
  columns = { xs: 1, sm: 2, md: 3, lg: 4, xl: 4 },
  showSparklines = true,
  showComparisons = true,
  showProgress = false,
  cardSize = 'medium',
  onMetricClick,
  refreshInterval,
  onRefresh,
  className = '',
}) => {
  const [metrics, setMetrics] = useState<MetricData[]>(initialMetrics);
  const [loading, setLoading] = useState<LoadingState>({ isLoading: false });
  const [screenSize, setScreenSize] = useState<'xs' | 'sm' | 'md' | 'lg' | 'xl'>('lg');

  /**
   * Handle screen resize
   */
  useEffect(() => {
    const handleResize = () => {
      const width = window.innerWidth;
      if (width < 576) setScreenSize('xs');
      else if (width < 768) setScreenSize('sm');
      else if (width < 992) setScreenSize('md');
      else if (width < 1200) setScreenSize('lg');
      else setScreenSize('xl');
    };

    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  /**
   * Refresh metrics data
   */
  const refreshMetrics = useCallback(async () => {
    if (!onRefresh) return;

    setLoading({ isLoading: true, message: 'Refreshing metrics...' });
    try {
      const newMetrics = await onRefresh();
      setMetrics(newMetrics);
    } catch (error) {
      console.error('Failed to refresh metrics:', error);
    } finally {
      setLoading({ isLoading: false });
    }
  }, [onRefresh]);

  /**
   * Auto-refresh setup
   */
  useEffect(() => {
    if (!refreshInterval || !onRefresh) return;

    const interval = setInterval(refreshMetrics, refreshInterval);
    return () => clearInterval(interval);
  }, [refreshInterval, refreshMetrics, onRefresh]);

  /**
   * Update metrics when prop changes
   */
  useEffect(() => {
    setMetrics(initialMetrics);
  }, [initialMetrics]);

  /**
   * Get grid columns for current screen size
   */
  const gridColumns = useMemo(() => {
    return columns[screenSize] || 4;
  }, [columns, screenSize]);

  return (
    <div
      className={`metrics-grid ${className}`}
      style={{
        ...styles.grid,
        gridTemplateColumns: `repeat(${gridColumns}, 1fr)`,
      }}
      role="region"
      aria-label="Metrics dashboard"
    >
      {metrics.map((metric) => (
        <MetricCard
          key={metric.id}
          metric={metric}
          showSparkline={showSparklines}
          showComparison={showComparisons}
          showProgress={showProgress}
          size={cardSize}
          onClick={onMetricClick}
          isLoading={loading.isLoading}
        />
      ))}
    </div>
  );
};

/**
 * Utility function to capitalize first letter
 */
function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Component styles
 */
const styles: Record<string, React.CSSProperties> = {
  grid: {
    display: 'grid',
    gap: 16,
    width: '100%',
  },
  card: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    border: '1px solid var(--color-border, #e0e0e0)',
    padding: 20,
    transition: 'transform var(--animation-duration, 200ms), box-shadow var(--animation-duration, 200ms)',
    position: 'relative',
    overflow: 'hidden',
  },
  cardHover: {
    transform: 'translateY(-2px)',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.1)',
  },
  cardSmall: {
    padding: 12,
  },
  cardMedium: {
    padding: 20,
  },
  cardLarge: {
    padding: 24,
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  headerLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
  },
  icon: {
    fontSize: 24,
    lineHeight: 1,
  },
  title: {
    margin: 0,
    fontSize: 14,
    fontWeight: 500,
    color: 'var(--color-text-secondary, #666)',
    lineHeight: 1.4,
  },
  category: {
    fontSize: 11,
    padding: '2px 8px',
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 12,
    color: 'var(--color-text-secondary, #666)',
    fontWeight: 500,
  },
  valueContainer: {
    display: 'flex',
    alignItems: 'baseline',
    gap: 8,
    marginBottom: 8,
  },
  value: {
    fontSize: 32,
    fontWeight: 700,
    color: 'var(--color-text, #333)',
    lineHeight: 1.2,
  },
  unit: {
    fontSize: 14,
    color: 'var(--color-text-secondary, #666)',
    fontWeight: 500,
  },
  comparison: {
    display: 'flex',
    alignItems: 'center',
    gap: 4,
    fontSize: 14,
    fontWeight: 600,
    marginBottom: 12,
  },
  trendIcon: {
    fontSize: 16,
    lineHeight: 1,
  },
  changeText: {
    fontWeight: 600,
  },
  previousValue: {
    fontSize: 12,
    marginLeft: 4,
    color: 'var(--color-text-secondary, #666)',
    fontWeight: 400,
  },
  progressContainer: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    marginBottom: 12,
  },
  progressBar: {
    flex: 1,
    height: 6,
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 3,
    overflow: 'hidden',
  },
  progressFill: {
    height: '100%',
    borderRadius: 3,
    transition: 'width var(--animation-duration, 200ms)',
  },
  progressText: {
    fontSize: 12,
    fontWeight: 600,
    color: 'var(--color-text-secondary, #666)',
    minWidth: 40,
    textAlign: 'right',
  },
  sparkline: {
    marginTop: 8,
    marginBottom: 8,
    opacity: 0.8,
  },
  target: {
    fontSize: 12,
    color: 'var(--color-text-secondary, #666)',
    marginTop: 8,
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
    height: 120,
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 4,
    animation: 'pulse 1.5s ease-in-out infinite',
  },
};

export default MetricsGrid;
