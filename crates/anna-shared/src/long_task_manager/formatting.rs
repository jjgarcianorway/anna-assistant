// v0.0.534: Long Task Manager - Formatting (Phase 110)
// Display formatting functions for tasks and manager

use crate::long_task_manager::record::LongTaskRecord;
use crate::long_task_manager::manager::LongTaskManager;

/// Format task for display
pub fn format_long_task(task: &LongTaskRecord) -> String {
    let mut output = format!(
        "{} [{}]\n  Type: {} | Status: {} | Progress: {}%\n  Description: {}",
        task.id, task.created_at, task.task_type, task.status, task.progress_pct, task.description
    );

    if let Some(est) = task.estimated_minutes {
        output.push_str(&format!("\n  Estimated: {} minutes", est));
    }

    if !task.chain_of_thought.is_empty() {
        output.push_str("\n  Thoughts:");
        for thought in &task.chain_of_thought {
            output.push_str(&format!("\n    - {}", thought));
        }
    }

    if let Some(result) = &task.result {
        output.push_str(&format!("\n  Result: {}", result));
    }

    if let Some(error) = &task.error {
        output.push_str(&format!("\n  Error: {}", error));
    }

    output
}

/// Format task compact
pub fn format_long_task_compact(task: &LongTaskRecord) -> String {
    format!(
        "{}: {} [{}] {}%",
        task.id, task.task_type, task.status, task.progress_pct
    )
}

/// Format task oneline
pub fn format_long_task_oneline(task: &LongTaskRecord) -> String {
    format!("{} [{}]", task.id, task.status)
}

/// Format manager summary
pub fn format_manager_summary(manager: &LongTaskManager) -> String {
    let mut output = String::new();
    output.push_str("=== Long Task Manager ===\n\n");

    output.push_str(&format!("Total Tasks: {}\n", manager.total()));
    output.push_str(&format!("Active: {}\n", manager.active().len()));
    output.push_str(&format!(
        "Waiting for Idle: {}\n\n",
        manager.waiting_for_idle().len()
    ));

    output.push_str("--- By Status ---\n");
    for (status, count) in manager.status_stats() {
        output.push_str(&format!("  {}: {}\n", status, count));
    }

    let active = manager.active();
    if !active.is_empty() {
        output.push_str("\n--- Active Tasks ---\n");
        for task in active {
            output.push_str(&format!("  {}\n", format_long_task_compact(task)));
        }
    }

    output
}
