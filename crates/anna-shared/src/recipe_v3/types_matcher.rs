//! Recipe matcher types (v0.0.423).
//!
//! Matching criteria for recipes.

use serde::{Deserialize, Serialize};

use super::types_enums::RecipeDomain;

/// Recipe matching criteria
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeMatcher {
    /// Target domain
    pub domain: RecipeDomain,
    /// Matching intents (e.g., "restart", "enable", "check")
    pub intents: Vec<String>,
    /// Required keywords for matching
    pub keywords: Vec<String>,
    /// Optional entity patterns (e.g., service names)
    pub entity_patterns: Vec<String>,
    /// Similarity key for fuzzy matching
    pub similarity_key: String,
}

impl RecipeMatcher {
    /// Create a new matcher
    pub fn new(domain: RecipeDomain) -> Self {
        Self {
            domain,
            ..Default::default()
        }
    }

    /// Add intents
    pub fn with_intents(mut self, intents: &[&str]) -> Self {
        self.intents = intents.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add keywords
    pub fn with_keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add entity patterns
    pub fn with_entities(mut self, patterns: &[&str]) -> Self {
        self.entity_patterns = patterns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set similarity key
    pub fn with_similarity_key(mut self, key: &str) -> Self {
        self.similarity_key = key.to_string();
        self
    }
}
