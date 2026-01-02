//! Follow-up hints for answer enrichment (v0.0.384).
//!
//! Generates contextual suggestions for what the user might want to know next.
//! Based on domain, query patterns, and successful past interactions.

mod types;
mod domain_hints;
mod generator;
mod formatter;

// Re-export public API
pub use types::FollowupHint;
pub use generator::generate_followup_hints;
pub use formatter::format_hints;
