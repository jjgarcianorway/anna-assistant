//! Verification steps for pre-action and post-action checks (v0.0.198).
//!
//! Ensures Anna verifies tool existence before action, and confirms
//! outcomes after applying changes.
//!
//! v0.0.198: Modularized into domain-focused submodules.

mod batch;
mod runners;
mod tests;
mod types;

// Re-export all types and functions
pub use batch::{PostActionVerify, PreActionVerify};
pub use runners::{expand_path, run_verification};
pub use types::{ServiceExpectedState, VerificationStep, VerifyExpectation, VerifyResult};
