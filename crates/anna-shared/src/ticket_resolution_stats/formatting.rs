//! Formatting utilities for ticket resolution statistics

use super::stats::TicketResolutionStats;

/// Format resolution stats for display
pub fn format_resolution_stats(stats: &TicketResolutionStats) -> String {
    let mut lines = vec!["=== Ticket Resolution Stats ===".to_string()];
    lines.push(String::new());

    if stats.records.is_empty() {
        lines.push("No resolutions yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total resolutions: {}", stats.total_count()));
    lines.push(format!("Anna: {} ({:.1}%)", stats.anna_count, stats.anna_rate()));
    lines.push(format!("Specialists: {}", stats.specialist_count));
    lines.push(format!("Recipes learned: {}", stats.recipes_learned));

    // Improvement
    if stats.anna_improving() {
        lines.push("Anna is improving over time!".to_string());
    }

    // Times
    lines.push(String::new());
    lines.push(format!("Avg resolution: {:.1} sec", stats.avg_resolution_time()));
    if let Some(fastest) = stats.fastest_resolution() {
        lines.push(format!("Fastest: {} sec", fastest));
    }
    if let Some(slowest) = stats.slowest_resolution() {
        lines.push(format!("Slowest: {} sec", slowest));
    }

    // By resolver
    if !stats.by_resolver.is_empty() {
        lines.push(String::new());
        lines.push("By resolver:".to_string());
        for (resolver, count) in &stats.by_resolver {
            lines.push(format!("  {}: {}", resolver, count));
        }
    }

    lines.join("\n")
}

/// Format resolution stats compact
pub fn format_resolution_stats_compact(stats: &TicketResolutionStats) -> String {
    format!(
        "Resolutions: {} total | Anna: {:.0}% | {} recipes learned",
        stats.total_count(),
        stats.anna_rate(),
        stats.recipes_learned
    )
}

/// Format resolution stats one-line
pub fn format_resolution_stats_oneline(stats: &TicketResolutionStats) -> String {
    format!(
        "{} resolved (Anna: {})",
        stats.total_count(),
        stats.anna_count
    )
}
