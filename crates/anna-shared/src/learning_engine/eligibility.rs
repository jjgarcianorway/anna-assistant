//! Learning eligibility checker (v0.0.427).
//!
//! Determines when to learn a new recipe from a ticket.
//! Only learns when:
//! - Specialist response is success/partial with high confidence
//! - Answer is grounded in probes/documentation
//! - Pattern is generalizable (not too user-specific)

// Re-export public types and functions
pub use super::eligibility_checks::check_eligibility;
pub use super::eligibility_extraction::{extract_intent, extract_params};
pub use super::eligibility_types::{EligibilityResult, SkipReason};
