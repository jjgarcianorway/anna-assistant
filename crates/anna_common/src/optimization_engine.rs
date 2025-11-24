//! v6.30.0: Optimization Engine
//!
//! Builds self-tuning optimization profiles based on insight meta telemetry.
//! Anna adjusts her behavior without user configuration.
//!
//! ## Purpose
//!
//! - Suppress noisy insights that never resolve
//! - Highlight high-value insights (accurate predictions)
//! - Tune detail levels based on user interaction patterns
//!
//! ## Design Principles
//!
//! 1. **Deterministic**: Pure rules-based, no randomness
//! 2. **Transparent**: Profiles are inspectable and explainable
//! 3. **Conservative**: Bias toward showing too much vs too little
//! 4. **Reversible**: All suppressions can be undone

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::insights_engine::Insight;
use crate::meta_telemetry::InsightMetaStats;
use crate::session_context::DetailPreference;

/// Optimization profile for self-tuning behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationProfile {
    /// Insight kinds to suppress (noisy, never resolved)
    pub suppressed_kinds: Vec<String>,

    /// Insight kinds to always show (high-value predictions)
    pub highlighted_kinds: Vec<String>,

    /// Preferred detail level (inferred from user behavior)
    pub preferred_detail: DetailPreference,

    /// When this profile was generated
    pub generated_at: DateTime<Utc>,
}

impl OptimizationProfile {
    /// Create a default optimization profile (no suppressions)
    pub fn default() -> Self {
        Self {
            suppressed_kinds: vec![],
            highlighted_kinds: vec![],
            preferred_detail: DetailPreference::Normal,
            generated_at: Utc::now(),
        }
    }

    /// Check if an insight should be suppressed
    pub fn should_suppress(&self, insight: &Insight) -> bool {
        // Never suppress Critical insights
        if insight.severity == crate::insights_engine::InsightSeverity::Critical {
            return false;
        }

        // Always show highlighted insights
        if self.highlighted_kinds.contains(&insight.detector) {
            return false;
        }

        // Suppress if in the suppression list
        self.suppressed_kinds.contains(&insight.detector)
    }

    /// Check if an insight should be highlighted
    pub fn should_highlight(&self, insight: &Insight) -> bool {
        self.highlighted_kinds.contains(&insight.detector)
    }
}

/// Build an optimization profile from meta telemetry data
///
/// Implements three rule clusters:
/// 1. Noisy insight suppression
/// 2. High-value insight highlighting
/// 3. Detail level tuning
pub fn build_optimization_profile(
    meta_stats: &[InsightMetaStats],
    detail_pref: DetailPreference,
) -> OptimizationProfile {
    let now = Utc::now();

    // Rule Cluster 1: Suppress noisy insights
    let suppressed = identify_noisy_insights(meta_stats, now);

    // Rule Cluster 2: Highlight high-value insights
    let highlighted = identify_high_value_insights(meta_stats, now);

    // Rule Cluster 3: Use provided detail preference (future: infer from user patterns)
    let preferred_detail = detail_pref;

    OptimizationProfile {
        suppressed_kinds: suppressed,
        highlighted_kinds: highlighted,
        preferred_detail,
        generated_at: now,
    }
}

/// Rule Cluster 1: Identify noisy insights to suppress
///
/// Criteria:
/// - Triggered 5+ times
/// - Unresolved for 7+ days
/// - Not worsening (no recent increases in severity)
fn identify_noisy_insights(stats: &[InsightMetaStats], now: DateTime<Utc>) -> Vec<String> {
    let mut noisy = Vec::new();

    for stat in stats {
        // Skip Critical insights (never suppress)
        if stat.severity == crate::insights_engine::InsightSeverity::Critical {
            continue;
        }

        // Check criteria
        let frequent = stat.trigger_count >= 5;
        let unresolved_long = stat.unresolved_for_days(7, now);
        let not_worsening = !is_worsening(stat, now);

        if frequent && unresolved_long && not_worsening {
            noisy.push(stat.insight_kind.clone());
        }
    }

    noisy
}

/// Rule Cluster 2: Identify high-value insights to highlight
///
/// Criteria:
/// - Successful predictions (resolved within reasonable time)
/// - 2+ successes in the last 30 days
fn identify_high_value_insights(stats: &[InsightMetaStats], now: DateTime<Utc>) -> Vec<String> {
    let mut high_value = Vec::new();

    for stat in stats {
        // Check if recently triggered and resolved
        let recent_trigger = stat.triggered_within_days(30, now);
        let has_resolution = stat.last_resolved_at.is_some();

        // Check for successful pattern: trigger → resolve within reasonable time
        if recent_trigger && has_resolution {
            if let (Some(triggered), Some(resolved)) = (stat.last_triggered_at, stat.last_resolved_at) {
                // Resolved within 7 days of last trigger
                let resolution_time = (resolved - triggered).num_days();
                if resolution_time >= 0 && resolution_time <= 7 {
                    // Check for multiple successes (trigger count as proxy)
                    if stat.trigger_count >= 2 {
                        high_value.push(stat.insight_kind.clone());
                    }
                }
            }
        }
    }

    high_value
}

/// Check if an insight is worsening (increasing in severity or frequency)
///
/// Currently simplified: check if recently triggered (within 3 days)
/// Future: Track severity changes over time
fn is_worsening(stat: &InsightMetaStats, now: DateTime<Utc>) -> bool {
    stat.triggered_within_days(3, now)
}

/// Generate a human-readable self-tuning report
///
/// Shows what Anna is learning about system behavior in 4-6 lines.
/// Three variants: short (2 lines), normal (4 lines), verbose (6 lines).
pub fn generate_self_tuning_report(
    meta_stats: &[InsightMetaStats],
    profile: &OptimizationProfile,
    detail: DetailPreference,
) -> Option<String> {
    if meta_stats.is_empty() {
        return None;
    }

    let now = Utc::now();
    let mut report = String::new();

    // Count insights by state
    let total_kinds = meta_stats.len();
    let suppressed_count = profile.suppressed_kinds.len();
    let highlighted_count = profile.highlighted_kinds.len();

    // Find most frequent insight
    let most_frequent = meta_stats.iter().max_by_key(|s| s.trigger_count);

    match detail {
        DetailPreference::Short => {
            // 2 lines: basics only
            report.push_str(&format!("**Self-Tuning Status**: Tracking {} insight types\n", total_kinds));
            if suppressed_count > 0 {
                report.push_str(&format!("Suppressing {} noisy patterns\n", suppressed_count));
            } else {
                report.push_str("No suppressions active\n");
            }
        }
        DetailPreference::Normal => {
            // 4 lines: include highlights
            report.push_str(&format!("**Self-Tuning Status**: Tracking {} insight types\n\n", total_kinds));
            report.push_str(&format!("• Suppressed (noisy): {}\n", suppressed_count));
            report.push_str(&format!("• Highlighted (high-value): {}\n", highlighted_count));

            if let Some(freq) = most_frequent {
                report.push_str(&format!("• Most frequent: {} ({} triggers)\n",
                    freq.insight_kind, freq.trigger_count));
            }
        }
        DetailPreference::Verbose => {
            // 6 lines: full details
            report.push_str(&format!("**Self-Tuning Status**: Tracking {} insight types\n\n", total_kinds));
            report.push_str(&format!("• Suppressed (noisy, unresolved 7+ days): {}\n", suppressed_count));
            if suppressed_count > 0 {
                let names: Vec<&str> = profile.suppressed_kinds.iter().take(3).map(|s| s.as_str()).collect();
                report.push_str(&format!("  → {}\n", names.join(", ")));
            }

            report.push_str(&format!("• Highlighted (accurate predictions): {}\n", highlighted_count));
            if highlighted_count > 0 {
                let names: Vec<&str> = profile.highlighted_kinds.iter().take(3).map(|s| s.as_str()).collect();
                report.push_str(&format!("  → {}\n", names.join(", ")));
            }

            if let Some(freq) = most_frequent {
                report.push_str(&format!("• Most frequent: {} ({} triggers)\n",
                    freq.insight_kind, freq.trigger_count));
            }
        }
    }

    Some(report.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::insights_engine::InsightSeverity;

    #[test]
    fn test_optimization_profile_default() {
        let profile = OptimizationProfile::default();

        assert!(profile.suppressed_kinds.is_empty());
        assert!(profile.highlighted_kinds.is_empty());
        assert_eq!(profile.preferred_detail, DetailPreference::Normal);
    }

    #[test]
    fn test_should_suppress_never_suppresses_critical() {
        let mut profile = OptimizationProfile::default();
        profile.suppressed_kinds.push("disk_space".to_string());

        let insight = Insight {
            id: "test_1".to_string(),
            timestamp: Utc::now(),
            title: "Critical disk".to_string(),
            severity: InsightSeverity::Critical,
            explanation: "Critical".to_string(),
            evidence: vec![],
            suggestion: None,
            detector: "disk_space".to_string(),
        };

        assert!(!profile.should_suppress(&insight));
    }

    #[test]
    fn test_should_suppress_respects_highlighted() {
        let mut profile = OptimizationProfile::default();
        profile.suppressed_kinds.push("boot_regression".to_string());
        profile.highlighted_kinds.push("boot_regression".to_string());

        let insight = Insight {
            id: "test_2".to_string(),
            timestamp: Utc::now(),
            title: "Boot slow".to_string(),
            severity: InsightSeverity::Warning,
            explanation: "Slow boot".to_string(),
            evidence: vec![],
            suggestion: None,
            detector: "boot_regression".to_string(),
        };

        // Highlighted overrides suppressed
        assert!(!profile.should_suppress(&insight));
    }

    #[test]
    fn test_identify_noisy_insights() {
        let now = Utc::now();
        let ten_days_ago = now - chrono::Duration::days(10);

        let mut stats = vec![
            InsightMetaStats {
                insight_kind: "noisy_insight".to_string(),
                severity: InsightSeverity::Info,
                trigger_count: 10,
                suppressed_count: 0,
                last_triggered_at: Some(ten_days_ago),
                last_shown_at: Some(ten_days_ago),
                last_resolved_at: None,
            },
            InsightMetaStats {
                insight_kind: "critical_noisy".to_string(),
                severity: InsightSeverity::Critical,
                trigger_count: 10,
                suppressed_count: 0,
                last_triggered_at: Some(ten_days_ago),
                last_shown_at: Some(ten_days_ago),
                last_resolved_at: None,
            },
        ];

        let noisy = identify_noisy_insights(&stats, now);

        // Should identify noisy_insight but not critical_noisy
        assert_eq!(noisy.len(), 1);
        assert!(noisy.contains(&"noisy_insight".to_string()));
        assert!(!noisy.contains(&"critical_noisy".to_string()));
    }

    #[test]
    fn test_identify_high_value_insights() {
        let now = Utc::now();
        let five_days_ago = now - chrono::Duration::days(5);
        let two_days_ago = now - chrono::Duration::days(2);

        let stats = vec![
            InsightMetaStats {
                insight_kind: "good_predictor".to_string(),
                severity: InsightSeverity::Warning,
                trigger_count: 3,
                suppressed_count: 0,
                last_triggered_at: Some(five_days_ago),
                last_shown_at: Some(five_days_ago),
                last_resolved_at: Some(two_days_ago),
            },
            InsightMetaStats {
                insight_kind: "never_resolved".to_string(),
                severity: InsightSeverity::Info,
                trigger_count: 5,
                suppressed_count: 0,
                last_triggered_at: Some(five_days_ago),
                last_shown_at: Some(five_days_ago),
                last_resolved_at: None,
            },
        ];

        let high_value = identify_high_value_insights(&stats, now);

        // Should identify good_predictor but not never_resolved
        assert_eq!(high_value.len(), 1);
        assert!(high_value.contains(&"good_predictor".to_string()));
    }

    #[test]
    fn test_build_optimization_profile() {
        let now = Utc::now();
        let ten_days_ago = now - chrono::Duration::days(10);
        let five_days_ago = now - chrono::Duration::days(5);
        let two_days_ago = now - chrono::Duration::days(2);

        let stats = vec![
            // Noisy insight
            InsightMetaStats {
                insight_kind: "noisy".to_string(),
                severity: InsightSeverity::Info,
                trigger_count: 8,
                suppressed_count: 0,
                last_triggered_at: Some(ten_days_ago),
                last_shown_at: Some(ten_days_ago),
                last_resolved_at: None,
            },
            // High-value insight
            InsightMetaStats {
                insight_kind: "predictor".to_string(),
                severity: InsightSeverity::Warning,
                trigger_count: 3,
                suppressed_count: 0,
                last_triggered_at: Some(five_days_ago),
                last_shown_at: Some(five_days_ago),
                last_resolved_at: Some(two_days_ago),
            },
        ];

        let profile = build_optimization_profile(&stats, DetailPreference::Verbose);

        assert_eq!(profile.suppressed_kinds.len(), 1);
        assert!(profile.suppressed_kinds.contains(&"noisy".to_string()));

        assert_eq!(profile.highlighted_kinds.len(), 1);
        assert!(profile.highlighted_kinds.contains(&"predictor".to_string()));

        assert_eq!(profile.preferred_detail, DetailPreference::Verbose);
    }

    #[test]
    fn test_generate_self_tuning_report_short() {
        let stats = vec![
            InsightMetaStats {
                insight_kind: "test1".to_string(),
                severity: InsightSeverity::Info,
                trigger_count: 5,
                suppressed_count: 0,
                last_triggered_at: Some(Utc::now()),
                last_shown_at: Some(Utc::now()),
                last_resolved_at: None,
            },
        ];

        let profile = OptimizationProfile {
            suppressed_kinds: vec!["test1".to_string()],
            highlighted_kinds: vec![],
            preferred_detail: DetailPreference::Short,
            generated_at: Utc::now(),
        };

        let report = generate_self_tuning_report(&stats, &profile, DetailPreference::Short);
        assert!(report.is_some());

        let text = report.unwrap();
        assert!(text.contains("Tracking 1 insight types"));
        assert!(text.contains("Suppressing 1 noisy patterns"));
    }

    #[test]
    fn test_generate_self_tuning_report_normal() {
        let stats = vec![
            InsightMetaStats {
                insight_kind: "frequent".to_string(),
                severity: InsightSeverity::Warning,
                trigger_count: 10,
                suppressed_count: 0,
                last_triggered_at: Some(Utc::now()),
                last_shown_at: Some(Utc::now()),
                last_resolved_at: None,
            },
        ];

        let profile = OptimizationProfile {
            suppressed_kinds: vec![],
            highlighted_kinds: vec!["good_one".to_string()],
            preferred_detail: DetailPreference::Normal,
            generated_at: Utc::now(),
        };

        let report = generate_self_tuning_report(&stats, &profile, DetailPreference::Normal);
        assert!(report.is_some());

        let text = report.unwrap();
        assert!(text.contains("Suppressed (noisy): 0"));
        assert!(text.contains("Highlighted (high-value): 1"));
        assert!(text.contains("Most frequent: frequent (10 triggers)"));
    }

    #[test]
    fn test_generate_self_tuning_report_verbose() {
        let stats = vec![
            InsightMetaStats {
                insight_kind: "noisy1".to_string(),
                severity: InsightSeverity::Info,
                trigger_count: 8,
                suppressed_count: 0,
                last_triggered_at: Some(Utc::now()),
                last_shown_at: Some(Utc::now()),
                last_resolved_at: None,
            },
            InsightMetaStats {
                insight_kind: "highlight1".to_string(),
                severity: InsightSeverity::Warning,
                trigger_count: 3,
                suppressed_count: 0,
                last_triggered_at: Some(Utc::now()),
                last_shown_at: Some(Utc::now()),
                last_resolved_at: Some(Utc::now()),
            },
        ];

        let profile = OptimizationProfile {
            suppressed_kinds: vec!["noisy1".to_string()],
            highlighted_kinds: vec!["highlight1".to_string()],
            preferred_detail: DetailPreference::Verbose,
            generated_at: Utc::now(),
        };

        let report = generate_self_tuning_report(&stats, &profile, DetailPreference::Verbose);
        assert!(report.is_some());

        let text = report.unwrap();
        assert!(text.contains("Suppressed (noisy, unresolved 7+ days): 1"));
        assert!(text.contains("→ noisy1"));
        assert!(text.contains("Highlighted (accurate predictions): 1"));
        assert!(text.contains("→ highlight1"));
    }
}
