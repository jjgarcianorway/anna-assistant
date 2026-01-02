//! LLM-based semantic similarity for recipe matching (v0.0.293).
//!
//! Uses the translator model to determine if a new query is semantically
//! similar to queries with learned recipes. This enables Anna to reuse
//! learned recipes for paraphrased queries that token matching would miss.
//!
//! Example: "how much disk space" and "what's my storage usage" are semantically
//! similar but share no common tokens.
//!
//! v0.0.293: Added domain guard to prevent cross-domain false matches.
//! v0.0.294: Stricter domain guard - domain-specific queries only match same-domain recipes.
//! v0.0.295: Skip recipes where query is in negative_match_patterns (learned from feedback).

mod domain;
mod matching;
mod prompts;
mod types;

// Re-export public API
pub use matching::{check_classification_similarity, check_semantic_similarity};
pub use types::SimilarityResult;
