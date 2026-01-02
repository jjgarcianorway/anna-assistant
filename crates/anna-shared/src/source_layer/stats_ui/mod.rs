//! Stats and UI Components (Part 5) - v0.0.443.
//!
//! Clean stats driven by ticket state machine:
//! - Breakdown by outcome (answered, parse_error, etc.)
//! - No XP gamification by default (--fun flag)
//!
//! Clean dialogs:
//! - Single question, enumerated choices
//! - Always allow Cancel
//! - Never duplicate
//! - --plain and --json modes

mod dialog;
mod progress;
mod stats;

#[cfg(test)]
mod tests;

// Re-export public types
pub use dialog::{ConfirmDialog, DialogChoice, DialogQuestion, DialogResult};
pub use progress::ProgressIndicator;
pub use stats::{CleanStats, OutputMode};
