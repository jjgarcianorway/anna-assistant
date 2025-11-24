//! v6.27.0: Proactive Commentary Engine
//!
//! The Proactive Commentary Engine generates context-aware commentary when answering
//! system-related questions. It uses real telemetry, trends, and insights to provide
//! relevant additional information without being noisy.
//!
//! ## Design Principles
//!
//! 1. **Evidence-Based**: Only comment when there's real data to support it
//! 2. **Severity-Aware**: Prioritize Critical > Warning > Info
//! 3. **Context-Aware**: Match insights to the user's query intent
//! 4. **Deterministic**: No LLM calls, pure rule-based logic
//! 5. **Respect Preferences**: Honor user's detail level preference
//!
//! ## Architecture
//!
//! - `match_insights_to_intent()`: Filters insights relevant to query intent
//! - `generate_proactive_commentary()`: Produces commentary from matched insights
//! - Integrated into unified_query_handler for Status/Diagnostics/WikiReasoning queries

use crate::insights_engine::{Insight, InsightSeverity};
use crate::session_context::{DetailPreference, QueryIntent, SessionContext};

/// Maximum number of insights to include in commentary
const MAX_INSIGHTS_IN_COMMENTARY: usize = 2;

/// Minimum severity level to trigger commentary
const MIN_COMMENTARY_SEVERITY: InsightSeverity = InsightSeverity::Warning;

/// Match insights to query intent
///
/// Filters insights to only those relevant to what the user asked about.
/// This ensures commentary is contextual, not random.
pub fn match_insights_to_intent(
    intent: &QueryIntent,
    insights: &[Insight],
) -> Vec<Insight> {
    let mut matched = Vec::new();

    for insight in insights {
        let is_relevant = match intent {
            // Status queries: show top severity insights
            QueryIntent::Status => insight.severity >= MIN_COMMENTARY_SEVERITY,

            // Diagnostics: all warnings and critical
            QueryIntent::Diagnostics => insight.severity >= MIN_COMMENTARY_SEVERITY,

            // Wiki reasoning: match by topic
            QueryIntent::WikiReasoning { topic } => {
                is_insight_relevant_to_topic(&insight.detector, topic)
            }

            // Generic/Config/ActionPlan: no proactive commentary
            _ => false,
        };

        if is_relevant {
            matched.push(insight.clone());
        }
    }

    // Sort by severity (Critical first)
    matched.sort_by(|a, b| {
        severity_priority(b.severity).cmp(&severity_priority(a.severity))
    });

    // Limit to top N
    matched.truncate(MAX_INSIGHTS_IN_COMMENTARY);

    matched
}

/// Check if insight detector is relevant to wiki topic
fn is_insight_relevant_to_topic(detector: &str, topic: &str) -> bool {
    let detector_lower = detector.to_lowercase();
    let topic_lower = topic.to_lowercase();

    // Disk-related
    if topic_lower.contains("disk") || topic_lower.contains("storage") {
        return detector_lower.contains("disk")
            || detector_lower.contains("space")
            || detector_lower.contains("filesystem");
    }

    // Network-related
    if topic_lower.contains("network") || topic_lower.contains("wifi") || topic_lower.contains("dns") {
        return detector_lower.contains("network")
            || detector_lower.contains("latency")
            || detector_lower.contains("connectivity");
    }

    // Boot/performance-related
    if topic_lower.contains("boot") || topic_lower.contains("performance") {
        return detector_lower.contains("boot")
            || detector_lower.contains("startup")
            || detector_lower.contains("cpu")
            || detector_lower.contains("memory");
    }

    // Service-related
    if topic_lower.contains("service") || topic_lower.contains("systemd") {
        return detector_lower.contains("service")
            || detector_lower.contains("systemd")
            || detector_lower.contains("flapping")
            || detector_lower.contains("degraded");
    }

    // Power-related
    if topic_lower.contains("power") || topic_lower.contains("battery") {
        return detector_lower.contains("power")
            || detector_lower.contains("battery")
            || detector_lower.contains("suspend");
    }

    false
}

/// Severity priority for sorting (higher = more important)
fn severity_priority(severity: InsightSeverity) -> u8 {
    match severity {
        InsightSeverity::Critical => 3,
        InsightSeverity::Warning => 2,
        InsightSeverity::Info => 1,
    }
}

/// Generate proactive commentary from insights
///
/// Produces a single commentary block appended to the main answer.
/// Returns None if no relevant insights or user prefers "short" answers.
pub fn generate_proactive_commentary(
    session_ctx: &SessionContext,
    intent: &QueryIntent,
    insights: &[Insight],
) -> Option<String> {
    // Respect user's detail preference
    if session_ctx.preferences.detail_level == DetailPreference::Short {
        return None;
    }

    // Match insights to intent
    let matched_insights = match_insights_to_intent(intent, insights);

    if matched_insights.is_empty() {
        return None;
    }

    // Build commentary
    let mut commentary = String::new();

    // Introduction
    commentary.push_str("\n\n---\n\n");

    if matched_insights.len() == 1 {
        commentary.push_str("**By the way:** ");
    } else {
        commentary.push_str("**By the way, I've noticed:**\n\n");
    }

    // Add each insight
    for (idx, insight) in matched_insights.iter().enumerate() {
        if matched_insights.len() == 1 {
            // Single insight: inline format
            commentary.push_str(&format!("{}", insight.explanation));
        } else {
            // Multiple insights: bulleted list
            commentary.push_str(&format!("• {}\n", insight.title));
        }

        // Add evidence for verbose mode
        if session_ctx.preferences.detail_level == DetailPreference::Verbose && !insight.evidence.is_empty() {
            commentary.push_str(&format!("  Evidence: {}\n", insight.evidence.join(", ")));
        }

        // Add suggestion if available
        if let Some(suggestion) = &insight.suggestion {
            commentary.push_str(&format!("  → {}\n", suggestion));
        }

        // Spacing between insights
        if idx < matched_insights.len() - 1 && matched_insights.len() > 1 {
            commentary.push('\n');
        }
    }

    Some(commentary.trim().to_string())
}

/// v6.30.0: Generate proactive commentary with optimization profile
///
/// Respects OptimizationProfile rules:
/// - Skips noisy insights unless Critical
/// - Always includes highlighted high-value insights
pub fn generate_proactive_commentary_with_optimization(
    session_ctx: &SessionContext,
    intent: &QueryIntent,
    insights: &[Insight],
    profile: &crate::optimization_engine::OptimizationProfile,
) -> Option<String> {
    // Filter insights based on optimization profile
    let filtered_insights: Vec<Insight> = insights
        .iter()
        .filter(|i| {
            // Always show Critical
            if i.severity == crate::insights_engine::InsightSeverity::Critical {
                return true;
            }
            // Skip suppressed insights
            !profile.should_suppress(i)
        })
        .cloned()
        .collect();

    // Use the original function with filtered insights
    generate_proactive_commentary(session_ctx, intent, &filtered_insights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_insight(detector: &str, severity: InsightSeverity, title: &str) -> Insight {
        Insight::new(detector, severity, title, "Test explanation")
            .with_evidence(vec!["evidence1".to_string()])
            .with_suggestion("Try fixing it")
    }

    #[test]
    fn test_match_insights_to_status_intent() {
        let insights = vec![
            create_test_insight("disk_space", InsightSeverity::Critical, "Disk full"),
            create_test_insight("boot_time", InsightSeverity::Info, "Slow boot"),
            create_test_insight("service_flap", InsightSeverity::Warning, "Service restarting"),
        ];

        let intent = QueryIntent::Status;
        let matched = match_insights_to_intent(&intent, &insights);

        // Should match Warning and Critical (not Info)
        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].severity, InsightSeverity::Critical); // Sorted by severity
        assert_eq!(matched[1].severity, InsightSeverity::Warning);
    }

    #[test]
    fn test_match_insights_to_wiki_disk_topic() {
        let insights = vec![
            create_test_insight("disk_space_detector", InsightSeverity::Warning, "Disk usage high"),
            create_test_insight("network_latency", InsightSeverity::Warning, "Network slow"),
            create_test_insight("boot_regression", InsightSeverity::Info, "Boot slow"),
        ];

        let intent = QueryIntent::WikiReasoning { topic: "DiskSpace".to_string() };
        let matched = match_insights_to_intent(&intent, &insights);

        // Should only match disk-related insight
        assert_eq!(matched.len(), 1);
        assert!(matched[0].detector.contains("disk"));
    }

    #[test]
    fn test_match_insights_to_wiki_network_topic() {
        let insights = vec![
            create_test_insight("disk_space_detector", InsightSeverity::Warning, "Disk usage high"),
            create_test_insight("network_latency", InsightSeverity::Critical, "Network degraded"),
            create_test_insight("network_service_correlation", InsightSeverity::Warning, "DNS issues"),
        ];

        let intent = QueryIntent::WikiReasoning { topic: "Networking".to_string() };
        let matched = match_insights_to_intent(&intent, &insights);

        // Should match both network insights, sorted by severity
        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].severity, InsightSeverity::Critical);
        assert!(matched[0].detector.contains("network"));
        assert_eq!(matched[1].severity, InsightSeverity::Warning);
    }

    #[test]
    fn test_generate_commentary_respects_short_preference() {
        let mut ctx = SessionContext::new();
        ctx.preferences.detail_level = DetailPreference::Short;

        let insights = vec![
            create_test_insight("disk", InsightSeverity::Critical, "Disk full"),
        ];

        let intent = QueryIntent::Status;
        let commentary = generate_proactive_commentary(&ctx, &intent, &insights);

        // Should return None for "short" preference
        assert!(commentary.is_none());
    }

    #[test]
    fn test_generate_commentary_single_insight() {
        let ctx = SessionContext::new();

        let insights = vec![
            create_test_insight("disk_space", InsightSeverity::Warning, "Disk usage increasing"),
        ];

        let intent = QueryIntent::Status;
        let commentary = generate_proactive_commentary(&ctx, &intent, &insights);

        assert!(commentary.is_some());
        let text = commentary.unwrap();
        assert!(text.contains("By the way:"));
        assert!(text.contains("Test explanation"));
    }

    #[test]
    fn test_generate_commentary_multiple_insights() {
        let ctx = SessionContext::new();

        let insights = vec![
            create_test_insight("disk", InsightSeverity::Critical, "Disk full"),
            create_test_insight("swap", InsightSeverity::Warning, "Swap pressure"),
        ];

        let intent = QueryIntent::Status;
        let commentary = generate_proactive_commentary(&ctx, &intent, &insights);

        assert!(commentary.is_some());
        let text = commentary.unwrap();
        assert!(text.contains("By the way, I've noticed:"));
        assert!(text.contains("• Disk full"));
        assert!(text.contains("• Swap pressure"));
    }

    #[test]
    fn test_no_commentary_for_generic_intent() {
        let ctx = SessionContext::new();

        let insights = vec![
            create_test_insight("disk", InsightSeverity::Critical, "Disk full"),
        ];

        let intent = QueryIntent::Generic;
        let commentary = generate_proactive_commentary(&ctx, &intent, &insights);

        // Generic queries should not get proactive commentary
        assert!(commentary.is_none());
    }

    #[test]
    fn test_severity_sorting() {
        let insights = vec![
            create_test_insight("a", InsightSeverity::Info, "Info item"),
            create_test_insight("b", InsightSeverity::Critical, "Critical item"),
            create_test_insight("c", InsightSeverity::Warning, "Warning item"),
        ];

        let intent = QueryIntent::Diagnostics;
        let matched = match_insights_to_intent(&intent, &insights);

        // Should be sorted Critical > Warning, Info filtered out
        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].severity, InsightSeverity::Critical);
        assert_eq!(matched[1].severity, InsightSeverity::Warning);
    }
}
