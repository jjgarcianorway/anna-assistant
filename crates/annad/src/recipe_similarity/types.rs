//! Types and constants for recipe similarity matching.

use anna_shared::recipe::Recipe;

/// Minimum similarity score (0-100) to consider queries equivalent
/// v0.0.290: Increased from 75 to 85 to reduce false positives
/// v0.0.293: Increased to 90 - small models are unreliable at semantic judgment
pub const SIMILARITY_THRESHOLD: u8 = 90;

/// Maximum number of recipe candidates to check with LLM
pub const MAX_CANDIDATES: usize = 5;

/// Result of semantic similarity check
#[derive(Debug, Clone)]
pub struct SimilarityResult {
    pub is_similar: bool,
    pub score: u8,
    pub matched_recipe: Option<Recipe>,
    pub original_query: String,
}

impl SimilarityResult {
    pub fn no_match() -> Self {
        Self {
            is_similar: false,
            score: 0,
            matched_recipe: None,
            original_query: String::new(),
        }
    }
}
