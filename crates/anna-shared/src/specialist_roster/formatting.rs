//! Roster formatting and display functions

use super::management::SpecialistRoster;

/// Format specialist roster for display
pub fn format_specialist_roster(roster: &SpecialistRoster) -> String {
    let mut lines = vec!["=== Specialist Roster ===".to_string()];
    lines.push(String::new());

    if roster.specialists.is_empty() {
        lines.push("No specialists registered.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total specialists: {}", roster.total_count()));
    lines.push(format!("Available: {}", roster.available_count()));
    lines.push(format!("Tickets resolved: {}", roster.total_tickets));

    // By level
    if !roster.by_level.is_empty() {
        lines.push(String::new());
        lines.push("By level:".to_string());
        for (level, count) in &roster.by_level {
            lines.push(format!("  {}: {}", level, count));
        }
    }

    // Top performer
    if let Some(top) = roster.top_performer() {
        lines.push(String::new());
        lines.push(format!(
            "Top performer: {} ({} tickets)",
            top.name, top.tickets_resolved
        ));
    }

    // List available
    let available = roster.available();
    if !available.is_empty() {
        lines.push(String::new());
        lines.push("Available now:".to_string());
        for s in available.iter().take(5) {
            lines.push(format!(
                "  {} - {} {} ({})",
                s.name,
                s.level.name(),
                s.department.name(),
                s.tickets_resolved
            ));
        }
    }

    lines.join("\n")
}

/// Format roster compact
pub fn format_roster_compact(roster: &SpecialistRoster) -> String {
    format!(
        "Team: {} specialists | {} available | {} tickets",
        roster.total_count(),
        roster.available_count(),
        roster.total_tickets
    )
}

/// Format roster one-line
pub fn format_roster_oneline(roster: &SpecialistRoster) -> String {
    format!(
        "{} specialists ({} available)",
        roster.total_count(),
        roster.available_count()
    )
}

/// Check if query is about specialists
pub fn is_specialist_roster_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "specialist",
        "specialists",
        "team member",
        "who is available",
        "available experts",
        "roster",
        "team roster",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about roster
pub fn roster_fun_fact(roster: &SpecialistRoster) -> String {
    if roster.specialists.is_empty() {
        return "No specialists on the team yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has {} specialists on the team.",
            roster.total_count()
        ),
        format!(
            "{} specialists are currently available.",
            roster.available_count()
        ),
        {
            if let Some(top) = roster.top_performer() {
                format!("{} is the top performer with {} tickets!", top.name, top.tickets_resolved)
            } else {
                "No resolutions yet.".to_string()
            }
        },
        format!(
            "The team has resolved {} tickets total.",
            roster.total_tickets
        ),
        format!(
            "{} juniors and {} seniors on the team.",
            roster.juniors().len(),
            roster.seniors().len()
        ),
    ];

    facts[roster.total_count() % facts.len()].clone()
}
