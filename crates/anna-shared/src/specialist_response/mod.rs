//! Unified specialist response schema (v0.0.409).
//!
//! This is the SINGLE source of truth for specialist JSON responses.
//! All specialists must produce exactly this structure.
//!
//! Key principles:
//! - can_answer is REQUIRED (boolean)
//! - All fields have sensible defaults via serde
//! - Parse failures are classified explicitly
//! - Validation is strict and separate from parsing

mod format;
mod parse;
mod types;

// Re-export all public types and functions to preserve the API
pub use format::format_parse_failure;
pub use parse::{extract_json, parse_specialist_output, timeout_outcome};
pub use types::{
    ActionCommand, ParseOutcome, RecommendedAction, UnifiedSpecialistResponse,
};
