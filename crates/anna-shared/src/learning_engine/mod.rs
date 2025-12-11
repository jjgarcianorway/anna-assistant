//! Learning Engine V1 (v0.0.427).
//!
//! Self-learning recipe system that:
//! - Learns from successful specialist responses
//! - Matches questions to existing recipes before calling LLM
//! - Tracks recipe usage and success rates
//! - Cites sources (man pages, Arch Wiki, help output)
//!
//! Key principles:
//! - Minimal hardcoding (only seed recipes for basic checks)
//! - Strong bias toward learning from probes and documentation
//! - Recipes are the first engine; LLM is the fallback
//! - Honest metrics about recipe hit rates

pub mod recipe;
pub mod evidence;
pub mod eligibility;
pub mod generator;
pub mod matcher;
pub mod executor;
pub mod storage;
pub mod stats;
pub mod seeds;

pub use recipe::*;
pub use evidence::*;
pub use eligibility::*;
pub use generator::*;
pub use matcher::*;
pub use executor::*;
pub use storage::*;
pub use stats::*;
pub use seeds::*;

/// Minimum confidence to learn from a ticket
pub const MIN_LEARN_CONFIDENCE: f32 = 0.8;

/// Minimum confidence for partial learning consideration
pub const MIN_PARTIAL_CONFIDENCE: f32 = 0.7;

/// Maximum evidence cache entries
pub const MAX_EVIDENCE_CACHE: usize = 500;

/// Evidence cache rolling window (days)
pub const EVIDENCE_CACHE_DAYS: u32 = 30;

/// Minimum recipe match score to use
pub const MIN_RECIPE_MATCH_SCORE: f32 = 0.70;

/// Score threshold for auto-execution without LLM
pub const AUTO_EXECUTE_SCORE: f32 = 0.85;

/// Maximum parameters in a learned recipe
pub const MAX_RECIPE_PARAMS: usize = 5;

/// Minimum uses before recipe is considered reliable
pub const MIN_RELIABLE_USES: u32 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(MIN_LEARN_CONFIDENCE > MIN_PARTIAL_CONFIDENCE);
        assert!(AUTO_EXECUTE_SCORE > MIN_RECIPE_MATCH_SCORE);
    }
}
