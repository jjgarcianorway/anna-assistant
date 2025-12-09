//! Pending clarification persistence for REPL continuity (v0.0.227).
//!
//! When Anna needs clarification, the pending question is saved so the user
//! can resume in a new REPL session without losing context.
//!
//! v0.0.36: Initial implementation.
//! v0.0.227: Modularized into domain-focused submodules.

mod persistence;
#[cfg(test)]
mod tests;
mod types;
mod verification;

// Re-export for backwards compatibility
pub use persistence::{clear_pending, has_pending, load_pending, pending_path, save_pending};
pub use types::{ParseResult, PendingClarification, VerifyResult};
pub use verification::verify_answer;
