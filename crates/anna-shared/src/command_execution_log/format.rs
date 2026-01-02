//! Formatting functions for execution log display

use super::log::ExecutionLog;

/// Format execution log for display
pub fn format_execution_log(log: &ExecutionLog) -> String {
    let mut lines = vec!["=== Command Execution Log ===".to_string()];
    lines.push(String::new());

    if log.records.is_empty() {
        lines.push("No commands executed yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total executions: {}", log.total_count()));
    lines.push(format!("Success rate: {:.1}%", log.success_rate()));
    lines.push(format!("Avg duration: {}ms", log.average_duration_ms()));
    lines.push(format!("Elevated (sudo): {}", log.elevated_count));

    // Most used
    let most_used = log.most_used(5);
    if !most_used.is_empty() {
        lines.push(String::new());
        lines.push("Most used commands:".to_string());
        for (cmd, count) in most_used {
            lines.push(format!("  {} ({} times)", cmd, count));
        }
    }

    // Recent executions
    let recent = log.recent(5);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent executions:".to_string());
        for exec in recent {
            let status = exec.status.symbol();
            let elevated = if exec.elevated { "[sudo]" } else { "" };
            lines.push(format!(
                "  [{}] {} {} ({}ms)",
                status, exec.command, elevated, exec.duration_ms
            ));
        }
    }

    // Failed commands
    let failed = log.most_failed(3);
    if !failed.is_empty() {
        lines.push(String::new());
        lines.push("Commands with failures:".to_string());
        for (cmd, count) in failed {
            lines.push(format!("  {} ({} failures)", cmd, count));
        }
    }

    lines.join("\n")
}

/// Format execution log compact
pub fn format_execution_log_compact(log: &ExecutionLog) -> String {
    format!(
        "Commands: {} ({:.1}% success) | Avg: {}ms | Sudo: {}",
        log.total_count(),
        log.success_rate(),
        log.average_duration_ms(),
        log.elevated_count
    )
}

/// Format execution log one-line
pub fn format_execution_log_oneline(log: &ExecutionLog) -> String {
    format!(
        "{} commands ({:.0}% ok)",
        log.total_count(),
        log.success_rate()
    )
}
