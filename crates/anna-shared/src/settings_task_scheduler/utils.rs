// v0.0.610: Settings Task Scheduler - Utilities (Phase 186)
// Utility functions for task scheduler

use super::scheduler::SettingsTaskScheduler;

/// Format scheduler
pub fn format_task_scheduler(scheduler: &SettingsTaskScheduler) -> String {
    let mut output = String::new();
    output.push_str("Settings Task Scheduler:\n");
    output.push_str(&format!("  Definitions: {}\n", scheduler.definition_count()));
    output.push_str(&format!("  Instances: {}\n", scheduler.instance_count()));
    output.push_str(&format!("  Pending: {}\n", scheduler.pending().len()));
    output.push_str(&format!("  Running: {}\n", scheduler.running().len()));
    output
}

/// Check if query is about task scheduler
pub fn is_task_scheduler_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("task schedule")
        || lower.contains("scheduled task")
        || lower.contains("task queue")
        || lower.contains("background task")
}

/// Fun fact about task scheduler
pub fn task_scheduler_fun_fact() -> &'static str {
    "Anna can schedule background tasks to maintain your settings automatically!"
}
