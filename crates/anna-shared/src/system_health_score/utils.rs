//! Utility functions for health score queries and messages

use super::types::{HealthGrade, SystemHealthScore};

/// Check if query is asking about health score
pub fn is_health_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "health score",
        "system health",
        "health check",
        "how healthy",
        "health status",
        "system status",
        "overall health",
        "check health",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

/// Generate a health summary message
pub fn health_summary_message(health: &SystemHealthScore) -> String {
    let grade = HealthGrade::from_score(health.overall_score);

    match grade {
        HealthGrade::A => "System is in excellent health. All systems go!".to_string(),
        HealthGrade::B => "System health is good. Minor optimizations possible.".to_string(),
        HealthGrade::C => format!(
            "System health is fair. {} issue{} to address.",
            health.warnings + health.critical_issues,
            if health.warnings + health.critical_issues == 1 { "" } else { "s" }
        ),
        HealthGrade::D => format!(
            "System health is poor. {} critical issue{} need attention.",
            health.critical_issues,
            if health.critical_issues == 1 { "" } else { "s" }
        ),
        HealthGrade::F => "System health is critical! Immediate action required.".to_string(),
    }
}
