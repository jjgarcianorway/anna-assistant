//! Ticket outcome tracking for honest stats (v0.0.428).
//!
//! A ticket counts as "resolved" only if:
//! - status == success OR
//! - status == partial with a clearly useful answer
//!
//! A ticket counts as "failed" if:
//! - status == failure
//! - final answer says "I don't know" without meaningful facts
//! - parse errors with no useful fallback

mod outcome_types;
mod outcome_determination;
mod outcome_stats;

#[cfg(test)]
mod tests;

pub use outcome_types::*;
pub use outcome_determination::*;
pub use outcome_stats::*;
