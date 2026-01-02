//! Formatters for strategic insights.

use super::types::{InsightPriority, StrategicInsight, StrategicSession};
use std::collections::HashMap;

/// Format insights for email notification
pub fn format_insights_email(session: &StrategicSession) -> String {
    let mut output = String::new();

    output.push_str("Weekly Strategic Analysis Report\n");
    output.push_str("================================\n\n");

    output.push_str(&format!(
        "Analysis period: {} days\n",
        session.days_analyzed
    ));
    output.push_str(&format!(
        "Tickets analyzed: {}\n",
        session.tickets_analyzed
    ));
    output.push_str(&format!(
        "Insights generated: {}\n\n",
        session.insights.len()
    ));

    // Group by priority
    let mut by_priority: HashMap<InsightPriority, Vec<&StrategicInsight>> = HashMap::new();
    for insight in &session.insights {
        by_priority.entry(insight.priority).or_default().push(insight);
    }

    for priority in [
        InsightPriority::Critical,
        InsightPriority::High,
        InsightPriority::Medium,
        InsightPriority::Low,
    ] {
        if let Some(insights) = by_priority.get(&priority) {
            output.push_str(&format!("\n[{}]\n", priority.display()));
            for insight in insights {
                output.push_str(&format!("\n• {}\n", insight.title));
                output.push_str(&format!("  Category: {}\n", insight.category.display()));
                output.push_str(&format!("  Team: {}\n", insight.team));
                output.push_str(&format!("  Analysis: {}\n", insight.analysis));
                output.push_str("  Recommendations:\n");
                for rec in &insight.recommendations {
                    output.push_str(&format!("    - {}\n", rec));
                }
            }
        }
    }

    output.push_str("\n--\nAnna Service Desk - Senior Strategic Analysis\n");

    output
}
