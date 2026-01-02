//! Recipe matching engine (v0.0.423).
//!
//! Matches incoming queries against available recipes using:
//! - Domain matching
//! - Intent matching
//! - Keyword similarity (Jaccard)
//! - Entity matching
//! - Precondition evaluation

mod matcher_core;
mod matcher_helpers;
mod matcher_types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use matcher_core::RecipeMatcher;
pub use matcher_types::{MatchBreakdown, MatchQuery, MatchResult};
