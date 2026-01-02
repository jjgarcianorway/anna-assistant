// v0.0.528: Team Specialist Roster - Formatting
// Display and formatting functions for specialists and rosters

use super::roster::TeamSpecialistRoster;
use super::specialist::Specialist;

/// Format specialist for display
pub fn format_specialist(spec: &Specialist) -> String {
    format!(
        "{} ({} {})\n  Model: {} | Status: {}\n  Tickets: {} | Avg Time: {}ms | Success: {:.1}%",
        spec.name,
        spec.seniority,
        spec.department,
        spec.llm_model,
        spec.status,
        spec.tickets_closed,
        spec.avg_resolution_ms,
        spec.success_rate
    )
}

/// Format specialist compact
pub fn format_specialist_compact(spec: &Specialist) -> String {
    format!(
        "{} [{}] - {} tickets ({:.0}%)",
        spec.name, spec.department, spec.tickets_closed, spec.success_rate
    )
}

/// Format specialist oneline
pub fn format_specialist_oneline(spec: &Specialist) -> String {
    format!("{} ({})", spec.name, spec.department)
}

/// Format roster summary
pub fn format_roster_summary(roster: &TeamSpecialistRoster) -> String {
    let mut output = String::new();
    output.push_str("=== IT Department Roster ===\n\n");

    output.push_str(&format!(
        "Total Specialists: {}\n",
        roster.total_count()
    ));
    output.push_str(&format!("Total Tickets Closed: {}\n", roster.total_tickets()));
    output.push_str(&format!(
        "Available Now: {}\n\n",
        roster.available().len()
    ));

    output.push_str("--- Top Performers ---\n");
    for spec in roster.top_performers(5) {
        output.push_str(&format!("  {}\n", format_specialist_compact(spec)));
    }

    output
}

/// Check if query is roster-related
pub fn is_roster_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("roster")
        || lower.contains("specialist")
        || lower.contains("team")
        || lower.contains("department")
        || lower.contains("expert")
        || lower.contains("junior")
        || lower.contains("senior")
        || lower.contains("available")
}

/// Fun fact about teams
pub fn roster_fun_fact() -> &'static str {
    "The most effective IT teams have a healthy mix of junior and senior specialists - juniors bring fresh perspectives while seniors provide battle-tested wisdom!"
}
