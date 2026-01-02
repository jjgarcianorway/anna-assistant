//! User-defined alarms and reminders (v0.0.456).
//!
//! Natural language alarm system:
//! - "Remind me every Monday at 9 about storage"
//! - "Alert me when disk is above 90%"
//! - "Notify me daily about failed services"
//!
//! v0.0.456: Initial implementation per VISION.md Phase 35.

mod parsing;
mod store;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use parsing::parse_alarm_request;
pub use store::AlarmStore;
pub use types::{AlarmCondition, AlarmSchedule, NotifyChannel, UserAlarm, Weekday};
