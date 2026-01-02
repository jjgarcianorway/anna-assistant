//! Types and constants for recipe matching.

use crate::recipe::Recipe;

/// Base minimum score threshold for recipe match (out of 100)
/// v0.0.373: Now dynamically adjusted based on recipe maturity
pub const BASE_MATCH_THRESHOLD: u32 = 60;

/// Minimum tokens that must match for a valid match
pub const MIN_MATCHING_TOKENS: usize = 2;

/// Result of matching a query against learned recipes
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The matched recipe
    pub recipe: Recipe,
    /// Match score (0-100)
    pub score: u32,
    /// Tokens that matched between query and recipe
    pub matched_tokens: Vec<String>,
    /// Whether this is a high-confidence match (can skip LLM)
    pub high_confidence: bool,
    /// Suggested parameter substitutions
    pub substitutions: Vec<(String, String)>,
}

impl MatchResult {
    /// Check if this match is strong enough to use without LLM
    /// v0.0.373: Uses dynamic threshold based on recipe maturity
    pub fn can_skip_llm(&self) -> bool {
        let threshold = super::threshold::dynamic_threshold(&self.recipe);
        self.high_confidence && self.score >= threshold
    }
}
