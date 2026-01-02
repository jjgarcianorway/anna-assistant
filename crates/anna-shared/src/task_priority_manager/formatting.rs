//! Task Manager Formatting Functions

use super::manager::TaskPriorityManager;

/// Format task manager for display
pub fn format_task_manager(manager: &TaskPriorityManager) -> String {
    let mut lines = vec!["=== Task Priority Manager ===".to_string()];
    lines.push(String::new());

    if manager.tasks.is_empty() {
        lines.push("No tasks in queue.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total tasks: {}", manager.total_count()));
    lines.push(format!("Pending: {}", manager.pending_count()));
    lines.push(format!("Completed: {}", manager.total_completed));

    // By priority
    if !manager.by_priority.is_empty() {
        lines.push(String::new());
        lines.push("By priority:".to_string());
        for (p, count) in &manager.by_priority {
            lines.push(format!("  {}: {}", p, count));
        }
    }

    // Pending tasks
    let pending = manager.pending();
    if !pending.is_empty() {
        lines.push(String::new());
        lines.push("Pending tasks:".to_string());
        for task in pending.iter().take(10) {
            lines.push(format!(
                "  [{}] {} - {}",
                task.priority.symbol(),
                task.id,
                task.description
            ));
        }
    }

    lines.join("\n")
}

/// Format task manager compact
pub fn format_task_manager_compact(manager: &TaskPriorityManager) -> String {
    format!(
        "Tasks: {} total | {} pending | {} completed",
        manager.total_count(),
        manager.pending_count(),
        manager.total_completed
    )
}

/// Format task manager one-line
pub fn format_task_manager_oneline(manager: &TaskPriorityManager) -> String {
    format!("{} tasks ({} pending)", manager.total_count(), manager.pending_count())
}
