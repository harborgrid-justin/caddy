//! Natural Language Processing for Accessibility
//!
//! Provides NLP-based accessibility analysis including:
//! - Readability scoring (Flesch-Kincaid, SMOG, etc.)
//! - Plain language suggestions
//! - Heading structure analysis
//! - Link text analysis
//! - Form label quality assessment

use crate::ai::{AIError, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// NLP analyzer for text accessibility
pub struct NLPAnalyzer {
    config: NLPConfig,
}

/// NLP analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLPConfig {
    /// Enable readability analysis
    pub enable_readability: bool,
    /// Enable plain language suggestions
    pub enable_plain_language: bool,
    /// Enable heading analysis
    pub enable_heading_analysis: bool,
    /// Enable link text analysis
    pub enable_link_analysis: bool,
    /// Enable form label analysis
    pub enable_form_analysis: bool,
    /// Target reading level (grade)
    pub target_reading_level: u8,
    /// Minimum confidence for suggestions
    pub suggestion_confidence_threshold: f64,
}

impl Default for NLPConfig {
    fn default() -> Self {
        Self {
            enable_readability: true,
            enable_plain_language: true,
            enable_heading_analysis: true,
            enable_link_analysis: true,
            enable_form_analysis: true,
            target_reading_level: 8, // 8th grade level
            suggestion_confidence_threshold: 0.7,
        }
    }
}

impl NLPAnalyzer {
    /// Create a new NLP analyzer
    pub fn new(config: NLPConfig) -> Self {
        Self { config }
    }

    /// Analyze text content for accessibility
    pub fn analyze_text(&self, text: &str) -> Result<TextAnalysis> {
        let mut analysis = TextAnalysis::default();

        if self.config.enable_readability {
            analysis.readability = Some(self.calculate_readability(text)?);
        }

        if self.config.enable_plain_language {
            analysis.plain_language_suggestions = self.analyze_plain_language(text)?;
        }

        Ok(analysis)
    }

    /// Calculate readability metrics
    pub fn calculate_readability(&self, text: &str) -> Result<ReadabilityScore> {
        let metrics = self.extract_text_metrics(text);

        // Flesch Reading Ease (0-100, higher is easier)
        let flesch_reading_ease = if metrics.total_sentences > 0 && metrics.total_words > 0 {
            206.835
                - 1.015 * (metrics.total_words as f64 / metrics.total_sentences as f64)
                - 84.6 * (metrics.total_syllables as f64 / metrics.total_words as f64)
        } else {
            0.0
        };

        // Flesch-Kincaid Grade Level
        let flesch_kincaid_grade = if metrics.total_sentences > 0 && metrics.total_words > 0 {
            0.39 * (metrics.total_words as f64 / metrics.total_sentences as f64)
                + 11.8 * (metrics.total_syllables as f64 / metrics.total_words as f64)
                - 15.59
        } else {
            0.0
        };

        // SMOG Index (for texts with 30+ sentences)
        let smog_index = if metrics.total_sentences >= 30 {
            let polysyllables = metrics.words_by_syllables.get(&3).unwrap_or(&0)
                + metrics.words_by_syllables.get(&4).unwrap_or(&0);
            1.0430 * ((polysyllables as f64 * 30.0 / metrics.total_sentences as f64).sqrt())
                + 3.1291
        } else {
            0.0
        };

        // Gunning Fog Index
        let complex_words = metrics.words_by_syllables.iter()
            .filter(|(syllables, _)| **syllables >= 3)
            .map(|(_, count)| count)
            .sum::<usize>();

        let gunning_fog = if metrics.total_sentences > 0 && metrics.total_words > 0 {
            0.4 * ((metrics.total_words as f64 / metrics.total_sentences as f64)
                + 100.0 * (complex_words as f64 / metrics.total_words as f64))
        } else {
            0.0
        };

        // Automated Readability Index (ARI)
        let ari = if metrics.total_sentences > 0 && metrics.total_words > 0 {
            4.71 * (metrics.total_characters as f64 / metrics.total_words as f64)
                + 0.5 * (metrics.total_words as f64 / metrics.total_sentences as f64)
                - 21.43
        } else {
            0.0
        };

        // Coleman-Liau Index
        let coleman_liau = if metrics.total_words > 0 {
            let l = (metrics.total_characters as f64 / metrics.total_words as f64) * 100.0;
            let s = (metrics.total_sentences as f64 / metrics.total_words as f64) * 100.0;
            0.0588 * l - 0.296 * s - 15.8
        } else {
            0.0
        };

        let avg_grade_level = (flesch_kincaid_grade + smog_index + gunning_fog + ari + coleman_liau) / 5.0;

        let is_accessible = avg_grade_level <= self.config.target_reading_level as f64;

        Ok(ReadabilityScore {
            flesch_reading_ease,
            flesch_kincaid_grade,
            smog_index,
            gunning_fog,
            automated_readability_index: ari,
            coleman_liau_index: coleman_liau,
            average_grade_level: avg_grade_level,
            is_accessible,
            metrics,
            recommendations: self.generate_readability_recommendations(avg_grade_level),
        })
    }

    /// Extract text metrics for readability calculation
    fn extract_text_metrics(&self, text: &str) -> ReadabilityMetrics {
        let sentences = self.split_sentences(text);
        let total_sentences = sentences.len();

        let mut total_words = 0;
        let mut total_syllables = 0;
        let mut total_characters = 0;
        let mut words_by_syllables: HashMap<usize, usize> = HashMap::new();
        let mut sentence_lengths = Vec::new();

        for sentence in &sentences {
            let words = self.split_words(sentence);
            let word_count = words.len();
            sentence_lengths.push(word_count);
            total_words += word_count;

            for word in words {
                let syllable_count = self.count_syllables(&word);
                total_syllables += syllable_count;
                total_characters += word.chars().count();
                *words_by_syllables.entry(syllable_count).or_insert(0) += 1;
            }
        }

        let avg_sentence_length = if total_sentences > 0 {
            total_words as f64 / total_sentences as f64
        } else {
            0.0
        };

        let avg_word_length = if total_words > 0 {
            total_characters as f64 / total_words as f64
        } else {
            0.0
        };

        ReadabilityMetrics {
            total_words,
            total_sentences,
            total_syllables,
            total_characters,
            words_by_syllables,
            sentence_lengths,
            average_sentence_length: avg_sentence_length,
            average_word_length: avg_word_length,
        }
    }

    /// Split text into sentences
    fn split_sentences(&self, text: &str) -> Vec<String> {
        let re = Regex::new(r"[.!?]+\s+").unwrap();
        re.split(text)
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Split sentence into words
    fn split_words(&self, sentence: &str) -> Vec<String> {
        let re = Regex::new(r"\W+").unwrap();
        re.split(sentence)
            .filter(|w| !w.trim().is_empty())
            .map(|w| w.to_lowercase())
            .collect()
    }

    /// Count syllables in a word (simplified algorithm)
    fn count_syllables(&self, word: &str) -> usize {
        let word = word.to_lowercase();
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];

        let mut count = 0;
        let mut previous_was_vowel = false;

        for ch in word.chars() {
            let is_vowel = vowels.contains(&ch);
            if is_vowel && !previous_was_vowel {
                count += 1;
            }
            previous_was_vowel = is_vowel;
        }

        // Handle silent 'e'
        if word.ends_with('e') && count > 1 {
            count -= 1;
        }

        // Ensure at least one syllable
        count.max(1)
    }

    /// Generate readability recommendations
    fn generate_readability_recommendations(&self, grade_level: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if grade_level > self.config.target_reading_level as f64 {
            recommendations.push(format!(
                "Text is at grade {} level, consider simplifying to grade {} or lower",
                grade_level.round(),
                self.config.target_reading_level
            ));

            recommendations.push("Use shorter sentences (15-20 words maximum)".to_string());
            recommendations.push("Replace complex words with simpler alternatives".to_string());
            recommendations.push("Break long paragraphs into shorter ones".to_string());
            recommendations.push("Use bullet points or lists for complex information".to_string());
        }

        recommendations
    }

    /// Analyze for plain language suggestions
    fn analyze_plain_language(&self, text: &str) -> Result<Vec<PlainLanguageSuggestion>> {
        let mut suggestions = Vec::new();

        // Check for passive voice
        suggestions.extend(self.detect_passive_voice(text));

        // Check for jargon and complex terms
        suggestions.extend(self.detect_jargon(text));

        // Check for wordy phrases
        suggestions.extend(self.detect_wordy_phrases(text));

        // Check for nominalization
        suggestions.extend(self.detect_nominalizations(text));

        Ok(suggestions)
    }

    /// Detect passive voice
    fn detect_passive_voice(&self, text: &str) -> Vec<PlainLanguageSuggestion> {
        let mut suggestions = Vec::new();
        let passive_patterns = vec![
            (r"\b(is|are|was|were|be|been|being)\s+\w+ed\b", "Consider using active voice"),
            (r"\b(is|are|was|were)\s+being\s+\w+ed\b", "Use active voice for clarity"),
        ];

        for (pattern, suggestion) in passive_patterns {
            let re = Regex::new(pattern).unwrap();
            for mat in re.find_iter(text) {
                suggestions.push(PlainLanguageSuggestion {
                    original_text: mat.as_str().to_string(),
                    suggestion: suggestion.to_string(),
                    replacement: None,
                    confidence: 0.8,
                    category: SuggestionCategory::PassiveVoice,
                    position: mat.start(),
                    length: mat.len(),
                });
            }
        }

        suggestions
    }

    /// Detect jargon and complex terms
    fn detect_jargon(&self, text: &str) -> Vec<PlainLanguageSuggestion> {
        let jargon_replacements = vec![
            ("utilize", "use"),
            ("facilitate", "help"),
            ("implement", "start" ),
            ("commence", "start"),
            ("terminate", "end"),
            ("prior to", "before"),
            ("subsequent to", "after"),
            ("in order to", "to"),
            ("due to the fact that", "because"),
        ];

        let mut suggestions = Vec::new();
        let text_lower = text.to_lowercase();

        for (jargon, replacement) in jargon_replacements {
            if let Some(pos) = text_lower.find(jargon) {
                suggestions.push(PlainLanguageSuggestion {
                    original_text: jargon.to_string(),
                    suggestion: format!("Replace '{}' with simpler term", jargon),
                    replacement: Some(replacement.to_string()),
                    confidence: 0.9,
                    category: SuggestionCategory::Jargon,
                    position: pos,
                    length: jargon.len(),
                });
            }
        }

        suggestions
    }

    /// Detect wordy phrases
    fn detect_wordy_phrases(&self, text: &str) -> Vec<PlainLanguageSuggestion> {
        let wordy_phrases = vec![
            ("at this point in time", "now"),
            ("in the event that", "if"),
            ("on a daily basis", "daily"),
            ("with regard to", "about"),
            ("for the purpose of", "to"),
        ];

        let mut suggestions = Vec::new();
        let text_lower = text.to_lowercase();

        for (wordy, concise) in wordy_phrases {
            if let Some(pos) = text_lower.find(wordy) {
                suggestions.push(PlainLanguageSuggestion {
                    original_text: wordy.to_string(),
                    suggestion: "Use more concise phrasing".to_string(),
                    replacement: Some(concise.to_string()),
                    confidence: 0.95,
                    category: SuggestionCategory::WordyPhrase,
                    position: pos,
                    length: wordy.len(),
                });
            }
        }

        suggestions
    }

    /// Detect nominalizations
    fn detect_nominalizations(&self, text: &str) -> Vec<PlainLanguageSuggestion> {
        let nominalizations = vec![
            ("implementation", "implement"),
            ("utilization", "use"),
            ("modification", "modify"),
            ("determination", "determine"),
        ];

        let mut suggestions = Vec::new();
        let text_lower = text.to_lowercase();

        for (noun, verb) in nominalizations {
            if let Some(pos) = text_lower.find(noun) {
                suggestions.push(PlainLanguageSuggestion {
                    original_text: noun.to_string(),
                    suggestion: "Consider using verb form".to_string(),
                    replacement: Some(verb.to_string()),
                    confidence: 0.75,
                    category: SuggestionCategory::Nominalization,
                    position: pos,
                    length: noun.len(),
                });
            }
        }

        suggestions
    }

    /// Analyze heading structure
    pub fn analyze_headings(&self, headings: &[HeadingElement]) -> Result<HeadingAnalysis> {
        let mut issues = Vec::new();
        let mut previous_level = 0;

        // Check for H1
        if headings.is_empty() || headings[0].level != 1 {
            issues.push("Page should have exactly one H1 heading".to_string());
        }

        // Check for heading level skips
        for heading in headings {
            if heading.level > previous_level + 1 {
                issues.push(format!(
                    "Heading level skip detected: H{} followed by H{}",
                    previous_level, heading.level
                ));
            }
            previous_level = heading.level;
        }

        // Check for empty headings
        for heading in headings {
            if heading.text.trim().is_empty() {
                issues.push(format!("Empty H{} heading detected", heading.level));
            }
        }

        let is_valid = issues.is_empty();
        let hierarchy_depth = headings.iter().map(|h| h.level).max().unwrap_or(0);

        Ok(HeadingAnalysis {
            total_headings: headings.len(),
            hierarchy_depth,
            is_valid_hierarchy: is_valid,
            issues,
            suggestions: if !is_valid {
                vec![
                    "Ensure proper heading hierarchy (H1 > H2 > H3, etc.)".to_string(),
                    "Use headings to structure content logically".to_string(),
                ]
            } else {
                Vec::new()
            },
        })
    }

    /// Analyze link text
    pub fn analyze_link_text(&self, link_text: &str, link_url: &str) -> Result<LinkTextAnalysis> {
        let mut issues = Vec::new();
        let text = link_text.trim();

        // Check for generic link text
        let generic_terms = vec!["click here", "here", "read more", "more", "link"];
        if generic_terms.iter().any(|term| text.to_lowercase() == *term) {
            issues.push("Link text is too generic and not descriptive".to_string());
        }

        // Check for URL as link text
        if text.starts_with("http") || text.starts_with("www") {
            issues.push("Avoid using URLs as link text".to_string());
        }

        // Check for very short link text
        if text.len() < 4 && !text.is_empty() {
            issues.push("Link text is too short to be meaningful".to_string());
        }

        // Check for very long link text
        if text.len() > 100 {
            issues.push("Link text is too long, consider shortening".to_string());
        }

        let is_accessible = issues.is_empty();

        Ok(LinkTextAnalysis {
            text: text.to_string(),
            url: link_url.to_string(),
            is_accessible,
            issues,
            suggestions: if !is_accessible {
                vec![
                    "Use descriptive text that indicates the link's purpose".to_string(),
                    "Avoid generic phrases like 'click here'".to_string(),
                    "Keep link text concise but meaningful".to_string(),
                ]
            } else {
                Vec::new()
            },
        })
    }

    /// Assess form label quality
    pub fn assess_form_label(&self, label_text: &str, input_type: &str) -> Result<FormLabelQuality> {
        let mut issues = Vec::new();
        let text = label_text.trim();

        // Check for empty label
        if text.is_empty() {
            issues.push("Label is empty".to_string());
        }

        // Check for single character labels
        if text.len() == 1 {
            issues.push("Label is too short to be descriptive".to_string());
        }

        // Check for placeholder-only patterns
        if text.contains("...") || text.ends_with(':') {
            issues.push("Label should be descriptive, not just a placeholder".to_string());
        }

        // Specific checks by input type
        match input_type {
            "email" => {
                if !text.to_lowercase().contains("email") {
                    issues.push("Email input should have 'email' in the label".to_string());
                }
            }
            "password" => {
                if !text.to_lowercase().contains("password") {
                    issues.push("Password input should have 'password' in the label".to_string());
                }
            }
            _ => {}
        }

        let is_accessible = issues.is_empty() && !text.is_empty();

        Ok(FormLabelQuality {
            label_text: text.to_string(),
            input_type: input_type.to_string(),
            is_accessible,
            issues,
            suggestions: if !is_accessible {
                vec![
                    "Provide clear, descriptive labels for all form inputs".to_string(),
                    "Labels should indicate what information is expected".to_string(),
                ]
            } else {
                Vec::new()
            },
        })
    }
}

/// Text analysis result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextAnalysis {
    pub readability: Option<ReadabilityScore>,
    pub plain_language_suggestions: Vec<PlainLanguageSuggestion>,
}

/// Readability score with multiple metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityScore {
    pub flesch_reading_ease: f64,
    pub flesch_kincaid_grade: f64,
    pub smog_index: f64,
    pub gunning_fog: f64,
    pub automated_readability_index: f64,
    pub coleman_liau_index: f64,
    pub average_grade_level: f64,
    pub is_accessible: bool,
    pub metrics: ReadabilityMetrics,
    pub recommendations: Vec<String>,
}

/// Text metrics for readability calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityMetrics {
    pub total_words: usize,
    pub total_sentences: usize,
    pub total_syllables: usize,
    pub total_characters: usize,
    pub words_by_syllables: HashMap<usize, usize>,
    pub sentence_lengths: Vec<usize>,
    pub average_sentence_length: f64,
    pub average_word_length: f64,
}

/// Plain language suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlainLanguageSuggestion {
    pub original_text: String,
    pub suggestion: String,
    pub replacement: Option<String>,
    pub confidence: f64,
    pub category: SuggestionCategory,
    pub position: usize,
    pub length: usize,
}

/// Suggestion category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionCategory {
    PassiveVoice,
    Jargon,
    WordyPhrase,
    Nominalization,
    ComplexSentence,
    Other,
}

/// Heading element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingElement {
    pub level: usize,
    pub text: String,
}

/// Heading structure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingAnalysis {
    pub total_headings: usize,
    pub hierarchy_depth: usize,
    pub is_valid_hierarchy: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Link text analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkTextAnalysis {
    pub text: String,
    pub url: String,
    pub is_accessible: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Form label quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormLabelQuality {
    pub label_text: String,
    pub input_type: String,
    pub is_accessible: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syllable_counting() {
        let analyzer = NLPAnalyzer::new(NLPConfig::default());

        assert_eq!(analyzer.count_syllables("hello"), 2);
        assert_eq!(analyzer.count_syllables("world"), 1);
        assert_eq!(analyzer.count_syllables("accessibility"), 6);
    }

    #[test]
    fn test_readability_calculation() {
        let analyzer = NLPAnalyzer::new(NLPConfig::default());
        let text = "This is a simple test. It has short sentences. Very easy to read.";

        let result = analyzer.calculate_readability(text);
        assert!(result.is_ok());

        let score = result.unwrap();
        assert!(score.flesch_reading_ease > 0.0);
    }

    #[test]
    fn test_plain_language_detection() {
        let analyzer = NLPAnalyzer::new(NLPConfig::default());
        let text = "We will utilize this tool in order to facilitate implementation.";

        let suggestions = analyzer.analyze_plain_language(text).unwrap();
        assert!(!suggestions.is_empty());
    }
}
