// v0.0.534: Long Task Manager Module (Phase 110)
// Manages long-running tasks with email notification per VISION.md

mod types;
mod record;
mod manager;
mod formatting;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API to preserve existing interface
pub use types::{LongTaskStatus, LongTaskType};
pub use record::LongTaskRecord;
pub use manager::LongTaskManager;
pub use formatting::{
    format_long_task,
    format_long_task_compact,
    format_long_task_oneline,
    format_manager_summary,
};
pub use utils::{is_long_task_query, long_task_fun_fact};
