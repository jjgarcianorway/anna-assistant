//! JSON parser for specialist responses (v0.0.428).
//!
//! Parses specialist JSON with multiple fallback strategies.
//! Never returns raw error messages to users.

mod core;
mod extraction;
mod lenient;
mod types;

// Re-export all public items
pub use core::{parse_specialist_response, parse_with_timeout};
pub use types::{timeout_outcome, ParseOutcome};

// Internal utilities are not re-exported:
// - extraction::extract_json (used internally)
// - lenient::try_lenient_parse (used internally)
// - types::truncate (used internally)
