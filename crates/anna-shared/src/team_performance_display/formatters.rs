//! Formatting functions for team performance display

use super::team_performance::TeamPerformance;

/// Format duration in milliseconds as human-readable
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else if ms < 3600000 {
        format!("{:.1}m", ms as f64 / 60000.0)
    } else {
        format!("{:.1}h", ms as f64 / 3600000.0)
    }
}

/// Format team performance as full display
pub fn format_team_performance(perf: &TeamPerformance) -> String {
    let mut lines = Vec::new();

    lines.push("=== Team Performance ===".to_string());
    lines.push(String::new());

    // Overall summary
    lines.push(format!("Total Tickets: {}", perf.total_tickets));
    lines.push(format!("Total Resolved: {}", perf.total_resolved));
    lines.push(format!("Overall Success Rate: {:.1}%", perf.overall_success_rate()));
    lines.push(String::new());

    // Highlights
    if let Some((name, metrics)) = perf.most_active() {
        lines.push(format!(
            "Most Active: {} ({} tickets)",
            name, metrics.tickets_handled
        ));
    }

    if let Some((name, metrics)) = perf.best_performing() {
        lines.push(format!(
            "Best Performing: {} ({:.1}% success)",
            name,
            metrics.success_rate()
        ));
    }

    if let Some((name, metrics)) = perf.fastest() {
        lines.push(format!(
            "Fastest: {} (avg {})",
            name,
            format_duration_ms(metrics.avg_resolution_ms())
        ));
    }

    lines.push(String::new());

    // Team breakdown
    lines.push("--- Team Breakdown ---".to_string());

    let teams = perf.by_activity();
    if teams.is_empty() {
        lines.push("  No team data yet.".to_string());
    } else {
        for (name, metrics) in teams {
            lines.push(format!("\n  {} Team:", name));
            lines.push(format!("    Tickets: {} handled, {} resolved", metrics.tickets_handled, metrics.tickets_resolved));
            lines.push(format!("    Success Rate: {:.1}%", metrics.success_rate()));
            lines.push(format!("    Escalation Rate: {:.1}%", metrics.escalation_rate()));

            if metrics.avg_resolution_ms() > 0 {
                lines.push(format!("    Avg Resolution: {}", format_duration_ms(metrics.avg_resolution_ms())));
            }

            if metrics.fastest_resolution_ms.is_some() || metrics.slowest_resolution_ms.is_some() {
                let fastest = metrics.fastest_resolution_ms.map(format_duration_ms).unwrap_or_else(|| "-".to_string());
                let slowest = metrics.slowest_resolution_ms.map(format_duration_ms).unwrap_or_else(|| "-".to_string());
                lines.push(format!("    Range: {} - {}", fastest, slowest));
            }

            lines.push(format!(
                "    Junior/Senior: {}/{}",
                metrics.junior_tickets, metrics.senior_tickets
            ));
        }
    }

    lines.join("\n")
}

/// Format team performance compact (for greetings)
pub fn format_team_performance_compact(perf: &TeamPerformance) -> String {
    let teams = perf.by_activity();
    if teams.is_empty() {
        return "No team activity yet.".to_string();
    }

    let top_teams: Vec<String> = teams
        .iter()
        .take(3)
        .map(|(name, m)| format!("{}: {}t", name, m.tickets_handled))
        .collect();

    format!(
        "Top teams: {} | {:.0}% overall success",
        top_teams.join(", "),
        perf.overall_success_rate()
    )
}

/// Format team performance one-line
pub fn format_team_performance_oneline(perf: &TeamPerformance) -> String {
    format!(
        "Teams: {} active, {} tickets, {:.0}% success",
        perf.teams.len(),
        perf.total_tickets,
        perf.overall_success_rate()
    )
}

/// Generate a fun fact about team performance
pub fn team_performance_fun_fact(perf: &TeamPerformance) -> Option<String> {
    if perf.teams.is_empty() {
        return None;
    }

    let facts = vec![
        perf.total_tickets >= 100,
        perf.overall_success_rate() >= 95.0,
        perf.teams.len() >= 5,
    ];

    let messages = vec![
        format!(
            "Century of teamwork! {} tickets handled across all teams.",
            perf.total_tickets
        ),
        format!(
            "Elite performance! {:.1}% overall success rate.",
            perf.overall_success_rate()
        ),
        format!(
            "Full house! {} different teams have been called into action.",
            perf.teams.len()
        ),
    ];

    for (i, fact) in facts.iter().enumerate() {
        if *fact {
            return Some(messages[i].clone());
        }
    }

    // Default fact
    if let Some((name, _)) = perf.most_active() {
        return Some(format!(
            "The {} team is the most consulted, handling the most tickets.",
            name
        ));
    }

    None
}

/// Check if query is asking about team performance
pub fn is_team_performance_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "team performance",
        "team stats",
        "how are the teams",
        "team metrics",
        "best team",
        "fastest team",
        "most active team",
        "team breakdown",
        "department stats",
        "team activity",
        "which team",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(500), "500ms");
        assert_eq!(format_duration_ms(1500), "1.5s");
        assert_eq!(format_duration_ms(90000), "1.5m");
        assert_eq!(format_duration_ms(5400000), "1.5h");
    }

    #[test]
    fn test_format_team_performance() {
        let mut perf = TeamPerformance::new();
        perf.record_ticket("Desktop", 5000, true, false);

        let output = format_team_performance(&perf);
        assert!(output.contains("Team Performance"));
        assert!(output.contains("Desktop"));
    }

    #[test]
    fn test_format_team_performance_compact() {
        let mut perf = TeamPerformance::new();
        perf.record_ticket("Desktop", 1000, true, false);

        let output = format_team_performance_compact(&perf);
        assert!(output.contains("Desktop"));
    }

    #[test]
    fn test_is_team_performance_query() {
        assert!(is_team_performance_query("show team performance"));
        assert!(is_team_performance_query("what are the team stats?"));
        assert!(is_team_performance_query("which team is best?"));
        assert!(!is_team_performance_query("how do I install vim?"));
    }

    #[test]
    fn test_team_performance_fun_fact() {
        let mut perf = TeamPerformance::new();

        // Empty - no fact
        assert!(team_performance_fun_fact(&perf).is_none());

        // Add some data
        perf.record_ticket("Desktop", 1000, true, false);
        let fact = team_performance_fun_fact(&perf);
        assert!(fact.is_some());
    }
}
