// v0.0.529: Escalation Tracker Module (Phase 105)
// Tracks ticket escalations between junior and senior specialists per VISION.md

mod formatting;
mod tracker;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use formatting::{
    format_escalation, format_escalation_compact, format_escalation_oneline,
    format_tracker_summary,
};
pub use tracker::EscalationTracker;
pub use types::{EscalationOutcome, EscalationReason, EscalationRecord};
pub use utils::{escalation_fun_fact, is_escalation_query};
