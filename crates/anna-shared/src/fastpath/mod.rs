//! Fast path engine for answering health/status queries without LLM (v0.0.185).
//!
//! Handles common "how is my computer" queries deterministically using:
//! - Cached snapshot (if fresh)
//! - Minimal probes (free + df + systemctl --failed) when snapshot is stale
//! - Known facts and recipes index
//!
//! Never calls specialist LLM for these query classes.
//!
//! v0.0.40: Uses RelevantHealthSummary for minimal, actionable responses.
//! v0.0.185: Modularized into domain-focused submodules.

mod answers;
mod classify;
mod engine;
mod tests;
mod types;

// Re-export main types and functions
pub use classify::classify_fast_path;
pub use engine::{find_matching_recipes, try_fast_path};
pub use types::{
    FastPathAnswer, FastPathClass, FastPathInput, FastPathPolicy, DEFAULT_SNAPSHOT_MAX_AGE,
};
