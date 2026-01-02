// v0.0.610: Settings Task Scheduler (Phase 186)
// Advanced task scheduling for settings operations

mod types;
mod task_definition;
mod task_instance;
mod scheduler;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{TaskFrequency, TaskType, TaskState};
pub use task_definition::TaskDefinition;
pub use task_instance::TaskInstance;
pub use scheduler::SettingsTaskScheduler;
pub use utils::{format_task_scheduler, is_task_scheduler_query, task_scheduler_fun_fact};
