//! Behavioral Insights Engine
//!
//! Phase 5.2: Observer Layer
//!
//! Analyzes observation history to detect patterns, trends, and behavioral anomalies.
//! All insights are internal-only and not yet exposed to users.

use crate::context::{self, Observation};
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use tracing::debug;

/// A detected pattern in system behavior
#[derive(Debug, Clone, Serialize)]
pub struct BehaviorInsight {
    /// Type of pattern detected
    pub pattern_type: PatternType,
    /// Issue key this pattern applies to
    pub issue_key: String,
    /// Human-readable description
    pub description: String,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    /// Supporting data
    pub metadata: InsightMetadata,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum PatternType {
    /// Issue appears and disappears frequently
    Flapping,
    /// Issue severity has increased over time
    Escalation,
    /// Issue has been visible for extended period without action
    LongTermUnaddressed,
    /// Machine profile changed (e.g., VM scenarios)
    ProfileTransition,
}

#[derive(Debug, Clone, Serialize)]
pub struct InsightMetadata {
    /// Number of occurrences or state changes
    pub occurrence_count: usize,
    /// Time span covered by this insight
    pub days_observed: i64,
    /// Additional context
    pub details: String,
}

/// Complete insight report for a time window
#[derive(Debug, Clone, Serialize)]
pub struct InsightReport {
    /// Time window analyzed (days back)
    pub analysis_window_days: i64,
    /// Total observations analyzed
    pub total_observations: usize,
    /// Top recurring issues (by appearance count)
    pub top_recurring_issues: Vec<RecurringIssue>,
    /// Detected behavioral patterns
    pub patterns: Vec<BehaviorInsight>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecurringIssue {
    pub issue_key: String,
    pub appearance_count: usize,
    pub first_seen: String,
    pub last_seen: String,
}

/// Generate behavioral insights from observation history
///
/// This is the main API for analyzing system behavior over time.
/// Currently internal-only, not exposed to users.
pub async fn generate_insights(days_back: i64) -> Result<InsightReport> {
    debug!("Generating behavioral insights for last {} days", days_back);

    // Get all observations in the time window
    let observations = context::get_all_observations(days_back).await?;

    debug!("Analyzing {} observations", observations.len());

    let total_observations = observations.len();

    // Detect patterns
    let mut patterns = Vec::new();

    // 1. Flapping Detector
    patterns.extend(detect_flapping(&observations, days_back)?);

    // 2. Escalation Detector
    patterns.extend(detect_escalations(&observations)?);

    // 3. Long-term Trend Detector
    patterns.extend(detect_longterm_unaddressed(&observations)?);

    // 4. Profile Transition Detector
    patterns.extend(detect_profile_transitions(&observations)?);

    // Calculate top recurring issues
    let top_recurring = calculate_top_recurring(&observations, 5);

    let report = InsightReport {
        analysis_window_days: days_back,
        total_observations,
        top_recurring_issues: top_recurring,
        patterns,
    };

    debug!(
        "Generated {} insights from {} observations",
        report.patterns.len(),
        total_observations
    );

    Ok(report)
}

/// Detect issues that appear/disappear more than 5 times in 2 weeks
fn detect_flapping(observations: &[Observation], days_back: i64) -> Result<Vec<BehaviorInsight>> {
    let mut insights = Vec::new();

    // Only check within 14-day window for flapping
    let flap_window_days = if days_back < 14 { days_back } else { 14 };

    let cutoff = chrono::Utc::now() - chrono::Duration::days(flap_window_days);
    let cutoff_str = cutoff.to_rfc3339();

    // Group observations by issue_key
    let mut issue_observations: HashMap<String, Vec<&Observation>> = HashMap::new();
    for obs in observations {
        if obs.timestamp >= cutoff_str {
            issue_observations
                .entry(obs.issue_key.clone())
                .or_insert_with(Vec::new)
                .push(obs);
        }
    }

    // Check each issue for flapping pattern
    for (issue_key, obs_list) in issue_observations {
        if obs_list.len() < 10 {
            continue; // Need at least 10 observations to detect flapping
        }

        // Count visibility state changes
        let mut state_changes = 0;
        let mut last_visible: Option<bool> = None;

        for obs in &obs_list {
            if let Some(prev_visible) = last_visible {
                if prev_visible != obs.visible {
                    state_changes += 1;
                }
            }
            last_visible = Some(obs.visible);
        }

        // Flapping threshold: >5 state changes in the window
        if state_changes > 5 {
            insights.push(BehaviorInsight {
                pattern_type: PatternType::Flapping,
                issue_key: issue_key.clone(),
                description: format!(
                    "Issue '{}' has appeared and disappeared {} times in {} days",
                    issue_key, state_changes, flap_window_days
                ),
                confidence: (state_changes as f64 / 10.0).min(1.0),
                metadata: InsightMetadata {
                    occurrence_count: state_changes,
                    days_observed: flap_window_days,
                    details: format!("{} visibility state changes", state_changes),
                },
            });
        }
    }

    Ok(insights)
}

/// Detect severity escalations (Info → Warning → Critical)
fn detect_escalations(observations: &[Observation]) -> Result<Vec<BehaviorInsight>> {
    let mut insights = Vec::new();

    // Group by issue_key and sort by timestamp (oldest first)
    let mut issue_observations: HashMap<String, Vec<&Observation>> = HashMap::new();
    for obs in observations {
        issue_observations
            .entry(obs.issue_key.clone())
            .or_insert_with(Vec::new)
            .push(obs);
    }

    for (issue_key, mut obs_list) in issue_observations {
        if obs_list.len() < 2 {
            continue; // Need at least 2 observations to detect escalation
        }

        // Sort by timestamp (oldest to newest)
        obs_list.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Check for severity increases
        let first_severity = obs_list.first().unwrap().severity;
        let last_severity = obs_list.last().unwrap().severity;

        if last_severity > first_severity {
            let severity_increase = last_severity - first_severity;

            // Escalation detected
            insights.push(BehaviorInsight {
                pattern_type: PatternType::Escalation,
                issue_key: issue_key.clone(),
                description: format!(
                    "Issue '{}' has escalated from severity {} to {}",
                    issue_key,
                    severity_name(first_severity),
                    severity_name(last_severity)
                ),
                confidence: (severity_increase as f64 / 2.0).min(1.0),
                metadata: InsightMetadata {
                    occurrence_count: obs_list.len(),
                    days_observed: calculate_days_span(&obs_list),
                    details: format!(
                        "Severity increased by {} levels",
                        severity_increase
                    ),
                },
            });
        }
    }

    Ok(insights)
}

/// Detect issues visible for >14 days without user action
fn detect_longterm_unaddressed(observations: &[Observation]) -> Result<Vec<BehaviorInsight>> {
    let mut insights = Vec::new();

    // Group by issue_key
    let mut issue_observations: HashMap<String, Vec<&Observation>> = HashMap::new();
    for obs in observations {
        if obs.visible && obs.decision.is_none() {
            // Only count observations where issue is visible and no decision made
            issue_observations
                .entry(obs.issue_key.clone())
                .or_insert_with(Vec::new)
                .push(obs);
        }
    }

    for (issue_key, obs_list) in issue_observations {
        if obs_list.is_empty() {
            continue;
        }

        let days_span = calculate_days_span(&obs_list);

        // Long-term threshold: >14 days
        if days_span > 14 {
            insights.push(BehaviorInsight {
                pattern_type: PatternType::LongTermUnaddressed,
                issue_key: issue_key.clone(),
                description: format!(
                    "Issue '{}' has been visible for {} days without user action",
                    issue_key, days_span
                ),
                confidence: ((days_span - 14) as f64 / 30.0).min(1.0),
                metadata: InsightMetadata {
                    occurrence_count: obs_list.len(),
                    days_observed: days_span,
                    details: format!(
                        "Visible for {} days across {} observations",
                        days_span, obs_list.len()
                    ),
                },
            });
        }
    }

    Ok(insights)
}

/// Detect machine profile transitions (e.g., Laptop → Desktop for VM scenarios)
fn detect_profile_transitions(observations: &[Observation]) -> Result<Vec<BehaviorInsight>> {
    let mut insights = Vec::new();

    // Group by issue_key and sort by timestamp
    let mut issue_observations: HashMap<String, Vec<&Observation>> = HashMap::new();
    for obs in observations {
        issue_observations
            .entry(obs.issue_key.clone())
            .or_insert_with(Vec::new)
            .push(obs);
    }

    for (issue_key, mut obs_list) in issue_observations {
        if obs_list.len() < 2 {
            continue;
        }

        // Sort by timestamp
        obs_list.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Check for profile changes
        let first_profile = &obs_list.first().unwrap().profile;
        let last_profile = &obs_list.last().unwrap().profile;

        if first_profile != last_profile {
            insights.push(BehaviorInsight {
                pattern_type: PatternType::ProfileTransition,
                issue_key: issue_key.clone(),
                description: format!(
                    "Machine profile changed from {} to {} while observing '{}'",
                    first_profile, last_profile, issue_key
                ),
                confidence: 1.0, // Profile changes are explicit
                metadata: InsightMetadata {
                    occurrence_count: obs_list.len(),
                    days_observed: calculate_days_span(&obs_list),
                    details: format!(
                        "Profile transition: {} → {}",
                        first_profile, last_profile
                    ),
                },
            });
        }
    }

    Ok(insights)
}

/// Calculate top N recurring issues
fn calculate_top_recurring(observations: &[Observation], limit: usize) -> Vec<RecurringIssue> {
    let mut issue_map: HashMap<String, (usize, String, String)> = HashMap::new();

    for obs in observations {
        issue_map
            .entry(obs.issue_key.clone())
            .and_modify(|(count, first, last)| {
                *count += 1;
                if obs.timestamp < *first {
                    *first = obs.timestamp.clone();
                }
                if obs.timestamp > *last {
                    *last = obs.timestamp.clone();
                }
            })
            .or_insert_with(|| (1, obs.timestamp.clone(), obs.timestamp.clone()));
    }

    let mut issues: Vec<_> = issue_map
        .into_iter()
        .map(|(key, (count, first, last))| RecurringIssue {
            issue_key: key,
            appearance_count: count,
            first_seen: first,
            last_seen: last,
        })
        .collect();

    // Sort by appearance count (descending)
    issues.sort_by(|a, b| b.appearance_count.cmp(&a.appearance_count));

    issues.into_iter().take(limit).collect()
}

/// Calculate days between first and last observation
fn calculate_days_span(observations: &[&Observation]) -> i64 {
    if observations.is_empty() {
        return 0;
    }

    let first = observations
        .iter()
        .map(|o| &o.timestamp)
        .min()
        .unwrap();
    let last = observations
        .iter()
        .map(|o| &o.timestamp)
        .max()
        .unwrap();

    if let (Ok(first_dt), Ok(last_dt)) = (
        chrono::DateTime::parse_from_rfc3339(first),
        chrono::DateTime::parse_from_rfc3339(last),
    ) {
        (last_dt - first_dt).num_days()
    } else {
        0
    }
}

/// Convert severity integer to human name
fn severity_name(severity: i32) -> &'static str {
    match severity {
        0 => "Info",
        1 => "Warning",
        2 => "Critical",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_name() {
        assert_eq!(severity_name(0), "Info");
        assert_eq!(severity_name(1), "Warning");
        assert_eq!(severity_name(2), "Critical");
    }

    #[test]
    fn test_calculate_days_span() {
        let obs1 = Observation {
            id: 1,
            timestamp: "2025-11-01T10:00:00Z".to_string(),
            issue_key: "test".to_string(),
            severity: 0,
            profile: "Laptop".to_string(),
            visible: true,
            decision: None,
        };

        let obs2 = Observation {
            id: 2,
            timestamp: "2025-11-15T10:00:00Z".to_string(),
            issue_key: "test".to_string(),
            severity: 0,
            profile: "Laptop".to_string(),
            visible: true,
            decision: None,
        };

        let observations = vec![&obs1, &obs2];
        let days = calculate_days_span(&observations);

        assert_eq!(days, 14);
    }
}
