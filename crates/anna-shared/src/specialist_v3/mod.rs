//! Specialist V3 - Strict JSON Contract (v0.0.425).
//!
//! This module enforces a strict JSON schema for ALL specialist responses.
//! Key principles:
//! - Single canonical schema - no free-form text
//! - Robust parsing with retry logic
//! - User-friendly error messages (no "Failed to parse" exposed)
//! - Everything grounded in probes and knowledge citations

pub mod parser;
pub mod prompt;
pub mod schema;
pub mod synthesize;

pub use parser::*;
pub use prompt::*;
pub use schema::*;
pub use synthesize::*;

/// Maximum retry attempts for parsing
pub const MAX_PARSE_RETRIES: usize = 1;

/// Default confidence for synthesized responses
pub const DEFAULT_CONFIDENCE: f32 = 0.5;

/// Minimum acceptable confidence
pub const MIN_USEFUL_CONFIDENCE: f32 = 0.3;

/// High confidence threshold
pub const HIGH_CONFIDENCE: f32 = 0.85;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(MIN_USEFUL_CONFIDENCE < DEFAULT_CONFIDENCE);
        assert!(DEFAULT_CONFIDENCE < HIGH_CONFIDENCE);
    }
}
