//! Formatting functions for fun statistics (v0.0.479).

use super::types::FunStats;

/// Format installation date from timestamp
pub fn format_install_date(timestamp: u64) -> String {
    if timestamp == 0 {
        return "Unknown".to_string();
    }

    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0);
    match dt {
        Some(d) => d.format("%Y-%m-%d").to_string(),
        None => "Unknown".to_string(),
    }
}

/// Format duration as human-readable time
pub fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{:.1}min", ms as f64 / 60_000.0)
    }
}

/// Format fun stats for display (full view)
pub fn format_fun_stats(stats: &FunStats) -> String {
    let mut output = String::new();

    output.push_str("Fun Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    // Installation & Usage
    output.push_str("📅 History\n");
    output.push_str("──────────────────────────────────────\n");
    output.push_str(&format!(
        "  Installed:         {}\n",
        format_install_date(stats.installation_date)
    ));
    output.push_str(&format!("  Days active:       {}\n", stats.days_active));
    output.push_str(&format!(
        "  Total requests:    {}\n",
        stats.total_requests
    ));
    output.push('\n');

    // Team stats
    output.push_str("👥 Teams\n");
    output.push_str("──────────────────────────────────────\n");
    if let Some(team) = &stats.most_consulted_team {
        output.push_str(&format!(
            "  Most consulted:    {} ({} times)\n",
            team, stats.most_consulted_count
        ));
    }
    if let Some(team) = &stats.lucky_team {
        output.push_str(&format!(
            "  Lucky team:        {} ({:.0}% success)\n",
            team,
            stats.lucky_team_rate * 100.0
        ));
    }
    output.push('\n');

    // Anna's independence
    output.push_str("🤖 Anna's Independence\n");
    output.push_str("──────────────────────────────────────\n");
    output.push_str(&format!(
        "  Solo answers:      {} ({:.1}%)\n",
        stats.anna_solo_count, stats.anna_solo_pct
    ));
    output.push_str(&format!(
        "  Recipes learned:   {}\n",
        stats.recipes_learned
    ));
    output.push('\n');

    // Response times
    output.push_str("⏱ Response Times\n");
    output.push_str("──────────────────────────────────────\n");
    output.push_str(&format!(
        "  Longest reply:     {}\n",
        format_duration(stats.longest_reply_ms)
    ));
    output.push_str(&format!(
        "  Shortest reply:    {}\n",
        format_duration(stats.shortest_reply_ms)
    ));
    output.push('\n');

    // Streaks
    output.push_str("🔥 Streaks\n");
    output.push_str("──────────────────────────────────────\n");
    output.push_str(&format!(
        "  Current streak:    {} days\n",
        stats.current_streak
    ));
    output.push_str(&format!(
        "  Best streak:       {} days\n",
        stats.best_streak
    ));

    // Add a fun fact at the end
    if let Some(fact) = super::fun_facts::generate_fun_fact(stats) {
        output.push('\n');
        output.push_str("💡 Fun Fact\n");
        output.push_str("──────────────────────────────────────\n");
        output.push_str(&format!("  {}\n", fact));
    }

    output
}

/// Format fun stats for compact display (greeting integration)
pub fn format_fun_stats_compact(stats: &FunStats) -> String {
    let mut parts = Vec::new();

    if stats.days_active > 0 {
        parts.push(format!("{} days together", stats.days_active));
    }

    if stats.total_requests > 0 {
        parts.push(format!("{} requests", stats.total_requests));
    }

    if stats.current_streak > 0 {
        parts.push(format!("{} day streak", stats.current_streak));
    }

    if parts.is_empty() {
        "No statistics yet".to_string()
    } else {
        parts.join(" | ")
    }
}
