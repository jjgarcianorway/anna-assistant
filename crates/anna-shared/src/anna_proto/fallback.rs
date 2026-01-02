//! Evidence-Only Fallback Mode (Part E) - v0.0.436.
//!
//! When a model call fails (timeout, parse failure, crash):
//! - Render gathered evidence in compact form
//! - State what couldn't be concluded without synthesis
//! - Propose next 1-2 probes deterministically
//! - Never claim confidence > 0.5 without synthesis

// Re-export public API from sibling modules
pub use super::fallback_builder::EvidenceFallback;
pub use super::fallback_types::{FallbackResponse, GatheredEvidence, MAX_FALLBACK_CONFIDENCE};
