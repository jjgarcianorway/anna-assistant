//! Alarm Scheduler - Phase 90
//!
//! Schedules recurring notifications via natural language.
//! VISION.md: "User can set specific alarms via natural language"
//! Example: "Please notify me every Monday at 9 about storage progression"

mod types;
mod scheduler;
mod utils;
mod query;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use types::{AlarmFrequency, AlarmRecord, AlarmScheduler, AlarmStatus, DayOfWeek};
pub use utils::{
    format_alarm_scheduler, format_alarm_scheduler_compact, format_alarm_scheduler_oneline,
    parse_day_of_week,
};
pub use query::{alarm_fun_fact, is_alarm_query};
