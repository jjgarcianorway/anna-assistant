//! Answer contract - enforces that answers contain only what was asked (v0.0.209).
//!
//! v0.0.74: Implements answer shaping to prevent over-sharing facts.
//! v0.0.209: Modularized into domain-focused submodules.
//!
//! # Design
//! - Translator output includes `requested_fields` and `verbosity`
//! - Final answers are validated against the contract
//! - Extra facts are trimmed unless teaching mode is enabled

mod contract;
mod tests;
mod trimming;
mod types;
mod validation;

// Re-export all types and functions
pub use contract::AnswerContract;
pub use trimming::trim_answer;
pub use types::{RequestedField, Verbosity};
pub use validation::{validate_answer, AnswerValidation};
