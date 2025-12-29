/**
 * Insights View - AI-powered accessibility insights dashboard
 *
 * Displays comprehensive analytics and predictions including:
 * - Trend predictions and forecasting
 * - Risk assessment and compliance scoring
 * - Prioritized recommendations
 * - Impact analysis and effort estimates
 * - Visual analytics and charts
 */

import React, { useState, useMemo } from 'react';
import type {
  InsightsViewProps,
  AIInsights,
  Recommendation,
  TrendPrediction,
  ComplianceRisk,
  Prediction,
} from './types';

type TabType = 'overview' | 'trends' | 'risks' | 'recommendations' | 'predictions';

export function InsightsView({
  insights,
  onRefresh,
  onRecommendationClick,
  timeRange,
}: InsightsViewProps) {
  const [activeTab, setActiveTab] = useState<TabType>('overview');

  if (!insights) {
    return (
      <div className="insights-view empty">
        <div className="empty-state">
          <h3>No insights available</h3>
          <p>Run an AI analysis to generate insights.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="insights-view">
      {/* Header */}
      <div className="insights-header">
        <div className="header-content">
          <h1>AI Insights Dashboard</h1>
          <p className="header-subtitle">
            AI-powered accessibility analytics and predictions
          </p>
        </div>

        <div className="header-actions">
          {timeRange && (
            <div className="time-range">
              <span className="time-range-label">Period:</span>
              <span className="time-range-value">
                {timeRange.start.toLocaleDateString()} - {timeRange.end.toLocaleDateString()}
              </span>
            </div>
          )}
          {onRefresh && (
            <button className="refresh-button" onClick={onRefresh}>
              Refresh
            </button>
          )}
        </div>
      </div>

      {/* Navigation Tabs */}
      <nav className="insights-tabs">
        <TabButton active={activeTab === 'overview'} onClick={() => setActiveTab('overview')}>
          Overview
        </TabButton>
        <TabButton active={activeTab === 'trends'} onClick={() => setActiveTab('trends')}>
          Trends
        </TabButton>
        <TabButton active={activeTab === 'risks'} onClick={() => setActiveTab('risks')}>
          Risk Assessment
        </TabButton>
        <TabButton
          active={activeTab === 'recommendations'}
          onClick={() => setActiveTab('recommendations')}
        >
          Recommendations
        </TabButton>
        <TabButton active={activeTab === 'predictions'} onClick={() => setActiveTab('predictions')}>
          Predictions
        </TabButton>
      </nav>

      {/* Tab Content */}
      <div className="insights-content">
        {activeTab === 'overview' && (
          <OverviewTab insights={insights} onRecommendationClick={onRecommendationClick} />
        )}
        {activeTab === 'trends' && <TrendsTab trends={insights.trends} />}
        {activeTab === 'risks' && <RiskTab riskAssessment={insights.riskAssessment} />}
        {activeTab === 'recommendations' && (
          <RecommendationsTab
            recommendations={insights.recommendations}
            onClick={onRecommendationClick}
          />
        )}
        {activeTab === 'predictions' && <PredictionsTab predictions={insights.predictions} />}
      </div>

      {/* Footer */}
      <div className="insights-footer">
        <p className="footer-text">
          Last updated: {new Date(insights.timestamp).toLocaleString()}
        </p>
        <p className="footer-note">
          AI confidence: {(insights.summary.avgConfidence * 100).toFixed(1)}%
        </p>
      </div>
    </div>
  );
}

// Tab Button Component
interface TabButtonProps {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}

function TabButton({ active, onClick, children }: TabButtonProps) {
  return (
    <button className={`tab-button ${active ? 'active' : ''}`} onClick={onClick}>
      {children}
    </button>
  );
}

// Overview Tab
interface OverviewTabProps {
  insights: AIInsights;
  onRecommendationClick?: (recommendation: Recommendation) => void;
}

function OverviewTab({ insights, onRecommendationClick }: OverviewTabProps) {
  const { summary, riskAssessment, recommendations } = insights;

  const topRecommendations = useMemo(
    () => recommendations.slice(0, 5),
    [recommendations]
  );

  return (
    <div className="overview-tab">
      {/* Key Metrics Grid */}
      <div className="metrics-grid">
        <MetricCard
          title="Issues Analyzed"
          value={summary.totalIssuesAnalyzed.toString()}
          icon="📊"
        />
        <MetricCard
          title="Compliance Score"
          value={`${summary.complianceScore.toFixed(1)}%`}
          icon="✓"
          trend={summary.complianceScore >= 80 ? 'success' : 'warning'}
        />
        <MetricCard
          title="Estimated Effort"
          value={`${summary.estimatedEffort.toFixed(0)} hrs`}
          icon="⏱️"
        />
        <MetricCard
          title="Risk Level"
          value={riskAssessment.riskLevel}
          icon="⚠️"
          trend={
            riskAssessment.riskLevel === 'Low' || riskAssessment.riskLevel === 'Minimal'
              ? 'success'
              : 'danger'
          }
        />
      </div>

      {/* Category Breakdown */}
      <div className="section">
        <h2>Issue Categories</h2>
        <div className="category-breakdown">
          {summary.topCategories.map((cat) => (
            <CategoryBar
              key={cat.category}
              category={cat.category}
              count={cat.count}
              percentage={cat.percentage}
            />
          ))}
        </div>
      </div>

      {/* Risk Assessment Summary */}
      <div className="section">
        <h2>Risk Assessment</h2>
        <div className="risk-summary">
          <div className="risk-score-container">
            <RiskGauge score={riskAssessment.riskScore} level={riskAssessment.riskLevel} />
          </div>
          <div className="risk-details">
            <div className="risk-stat">
              <span className="label">Total Violations:</span>
              <span className="value">{riskAssessment.totalViolations}</span>
            </div>
            <div className="risk-stat">
              <span className="label">Compliance:</span>
              <span className="value">{riskAssessment.compliancePercentage.toFixed(1)}%</span>
            </div>
            <div className="risk-stat">
              <span className="label">Time to Compliance:</span>
              <span className="value">
                {riskAssessment.estimatedTimeToComplianceDays} days
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Top Recommendations */}
      <div className="section">
        <h2>Top Recommendations</h2>
        <div className="recommendations-preview">
          {topRecommendations.map((rec) => (
            <RecommendationCard
              key={rec.id}
              recommendation={rec}
              onClick={onRecommendationClick}
              compact
            />
          ))}
        </div>
      </div>
    </div>
  );
}

// Trends Tab
interface TrendsTabProps {
  trends: TrendPrediction[];
}

function TrendsTab({ trends }: TrendsTabProps) {
  if (trends.length === 0) {
    return (
      <div className="empty-state">
        <p>No trend data available. More historical data is needed for predictions.</p>
      </div>
    );
  }

  return (
    <div className="trends-tab">
      <h2>Accessibility Trends</h2>
      <p className="section-description">
        AI-powered predictions based on historical data analysis
      </p>

      {trends.map((trend, index) => (
        <TrendCard key={index} trend={trend} />
      ))}
    </div>
  );
}

// Risk Tab
interface RiskTabProps {
  riskAssessment: ComplianceRisk;
}

function RiskTab({ riskAssessment }: RiskTabProps) {
  return (
    <div className="risk-tab">
      <h2>Risk Assessment</h2>

      {/* Risk Score Visualization */}
      <div className="risk-visualization">
        <RiskGauge score={riskAssessment.riskScore} level={riskAssessment.riskLevel} large />
        <div className="risk-interpretation">
          <h3>Risk Level: {riskAssessment.riskLevel}</h3>
          <p className="risk-description">
            {getRiskDescription(riskAssessment.riskLevel)}
          </p>
        </div>
      </div>

      {/* Severity Breakdown */}
      <div className="section">
        <h3>Violations by Severity</h3>
        <div className="severity-breakdown">
          {Object.entries(riskAssessment.severityBreakdown).map(([severity, count]) => (
            <div key={severity} className="severity-item">
              <span className={`severity-badge severity-${severity.toLowerCase()}`}>
                {severity}
              </span>
              <span className="severity-count">{count}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Mitigation Strategies */}
      <div className="section">
        <h3>Recommended Mitigation Strategies</h3>
        <ul className="mitigation-list">
          {riskAssessment.mitigationStrategies.map((strategy, index) => (
            <li key={index} className="mitigation-item">
              {strategy}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

// Recommendations Tab
interface RecommendationsTabProps {
  recommendations: Recommendation[];
  onClick?: (recommendation: Recommendation) => void;
}

function RecommendationsTab({ recommendations, onClick }: RecommendationsTabProps) {
  const [filter, setFilter] = useState<'all' | 'high' | 'medium' | 'low'>('all');

  const filtered = useMemo(() => {
    if (filter === 'all') return recommendations;
    return recommendations.filter(
      (rec) => rec.impact.toLowerCase() === filter
    );
  }, [recommendations, filter]);

  return (
    <div className="recommendations-tab">
      <div className="tab-header">
        <h2>AI Recommendations</h2>
        <div className="filter-controls">
          <label htmlFor="impact-filter">Filter by impact:</label>
          <select
            id="impact-filter"
            value={filter}
            onChange={(e) => setFilter(e.target.value as any)}
          >
            <option value="all">All</option>
            <option value="high">High impact</option>
            <option value="medium">Medium impact</option>
            <option value="low">Low impact</option>
          </select>
        </div>
      </div>

      <div className="recommendations-list">
        {filtered.map((rec) => (
          <RecommendationCard key={rec.id} recommendation={rec} onClick={onClick} />
        ))}
      </div>
    </div>
  );
}

// Predictions Tab
interface PredictionsTabProps {
  predictions: Prediction[];
}

function PredictionsTab({ predictions }: PredictionsTabProps) {
  return (
    <div className="predictions-tab">
      <h2>AI Predictions</h2>
      <p className="section-description">
        Forward-looking predictions based on current trends and patterns
      </p>

      <div className="predictions-grid">
        {predictions.map((pred, index) => (
          <PredictionCard key={index} prediction={pred} />
        ))}
      </div>
    </div>
  );
}

// Component: Metric Card
interface MetricCardProps {
  title: string;
  value: string;
  icon?: string;
  trend?: 'success' | 'warning' | 'danger';
}

function MetricCard({ title, value, icon, trend }: MetricCardProps) {
  return (
    <div className={`metric-card ${trend ? `trend-${trend}` : ''}`}>
      {icon && <div className="metric-icon">{icon}</div>}
      <div className="metric-content">
        <h4>{title}</h4>
        <div className="metric-value">{value}</div>
      </div>
    </div>
  );
}

// Component: Category Bar
interface CategoryBarProps {
  category: string;
  count: number;
  percentage: number;
}

function CategoryBar({ category, count, percentage }: CategoryBarProps) {
  return (
    <div className="category-bar">
      <div className="category-info">
        <span className="category-name">{category}</span>
        <span className="category-count">{count}</span>
      </div>
      <div className="progress-bar">
        <div className="progress-fill" style={{ width: `${percentage}%` }} />
      </div>
      <span className="category-percentage">{percentage.toFixed(1)}%</span>
    </div>
  );
}

// Component: Risk Gauge
interface RiskGaugeProps {
  score: number;
  level: string;
  large?: boolean;
}

function RiskGauge({ score, level, large = false }: RiskGaugeProps) {
  const getColor = () => {
    if (score >= 80) return '#ef4444';
    if (score >= 60) return '#f97316';
    if (score >= 40) return '#eab308';
    if (score >= 20) return '#84cc16';
    return '#22c55e';
  };

  return (
    <div className={`risk-gauge ${large ? 'large' : ''}`}>
      <svg viewBox="0 0 200 120" className="gauge-svg">
        <path
          d="M 20 100 A 80 80 0 0 1 180 100"
          fill="none"
          stroke="#e5e7eb"
          strokeWidth="20"
        />
        <path
          d="M 20 100 A 80 80 0 0 1 180 100"
          fill="none"
          stroke={getColor()}
          strokeWidth="20"
          strokeDasharray={`${(score / 100) * 251.2} 251.2`}
        />
        <text x="100" y="90" textAnchor="middle" className="gauge-score">
          {score.toFixed(0)}
        </text>
        <text x="100" y="110" textAnchor="middle" className="gauge-label">
          {level}
        </text>
      </svg>
    </div>
  );
}

// Component: Trend Card
interface TrendCardProps {
  trend: TrendPrediction;
}

function TrendCard({ trend }: TrendCardProps) {
  const directionIcon =
    trend.trendDirection === 'Increasing' ? '📈' : trend.trendDirection === 'Decreasing' ? '📉' : '➡️';

  return (
    <div className="trend-card">
      <div className="trend-header">
        <span className="trend-icon">{directionIcon}</span>
        <h3 className="trend-title">Issue Trend</h3>
        <span className={`trend-direction trend-${trend.trendDirection.toLowerCase()}`}>
          {trend.trendDirection}
        </span>
      </div>

      <div className="trend-data">
        <div className="trend-stat">
          <span className="stat-label">Current:</span>
          <span className="stat-value">{trend.currentValue.toFixed(0)}</span>
        </div>
        <div className="trend-stat">
          <span className="stat-label">Predicted:</span>
          <span className="stat-value">{trend.predictedValue.toFixed(0)}</span>
        </div>
        <div className="trend-stat">
          <span className="stat-label">Confidence:</span>
          <span className="stat-value">{(trend.confidence * 100).toFixed(0)}%</span>
        </div>
      </div>

      {trend.issues.length > 0 && (
        <div className="trend-issues">
          <h4>Identified Trends:</h4>
          <ul>
            {trend.issues.map((issue, index) => (
              <li key={index}>
                <strong>{issue.category}:</strong> {issue.description}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

// Component: Recommendation Card
interface RecommendationCardProps {
  recommendation: Recommendation;
  onClick?: (recommendation: Recommendation) => void;
  compact?: boolean;
}

function RecommendationCard({ recommendation, onClick, compact = false }: RecommendationCardProps) {
  return (
    <div
      className={`recommendation-card ${compact ? 'compact' : ''}`}
      onClick={() => onClick?.(recommendation)}
      role={onClick ? 'button' : undefined}
      tabIndex={onClick ? 0 : undefined}
    >
      <div className="recommendation-header">
        <h3>{recommendation.title}</h3>
        <div className="recommendation-badges">
          <span className={`impact-badge impact-${recommendation.impact.toLowerCase()}`}>
            {recommendation.impact} impact
          </span>
          <span className={`effort-badge effort-${recommendation.effort.toLowerCase()}`}>
            {recommendation.effort} effort
          </span>
        </div>
      </div>

      <p className="recommendation-description">{recommendation.description}</p>

      {!compact && recommendation.actionItems.length > 0 && (
        <div className="action-items">
          <h4>Action Items:</h4>
          <ul>
            {recommendation.actionItems.map((item, index) => (
              <li key={index}>{item}</li>
            ))}
          </ul>
        </div>
      )}

      <div className="recommendation-footer">
        <span className="category-tag">{recommendation.category}</span>
        <span className="priority-badge">Priority: {recommendation.priority}</span>
      </div>
    </div>
  );
}

// Component: Prediction Card
interface PredictionCardProps {
  prediction: Prediction;
}

function PredictionCard({ prediction }: PredictionCardProps) {
  return (
    <div className="prediction-card">
      <h3>{prediction.type}</h3>
      <p className="prediction-text">{prediction.prediction}</p>
      <div className="prediction-meta">
        <div className="meta-item">
          <span className="meta-label">Timeframe:</span>
          <span className="meta-value">{prediction.timeframe}</span>
        </div>
        <div className="meta-item">
          <span className="meta-label">Confidence:</span>
          <span className="meta-value">{(prediction.confidence * 100).toFixed(0)}%</span>
        </div>
        <div className="meta-item">
          <span className="meta-label">Impact:</span>
          <span className="meta-value">{prediction.impact}</span>
        </div>
      </div>
    </div>
  );
}

// Utility function
function getRiskDescription(level: string): string {
  switch (level) {
    case 'Critical':
      return 'Immediate action required. Critical accessibility barriers detected.';
    case 'High':
      return 'High priority issues found. Address these soon to improve compliance.';
    case 'Medium':
      return 'Moderate accessibility concerns. Schedule fixes in upcoming sprints.';
    case 'Low':
      return 'Minor issues detected. Address during regular maintenance.';
    case 'Minimal':
      return 'Excellent! Very few accessibility issues detected.';
    default:
      return 'Risk level assessment unavailable.';
  }
}
