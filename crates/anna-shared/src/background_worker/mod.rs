//! Background Worker System (v0.0.430)
//!
//! Anna's background work system for:
//! - Long-running ticket analysis
//! - Idle-time learning and doc indexing
//! - User reminders and monitors
//! - Alert notifications
//!
//! Design principles:
//! - Only runs low-priority jobs when CPU is idle
//! - All jobs are durable (persisted to disk)
//! - No notifications without explicit user consent
//! - Rate-limited alerts (no spam)

pub mod executor;
pub mod idle_learning;
pub mod idle_learning_handlers;
pub mod idle_learning_types;
pub mod job;
pub mod job_result;
pub mod job_types;
pub mod monitors;
pub mod notification;
pub mod notifications;
pub mod scheduler;
pub mod storage;

pub use idle_learning::*;
pub use idle_learning_types::*;
pub use job::*;
pub use job_result::*;
pub use job_types::*;
pub use monitors::*;
pub use notification::*;
pub use notifications::*;
pub use scheduler::*;

/// Default idle CPU threshold for low-priority jobs (0.0-1.0)
pub const IDLE_CPU_THRESHOLD: f32 = 0.3;

/// Default check interval for scheduler (seconds)
pub const SCHEDULER_CHECK_INTERVAL_SECS: u64 = 60;

/// Long ticket threshold (seconds of active processing)
pub const LONG_TICKET_THRESHOLD_SECS: u64 = 30;

/// Alert cooldown period (hours) to prevent spam
pub const ALERT_COOLDOWN_HOURS: u64 = 24;

/// Maximum jobs per day for idle learning
pub const MAX_IDLE_JOBS_PER_DAY: usize = 10;

/// Job storage path
pub const JOBS_FILE: &str = "jobs.json";

/// Monitors storage path
pub const MONITORS_FILE: &str = "monitors.json";

/// Pending messages path (for user on next annactl open)
pub const PENDING_MESSAGES_FILE: &str = "pending_messages.json";

#[cfg(test)]
mod tests;
