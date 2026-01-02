//! Learning Without Hardcoding (Part E) - v0.0.441.
//!
//! Anna does NOT learn answers.
//! Anna learns: Which facts answer which intents.
//!
//! Learning record:
//! {
//!   "intent": "boot.slow_service",
//!   "required_facts": ["boot.blame"],
//!   "confidence": 0.92
//! }
//!
//! Next time:
//! - Anna skips LLM
//! - Runs known probes
//! - Assembles answer deterministically
//!
//! This avoids case-by-case hardcoding.

mod decision;
mod seeds;
mod store;
mod types;

// Re-export all public types
pub use decision::{decide_fast_path, FastPathDecision};
pub use store::IntentLearningStore;
pub use types::{IntentFactMapping, LearningStats};
