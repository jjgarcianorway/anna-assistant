//! Specialist Protocol V4 (v0.0.428).
//!
//! Strict, parseable, resilient specialist communication:
//! - Single JSON schema that all specialists must follow
//! - No-bullshit policy: no invented facts, no vague how-tos
//! - Graceful timeout/error degradation to honest fallbacks
//! - Stats reflect real user-facing success, not internal metrics
//!
//! Key principles:
//! - Everything in summary/key_facts must trace to evidence
//! - Status semantics are strict: success/partial/failure
//! - Parse failures become honest "failure" outcomes
//! - No generic tutorials when user asked for current state

pub mod fallback;
pub mod fallback_extractors;
pub mod fallback_responses;
pub mod fallback_types;
pub mod guardrails;
pub mod outcome;
pub mod parser;
pub mod schema;
pub mod schema_actions;
pub mod schema_evidence;
pub mod schema_types;
pub mod validation_checks;
pub mod validation_core;
pub mod validation_types;

#[cfg(test)]
mod validation_tests;

pub use fallback::{
    debug_error_message, extract_facts_from_probes, generate_fallback, generate_failure_response,
    generate_partial_response, truncate, user_friendly_error_message, ExtractedFact,
    FallbackContext, FallbackReason,
};
pub use guardrails::*;
pub use outcome::*;
pub use parser::*;
pub use schema::*;
pub use validation_checks::*;
pub use validation_core::*;
pub use validation_types::*;

/// Minimum confidence for learning recipes from a response
pub const MIN_LEARN_CONFIDENCE: f32 = 0.8;

/// Minimum confidence for suggesting actions
pub const MIN_ACTION_CONFIDENCE: f32 = 0.6;

/// Maximum response latency before warning (ms)
pub const RESPONSE_LATENCY_WARNING_MS: u64 = 5000;

/// Maximum retries for specialist calls
pub const MAX_SPECIALIST_RETRIES: u32 = 1;

/// Maximum summary length (characters)
pub const MAX_SUMMARY_LENGTH: usize = 500;

/// Maximum key facts
pub const MAX_KEY_FACTS: usize = 10;

/// Maximum recommendations
pub const MAX_RECOMMENDATIONS: usize = 5;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_acceptance.rs"]
mod tests_acceptance;

#[cfg(test)]
#[path = "tests_classification.rs"]
mod tests_classification;

#[cfg(test)]
#[path = "tests_validation.rs"]
mod tests_validation;
