//! Utility functions for ticket resolution statistics

use super::stats::TicketResolutionStats;

/// Check if query is about resolution stats
pub fn is_resolution_stats_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "resolution stats",
        "tickets resolved",
        "who resolved",
        "anna vs specialist",
        "ticket stats",
        "resolution rate",
        "tickets closed",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about resolution stats
pub fn resolution_fun_fact(stats: &TicketResolutionStats) -> String {
    if stats.records.is_empty() {
        return "No ticket resolutions yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has resolved {} tickets on her own!",
            stats.anna_count
        ),
        format!(
            "Anna's resolution rate is {:.1}%.",
            stats.anna_rate()
        ),
        format!(
            "{} recipes were learned from resolutions.",
            stats.recipes_learned
        ),
        format!(
            "Average resolution time is {:.1} seconds.",
            stats.avg_resolution_time()
        ),
        {
            if stats.anna_improving() {
                "Anna is getting better over time!".to_string()
            } else {
                format!("Specialists resolved {} tickets.", stats.specialist_count)
            }
        },
    ];

    facts[stats.total_count() % facts.len()].clone()
}
