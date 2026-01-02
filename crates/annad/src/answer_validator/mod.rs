//! Answer validation with LLM-based self-healing (v0.0.376).
//!
//! Every answer goes through validation before reaching the user:
//! 1. Extract claims from answer
//! 2. Verify claims against evidence
//! 3. If validation fails, regenerate with explicit constraints
//! 4. Retry until score >= threshold or max attempts
//!
//! v0.0.376: Domain-specific validation thresholds
//! - Security: 90 (high stakes, must be accurate)
//! - System: 80 (standard reliability)
//! - Network: 75 (often partial visibility)
//! - Storage: 80 (standard reliability)
//! - Packages: 75 (version info can vary)
//!
//! This implements the principle: "Any answer gathered must always be run
//! against the specialists to know if it's the right answer or not."

mod healing;
mod orchestrator;
mod thresholds;
mod types;
mod validation;

#[cfg(test)]
mod tests;

// Re-export public API
pub use orchestrator::{validate_and_heal, validate_and_heal_with_domain};
pub use types::{ValidationIssue, ValidationResult};
