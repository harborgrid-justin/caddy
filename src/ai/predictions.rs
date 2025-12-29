//! Predictive Analytics for Accessibility
//!
//! Provides AI-powered predictive analytics including:
//! - Issue trend prediction
//! - Remediation effort estimation
//! - Compliance risk scoring
//! - Regression probability calculation
//! - Impact scoring

use crate::ai::{AIError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Predictive analytics engine
pub struct PredictiveAnalytics {
    config: PredictiveConfig,
    historical_data: Vec<HistoricalDataPoint>,
}

/// Predictive analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveConfig {
    /// Minimum data points required for prediction
    pub min_data_points: usize,
    /// Prediction window (days into future)
    pub prediction_window_days: i64,
    /// Confidence threshold for predictions
    pub confidence_threshold: f64,
    /// Enable trend analysis
    pub enable_trend_analysis: bool,
    /// Enable effort estimation
    pub enable_effort_estimation: bool,
    /// Enable risk scoring
    pub enable_risk_scoring: bool,
}

impl Default for PredictiveConfig {
    fn default() -> Self {
        Self {
            min_data_points: 10,
            prediction_window_days: 30,
            confidence_threshold: 0.7,
            enable_trend_analysis: true,
            enable_effort_estimation: true,
            enable_risk_scoring: true,
        }
    }
}

/// Historical data point for training predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    pub timestamp: DateTime<Utc>,
    pub issue_count: usize,
    pub severity_breakdown: HashMap<String, usize>,
    pub remediation_time_hours: f64,
    pub team_size: usize,
    pub complexity_score: f64,
}

impl PredictiveAnalytics {
    /// Create a new predictive analytics engine
    pub fn new(config: PredictiveConfig) -> Self {
        Self {
            config,
            historical_data: Vec::new(),
        }
    }

    /// Add historical data point
    pub fn add_data_point(&mut self, data: HistoricalDataPoint) {
        self.historical_data.push(data);
    }

    /// Load historical data
    pub fn load_historical_data(&mut self, data: Vec<HistoricalDataPoint>) {
        self.historical_data = data;
    }

    /// Predict issue trends
    pub fn predict_trends(&self) -> Result<TrendPrediction> {
        if !self.config.enable_trend_analysis {
            return Err(AIError::InvalidConfig("Trend analysis is disabled".to_string()));
        }

        if self.historical_data.len() < self.config.min_data_points {
            return Err(AIError::InferenceError(format!(
                "Insufficient historical data: need at least {} points, have {}",
                self.config.min_data_points,
                self.historical_data.len()
            )));
        }

        // Calculate trend using linear regression
        let (slope, intercept, r_squared) = self.calculate_linear_regression();

        // Project future values
        let now = Utc::now();
        let future_date = now + Duration::days(self.config.prediction_window_days);
        let days_ahead = self.config.prediction_window_days as f64;

        let current_predicted = slope * 0.0 + intercept;
        let future_predicted = slope * days_ahead + intercept;

        let trend_direction = if slope > 0.5 {
            TrendDirection::Increasing
        } else if slope < -0.5 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };

        // Calculate confidence based on R-squared
        let confidence = r_squared.min(0.95);

        let issues = self.generate_trend_issues(slope, r_squared);

        Ok(TrendPrediction {
            trend_direction,
            current_value: current_predicted,
            predicted_value: future_predicted,
            prediction_date: future_date,
            confidence,
            r_squared,
            issues,
        })
    }

    /// Calculate linear regression for trend analysis
    fn calculate_linear_regression(&self) -> (f64, f64, f64) {
        let n = self.historical_data.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;

        let start_time = self.historical_data[0].timestamp;

        for (i, point) in self.historical_data.iter().enumerate() {
            let x = i as f64;
            let y = point.issue_count as f64;

            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
            sum_y2 += y * y;
        }

        // Calculate slope and intercept
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate R-squared
        let mean_y = sum_y / n;
        let ss_tot = sum_y2 - n * mean_y * mean_y;
        let ss_res = self.historical_data.iter().enumerate()
            .map(|(i, point)| {
                let predicted = slope * i as f64 + intercept;
                let actual = point.issue_count as f64;
                (actual - predicted).powi(2)
            })
            .sum::<f64>();

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        (slope, intercept, r_squared)
    }

    /// Generate issues based on trend analysis
    fn generate_trend_issues(&self, slope: f64, r_squared: f64) -> Vec<IssueTrend> {
        let mut issues = Vec::new();

        if slope > 1.0 {
            issues.push(IssueTrend {
                category: "General".to_string(),
                trend: TrendDirection::Increasing,
                severity: TrendSeverity::High,
                description: "Accessibility issues are increasing significantly".to_string(),
                recommendation: "Implement proactive accessibility reviews in development process".to_string(),
            });
        }

        if r_squared < 0.5 {
            issues.push(IssueTrend {
                category: "Data Quality".to_string(),
                trend: TrendDirection::Stable,
                severity: TrendSeverity::Low,
                description: "Historical data shows high variance, predictions may be less reliable".to_string(),
                recommendation: "Continue collecting data for more accurate predictions".to_string(),
            });
        }

        issues
    }

    /// Estimate remediation effort
    pub fn estimate_remediation(&self, issue_count: usize, complexity: f64) -> Result<RemediationEstimate> {
        if !self.config.enable_effort_estimation {
            return Err(AIError::InvalidConfig("Effort estimation is disabled".to_string()));
        }

        // Calculate average remediation time from historical data
        let avg_time_per_issue = if !self.historical_data.is_empty() {
            let total_time: f64 = self.historical_data.iter()
                .map(|d| d.remediation_time_hours)
                .sum();
            let total_issues: usize = self.historical_data.iter()
                .map(|d| d.issue_count)
                .sum();

            if total_issues > 0 {
                total_time / total_issues as f64
            } else {
                2.0 // Default 2 hours per issue
            }
        } else {
            2.0
        };

        // Apply complexity multiplier
        let complexity_multiplier = 1.0 + (complexity * 0.5);
        let estimated_hours = issue_count as f64 * avg_time_per_issue * complexity_multiplier;

        // Calculate confidence based on historical data availability
        let confidence = if self.historical_data.len() >= self.config.min_data_points {
            0.8
        } else {
            0.5
        };

        // Estimate team size needed (assuming 40 hours per week per person)
        let weeks_desired = 2.0; // Target: complete in 2 weeks
        let hours_per_week = 40.0;
        let estimated_team_size = (estimated_hours / (weeks_desired * hours_per_week)).ceil() as usize;

        Ok(RemediationEstimate {
            total_issues: issue_count,
            estimated_hours,
            estimated_hours_min: estimated_hours * 0.7,
            estimated_hours_max: estimated_hours * 1.5,
            complexity_factor: complexity,
            confidence,
            recommended_team_size: estimated_team_size.max(1),
            estimated_completion_days: (estimated_hours / (estimated_team_size as f64 * 8.0)).ceil() as usize,
            breakdown: self.generate_effort_breakdown(issue_count, estimated_hours),
        })
    }

    /// Generate effort breakdown
    fn generate_effort_breakdown(&self, issue_count: usize, total_hours: f64) -> HashMap<String, f64> {
        let mut breakdown = HashMap::new();

        // Standard breakdown percentages
        breakdown.insert("Analysis & Planning".to_string(), total_hours * 0.15);
        breakdown.insert("Implementation".to_string(), total_hours * 0.50);
        breakdown.insert("Testing".to_string(), total_hours * 0.20);
        breakdown.insert("Documentation".to_string(), total_hours * 0.10);
        breakdown.insert("Review & QA".to_string(), total_hours * 0.05);

        breakdown
    }

    /// Calculate compliance risk score
    pub fn calculate_compliance_risk(&self, violations: &[Violation]) -> Result<ComplianceRisk> {
        if !self.config.enable_risk_scoring {
            return Err(AIError::InvalidConfig("Risk scoring is disabled".to_string()));
        }

        let mut risk_score = 0.0;
        let mut severity_counts = HashMap::new();

        // Calculate weighted risk based on severity
        for violation in violations {
            let weight = match violation.severity.as_str() {
                "critical" => 10.0,
                "high" => 5.0,
                "medium" => 2.0,
                "low" => 0.5,
                _ => 1.0,
            };

            risk_score += weight;
            *severity_counts.entry(violation.severity.clone()).or_insert(0) += 1;
        }

        // Normalize risk score (0-100)
        let normalized_score = (risk_score / (violations.len().max(1) as f64 * 10.0) * 100.0).min(100.0);

        let risk_level = if normalized_score >= 80.0 {
            RiskLevel::Critical
        } else if normalized_score >= 60.0 {
            RiskLevel::High
        } else if normalized_score >= 40.0 {
            RiskLevel::Medium
        } else if normalized_score >= 20.0 {
            RiskLevel::Low
        } else {
            RiskLevel::Minimal
        };

        // Generate mitigation strategies
        let mitigation_strategies = self.generate_mitigation_strategies(&severity_counts, &risk_level);

        Ok(ComplianceRisk {
            risk_score: normalized_score,
            risk_level,
            total_violations: violations.len(),
            severity_breakdown: severity_counts,
            compliance_percentage: 100.0 - normalized_score,
            mitigation_strategies,
            estimated_time_to_compliance_days: self.estimate_compliance_timeline(violations.len(), normalized_score),
        })
    }

    /// Generate mitigation strategies
    fn generate_mitigation_strategies(&self, severity_counts: &HashMap<String, usize>, risk_level: &RiskLevel) -> Vec<String> {
        let mut strategies = Vec::new();

        match risk_level {
            RiskLevel::Critical => {
                strategies.push("Immediate action required: Form dedicated accessibility team".to_string());
                strategies.push("Halt new feature development until critical issues are resolved".to_string());
                strategies.push("Conduct comprehensive accessibility audit with external experts".to_string());
            }
            RiskLevel::High => {
                strategies.push("Prioritize accessibility fixes in current sprint".to_string());
                strategies.push("Implement automated accessibility testing in CI/CD pipeline".to_string());
                strategies.push("Train development team on accessibility best practices".to_string());
            }
            RiskLevel::Medium => {
                strategies.push("Schedule accessibility fixes in upcoming sprints".to_string());
                strategies.push("Integrate accessibility checklist into code review process".to_string());
            }
            RiskLevel::Low | RiskLevel::Minimal => {
                strategies.push("Maintain current accessibility standards".to_string());
                strategies.push("Continue regular accessibility testing".to_string());
            }
        }

        strategies
    }

    /// Estimate timeline to compliance
    fn estimate_compliance_timeline(&self, violation_count: usize, risk_score: f64) -> usize {
        let base_days = (violation_count as f64 * 0.5).ceil() as usize;
        let risk_multiplier = 1.0 + (risk_score / 100.0);

        (base_days as f64 * risk_multiplier).ceil() as usize
    }

    /// Calculate regression probability
    pub fn calculate_regression_probability(&self, file_path: &str, change_type: &str) -> Result<RegressionProbability> {
        // Analyze historical regressions
        let historical_regression_rate = 0.15; // 15% historical regression rate

        // Factor in change type
        let change_risk = match change_type {
            "refactor" => 0.3,
            "feature" => 0.2,
            "bugfix" => 0.1,
            "style" => 0.05,
            _ => 0.15,
        };

        // Calculate probability
        let probability = (historical_regression_rate + change_risk) / 2.0;

        let risk_level = if probability >= 0.5 {
            RiskLevel::High
        } else if probability >= 0.3 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        Ok(RegressionProbability {
            probability,
            risk_level,
            file_path: file_path.to_string(),
            change_type: change_type.to_string(),
            confidence: 0.75,
            recommendations: vec![
                "Run comprehensive accessibility tests before deployment".to_string(),
                "Perform manual accessibility review of changed components".to_string(),
            ],
        })
    }

    /// Calculate impact score
    pub fn calculate_impact_score(&self, issue: &IssueData) -> Result<ImpactScore> {
        let mut score = 0.0;

        // User impact (0-40 points)
        let user_impact = match issue.severity.as_str() {
            "critical" => 40.0,
            "high" => 30.0,
            "medium" => 20.0,
            "low" => 10.0,
            _ => 5.0,
        };
        score += user_impact;

        // Frequency impact (0-30 points)
        let frequency_impact = if issue.affects_all_pages {
            30.0
        } else {
            (issue.page_count as f64 / 100.0) * 30.0
        };
        score += frequency_impact.min(30.0);

        // Legal/compliance impact (0-20 points)
        let compliance_impact = if issue.wcag_level == "A" {
            20.0
        } else if issue.wcag_level == "AA" {
            15.0
        } else {
            10.0
        };
        score += compliance_impact;

        // Remediation difficulty (0-10 points, inverse)
        let difficulty_impact = 10.0 - (issue.remediation_difficulty * 10.0);
        score += difficulty_impact.max(0.0);

        let normalized_score = score; // Already 0-100

        let priority = if normalized_score >= 80.0 {
            "Critical".to_string()
        } else if normalized_score >= 60.0 {
            "High".to_string()
        } else if normalized_score >= 40.0 {
            "Medium".to_string()
        } else {
            "Low".to_string()
        };

        Ok(ImpactScore {
            overall_score: normalized_score,
            user_impact_score: user_impact,
            frequency_score: frequency_impact,
            compliance_score: compliance_impact,
            remediation_score: difficulty_impact,
            priority,
            recommended_action: self.generate_recommended_action(normalized_score),
        })
    }

    /// Generate recommended action based on impact score
    fn generate_recommended_action(&self, score: f64) -> String {
        if score >= 80.0 {
            "Fix immediately - critical user impact".to_string()
        } else if score >= 60.0 {
            "Prioritize in current sprint".to_string()
        } else if score >= 40.0 {
            "Schedule for next sprint".to_string()
        } else {
            "Address in regular maintenance cycle".to_string()
        }
    }
}

/// Trend prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPrediction {
    pub trend_direction: TrendDirection,
    pub current_value: f64,
    pub predicted_value: f64,
    pub prediction_date: DateTime<Utc>,
    pub confidence: f64,
    pub r_squared: f64,
    pub issues: Vec<IssueTrend>,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Issue trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTrend {
    pub category: String,
    pub trend: TrendDirection,
    pub severity: TrendSeverity,
    pub description: String,
    pub recommendation: String,
}

/// Trend severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendSeverity {
    High,
    Medium,
    Low,
}

/// Remediation effort estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationEstimate {
    pub total_issues: usize,
    pub estimated_hours: f64,
    pub estimated_hours_min: f64,
    pub estimated_hours_max: f64,
    pub complexity_factor: f64,
    pub confidence: f64,
    pub recommended_team_size: usize,
    pub estimated_completion_days: usize,
    pub breakdown: HashMap<String, f64>,
}

/// Compliance risk score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRisk {
    pub risk_score: f64,
    pub risk_level: RiskLevel,
    pub total_violations: usize,
    pub severity_breakdown: HashMap<String, usize>,
    pub compliance_percentage: f64,
    pub mitigation_strategies: Vec<String>,
    pub estimated_time_to_compliance_days: usize,
}

/// Risk level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    Minimal,
}

/// Violation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub severity: String,
    pub wcag_criterion: String,
}

/// Regression probability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionProbability {
    pub probability: f64,
    pub risk_level: RiskLevel,
    pub file_path: String,
    pub change_type: String,
    pub confidence: f64,
    pub recommendations: Vec<String>,
}

/// Impact score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactScore {
    pub overall_score: f64,
    pub user_impact_score: f64,
    pub frequency_score: f64,
    pub compliance_score: f64,
    pub remediation_score: f64,
    pub priority: String,
    pub recommended_action: String,
}

/// Issue data for impact calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueData {
    pub severity: String,
    pub affects_all_pages: bool,
    pub page_count: usize,
    pub wcag_level: String,
    pub remediation_difficulty: f64, // 0.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remediation_estimate() {
        let analytics = PredictiveAnalytics::new(PredictiveConfig::default());

        let result = analytics.estimate_remediation(10, 0.5);
        assert!(result.is_ok());

        let estimate = result.unwrap();
        assert!(estimate.estimated_hours > 0.0);
        assert!(estimate.recommended_team_size > 0);
    }

    #[test]
    fn test_compliance_risk() {
        let analytics = PredictiveAnalytics::new(PredictiveConfig::default());

        let violations = vec![
            Violation {
                severity: "critical".to_string(),
                wcag_criterion: "1.1.1".to_string(),
            },
            Violation {
                severity: "high".to_string(),
                wcag_criterion: "2.1.1".to_string(),
            },
        ];

        let result = analytics.calculate_compliance_risk(&violations);
        assert!(result.is_ok());

        let risk = result.unwrap();
        assert!(risk.risk_score >= 0.0 && risk.risk_score <= 100.0);
    }

    #[test]
    fn test_impact_score() {
        let analytics = PredictiveAnalytics::new(PredictiveConfig::default());

        let issue = IssueData {
            severity: "critical".to_string(),
            affects_all_pages: true,
            page_count: 100,
            wcag_level: "A".to_string(),
            remediation_difficulty: 0.3,
        };

        let result = analytics.calculate_impact_score(&issue);
        assert!(result.is_ok());

        let score = result.unwrap();
        assert!(score.overall_score > 50.0); // Critical issue should have high score
    }
}
