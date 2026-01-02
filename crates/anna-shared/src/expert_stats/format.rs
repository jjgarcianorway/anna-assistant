//! Formatting functions for expert statistics.

use super::tracker::ExpertStatsTracker;

/// Format expert stats for display
pub fn format_expert_stats(tracker: &ExpertStatsTracker) -> String {
    let mut output = String::new();

    output.push_str("Expert Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    if tracker.total_tickets() == 0 {
        output.push_str("No tickets recorded yet.\n");
        return output;
    }

    output.push_str(&format!(
        "Total Tickets: {}\n",
        tracker.total_tickets()
    ));
    output.push_str(&format!(
        "  Anna Solo: {} ({:.1}%)\n",
        tracker.anna_solo,
        tracker.anna_share()
    ));
    output.push_str(&format!(
        "  Junior:    {} ({:.1}%)\n",
        tracker.junior_total,
        tracker.junior_share()
    ));
    output.push_str(&format!(
        "  Senior:    {} ({:.1}%)\n\n",
        tracker.senior_total,
        tracker.senior_share()
    ));

    let top = tracker.top_performers(5);
    if !top.is_empty() {
        output.push_str("Top Performers:\n");
        for (id, stats) in top {
            let name = tracker
                .experts
                .get(id)
                .map(|e| format!("{} ({})", e.name, e.level.short_name()))
                .unwrap_or_else(|| id.to_string());

            output.push_str(&format!(
                "  {} - {} closed, {:.0}% confidence\n",
                name,
                stats.tickets_closed,
                stats.avg_confidence * 100.0
            ));
        }
    }

    output
}

/// Format compact expert stats
pub fn format_expert_stats_compact(tracker: &ExpertStatsTracker) -> String {
    if tracker.total_tickets() == 0 {
        return "No tickets yet".to_string();
    }

    format!(
        "{} tickets: Anna {:.0}%, Jr {:.0}%, Sr {:.0}%",
        tracker.total_tickets(),
        tracker.anna_share(),
        tracker.junior_share(),
        tracker.senior_share()
    )
}
