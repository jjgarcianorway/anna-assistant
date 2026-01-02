//! JSON Validator (Part B) - v0.0.440.
//!
//! Before Anna accepts a specialist response:
//! - Parse JSON
//! - Validate schema
//! - Validate case_id matches
//!
//! If validation fails:
//! - Mark specialist_response_invalid=true
//! - Trigger retry with repair prompt

mod validator_batch;
mod validator_core;
mod validator_types;
mod validator_utils;

// Re-export all public types
pub use validator_batch::BatchValidator;
pub use validator_core::SrcValidator;
pub use validator_types::{
    ValidationError, ValidationResult, MAX_RESPONSE_CHARS, MAX_RESPONSE_TOKENS,
};
