//! Intake module for query analysis and clarification planning (v0.0.180).
//!
//! Determines what clarifications are needed and how to verify them.
//! Supports the "research first, ask second, verify always" philosophy.
//!
//! v0.0.180: Modularized into domain-focused submodules.

mod analyze;
mod question;
mod result;
mod slot;
mod tests;
mod verify_plan;

// Re-export main types
pub use analyze::{analyze_intake, check_slot_satisfied};
pub use question::ClarificationQuestion;
pub use result::{IntakeResult, VerificationResult};
pub use slot::{generate_clarification, ClarificationSlot};
pub use verify_plan::VerifyPlan;
