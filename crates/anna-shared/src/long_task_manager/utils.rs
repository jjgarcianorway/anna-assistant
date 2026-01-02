// v0.0.534: Long Task Manager - Utils (Phase 110)
// Query detection and utility functions

/// Check if query is long-task related
pub fn is_long_task_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("long task")
        || lower.contains("background")
        || lower.contains("research")
        || lower.contains("idle")
        || lower.contains("email when done")
        || lower.contains("takes a while")
}

/// Fun fact about long tasks
pub fn long_task_fun_fact() -> &'static str {
    "Anna can research complex questions when your machine is idle and email you with a complete chain of thought - like having a researcher on call 24/7!"
}
