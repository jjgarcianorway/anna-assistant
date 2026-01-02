// v0.0.539: Team Consultation Tracker - Formatting and Utilities
// Formatting functions and helper utilities

use super::record::ConsultationRecord;
use super::tracker::TeamConsultationTracker;

/// Format consultation record
pub fn format_consultation(record: &ConsultationRecord) -> String {
    let mut output = format!(
        "Consultation {} [{}]\n  Department: {} | Seniority: {}\n  Outcome: {} | Interactions: {}\n",
        record.id, record.timestamp.format("%Y-%m-%d %H:%M"),
        record.department, record.seniority,
        record.outcome, record.interaction_count
    );

    if let Some(dur) = record.duration_ms {
        output.push_str(&format!("  Duration: {}ms\n", dur));
    }

    output
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &TeamConsultationTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Team Consultation Stats ===\n\n");

    output.push_str(&format!("Total Consultations: {}\n", tracker.total()));
    output.push_str(&format!("Resolution Rate: {:.1}%\n", tracker.resolution_rate()));
    output.push_str(&format!("Escalation Rate: {:.1}%\n", tracker.escalation_rate()));

    if let Some(avg) = tracker.average_interactions() {
        output.push_str(&format!("Avg Interactions: {:.1}\n", avg));
    }

    output.push_str("\nMost Consulted Teams:\n");
    for (dept, count) in tracker.department_stats().iter().take(5) {
        output.push_str(&format!("  {}: {}\n", dept, count));
    }

    output
}

/// Check if query is team-related
pub fn is_team_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("team")
        || lower.contains("specialist")
        || lower.contains("consulted")
        || lower.contains("department")
        || lower.contains("escalation")
}

/// Fun fact about team consultations
pub fn team_consultation_fun_fact() -> &'static str {
    "The 'most consulted team' stat shows which IT department Anna reaches out to most often. Network issues? Desktop configs? The stats tell the story!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_team_query() {
        assert!(is_team_query("Which team was consulted most?"));
        assert!(is_team_query("Show escalation stats"));
        assert!(!is_team_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = team_consultation_fun_fact();
        assert!(fact.contains("team") || fact.contains("consulted"));
    }
}
