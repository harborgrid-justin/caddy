/**
 * CADDY Enterprise Executive Overview Component v0.4.0
 *
 * C-suite executive summary view with key metrics, strategic initiatives,
 * risk indicators, and actionable insights for senior leadership.
 */

import React, { useState, useEffect, useMemo } from 'react';
import type {
  ExecutiveOverview as ExecutiveOverviewData,
  MetricData,
  Initiative,
  RiskIndicator,
  Highlight,
  Recommendation,
  TimeRange,
} from './types';
import { useDashboard } from './DashboardLayout';

/**
 * Executive overview props
 */
export interface ExecutiveOverviewProps {
  /** Overview data */
  data: ExecutiveOverviewData;
  /** Time period selector */
  showPeriodSelector?: boolean;
  /** On period change */
  onPeriodChange?: (period: TimeRange) => void;
  /** Show print button */
  showPrintButton?: boolean;
  /** Custom class name */
  className?: string;
}

/**
 * Executive overview component
 */
export const ExecutiveOverview: React.FC<ExecutiveOverviewProps> = ({
  data,
  showPeriodSelector = true,
  onPeriodChange,
  showPrintButton = true,
  className = '',
}) => {
  const [selectedPeriod, setSelectedPeriod] = useState<TimeRange>(data.period);
  const { theme, accessibility } = useDashboard();

  /**
   * Handle period change
   */
  const handlePeriodChange = (period: TimeRange) => {
    setSelectedPeriod(period);
    if (onPeriodChange) {
      onPeriodChange(period);
    }
  };

  /**
   * Handle print
   */
  const handlePrint = () => {
    window.print();
  };

  /**
   * Calculate overall health score
   */
  const healthScore = useMemo(() => {
    if (!data.performance) return 0;
    return data.performance.score;
  }, [data.performance]);

  /**
   * Get health color
   */
  const getHealthColor = (score: number): string => {
    if (score >= 80) return 'var(--color-success, #4caf50)';
    if (score >= 60) return 'var(--color-warning, #ff9800)';
    return 'var(--color-error, #f44336)';
  };

  return (
    <div
      className={`executive-overview ${className}`}
      style={styles.container}
      role="article"
      aria-label="Executive overview"
    >
      {/* Header */}
      <div style={styles.header}>
        <div style={styles.headerLeft}>
          <h1 style={styles.title}>Executive Overview</h1>
          <p style={styles.subtitle}>
            Generated on {new Date(data.generatedAt).toLocaleDateString('en-US', {
              weekday: 'long',
              year: 'numeric',
              month: 'long',
              day: 'numeric',
            })}
          </p>
        </div>

        <div style={styles.headerRight}>
          {/* Period selector */}
          {showPeriodSelector && (
            <select
              value={selectedPeriod}
              onChange={(e) => handlePeriodChange(e.target.value as TimeRange)}
              style={styles.periodSelector}
              aria-label="Select time period"
            >
              <option value="1h">Last Hour</option>
              <option value="24h">Last 24 Hours</option>
              <option value="7d">Last 7 Days</option>
              <option value="30d">Last 30 Days</option>
              <option value="90d">Last 90 Days</option>
              <option value="1y">Last Year</option>
            </select>
          )}

          {/* Print button */}
          {showPrintButton && (
            <button
              onClick={handlePrint}
              style={styles.printButton}
              aria-label="Print overview"
            >
              🖨️ Print
            </button>
          )}
        </div>
      </div>

      {/* Overall Health Score */}
      <div style={styles.healthSection}>
        <div style={styles.healthCard}>
          <h2 style={styles.sectionTitle}>Overall Health Score</h2>
          <div
            style={{
              ...styles.healthScore,
              color: getHealthColor(healthScore),
            }}
            role="meter"
            aria-valuenow={healthScore}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-label={`Health score: ${healthScore} out of 100`}
          >
            {healthScore}
            <span style={styles.healthScoreUnit}>/100</span>
          </div>
          <div
            style={{
              ...styles.healthBar,
              width: `${healthScore}%`,
              backgroundColor: getHealthColor(healthScore),
            }}
            role="presentation"
          />
          <p style={styles.healthTrend}>
            {data.performance.trend === 'up' && '↑'}
            {data.performance.trend === 'down' && '↓'}
            {data.performance.trend === 'neutral' && '→'}
            {' '}
            {data.performance.trend === 'up' ? 'Improving' : data.performance.trend === 'down' ? 'Declining' : 'Stable'}
          </p>
        </div>
      </div>

      {/* Key Metrics */}
      <section style={styles.section} aria-labelledby="key-metrics-title">
        <h2 style={styles.sectionTitle} id="key-metrics-title">
          Key Performance Indicators
        </h2>
        <div style={styles.metricsGrid}>
          {data.keyMetrics.map((metric) => (
            <ExecutiveMetricCard key={metric.id} metric={metric} />
          ))}
        </div>
      </section>

      {/* Revenue Summary */}
      <section style={styles.section} aria-labelledby="revenue-title">
        <h2 style={styles.sectionTitle} id="revenue-title">
          Revenue Summary
        </h2>
        <RevenueSummary data={data.revenue} />
      </section>

      {/* Strategic Initiatives */}
      <section style={styles.section} aria-labelledby="initiatives-title">
        <h2 style={styles.sectionTitle} id="initiatives-title">
          Strategic Initiatives
        </h2>
        <InitiativesGrid initiatives={data.initiatives} />
      </section>

      {/* Risk Dashboard */}
      <section style={styles.section} aria-labelledby="risks-title">
        <h2 style={styles.sectionTitle} id="risks-title">
          Risk Dashboard
        </h2>
        <RiskDashboard risks={data.risks} />
      </section>

      {/* Highlights */}
      <section style={styles.section} aria-labelledby="highlights-title">
        <h2 style={styles.sectionTitle} id="highlights-title">
          Key Highlights
        </h2>
        <HighlightsList highlights={data.highlights} />
      </section>

      {/* Recommendations */}
      <section style={styles.section} aria-labelledby="recommendations-title">
        <h2 style={styles.sectionTitle} id="recommendations-title">
          Strategic Recommendations
        </h2>
        <RecommendationsList recommendations={data.recommendations} />
      </section>
    </div>
  );
};

/**
 * Executive metric card
 */
const ExecutiveMetricCard: React.FC<{ metric: MetricData }> = ({ metric }) => {
  const getTrendColor = (trend: string) => {
    if (trend === 'up') return 'var(--color-success, #4caf50)';
    if (trend === 'down') return 'var(--color-error, #f44336)';
    return 'var(--color-text-secondary, #666)';
  };

  return (
    <div style={styles.metricCard}>
      <div style={styles.metricHeader}>
        <span style={styles.metricIcon}>{metric.icon || '📊'}</span>
        <span style={styles.metricCategory}>{metric.category}</span>
      </div>
      <h3 style={styles.metricName}>{metric.name}</h3>
      <div style={styles.metricValue}>{metric.formattedValue}</div>
      {metric.changePercent !== undefined && (
        <div style={{ ...styles.metricChange, color: getTrendColor(metric.trend) }}>
          {metric.trend === 'up' && '↑'}
          {metric.trend === 'down' && '↓'}
          {metric.trend === 'neutral' && '→'}
          {' '}
          {metric.changePercent >= 0 ? '+' : ''}
          {metric.changePercent.toFixed(1)}%
        </div>
      )}
    </div>
  );
};

/**
 * Revenue summary component
 */
const RevenueSummary: React.FC<{ data: any }> = ({ data }) => {
  return (
    <div style={styles.revenueSummary}>
      <div style={styles.revenueCard}>
        <h3 style={styles.revenueLabel}>Total Revenue</h3>
        <div style={styles.revenueValue}>
          ${(data.total / 1000000).toFixed(2)}M
        </div>
        <div style={{ ...styles.revenueGrowth, color: data.growth >= 0 ? 'var(--color-success, #4caf50)' : 'var(--color-error, #f44336)' }}>
          {data.growth >= 0 ? '↑' : '↓'} {Math.abs(data.growth).toFixed(1)}% Growth
        </div>
      </div>

      {data.target && (
        <div style={styles.revenueCard}>
          <h3 style={styles.revenueLabel}>Target Attainment</h3>
          <div style={styles.revenueValue}>
            {((data.total / data.target) * 100).toFixed(1)}%
          </div>
          <div style={styles.revenueProgressBar}>
            <div
              style={{
                ...styles.revenueProgressFill,
                width: `${Math.min(100, (data.total / data.target) * 100)}%`,
              }}
            />
          </div>
        </div>
      )}

      <div style={styles.revenueBreakdown}>
        <h3 style={styles.revenueLabel}>Revenue by Segment</h3>
        {Object.entries(data.bySegment || {}).map(([segment, value]: [string, any]) => (
          <div key={segment} style={styles.revenueSegment}>
            <span style={styles.revenueSegmentName}>{segment}</span>
            <span style={styles.revenueSegmentValue}>
              ${(value / 1000000).toFixed(2)}M
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};

/**
 * Initiatives grid component
 */
const InitiativesGrid: React.FC<{ initiatives: Initiative[] }> = ({ initiatives }) => {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'var(--color-success, #4caf50)';
      case 'in-progress':
        return 'var(--color-info, #2196f3)';
      case 'on-hold':
        return 'var(--color-warning, #ff9800)';
      case 'cancelled':
        return 'var(--color-error, #f44336)';
      default:
        return 'var(--color-text-secondary, #666)';
    }
  };

  const getPriorityBadge = (priority: string) => {
    const colors = {
      critical: '#f44336',
      high: '#ff9800',
      medium: '#2196f3',
      low: '#4caf50',
    };
    return colors[priority as keyof typeof colors] || '#666';
  };

  return (
    <div style={styles.initiativesGrid}>
      {initiatives.map((initiative) => (
        <div key={initiative.id} style={styles.initiativeCard}>
          <div style={styles.initiativeHeader}>
            <h3 style={styles.initiativeName}>{initiative.name}</h3>
            <span
              style={{
                ...styles.priorityBadge,
                backgroundColor: getPriorityBadge(initiative.priority),
              }}
            >
              {initiative.priority}
            </span>
          </div>
          <p style={styles.initiativeDescription}>{initiative.description}</p>
          <div style={styles.initiativeProgress}>
            <div style={styles.initiativeProgressHeader}>
              <span style={{ ...styles.initiativeStatus, color: getStatusColor(initiative.status) }}>
                {initiative.status}
              </span>
              <span style={styles.initiativeProgressText}>
                {initiative.progress}%
              </span>
            </div>
            <div style={styles.progressBar}>
              <div
                style={{
                  ...styles.progressFill,
                  width: `${initiative.progress}%`,
                  backgroundColor: getStatusColor(initiative.status),
                }}
              />
            </div>
          </div>
          <div style={styles.initiativeFooter}>
            <span style={styles.initiativeOwner}>Owner: {initiative.owner}</span>
            <span style={styles.initiativeDate}>
              Due: {new Date(initiative.targetDate).toLocaleDateString()}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
};

/**
 * Risk dashboard component
 */
const RiskDashboard: React.FC<{ risks: RiskIndicator[] }> = ({ risks }) => {
  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return '#d32f2f';
      case 'error':
        return '#f44336';
      case 'warning':
        return '#ff9800';
      default:
        return '#2196f3';
    }
  };

  const criticalRisks = risks.filter((r) => r.severity === 'critical' && !r.mitigated);
  const highRisks = risks.filter((r) => r.severity === 'error' && !r.mitigated);

  return (
    <div style={styles.riskDashboard}>
      <div style={styles.riskSummary}>
        <div style={{ ...styles.riskSummaryCard, borderColor: '#d32f2f' }}>
          <div style={styles.riskSummaryCount}>{criticalRisks.length}</div>
          <div style={styles.riskSummaryLabel}>Critical Risks</div>
        </div>
        <div style={{ ...styles.riskSummaryCard, borderColor: '#f44336' }}>
          <div style={styles.riskSummaryCount}>{highRisks.length}</div>
          <div style={styles.riskSummaryLabel}>High Risks</div>
        </div>
        <div style={{ ...styles.riskSummaryCard, borderColor: '#4caf50' }}>
          <div style={styles.riskSummaryCount}>
            {risks.filter((r) => r.mitigated).length}
          </div>
          <div style={styles.riskSummaryLabel}>Mitigated</div>
        </div>
      </div>

      <div style={styles.riskList}>
        {risks.map((risk) => (
          <div
            key={risk.id}
            style={{
              ...styles.riskItem,
              borderLeftColor: getSeverityColor(risk.severity),
            }}
          >
            <div style={styles.riskItemHeader}>
              <h4 style={styles.riskCategory}>{risk.category}</h4>
              <span
                style={{
                  ...styles.riskSeverity,
                  backgroundColor: getSeverityColor(risk.severity),
                }}
              >
                {risk.severity}
              </span>
            </div>
            <p style={styles.riskDescription}>{risk.description}</p>
            <div style={styles.riskMetrics}>
              <div style={styles.riskMetric}>
                <span style={styles.riskMetricLabel}>Probability:</span>
                <span style={styles.riskMetricValue}>{risk.probability}%</span>
              </div>
              <div style={styles.riskMetric}>
                <span style={styles.riskMetricLabel}>Impact:</span>
                <span style={styles.riskMetricValue}>{risk.impact}%</span>
              </div>
              <div style={styles.riskMetric}>
                <span style={styles.riskMetricLabel}>Score:</span>
                <span style={styles.riskMetricValue}>{risk.score}</span>
              </div>
            </div>
            {risk.mitigated && (
              <div style={styles.riskMitigated}>✓ Mitigated</div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

/**
 * Highlights list component
 */
const HighlightsList: React.FC<{ highlights: Highlight[] }> = ({ highlights }) => {
  const getHighlightIcon = (type: string) => {
    switch (type) {
      case 'achievement':
        return '🏆';
      case 'milestone':
        return '🎯';
      case 'alert':
        return '⚠️';
      case 'insight':
        return '💡';
      default:
        return '📌';
    }
  };

  return (
    <div style={styles.highlightsList}>
      {highlights.map((highlight) => (
        <div key={highlight.id} style={styles.highlightItem}>
          <span style={styles.highlightIcon}>
            {highlight.icon || getHighlightIcon(highlight.type)}
          </span>
          <div style={styles.highlightContent}>
            <h4 style={styles.highlightTitle}>{highlight.title}</h4>
            <p style={styles.highlightDescription}>{highlight.description}</p>
            <span style={styles.highlightTime}>
              {new Date(highlight.timestamp).toLocaleString()}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
};

/**
 * Recommendations list component
 */
const RecommendationsList: React.FC<{ recommendations: Recommendation[] }> = ({
  recommendations,
}) => {
  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high':
        return '#f44336';
      case 'medium':
        return '#ff9800';
      default:
        return '#4caf50';
    }
  };

  return (
    <div style={styles.recommendationsList}>
      {recommendations.map((rec) => (
        <div key={rec.id} style={styles.recommendationItem}>
          <div style={styles.recommendationHeader}>
            <h4 style={styles.recommendationTitle}>{rec.title}</h4>
            <span
              style={{
                ...styles.recommendationPriority,
                backgroundColor: getPriorityColor(rec.priority),
              }}
            >
              {rec.priority} priority
            </span>
          </div>
          <p style={styles.recommendationDescription}>{rec.description}</p>
          <div style={styles.recommendationMeta}>
            <span style={styles.recommendationCategory}>
              Category: {rec.category}
            </span>
            <span style={styles.recommendationConfidence}>
              Confidence: {rec.confidence}%
            </span>
          </div>
          <div style={styles.recommendationImpact}>
            <strong>Expected Impact:</strong> {rec.impact}
          </div>
          {rec.actions && rec.actions.length > 0 && (
            <div style={styles.recommendationActions}>
              <strong>Action Items:</strong>
              <ul style={styles.actionsList}>
                {rec.actions.map((action, index) => (
                  <li key={index} style={styles.actionItem}>
                    {action}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

/**
 * Component styles
 */
const styles: Record<string, React.CSSProperties> = {
  container: {
    backgroundColor: 'var(--color-background, #f5f5f5)',
    padding: 32,
    minHeight: '100vh',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 32,
    paddingBottom: 24,
    borderBottom: '2px solid var(--color-divider, #e0e0e0)',
  },
  headerLeft: {},
  headerRight: {
    display: 'flex',
    gap: 12,
    alignItems: 'center',
  },
  title: {
    margin: 0,
    fontSize: 32,
    fontWeight: 700,
    color: 'var(--color-text, #333)',
  },
  subtitle: {
    margin: '8px 0 0 0',
    fontSize: 14,
    color: 'var(--color-text-secondary, #666)',
  },
  periodSelector: {
    padding: '8px 16px',
    border: '1px solid var(--color-border, #e0e0e0)',
    borderRadius: 4,
    backgroundColor: 'var(--color-surface, #fff)',
    color: 'var(--color-text, #333)',
    fontSize: 14,
    cursor: 'pointer',
  },
  printButton: {
    padding: '8px 16px',
    backgroundColor: 'var(--color-primary, #1976d2)',
    color: '#fff',
    border: 'none',
    borderRadius: 4,
    cursor: 'pointer',
    fontSize: 14,
    fontWeight: 500,
  },
  healthSection: {
    marginBottom: 32,
  },
  healthCard: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    padding: 32,
    textAlign: 'center',
  },
  healthScore: {
    fontSize: 72,
    fontWeight: 700,
    margin: '16px 0',
  },
  healthScoreUnit: {
    fontSize: 32,
    opacity: 0.6,
  },
  healthBar: {
    height: 8,
    backgroundColor: 'var(--color-success, #4caf50)',
    borderRadius: 4,
    margin: '16px 0',
    transition: 'width 1s ease',
  },
  healthTrend: {
    fontSize: 18,
    fontWeight: 600,
    margin: 0,
  },
  section: {
    marginBottom: 32,
  },
  sectionTitle: {
    margin: '0 0 16px 0',
    fontSize: 24,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
  },
  metricsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
    gap: 16,
  },
  metricCard: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    padding: 20,
    border: '1px solid var(--color-border, #e0e0e0)',
  },
  metricHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  metricIcon: {
    fontSize: 24,
  },
  metricCategory: {
    fontSize: 11,
    padding: '2px 8px',
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 12,
    fontWeight: 500,
  },
  metricName: {
    margin: '0 0 8px 0',
    fontSize: 14,
    fontWeight: 500,
    color: 'var(--color-text-secondary, #666)',
  },
  metricValue: {
    fontSize: 28,
    fontWeight: 700,
    color: 'var(--color-text, #333)',
    marginBottom: 8,
  },
  metricChange: {
    fontSize: 14,
    fontWeight: 600,
  },
  revenueSummary: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
    gap: 16,
  },
  revenueCard: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    padding: 24,
    border: '1px solid var(--color-border, #e0e0e0)',
  },
  revenueLabel: {
    margin: '0 0 12px 0',
    fontSize: 14,
    fontWeight: 500,
    color: 'var(--color-text-secondary, #666)',
  },
  revenueValue: {
    fontSize: 36,
    fontWeight: 700,
    color: 'var(--color-text, #333)',
    marginBottom: 8,
  },
  revenueGrowth: {
    fontSize: 16,
    fontWeight: 600,
  },
  revenueProgressBar: {
    height: 8,
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 4,
    overflow: 'hidden',
  },
  revenueProgressFill: {
    height: '100%',
    backgroundColor: 'var(--color-success, #4caf50)',
    transition: 'width 1s ease',
  },
  revenueBreakdown: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    padding: 24,
    border: '1px solid var(--color-border, #e0e0e0)',
  },
  revenueSegment: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '8px 0',
    borderBottom: '1px solid var(--color-divider, #e0e0e0)',
  },
  revenueSegmentName: {
    fontSize: 14,
    color: 'var(--color-text, #333)',
  },
  revenueSegmentValue: {
    fontSize: 14,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
  },
  initiativesGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))',
    gap: 16,
  },
  initiativeCard: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    padding: 20,
    border: '1px solid var(--color-border, #e0e0e0)',
  },
  initiativeHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  initiativeName: {
    margin: 0,
    fontSize: 16,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
    flex: 1,
  },
  priorityBadge: {
    fontSize: 10,
    padding: '4px 8px',
    borderRadius: 12,
    color: '#fff',
    fontWeight: 600,
    textTransform: 'uppercase',
  },
  initiativeDescription: {
    fontSize: 13,
    color: 'var(--color-text-secondary, #666)',
    marginBottom: 16,
  },
  initiativeProgress: {
    marginBottom: 12,
  },
  initiativeProgressHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    marginBottom: 8,
    fontSize: 12,
  },
  initiativeStatus: {
    fontWeight: 600,
    textTransform: 'capitalize',
  },
  initiativeProgressText: {
    fontWeight: 600,
  },
  progressBar: {
    height: 6,
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 3,
    overflow: 'hidden',
  },
  progressFill: {
    height: '100%',
    transition: 'width 0.5s ease',
  },
  initiativeFooter: {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: 11,
    color: 'var(--color-text-secondary, #999)',
  },
  initiativeOwner: {},
  initiativeDate: {},
  riskDashboard: {},
  riskSummary: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: 16,
    marginBottom: 24,
  },
  riskSummaryCard: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    padding: 24,
    textAlign: 'center',
    borderLeft: '4px solid',
  },
  riskSummaryCount: {
    fontSize: 48,
    fontWeight: 700,
    color: 'var(--color-text, #333)',
  },
  riskSummaryLabel: {
    fontSize: 14,
    color: 'var(--color-text-secondary, #666)',
    marginTop: 8,
  },
  riskList: {
    display: 'flex',
    flexDirection: 'column',
    gap: 12,
  },
  riskItem: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    padding: 20,
    borderLeft: '4px solid',
  },
  riskItemHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  riskCategory: {
    margin: 0,
    fontSize: 16,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
  },
  riskSeverity: {
    fontSize: 11,
    padding: '4px 12px',
    borderRadius: 12,
    color: '#fff',
    fontWeight: 600,
    textTransform: 'uppercase',
  },
  riskDescription: {
    fontSize: 14,
    color: 'var(--color-text, #333)',
    marginBottom: 12,
  },
  riskMetrics: {
    display: 'flex',
    gap: 24,
    marginBottom: 8,
  },
  riskMetric: {
    fontSize: 12,
  },
  riskMetricLabel: {
    color: 'var(--color-text-secondary, #666)',
    marginRight: 4,
  },
  riskMetricValue: {
    fontWeight: 600,
    color: 'var(--color-text, #333)',
  },
  riskMitigated: {
    display: 'inline-block',
    padding: '4px 12px',
    backgroundColor: '#e8f5e9',
    color: '#4caf50',
    borderRadius: 12,
    fontSize: 12,
    fontWeight: 600,
  },
  highlightsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: 12,
  },
  highlightItem: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    padding: 20,
    display: 'flex',
    gap: 16,
    alignItems: 'flex-start',
  },
  highlightIcon: {
    fontSize: 32,
    lineHeight: 1,
  },
  highlightContent: {
    flex: 1,
  },
  highlightTitle: {
    margin: '0 0 8px 0',
    fontSize: 16,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
  },
  highlightDescription: {
    margin: '0 0 8px 0',
    fontSize: 14,
    color: 'var(--color-text-secondary, #666)',
  },
  highlightTime: {
    fontSize: 12,
    color: 'var(--color-text-secondary, #999)',
  },
  recommendationsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: 16,
  },
  recommendationItem: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    padding: 24,
    border: '1px solid var(--color-border, #e0e0e0)',
  },
  recommendationHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  recommendationTitle: {
    margin: 0,
    fontSize: 18,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
    flex: 1,
  },
  recommendationPriority: {
    fontSize: 11,
    padding: '4px 12px',
    borderRadius: 12,
    color: '#fff',
    fontWeight: 600,
    textTransform: 'uppercase',
  },
  recommendationDescription: {
    fontSize: 14,
    color: 'var(--color-text, #333)',
    marginBottom: 12,
    lineHeight: 1.6,
  },
  recommendationMeta: {
    display: 'flex',
    gap: 24,
    marginBottom: 12,
    fontSize: 13,
    color: 'var(--color-text-secondary, #666)',
  },
  recommendationCategory: {},
  recommendationConfidence: {},
  recommendationImpact: {
    fontSize: 14,
    color: 'var(--color-text, #333)',
    marginBottom: 12,
    padding: 12,
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 4,
  },
  recommendationActions: {
    fontSize: 14,
  },
  actionsList: {
    margin: '8px 0 0 0',
    paddingLeft: 24,
  },
  actionItem: {
    marginBottom: 4,
    color: 'var(--color-text, #333)',
  },
};

export default ExecutiveOverview;
