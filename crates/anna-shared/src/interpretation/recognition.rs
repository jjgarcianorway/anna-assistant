//! Recognition - Detect when issues resolve.
//!
//! Detects when:
//! - A previously reported issue or warning is no longer present
//! - System state converges back to baseline or a new stable baseline

use crate::monitor::{IssueStore, IssueType, Severity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type of resolution observed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Resolution {
    /// Issue no longer present in active issues
    IssueCleared,
    /// State returned to original baseline
    ReturnedToBaseline,
    /// State stabilized at new baseline
    NewBaselineEstablished,
}

/// A detected resolution event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionEvent {
    /// Unique ID for this resolution event
    pub id: String,
    /// The issue type that resolved
    pub issue_type: IssueType,
    /// Original issue summary
    pub original_summary: String,
    /// Type of resolution
    pub resolution: Resolution,
    /// When the resolution was detected
    pub detected_at: DateTime<Utc>,
    /// Original detection time of the issue
    pub issue_detected_at: Option<DateTime<Utc>>,
    /// Evidence supporting the resolution (hash, state description)
    pub evidence: String,
}

/// Detect resolutions by comparing current state to stored issue history.
pub fn detect_resolutions() -> Vec<ResolutionEvent> {
    let mut resolutions = Vec::new();

    // Load issue store
    let store = match IssueStore::load() {
        Ok(s) => s,
        Err(_) => return resolutions,
    };

    // Check history for issues that were active but are now resolved
    for historical_issue in &store.history {
        // Only process recently resolved issues (within last hour)
        if let Ok(detected_at) = chrono::DateTime::parse_from_rfc3339(&historical_issue.detected_at)
        {
            let detected_utc: DateTime<Utc> = detected_at.into();
            let age = Utc::now() - detected_utc;

            // Only report resolutions from last hour (avoid old noise)
            if age.num_hours() < 1 {
                // Check if this issue is no longer in active issues
                let still_active = store.active_issues.iter().any(|active| {
                    active.issue_type == historical_issue.issue_type
                        && active.summary == historical_issue.summary
                });

                if !still_active && historical_issue.acknowledged {
                    resolutions.push(ResolutionEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        issue_type: historical_issue.issue_type.clone(),
                        original_summary: historical_issue.summary.clone(),
                        resolution: Resolution::IssueCleared,
                        detected_at: Utc::now(),
                        issue_detected_at: Some(detected_utc),
                        evidence: format!(
                            "Issue '{}' moved to history (acknowledged/resolved)",
                            historical_issue.summary
                        ),
                    });
                }
            }
        }
    }

    // Check for baseline convergence
    // (This would compare current hashes to stored baselines)
    // For now, we detect config changes that have reverted
    if let Some(baseline_resolutions) = detect_baseline_convergence() {
        resolutions.extend(baseline_resolutions);
    }

    resolutions
}

/// Detect when system state converges back to baseline.
fn detect_baseline_convergence() -> Option<Vec<ResolutionEvent>> {
    use crate::monitor::SystemBaseline;

    let baseline = SystemBaseline::load()?;
    let changes = baseline.compare();

    // If no config changes, any previously reported config changes are resolved
    if changes.config_changed.is_empty() {
        // Check if we had config change warnings that are now cleared
        let store = IssueStore::load().ok()?;

        let config_resolutions: Vec<ResolutionEvent> = store
            .history
            .iter()
            .filter(|issue| matches!(issue.issue_type, IssueType::ConfigChanged))
            .filter(|issue| {
                // Recently acknowledged
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&issue.detected_at) {
                    let age = Utc::now() - dt.with_timezone(&Utc);
                    age.num_hours() < 24 && issue.acknowledged
                } else {
                    false
                }
            })
            .map(|issue| ResolutionEvent {
                id: uuid::Uuid::new_v4().to_string(),
                issue_type: issue.issue_type.clone(),
                original_summary: issue.summary.clone(),
                resolution: Resolution::ReturnedToBaseline,
                detected_at: Utc::now(),
                issue_detected_at: chrono::DateTime::parse_from_rfc3339(&issue.detected_at)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc)),
                evidence: "Config file hash now matches baseline".to_string(),
            })
            .collect();

        if !config_resolutions.is_empty() {
            return Some(config_resolutions);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_types() {
        assert_eq!(Resolution::IssueCleared, Resolution::IssueCleared);
        assert_ne!(Resolution::IssueCleared, Resolution::ReturnedToBaseline);
    }

    #[test]
    fn test_resolution_event_creation() {
        let event = ResolutionEvent {
            id: "test-123".to_string(),
            issue_type: IssueType::ConfigChanged,
            original_summary: "Config changed: group".to_string(),
            resolution: Resolution::IssueCleared,
            detected_at: Utc::now(),
            issue_detected_at: None,
            evidence: "Issue no longer active".to_string(),
        };

        assert_eq!(event.issue_type, IssueType::ConfigChanged);
        assert_eq!(event.resolution, Resolution::IssueCleared);
    }
}
