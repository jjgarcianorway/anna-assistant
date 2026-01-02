//! Task Manager Utility Functions

use super::manager::TaskPriorityManager;

/// Check if query is about tasks
pub fn is_task_manager_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "task queue",
        "pending tasks",
        "task priority",
        "what tasks",
        "next task",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about tasks
pub fn task_manager_fun_fact(manager: &TaskPriorityManager) -> String {
    if manager.tasks.is_empty() {
        return "No tasks in the queue!".to_string();
    }

    let facts = [
        format!("Anna has {} tasks in the queue.", manager.total_count()),
        format!("{} tasks are pending.", manager.pending_count()),
        format!("{} tasks have been completed.", manager.total_completed),
    ];

    facts[manager.total_count() % facts.len()].clone()
}
