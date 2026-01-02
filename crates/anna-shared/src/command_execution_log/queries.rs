//! Query detection and fun fact generation for execution logs

use super::log::ExecutionLog;

/// Check if query is about execution log
pub fn is_execution_log_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "execution log",
        "command log",
        "commands executed",
        "commands run",
        "executed commands",
        "command history",
        "what commands",
        "ran commands",
        "execution history",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about executions
pub fn execution_fun_fact(log: &ExecutionLog) -> String {
    if log.records.is_empty() {
        return "No commands executed yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has executed {} commands with a {:.1}% success rate.",
            log.total_count(),
            log.success_rate()
        ),
        format!(
            "{} commands were run with elevated privileges.",
            log.elevated_count
        ),
        {
            if let Some((cmd, count)) = log.most_used(1).first() {
                format!("The most frequently used command is '{}' ({} times).", cmd, count)
            } else {
                "No command patterns detected yet.".to_string()
            }
        },
        format!(
            "Average command execution time: {}ms.",
            log.average_duration_ms()
        ),
    ];

    facts[log.total_count() % facts.len()].clone()
}
