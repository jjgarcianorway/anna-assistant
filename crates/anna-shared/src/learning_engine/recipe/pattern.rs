//! Recipe pattern matching logic (v0.0.427).
//!
//! Defines how questions are matched to recipes using:
//! - Intent classification
//! - Keyword matching
//! - Required and optional signal matching

use serde::{Deserialize, Serialize};

/// Pattern for matching questions to recipes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipePattern {
    /// Primary intent (e.g., "debug_failed_service", "check_free_ram")
    pub intent: String,
    /// Keywords that should appear in the question
    pub keywords: Vec<String>,
    /// Required probe signals (e.g., ["probe:systemd_failed_units"])
    #[serde(default)]
    pub required_signals: Vec<String>,
    /// Optional signals that improve match quality
    #[serde(default)]
    pub optional_signals: Vec<String>,
}

impl RecipePattern {
    /// Create a new pattern with intent
    pub fn new(intent: &str) -> Self {
        Self {
            intent: intent.to_string(),
            ..Default::default()
        }
    }

    /// Add keywords
    pub fn with_keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add required signals
    pub fn with_required_signals(mut self, signals: &[&str]) -> Self {
        self.required_signals = signals.iter().map(|s| s.to_string()).collect();
        self
    }
}
