//! GUARD: Evidence-aware contradiction and invention detection (v0.0.194).
//!
//! Reuses the same `Claim` extraction as ANCHOR but outputs contradiction/invention
//! signals instead of grounding ratios.
//!
//! # Invention Detection Rules
//!
//! - **Contradiction**: always flags `invention_detected = true`
//! - **Unverifiable + evidence_required**: flags `invention_detected = true`
//! - **Unverifiable + !evidence_required**: does NOT flag invention (but is counted)
//! - **Verified**: never flags invention
//!
//! v0.0.194: Modularized into domain-focused submodules.

mod tests;
mod types;
mod verify;

// Re-export all types and functions
pub use types::{GuardItem, GuardReport, VerifyResult};
pub use verify::run_guard;
