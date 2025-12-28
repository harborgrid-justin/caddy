// CADDY v0.1.5 - Data Protection and DLP
// Data classification, sensitive data detection, and redaction

use crate::enterprise::security::{SecurityError, SecurityResult, SecurityLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

/// Data classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataClassification {
    pub classification_id: String,
    pub security_level: SecurityLevel,
    pub data_category: DataCategory,
    pub sensitivity_score: u8, // 0-100
    pub retention_policy: RetentionPolicy,
    pub access_restrictions: Vec<String>,
    pub compliance_tags: Vec<String>,
}

/// Data category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataCategory {
    PersonalData,
    FinancialData,
    HealthData,
    IntellectualProperty,
    BusinessConfidential,
    SystemData,
    PublicData,
}

/// Retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub retain_days: Option<u32>,
    pub auto_delete: bool,
    pub archive_after_days: Option<u32>,
    pub legal_hold: bool,
}

/// DLP rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlpRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub pattern_type: PatternType,
    pub pattern: String,
    pub action: DlpAction,
    pub severity: DlpSeverity,
    pub enabled: bool,
}

/// Pattern type for DLP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Regex,
    Keyword,
    Fingerprint,
    MachineLearning,
}

/// DLP action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DlpAction {
    Alert,
    Block,
    Redact,
    Encrypt,
    Quarantine,
}

/// DLP severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum DlpSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Sensitive data detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveDataMatch {
    pub rule_id: String,
    pub pattern_type: String,
    pub match_start: usize,
    pub match_end: usize,
    pub matched_text: String,
    pub confidence: f32,
    pub severity: DlpSeverity,
}

/// DLP scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlpScanResult {
    pub scan_id: String,
    pub timestamp: i64,
    pub matches: Vec<SensitiveDataMatch>,
    pub total_violations: usize,
    pub highest_severity: DlpSeverity,
    pub recommended_action: DlpAction,
}

/// Redaction policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionPolicy {
    pub policy_id: String,
    pub redaction_type: RedactionType,
    pub preserve_format: bool,
    pub replacement_char: char,
    pub partial_redaction: bool,
    pub show_last_chars: usize,
}

/// Redaction type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedactionType {
    FullRedaction,
    PartialRedaction,
    Masking,
    Hashing,
    Tokenization,
}

/// Redacted content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactedContent {
    pub original_length: usize,
    pub redacted_text: String,
    pub redaction_map: Vec<RedactionRange>,
    pub reversible: bool,
}

/// Redaction range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRange {
    pub start: usize,
    pub end: usize,
    pub redaction_type: String,
    pub token_id: Option<String>,
}

/// Main data protection service
pub struct DataProtection {
    classifications: HashMap<String, DataClassification>,
    dlp_rules: HashMap<String, DlpRule>,
    redaction_policies: HashMap<String, RedactionPolicy>,
    predefined_patterns: HashMap<String, String>,
}

impl DataProtection {
    /// Create a new data protection service
    pub fn new() -> Self {
        let mut service = Self {
            classifications: HashMap::new(),
            dlp_rules: HashMap::new(),
            redaction_policies: HashMap::new(),
            predefined_patterns: HashMap::new(),
        };

        service.initialize_predefined_patterns();
        service.initialize_default_rules();

        service
    }

    /// Classify data
    pub fn classify_data(
        &self,
        data: &str,
        context: &HashMap<String, String>,
    ) -> SecurityResult<DataClassification> {
        // Scan for sensitive patterns
        let scan_result = self.scan_for_sensitive_data(data)?;

        // Determine classification based on findings
        let (security_level, data_category, sensitivity_score) = if scan_result.total_violations > 0 {
            match scan_result.highest_severity {
                DlpSeverity::Critical => (SecurityLevel::TopSecret, DataCategory::PersonalData, 95),
                DlpSeverity::High => (SecurityLevel::Secret, DataCategory::FinancialData, 80),
                DlpSeverity::Medium => (SecurityLevel::Confidential, DataCategory::BusinessConfidential, 60),
                DlpSeverity::Low => (SecurityLevel::Internal, DataCategory::SystemData, 30),
            }
        } else {
            (SecurityLevel::Internal, DataCategory::PublicData, 10)
        };

        // Apply context-based classification
        let (adjusted_level, adjusted_category) = self.apply_context_classification(
            security_level,
            data_category,
            context,
        );

        Ok(DataClassification {
            classification_id: format!("class_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            security_level: adjusted_level,
            data_category: adjusted_category.clone(),
            sensitivity_score,
            retention_policy: self.default_retention_policy(&adjusted_level),
            access_restrictions: vec![format!("{:?}", adjusted_level)],
            compliance_tags: self.determine_compliance_tags(&adjusted_category),
        })
    }

    /// Scan for sensitive data
    pub fn scan_for_sensitive_data(&self, data: &str) -> SecurityResult<DlpScanResult> {
        let mut matches = Vec::new();
        let mut highest_severity = DlpSeverity::Low;

        for rule in self.dlp_rules.values().filter(|r| r.enabled) {
            let rule_matches = self.apply_dlp_rule(data, rule)?;

            for m in rule_matches {
                if m.severity > highest_severity {
                    highest_severity = m.severity;
                }
                matches.push(m);
            }
        }

        let recommended_action = match highest_severity {
            DlpSeverity::Critical => DlpAction::Block,
            DlpSeverity::High => DlpAction::Redact,
            DlpSeverity::Medium => DlpAction::Encrypt,
            DlpSeverity::Low => DlpAction::Alert,
        };

        Ok(DlpScanResult {
            scan_id: format!("scan_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            timestamp: chrono::Utc::now().timestamp(),
            total_violations: matches.len(),
            matches,
            highest_severity,
            recommended_action,
        })
    }

    /// Detect specific sensitive data types
    pub fn detect_credit_cards(&self, text: &str) -> Vec<SensitiveDataMatch> {
        let pattern = self.predefined_patterns.get("credit_card").unwrap();
        self.find_pattern_matches(text, pattern, "credit_card", DlpSeverity::Critical)
    }

    pub fn detect_ssn(&self, text: &str) -> Vec<SensitiveDataMatch> {
        let pattern = self.predefined_patterns.get("ssn").unwrap();
        self.find_pattern_matches(text, pattern, "ssn", DlpSeverity::Critical)
    }

    pub fn detect_email_addresses(&self, text: &str) -> Vec<SensitiveDataMatch> {
        let pattern = self.predefined_patterns.get("email").unwrap();
        self.find_pattern_matches(text, pattern, "email", DlpSeverity::Low)
    }

    pub fn detect_phone_numbers(&self, text: &str) -> Vec<SensitiveDataMatch> {
        let pattern = self.predefined_patterns.get("phone").unwrap();
        self.find_pattern_matches(text, pattern, "phone", DlpSeverity::Medium)
    }

    pub fn detect_ip_addresses(&self, text: &str) -> Vec<SensitiveDataMatch> {
        let pattern = self.predefined_patterns.get("ip_address").unwrap();
        self.find_pattern_matches(text, pattern, "ip_address", DlpSeverity::Low)
    }

    /// Redact sensitive data
    pub fn redact_data(
        &self,
        data: &str,
        policy: &RedactionPolicy,
    ) -> SecurityResult<RedactedContent> {
        let scan_result = self.scan_for_sensitive_data(data)?;

        let mut redacted = data.to_string();
        let mut redaction_map = Vec::new();

        // Sort matches by position (reverse order to maintain indices)
        let mut sorted_matches = scan_result.matches;
        sorted_matches.sort_by(|a, b| b.match_start.cmp(&a.match_start));

        for m in sorted_matches {
            let redacted_text = match policy.redaction_type {
                RedactionType::FullRedaction => {
                    policy.replacement_char.to_string().repeat(m.match_end - m.match_start)
                }
                RedactionType::PartialRedaction => {
                    let show_chars = policy.show_last_chars.min(m.match_end - m.match_start);
                    let redact_chars = (m.match_end - m.match_start) - show_chars;
                    format!(
                        "{}{}",
                        policy.replacement_char.to_string().repeat(redact_chars),
                        &data[m.match_end - show_chars..m.match_end]
                    )
                }
                RedactionType::Masking => {
                    if policy.preserve_format {
                        self.mask_preserving_format(&data[m.match_start..m.match_end], policy.replacement_char)
                    } else {
                        policy.replacement_char.to_string().repeat(m.match_end - m.match_start)
                    }
                }
                RedactionType::Hashing => {
                    format!("[HASH:{}]", self.simple_hash(&data[m.match_start..m.match_end]))
                }
                RedactionType::Tokenization => {
                    let token_id = format!("TOKEN_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
                    format!("[{}]", token_id)
                }
            };

            redacted.replace_range(m.match_start..m.match_end, &redacted_text);

            redaction_map.push(RedactionRange {
                start: m.match_start,
                end: m.match_end,
                redaction_type: format!("{:?}", policy.redaction_type),
                token_id: None,
            });
        }

        Ok(RedactedContent {
            original_length: data.len(),
            redacted_text: redacted,
            redaction_map,
            reversible: matches!(policy.redaction_type, RedactionType::Tokenization),
        })
    }

    /// Add DLP rule
    pub fn add_dlp_rule(&mut self, rule: DlpRule) {
        self.dlp_rules.insert(rule.rule_id.clone(), rule);
    }

    /// Remove DLP rule
    pub fn remove_dlp_rule(&mut self, rule_id: &str) -> SecurityResult<()> {
        self.dlp_rules.remove(rule_id)
            .ok_or_else(|| SecurityError::InvalidInput(format!("Rule not found: {}", rule_id)))?;
        Ok(())
    }

    /// Get all DLP rules
    pub fn get_dlp_rules(&self) -> Vec<DlpRule> {
        self.dlp_rules.values().cloned().collect()
    }

    /// Add redaction policy
    pub fn add_redaction_policy(&mut self, policy: RedactionPolicy) {
        self.redaction_policies.insert(policy.policy_id.clone(), policy);
    }

    /// Get redaction policy
    pub fn get_redaction_policy(&self, policy_id: &str) -> SecurityResult<RedactionPolicy> {
        self.redaction_policies.get(policy_id)
            .cloned()
            .ok_or_else(|| SecurityError::InvalidInput(format!("Policy not found: {}", policy_id)))
    }

    // Helper methods

    fn initialize_predefined_patterns(&mut self) {
        // Credit card pattern (simplified)
        self.predefined_patterns.insert(
            "credit_card".to_string(),
            r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b".to_string(),
        );

        // SSN pattern
        self.predefined_patterns.insert(
            "ssn".to_string(),
            r"\b\d{3}-\d{2}-\d{4}\b".to_string(),
        );

        // Email pattern
        self.predefined_patterns.insert(
            "email".to_string(),
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
        );

        // Phone pattern (US format)
        self.predefined_patterns.insert(
            "phone".to_string(),
            r"\b(\+1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b".to_string(),
        );

        // IP address pattern
        self.predefined_patterns.insert(
            "ip_address".to_string(),
            r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b".to_string(),
        );

        // API key pattern
        self.predefined_patterns.insert(
            "api_key".to_string(),
            r"\b[A-Za-z0-9_-]{32,}\b".to_string(),
        );
    }

    fn initialize_default_rules(&mut self) {
        // Credit card detection rule
        self.add_dlp_rule(DlpRule {
            rule_id: "rule_credit_card".to_string(),
            name: "Credit Card Detection".to_string(),
            description: "Detects credit card numbers".to_string(),
            pattern_type: PatternType::Regex,
            pattern: self.predefined_patterns.get("credit_card").unwrap().clone(),
            action: DlpAction::Redact,
            severity: DlpSeverity::Critical,
            enabled: true,
        });

        // SSN detection rule
        self.add_dlp_rule(DlpRule {
            rule_id: "rule_ssn".to_string(),
            name: "SSN Detection".to_string(),
            description: "Detects Social Security Numbers".to_string(),
            pattern_type: PatternType::Regex,
            pattern: self.predefined_patterns.get("ssn").unwrap().clone(),
            action: DlpAction::Block,
            severity: DlpSeverity::Critical,
            enabled: true,
        });

        // Email detection rule
        self.add_dlp_rule(DlpRule {
            rule_id: "rule_email".to_string(),
            name: "Email Detection".to_string(),
            description: "Detects email addresses".to_string(),
            pattern_type: PatternType::Regex,
            pattern: self.predefined_patterns.get("email").unwrap().clone(),
            action: DlpAction::Alert,
            severity: DlpSeverity::Low,
            enabled: true,
        });
    }

    fn apply_dlp_rule(&self, data: &str, rule: &DlpRule) -> SecurityResult<Vec<SensitiveDataMatch>> {
        match rule.pattern_type {
            PatternType::Regex => {
                Ok(self.find_pattern_matches(data, &rule.pattern, &rule.rule_id, rule.severity))
            }
            PatternType::Keyword => {
                Ok(self.find_keyword_matches(data, &rule.pattern, &rule.rule_id, rule.severity))
            }
            PatternType::Fingerprint => {
                // Placeholder for fingerprinting
                Ok(Vec::new())
            }
            PatternType::MachineLearning => {
                // Placeholder for ML detection
                Ok(Vec::new())
            }
        }
    }

    fn find_pattern_matches(
        &self,
        text: &str,
        pattern: &str,
        rule_id: &str,
        severity: DlpSeverity,
    ) -> Vec<SensitiveDataMatch> {
        let mut matches = Vec::new();

        // Simple pattern matching (in production, use regex crate properly)
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.find_iter(text) {
                matches.push(SensitiveDataMatch {
                    rule_id: rule_id.to_string(),
                    pattern_type: "regex".to_string(),
                    match_start: cap.start(),
                    match_end: cap.end(),
                    matched_text: cap.as_str().to_string(),
                    confidence: 0.95,
                    severity,
                });
            }
        }

        matches
    }

    fn find_keyword_matches(
        &self,
        text: &str,
        keyword: &str,
        rule_id: &str,
        severity: DlpSeverity,
    ) -> Vec<SensitiveDataMatch> {
        let mut matches = Vec::new();
        let lower_text = text.to_lowercase();
        let lower_keyword = keyword.to_lowercase();

        let mut start = 0;
        while let Some(pos) = lower_text[start..].find(&lower_keyword) {
            let match_start = start + pos;
            let match_end = match_start + keyword.len();

            matches.push(SensitiveDataMatch {
                rule_id: rule_id.to_string(),
                pattern_type: "keyword".to_string(),
                match_start,
                match_end,
                matched_text: text[match_start..match_end].to_string(),
                confidence: 1.0,
                severity,
            });

            start = match_end;
        }

        matches
    }

    fn mask_preserving_format(&self, text: &str, mask_char: char) -> String {
        text.chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    mask_char
                } else {
                    c
                }
            })
            .collect()
    }

    fn simple_hash(&self, text: &str) -> String {
        let mut hash = 0u32;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        format!("{:08x}", hash)
    }

    fn apply_context_classification(
        &self,
        level: SecurityLevel,
        category: DataCategory,
        context: &HashMap<String, String>,
    ) -> (SecurityLevel, DataCategory) {
        // Adjust based on context
        if context.get("source").map(|s| s.as_str()) == Some("healthcare") {
            return (SecurityLevel::Secret, DataCategory::HealthData);
        }

        if context.get("contains_pii").map(|s| s.as_str()) == Some("true") {
            return (level.max(SecurityLevel::Confidential), DataCategory::PersonalData);
        }

        (level, category)
    }

    fn default_retention_policy(&self, level: &SecurityLevel) -> RetentionPolicy {
        match level {
            SecurityLevel::TopSecret => RetentionPolicy {
                retain_days: Some(2555), // 7 years
                auto_delete: false,
                archive_after_days: Some(365),
                legal_hold: true,
            },
            SecurityLevel::Secret => RetentionPolicy {
                retain_days: Some(1825), // 5 years
                auto_delete: false,
                archive_after_days: Some(180),
                legal_hold: false,
            },
            SecurityLevel::Confidential => RetentionPolicy {
                retain_days: Some(1095), // 3 years
                auto_delete: true,
                archive_after_days: Some(90),
                legal_hold: false,
            },
            _ => RetentionPolicy {
                retain_days: Some(365), // 1 year
                auto_delete: true,
                archive_after_days: Some(30),
                legal_hold: false,
            },
        }
    }

    fn determine_compliance_tags(&self, category: &DataCategory) -> Vec<String> {
        match category {
            DataCategory::PersonalData => vec!["GDPR".to_string(), "CCPA".to_string()],
            DataCategory::FinancialData => vec!["PCI-DSS".to_string(), "SOX".to_string()],
            DataCategory::HealthData => vec!["HIPAA".to_string(), "HITECH".to_string()],
            _ => vec![],
        }
    }
}

impl Default for DataProtection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_card_detection() {
        let protection = DataProtection::new();
        let text = "My credit card is 4532-1234-5678-9010";

        let matches = protection.detect_credit_cards(text);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_data_redaction() {
        let protection = DataProtection::new();
        let text = "Contact: john@example.com, Phone: 555-123-4567";

        let policy = RedactionPolicy {
            policy_id: "test_policy".to_string(),
            redaction_type: RedactionType::FullRedaction,
            preserve_format: false,
            replacement_char: '*',
            partial_redaction: false,
            show_last_chars: 0,
        };

        let result = protection.redact_data(text, &policy).unwrap();
        assert!(result.redacted_text.contains('*'));
    }

    #[test]
    fn test_data_classification() {
        let protection = DataProtection::new();
        let data = "User SSN: 123-45-6789";
        let context = HashMap::new();

        let classification = protection.classify_data(data, &context).unwrap();
        assert!(classification.sensitivity_score > 50);
    }
}
