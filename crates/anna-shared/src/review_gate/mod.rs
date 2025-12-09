//! Review gate: deterministic-first hybrid review logic (v0.0.228).
//!
//! Pure function that decides Accept/Revise/Escalate/Clarify based on
//! existing deterministic signals (reliability, grounding, guard).
//!
//! LLM review is only invoked when the gate returns "unclear".
//!
//! v0.0.228: Modularized into domain-focused submodules.

mod logic;
#[cfg(test)]
mod tests;
mod types;

// Re-export for backwards compatibility
pub use logic::{deterministic_review_gate, deterministic_review_gate_with_thresholds};
pub use types::{GateOutcome, GateThresholds, ReviewContext};
