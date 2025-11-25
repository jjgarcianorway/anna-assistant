//! Reflection Engine - Anna's self-aware status block
//!
//! v6.35.0: Presence & Reflection v1
//!
//! ## Purpose
//!
//! The Reflection Engine generates Anna's "self-aware" top block for the status command.
//! This is NOT a health summary - it's Anna reflecting on her own observations about
//! the system state, combining insights, predictions, and usage patterns into 1-3 short lines.
//!
//! ## Design Principles
//!
//! 1. **Deterministic**: Uses existing insights/predictions/summaries, no LLM
//! 2. **Concise**: At most 3 short lines (under 80 chars each)
//! 3. **Context-Aware**: Considers usage stats and system trends
//! 4. **Natural Voice**: Written as if Anna is thinking aloud, not a status report
//!
//! ## Architecture
//!
//! Input sources (all already computed):
//! - InsightsEngine: High-value insights about system state
//! - PredictiveDiagnostics: Forecasted future risks
//! - InsightSummaries: High-level system summaries
//! - UsageStats: User presence and query patterns
//!
//! Output: 1-3 lines of reflection text (compact OutputEngine style)

use crate::historian::UsageStats;
use crate::insights_engine::{Insight, InsightSeverity};

/// Build Anna's reflection block for status command
///
/// This function synthesizes existing insights and usage stats
/// into a short, natural-language reflection on system state.
///
/// Returns at most 3 lines of text (Vec<String>). Empty vec if nothing notable to reflect on.
pub fn build_anna_reflection(
    insights: &[Insight],
    usage_stats: Option<&UsageStats>,
) -> Vec<String> {
    let mut lines = Vec::new();

    // Line 1: Usage pattern commentary (if applicable)
    if let Some(usage_line) = build_usage_commentary(usage_stats) {
        lines.push(usage_line);
    }

    // Line 2-3: System state reflection (from insights)
    let system_reflection = build_system_reflection(insights);
    lines.extend(system_reflection);

    // Never return more than 3 lines
    lines.truncate(3);

    lines
}

/// Build usage pattern commentary
///
/// Only fires when total_queries > 20 (enough data for patterns)
///
/// Examples:
/// - "Haven't seen you in a week - checking if anything broke while you were away."
/// - "You've been using Anna more intensively this month (47 queries vs usual 12)."
pub fn build_usage_commentary(usage_stats: Option<&UsageStats>) -> Option<String> {
    let stats = usage_stats?;

    // Only comment if we have enough data (>20 queries)
    if stats.total_queries < 20 {
        return None;
    }

    // Calculate hours since last query
    let now = chrono::Utc::now();
    let hours_since_last = (now - stats.last_seen_at).num_hours();

    // Pattern 1: Long absence (>7 days)
    if hours_since_last > 168 {
        let days = hours_since_last / 24;
        return Some(format!(
            "Haven't seen you in {} days - checking if anything broke while you were away.",
            days
        ));
    }

    // Pattern 2: Intense recent usage (queries_last_7d significantly higher than average)
    let avg_weekly = (stats.total_queries as f64) / ((now - stats.first_seen_at).num_weeks() as f64).max(1.0);
    if stats.queries_last_7d as f64 > avg_weekly * 1.5 {
        return Some(format!(
            "You've been using Anna more intensively this week ({} queries vs usual {:.0}).",
            stats.queries_last_7d,
            avg_weekly
        ));
    }

    // Pattern 3: Quiet period (queries_last_7d is 0 but user has been active before)
    if stats.queries_last_7d == 0 && stats.total_queries > 30 {
        return Some("Quiet week - no queries in the last 7 days.".to_string());
    }

    None
}

/// Build system state reflection from insights
///
/// Returns 0-2 lines reflecting on current system state
fn build_system_reflection(insights: &[Insight]) -> Vec<String> {
    let mut lines = Vec::new();

    // Check for critical insights first
    let critical_insights: Vec<&Insight> = insights
        .iter()
        .filter(|i| i.severity == InsightSeverity::Critical)
        .collect();

    if !critical_insights.is_empty() {
        // Pick the most relevant critical insight
        if let Some(insight) = critical_insights.first() {
            lines.push(format!("Critical: {}", insight.title));
        }
    } else {
        // No critical issues - check for warning-level insights
        let warnings: Vec<&Insight> = insights
            .iter()
            .filter(|i| i.severity == InsightSeverity::Warning)
            .collect();

        if let Some(warning) = warnings.first() {
            lines.push(format!("Note: {}", warning.title));
        } else if !insights.is_empty() {
            // Have some info-level insights
            if let Some(info) = insights.first() {
                lines.push(info.title.clone());
            }
        } else {
            // No insights at all - system looks healthy
            lines.push("System looks healthy.".to_string());
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration, Utc};

    fn mock_usage_stats(
        total_queries: u64,
        queries_last_7d: u64,
        hours_ago: i64,
    ) -> UsageStats {
        let now = Utc::now();
        UsageStats {
            first_seen_at: now - Duration::weeks(10), // 10 weeks ago
            last_seen_at: now - Duration::hours(hours_ago),
            total_queries,
            queries_last_7d,
            queries_last_30d: queries_last_7d * 4,
        }
    }

    #[test]
    fn test_usage_commentary_long_absence() {
        let stats = mock_usage_stats(100, 0, 200); // 200 hours = ~8 days

        let commentary = build_usage_commentary(Some(&stats));
        assert!(commentary.is_some());
        let msg = commentary.unwrap();
        assert!(msg.contains("Haven't seen you"));
        assert!(msg.contains("days"));
    }

    #[test]
    fn test_usage_commentary_intense_usage() {
        // Average weekly: 100 queries / 10 weeks = 10 per week
        // Current week: 20 queries (2x average)
        let stats = mock_usage_stats(100, 20, 2);

        let commentary = build_usage_commentary(Some(&stats));
        assert!(commentary.is_some());
        let msg = commentary.unwrap();
        assert!(msg.contains("intensively"));
        assert!(msg.contains("20 queries"));
    }

    #[test]
    fn test_usage_commentary_quiet_week() {
        let stats = mock_usage_stats(50, 0, 2); // 50 total queries, 0 this week

        let commentary = build_usage_commentary(Some(&stats));
        assert!(commentary.is_some());
        assert!(commentary.unwrap().contains("Quiet week"));
    }

    #[test]
    fn test_usage_commentary_not_enough_data() {
        let stats = mock_usage_stats(10, 2, 2); // Only 10 total queries

        let commentary = build_usage_commentary(Some(&stats));
        assert!(commentary.is_none()); // Should not comment with < 20 queries
    }

    #[test]
    fn test_system_reflection_critical_insight() {
        let insight = Insight {
            id: "critical_1".to_string(),
            timestamp: Utc::now(),
            title: "Disk space critically low".to_string(),
            severity: InsightSeverity::Critical,
            explanation: "Root filesystem at 98%".to_string(),
            evidence: vec![],
            suggestion: None,
            detector: "disk_space".to_string(),
        };

        let reflection = build_system_reflection(&[insight]);
        assert_eq!(reflection.len(), 1);
        assert!(reflection[0].contains("Critical"));
        assert!(reflection[0].contains("Disk space critically low"));
    }

    #[test]
    fn test_system_reflection_warning() {
        let warning = Insight {
            id: "warn_1".to_string(),
            timestamp: Utc::now(),
            title: "Memory usage trending upward".to_string(),
            severity: InsightSeverity::Warning,
            explanation: "Memory usage increasing".to_string(),
            evidence: vec![],
            suggestion: None,
            detector: "memory".to_string(),
        };

        let reflection = build_system_reflection(&[warning]);
        assert_eq!(reflection.len(), 1);
        assert!(reflection[0].contains("Note"));
        assert!(reflection[0].contains("Memory usage trending upward"));
    }

    #[test]
    fn test_system_reflection_healthy() {
        let reflection = build_system_reflection(&[]);
        assert_eq!(reflection.len(), 1);
        assert_eq!(reflection[0], "System looks healthy.");
    }

    #[test]
    fn test_build_anna_reflection_max_3_lines() {
        // Create scenario with usage stats + critical insight
        let stats = mock_usage_stats(100, 20, 2);
        let insight = Insight {
            id: "crit_1".to_string(),
            timestamp: Utc::now(),
            title: "Service failed".to_string(),
            severity: InsightSeverity::Critical,
            explanation: "Test".to_string(),
            evidence: vec![],
            suggestion: None,
            detector: "test".to_string(),
        };

        let reflection = build_anna_reflection(&[insight], Some(&stats));

        // Should never exceed 3 lines
        assert!(reflection.len() <= 3);
    }
}
