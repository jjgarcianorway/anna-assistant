//! Monitor and reminder system (v0.0.430).
//!
//! User-defined monitors check conditions and trigger alerts.
//! Reminders fire at scheduled times.

mod checks;
mod reminder;
mod storage;
mod types;

// Re-export all public types
pub use reminder::{Reminder, ReminderSchedule};
pub use storage::MonitorStorage;
pub use types::{Monitor, MonitorCheck, MonitorCheckResult, MonitorStatus, ThresholdCondition};
