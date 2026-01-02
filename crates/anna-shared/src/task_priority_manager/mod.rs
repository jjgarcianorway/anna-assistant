//! Task Priority Manager - Phase 99
//!
//! Manages and prioritizes Anna's task queue.
//! Ensures critical tasks are handled first.

mod priority;
mod state;
mod task;
mod manager;
mod formatting;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use priority::TaskPriority;
pub use state::TaskState;
pub use task::ManagedTask;
pub use manager::TaskPriorityManager;
pub use formatting::{
    format_task_manager,
    format_task_manager_compact,
    format_task_manager_oneline,
};
pub use utils::{
    is_task_manager_query,
    task_manager_fun_fact,
};
