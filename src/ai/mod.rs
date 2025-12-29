//! AI-Powered Accessibility Analysis Engine for CADDY v0.3.0
//!
//! This module provides comprehensive AI-powered accessibility analysis including:
//! - Multi-model orchestration for various AI tasks
//! - Computer vision for visual accessibility analysis
//! - Natural language processing for content analysis
//! - Predictive analytics for accessibility trends
//! - AI-powered suggestions and remediation
//!
//! # Features
//!
//! - Multi-model inference pipeline with A/B testing
//! - GPU acceleration support
//! - Batch processing optimization
//! - Real-time analysis and suggestions
//! - Model versioning and rollback
//!
//! # Example
//!
//! ```no_run
//! use caddy::ai::{AIEngine, EngineConfig};
//!
//! let config = EngineConfig::default();
//! let engine = AIEngine::new(config)?;
//!
//! // Analyze an image for accessibility
//! let results = engine.analyze_image(image_data).await?;
//! ```

pub mod engine;
pub mod nlp;
pub mod predictions;
pub mod suggestions;
pub mod vision;

pub use engine::{
    AIEngine, EngineConfig, InferencePipeline, ModelVersion, ABTestConfig,
    BatchProcessor, GPUAccelerator,
};
pub use nlp::{
    NLPAnalyzer, ReadabilityScore, ReadabilityMetrics, PlainLanguageSuggestion,
    HeadingAnalysis, LinkTextAnalysis, FormLabelQuality,
};
pub use predictions::{
    PredictiveAnalytics, IssueTrend, RemediationEstimate, ComplianceRisk,
    RegressionProbability, ImpactScore, TrendPrediction,
};
pub use suggestions::{
    SuggestionEngine, AutoFix, CodeCompletion, AltTextSuggestion,
    ARIASuggestion, BestPracticeSuggestion, SuggestionConfidence,
};
pub use vision::{
    VisionAnalyzer, AltTextGeneration, ColorContrastAnalysis, VisualHierarchy,
    IconRecognition, ChartAnalysis, ImageAccessibility,
};

use std::error::Error as StdError;
use std::fmt;

/// Result type for AI operations
pub type Result<T> = std::result::Result<T, AIError>;

/// Errors that can occur in the AI engine
#[derive(Debug, Clone)]
pub enum AIError {
    /// Model not found or failed to load
    ModelError(String),
    /// Invalid configuration
    InvalidConfig(String),
    /// Inference error
    InferenceError(String),
    /// GPU acceleration error
    GPUError(String),
    /// Batch processing error
    BatchError(String),
    /// Version mismatch
    VersionError(String),
    /// Resource exhaustion
    ResourceError(String),
    /// Network error (for remote models)
    NetworkError(String),
    /// Serialization error
    SerializationError(String),
    /// I/O error
    IoError(String),
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModelError(msg) => write!(f, "Model error: {}", msg),
            Self::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            Self::GPUError(msg) => write!(f, "GPU error: {}", msg),
            Self::BatchError(msg) => write!(f, "Batch processing error: {}", msg),
            Self::VersionError(msg) => write!(f, "Version error: {}", msg),
            Self::ResourceError(msg) => write!(f, "Resource error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl StdError for AIError {}

impl From<std::io::Error> for AIError {
    fn from(err: std::io::Error) -> Self {
        AIError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for AIError {
    fn from(err: serde_json::Error) -> Self {
        AIError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AIError::ModelError("Failed to load model".to_string());
        assert_eq!(format!("{}", err), "Model error: Failed to load model");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let ai_err: AIError = io_err.into();
        assert!(matches!(ai_err, AIError::IoError(_)));
    }
}
