//! Pattern detection for strategic analysis.

use super::types::{DetectedPattern, InsightCategory, StrategicInsight, TicketSummary};
use super::utils::{days_between, generate_insight_id, now_timestamp};
use crate::teams::Team;
use std::collections::HashMap;

/// Pattern detection for strategic analysis
pub struct PatternDetector;

impl PatternDetector {
    /// Detect recurring patterns in ticket data
    pub fn detect_patterns(ticket_summaries: &[TicketSummary]) -> Vec<DetectedPattern> {
        let mut patterns = vec![];

        // Group by team
        let by_team = group_by_team(ticket_summaries);

        for (team, tickets) in by_team {
            // Look for repeated query patterns
            let query_freq = query_frequency(&tickets);
            for (query_type, count) in query_freq {
                if count >= 3 {
                    patterns.push(DetectedPattern {
                        team,
                        pattern_type: query_type,
                        occurrence_count: count,
                        first_seen: tickets.first().map(|t| t.created_at).unwrap_or(0),
                        last_seen: tickets.last().map(|t| t.created_at).unwrap_or(0),
                    });
                }
            }
        }

        patterns
    }

    /// Generate insights from patterns
    pub fn patterns_to_insights(
        patterns: Vec<DetectedPattern>,
        specialist: &str,
    ) -> Vec<StrategicInsight> {
        patterns
            .into_iter()
            .map(|p| {
                let (category, title, analysis, recommendations) = insight_for_pattern(&p);

                StrategicInsight {
                    id: generate_insight_id(),
                    team: p.team,
                    specialist: specialist.to_string(),
                    category,
                    title,
                    analysis,
                    recommendations,
                    priority: priority_for_count(p.occurrence_count),
                    generated_at: now_timestamp(),
                    ticket_count: p.occurrence_count,
                    period_days: days_between(p.first_seen, p.last_seen),
                }
            })
            .collect()
    }
}

fn group_by_team(tickets: &[TicketSummary]) -> HashMap<Team, Vec<&TicketSummary>> {
    let mut by_team: HashMap<Team, Vec<&TicketSummary>> = HashMap::new();
    for ticket in tickets {
        by_team.entry(ticket.team).or_default().push(ticket);
    }
    by_team
}

fn query_frequency(tickets: &[&TicketSummary]) -> HashMap<String, usize> {
    let mut freq = HashMap::new();
    for ticket in tickets {
        *freq.entry(ticket.query_type.clone()).or_insert(0) += 1;
    }
    freq
}

fn insight_for_pattern(
    pattern: &DetectedPattern,
) -> (InsightCategory, String, String, Vec<String>) {
    let category = match pattern.pattern_type.as_str() {
        "disk_space" | "storage" => InsightCategory::CapacityPlanning,
        "service_failed" | "systemd" => InsightCategory::MaintenanceSuggestion,
        "performance" | "slow" => InsightCategory::PerformanceTrend,
        "security" | "permission" => InsightCategory::SecurityConcern,
        _ => InsightCategory::RecurringPattern,
    };

    let title = format!(
        "{} pattern: {} ({} occurrences)",
        pattern.team, pattern.pattern_type, pattern.occurrence_count
    );

    let analysis = format!(
        "The {} team has seen {} tickets about '{}' in the last {} days. \
         This recurring pattern suggests a systematic issue that may benefit from proactive resolution.",
        pattern.team,
        pattern.occurrence_count,
        pattern.pattern_type,
        days_between(pattern.first_seen, pattern.last_seen)
    );

    let recommendations = vec![
        format!(
            "Consider creating a scheduled check for {} issues",
            pattern.pattern_type
        ),
        "Review system configuration for root causes".to_string(),
        "Add monitoring for early detection".to_string(),
    ];

    (category, title, analysis, recommendations)
}

fn priority_for_count(count: usize) -> super::types::InsightPriority {
    use super::types::InsightPriority;
    match count {
        0..=2 => InsightPriority::Low,
        3..=5 => InsightPriority::Medium,
        6..=10 => InsightPriority::High,
        _ => InsightPriority::Critical,
    }
}
