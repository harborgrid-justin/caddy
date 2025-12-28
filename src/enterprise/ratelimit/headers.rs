//! Rate Limit HTTP Headers
//!
//! This module provides standardized HTTP headers for rate limiting:
//! - X-RateLimit-Limit
//! - X-RateLimit-Remaining
//! - X-RateLimit-Reset
//! - Retry-After
//! - RateLimit (RFC draft)

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use super::algorithm::Decision;

// ============================================================================
// Rate Limit Header Standards
// ============================================================================

/// Rate limit header standard to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeaderStandard {
    /// Traditional X-RateLimit-* headers (most common)
    Traditional,
    /// IETF RateLimit header fields (RFC draft)
    IETF,
    /// GitHub-style headers
    GitHub,
    /// Twitter-style headers
    Twitter,
    /// Custom headers
    Custom,
}

/// Rate limit information for headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Maximum requests allowed in the time window
    pub limit: u64,
    /// Remaining requests in current window
    pub remaining: u64,
    /// Time when the rate limit resets (UNIX timestamp)
    pub reset: u64,
    /// Time window in seconds
    pub window: u64,
    /// Retry after duration (if rate limited)
    pub retry_after: Option<u64>,
    /// Policy name or identifier
    pub policy: Option<String>,
}

impl RateLimitInfo {
    /// Create from a rate limit decision
    pub fn from_decision(decision: Decision, limit: u64, window: u64) -> Self {
        match decision {
            Decision::Allowed {
                remaining,
                reset_after,
            } => {
                let now_secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                Self {
                    limit,
                    remaining,
                    reset: now_secs + reset_after,
                    window,
                    retry_after: None,
                    policy: None,
                }
            }
            Decision::Denied { retry_after, .. } => {
                let now_secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                Self {
                    limit,
                    remaining: 0,
                    reset: now_secs + retry_after,
                    window,
                    retry_after: Some(retry_after),
                    policy: None,
                }
            }
        }
    }

    /// Set policy identifier
    pub fn with_policy(mut self, policy: String) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Check if rate limited
    pub fn is_rate_limited(&self) -> bool {
        self.retry_after.is_some()
    }

    /// Get time until reset
    pub fn time_until_reset(&self) -> Duration {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Duration::from_secs(self.reset.saturating_sub(now_secs))
    }
}

// ============================================================================
// Header Builder
// ============================================================================

/// Builder for rate limit headers
pub struct RateLimitHeaders {
    /// Header standard to use
    standard: HeaderStandard,
    /// Rate limit information
    info: RateLimitInfo,
    /// Additional custom headers
    custom_headers: HashMap<String, String>,
}

impl RateLimitHeaders {
    /// Create a new header builder
    pub fn new(standard: HeaderStandard, info: RateLimitInfo) -> Self {
        Self {
            standard,
            info,
            custom_headers: HashMap::new(),
        }
    }

    /// Add a custom header
    pub fn add_custom_header(mut self, name: String, value: String) -> Self {
        self.custom_headers.insert(name, value);
        self
    }

    /// Build headers as a map
    pub fn build(&self) -> HashMap<String, String> {
        let mut headers = match self.standard {
            HeaderStandard::Traditional => self.build_traditional(),
            HeaderStandard::IETF => self.build_ietf(),
            HeaderStandard::GitHub => self.build_github(),
            HeaderStandard::Twitter => self.build_twitter(),
            HeaderStandard::Custom => HashMap::new(),
        };

        // Add custom headers
        headers.extend(self.custom_headers.clone());

        headers
    }

    /// Build traditional X-RateLimit-* headers
    fn build_traditional(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        headers.insert(
            "X-RateLimit-Limit".to_string(),
            self.info.limit.to_string(),
        );

        headers.insert(
            "X-RateLimit-Remaining".to_string(),
            self.info.remaining.to_string(),
        );

        headers.insert(
            "X-RateLimit-Reset".to_string(),
            self.info.reset.to_string(),
        );

        if let Some(retry_after) = self.info.retry_after {
            headers.insert("Retry-After".to_string(), retry_after.to_string());
        }

        if let Some(policy) = &self.info.policy {
            headers.insert("X-RateLimit-Policy".to_string(), policy.clone());
        }

        headers
    }

    /// Build IETF RateLimit header (RFC draft)
    fn build_ietf(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        // RateLimit header format: limit, remaining, reset
        let ratelimit_value = format!(
            "limit={}, remaining={}, reset={}",
            self.info.limit, self.info.remaining, self.info.reset
        );

        headers.insert("RateLimit".to_string(), ratelimit_value);

        // RateLimit-Policy header (optional)
        if let Some(_policy) = &self.info.policy {
            let policy_value = format!("{};w={}", self.info.limit, self.info.window);
            headers.insert("RateLimit-Policy".to_string(), policy_value);
        }

        if let Some(retry_after) = self.info.retry_after {
            headers.insert("Retry-After".to_string(), retry_after.to_string());
        }

        headers
    }

    /// Build GitHub-style headers
    fn build_github(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        headers.insert(
            "X-RateLimit-Limit".to_string(),
            self.info.limit.to_string(),
        );

        headers.insert(
            "X-RateLimit-Remaining".to_string(),
            self.info.remaining.to_string(),
        );

        headers.insert(
            "X-RateLimit-Reset".to_string(),
            self.info.reset.to_string(),
        );

        // GitHub also includes X-RateLimit-Used
        let used = self.info.limit - self.info.remaining;
        headers.insert("X-RateLimit-Used".to_string(), used.to_string());

        // GitHub uses X-RateLimit-Resource for policy
        if let Some(policy) = &self.info.policy {
            headers.insert("X-RateLimit-Resource".to_string(), policy.clone());
        }

        if let Some(retry_after) = self.info.retry_after {
            headers.insert("Retry-After".to_string(), retry_after.to_string());
        }

        headers
    }

    /// Build Twitter-style headers
    fn build_twitter(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        headers.insert(
            "x-rate-limit-limit".to_string(),
            self.info.limit.to_string(),
        );

        headers.insert(
            "x-rate-limit-remaining".to_string(),
            self.info.remaining.to_string(),
        );

        headers.insert(
            "x-rate-limit-reset".to_string(),
            self.info.reset.to_string(),
        );

        if let Some(retry_after) = self.info.retry_after {
            headers.insert("retry-after".to_string(), retry_after.to_string());
        }

        headers
    }

    /// Format headers for HTTP response
    pub fn format_http(&self) -> Vec<String> {
        self.build()
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect()
    }
}

// ============================================================================
// Retry-After Header Handling
// ============================================================================

/// Retry-After header format
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryAfterFormat {
    /// Delay in seconds
    Seconds(u64),
    /// HTTP-date (RFC 7231)
    HttpDate(String),
}

impl RetryAfterFormat {
    /// Create from seconds
    pub fn from_seconds(seconds: u64) -> Self {
        Self::Seconds(seconds)
    }

    /// Create from duration
    pub fn from_duration(duration: Duration) -> Self {
        Self::Seconds(duration.as_secs())
    }

    /// Create from timestamp
    pub fn from_timestamp(timestamp: u64) -> Self {
        // Convert UNIX timestamp to HTTP-date format
        let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());

        let http_date = datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        Self::HttpDate(http_date)
    }

    /// Convert to header value
    pub fn to_header_value(&self) -> String {
        match self {
            Self::Seconds(secs) => secs.to_string(),
            Self::HttpDate(date) => date.clone(),
        }
    }
}

// ============================================================================
// Header Parser
// ============================================================================

/// Parser for rate limit headers
pub struct RateLimitHeaderParser;

impl RateLimitHeaderParser {
    /// Parse traditional headers
    pub fn parse_traditional(headers: &HashMap<String, String>) -> Option<RateLimitInfo> {
        let limit = headers
            .get("X-RateLimit-Limit")
            .and_then(|v| v.parse().ok())?;

        let remaining = headers
            .get("X-RateLimit-Remaining")
            .and_then(|v| v.parse().ok())?;

        let reset = headers
            .get("X-RateLimit-Reset")
            .and_then(|v| v.parse().ok())?;

        let retry_after = headers
            .get("Retry-After")
            .and_then(|v| v.parse().ok());

        let policy = headers.get("X-RateLimit-Policy").cloned();

        Some(RateLimitInfo {
            limit,
            remaining,
            reset,
            window: 0, // Not included in traditional headers
            retry_after,
            policy,
        })
    }

    /// Parse IETF RateLimit header
    pub fn parse_ietf(headers: &HashMap<String, String>) -> Option<RateLimitInfo> {
        let ratelimit = headers.get("RateLimit")?;

        // Parse: limit=100, remaining=50, reset=1234567890
        let mut limit = None;
        let mut remaining = None;
        let mut reset = None;

        for part in ratelimit.split(',') {
            let part = part.trim();
            if let Some((key, value)) = part.split_once('=') {
                match key.trim() {
                    "limit" => limit = value.trim().parse().ok(),
                    "remaining" => remaining = value.trim().parse().ok(),
                    "reset" => reset = value.trim().parse().ok(),
                    _ => {}
                }
            }
        }

        let retry_after = headers
            .get("Retry-After")
            .and_then(|v| v.parse().ok());

        let policy = headers.get("RateLimit-Policy").cloned();

        Some(RateLimitInfo {
            limit: limit?,
            remaining: remaining?,
            reset: reset?,
            window: 0,
            retry_after,
            policy,
        })
    }

    /// Parse Retry-After header
    pub fn parse_retry_after(value: &str) -> Option<RetryAfterFormat> {
        // Try parsing as seconds first
        if let Ok(seconds) = value.parse::<u64>() {
            return Some(RetryAfterFormat::Seconds(seconds));
        }

        // Try parsing as HTTP-date
        if value.contains(',') {
            return Some(RetryAfterFormat::HttpDate(value.to_string()));
        }

        None
    }
}

// ============================================================================
// Response Builder
// ============================================================================

/// HTTP response with rate limit information
#[derive(Debug, Clone)]
pub struct RateLimitResponse {
    /// HTTP status code
    pub status_code: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body (optional)
    pub body: Option<String>,
}

impl RateLimitResponse {
    /// Create a success response (200 OK)
    pub fn ok(info: RateLimitInfo, standard: HeaderStandard) -> Self {
        let headers = RateLimitHeaders::new(standard, info).build();

        Self {
            status_code: 200,
            headers,
            body: None,
        }
    }

    /// Create a rate limited response (429 Too Many Requests)
    pub fn rate_limited(info: RateLimitInfo, standard: HeaderStandard) -> Self {
        let headers = RateLimitHeaders::new(standard, info.clone()).build();

        let body = serde_json::json!({
            "error": "rate_limit_exceeded",
            "message": "Rate limit exceeded. Please retry after the specified time.",
            "limit": info.limit,
            "retry_after": info.retry_after,
            "reset": info.reset
        })
        .to_string();

        Self {
            status_code: 429,
            headers,
            body: Some(body),
        }
    }

    /// Create a custom response
    pub fn custom(
        status_code: u16,
        info: RateLimitInfo,
        standard: HeaderStandard,
        body: Option<String>,
    ) -> Self {
        let headers = RateLimitHeaders::new(standard, info).build();

        Self {
            status_code,
            headers,
            body,
        }
    }

    /// Add a custom header
    pub fn with_header(mut self, name: String, value: String) -> Self {
        self.headers.insert(name, value);
        self
    }

    /// Format as HTTP response string
    pub fn format_http(&self) -> String {
        let mut response = format!("HTTP/1.1 {} {}\r\n", self.status_code, self.status_text());

        for (name, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", name, value));
        }

        if let Some(body) = &self.body {
            response.push_str(&format!("Content-Length: {}\r\n", body.len()));
            response.push_str("Content-Type: application/json\r\n");
            response.push_str("\r\n");
            response.push_str(body);
        } else {
            response.push_str("\r\n");
        }

        response
    }

    /// Get status text for code
    fn status_text(&self) -> &str {
        match self.status_code {
            200 => "OK",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            503 => "Service Unavailable",
            _ => "Unknown",
        }
    }
}

// ============================================================================
// Header Middleware
// ============================================================================

/// Middleware for automatically adding rate limit headers
pub struct RateLimitHeaderMiddleware {
    /// Header standard to use
    standard: HeaderStandard,
    /// Include headers on successful requests
    include_on_success: bool,
}

impl RateLimitHeaderMiddleware {
    /// Create new middleware
    pub fn new(standard: HeaderStandard) -> Self {
        Self {
            standard,
            include_on_success: true,
        }
    }

    /// Set whether to include headers on successful requests
    pub fn include_on_success(mut self, include: bool) -> Self {
        self.include_on_success = include;
        self
    }

    /// Apply headers to response
    pub fn apply(
        &self,
        info: RateLimitInfo,
        existing_headers: &mut HashMap<String, String>,
    ) {
        if !self.include_on_success && !info.is_rate_limited() {
            return;
        }

        let headers = RateLimitHeaders::new(self.standard, info).build();
        existing_headers.extend(headers);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_info_from_decision() {
        let decision = Decision::Allowed {
            remaining: 50,
            reset_after: 60,
        };

        let info = RateLimitInfo::from_decision(decision, 100, 60);

        assert_eq!(info.limit, 100);
        assert_eq!(info.remaining, 50);
        assert_eq!(info.window, 60);
        assert!(info.retry_after.is_none());
    }

    #[test]
    fn test_rate_limit_info_denied() {
        let decision = Decision::Denied {
            retry_after: 30,
            limit: 100,
        };

        let info = RateLimitInfo::from_decision(decision, 100, 60);

        assert_eq!(info.remaining, 0);
        assert_eq!(info.retry_after, Some(30));
        assert!(info.is_rate_limited());
    }

    #[test]
    fn test_traditional_headers() {
        let info = RateLimitInfo {
            limit: 100,
            remaining: 50,
            reset: 1234567890,
            window: 60,
            retry_after: None,
            policy: Some("default".to_string()),
        };

        let headers = RateLimitHeaders::new(HeaderStandard::Traditional, info).build();

        assert_eq!(headers.get("X-RateLimit-Limit"), Some(&"100".to_string()));
        assert_eq!(headers.get("X-RateLimit-Remaining"), Some(&"50".to_string()));
        assert_eq!(headers.get("X-RateLimit-Reset"), Some(&"1234567890".to_string()));
        assert_eq!(headers.get("X-RateLimit-Policy"), Some(&"default".to_string()));
    }

    #[test]
    fn test_ietf_headers() {
        let info = RateLimitInfo {
            limit: 100,
            remaining: 50,
            reset: 1234567890,
            window: 60,
            retry_after: None,
            policy: None,
        };

        let headers = RateLimitHeaders::new(HeaderStandard::IETF, info).build();

        assert!(headers.contains_key("RateLimit"));
        let ratelimit = headers.get("RateLimit").unwrap();
        assert!(ratelimit.contains("limit=100"));
        assert!(ratelimit.contains("remaining=50"));
        assert!(ratelimit.contains("reset=1234567890"));
    }

    #[test]
    fn test_github_headers() {
        let info = RateLimitInfo {
            limit: 100,
            remaining: 50,
            reset: 1234567890,
            window: 60,
            retry_after: None,
            policy: None,
        };

        let headers = RateLimitHeaders::new(HeaderStandard::GitHub, info).build();

        assert_eq!(headers.get("X-RateLimit-Limit"), Some(&"100".to_string()));
        assert_eq!(headers.get("X-RateLimit-Used"), Some(&"50".to_string()));
    }

    #[test]
    fn test_retry_after_format() {
        let seconds = RetryAfterFormat::from_seconds(120);
        assert_eq!(seconds.to_header_value(), "120");

        let duration = RetryAfterFormat::from_duration(Duration::from_secs(60));
        assert_eq!(duration.to_header_value(), "60");

        let timestamp = RetryAfterFormat::from_timestamp(1234567890);
        assert!(matches!(timestamp, RetryAfterFormat::HttpDate(_)));
    }

    #[test]
    fn test_header_parser() {
        let mut headers = HashMap::new();
        headers.insert("X-RateLimit-Limit".to_string(), "100".to_string());
        headers.insert("X-RateLimit-Remaining".to_string(), "50".to_string());
        headers.insert("X-RateLimit-Reset".to_string(), "1234567890".to_string());

        let info = RateLimitHeaderParser::parse_traditional(&headers).unwrap();

        assert_eq!(info.limit, 100);
        assert_eq!(info.remaining, 50);
        assert_eq!(info.reset, 1234567890);
    }

    #[test]
    fn test_rate_limit_response() {
        let info = RateLimitInfo {
            limit: 100,
            remaining: 0,
            reset: 1234567890,
            window: 60,
            retry_after: Some(60),
            policy: None,
        };

        let response = RateLimitResponse::rate_limited(info, HeaderStandard::Traditional);

        assert_eq!(response.status_code, 429);
        assert!(response.headers.contains_key("X-RateLimit-Limit"));
        assert!(response.body.is_some());
    }

    #[test]
    fn test_custom_headers() {
        let info = RateLimitInfo {
            limit: 100,
            remaining: 50,
            reset: 1234567890,
            window: 60,
            retry_after: None,
            policy: None,
        };

        let headers = RateLimitHeaders::new(HeaderStandard::Traditional, info)
            .add_custom_header("X-Custom-Header".to_string(), "custom-value".to_string())
            .build();

        assert_eq!(headers.get("X-Custom-Header"), Some(&"custom-value".to_string()));
    }

    #[test]
    fn test_middleware() {
        let middleware = RateLimitHeaderMiddleware::new(HeaderStandard::Traditional);

        let info = RateLimitInfo {
            limit: 100,
            remaining: 50,
            reset: 1234567890,
            window: 60,
            retry_after: None,
            policy: None,
        };

        let mut headers = HashMap::new();
        middleware.apply(info, &mut headers);

        assert!(headers.contains_key("X-RateLimit-Limit"));
        assert!(headers.contains_key("X-RateLimit-Remaining"));
    }
}
