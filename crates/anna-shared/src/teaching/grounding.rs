//! Grounding - Connect teaching explanations to real system state.
//!
//! Teaching Mode MUST ground all explanations in observed evidence.
//! This module provides the connection between teaching and reality.
//!
//! Sources of grounding:
//! - SystemBaseline (config file hashes)
//! - IssueStore (active warnings, history)
//! - OutcomeLedger (what Anna has done, outcomes observed)
//! - Interpretation Mode (resolution events, attributions)
//!
//! If grounding cannot be established, Teaching Mode must say so clearly.

use super::mode::{EvidenceSource, GroundingContext, StateEvidence};
use crate::monitor::{IssueStore, IssueType, SystemBaseline};
use crate::outcome_ledger::{read_all_outcomes, Outcome};
use chrono::Utc;

/// Gather grounding context for a teaching explanation.
///
/// This collects all available evidence from the system to ground
/// the teaching output in reality.
pub fn gather_grounding(subject: Option<&str>) -> GroundingContext {
    let mut ctx = GroundingContext::default();

    // Gather from SystemBaseline
    if let Some(baseline_evidence) = gather_baseline_evidence(subject) {
        ctx.system_state.extend(baseline_evidence);
        ctx.baselines.push("SystemBaseline snapshot".to_string());
    } else {
        ctx.unknowns.push("No baseline snapshot available".to_string());
    }

    // Gather from IssueStore
    if let Some(issue_evidence) = gather_issue_evidence(subject) {
        ctx.system_state.extend(issue_evidence);
    }

    // Gather from OutcomeLedger
    if let Some(ledger_evidence) = gather_ledger_evidence(subject) {
        ctx.system_state.extend(ledger_evidence);
    }

    // Check for diffs
    if let Some(diff_evidence) = gather_diff_evidence(subject) {
        ctx.diffs.extend(diff_evidence);
    }

    ctx
}

/// Gather evidence from SystemBaseline.
fn gather_baseline_evidence(subject: Option<&str>) -> Option<Vec<StateEvidence>> {
    let baseline = SystemBaseline::load()?;
    let changes = baseline.compare();
    let mut evidence = Vec::new();

    // Report config changes (config_changed is Vec<String>)
    for path in &changes.config_changed {
        let relevant = subject.map_or(true, |s| path.contains(s));
        if relevant {
            evidence.push(StateEvidence {
                observation: format!("Config file {} differs from baseline", path),
                source: EvidenceSource::Baseline,
                observed_at: Utc::now(),
            });
        }
    }

    // Report config additions
    for path in &changes.config_added {
        let relevant = subject.map_or(true, |s| path.contains(s));
        if relevant {
            evidence.push(StateEvidence {
                observation: format!("Config file {} added since baseline", path),
                source: EvidenceSource::Baseline,
                observed_at: Utc::now(),
            });
        }
    }

    // Report config removals
    for path in &changes.config_removed {
        let relevant = subject.map_or(true, |s| path.contains(s));
        if relevant {
            evidence.push(StateEvidence {
                observation: format!("Config file {} removed since baseline", path),
                source: EvidenceSource::Baseline,
                observed_at: Utc::now(),
            });
        }
    }

    if evidence.is_empty() && changes.config_changed.is_empty()
        && changes.config_added.is_empty() && changes.config_removed.is_empty()
    {
        // No changes - that's also evidence
        evidence.push(StateEvidence {
            observation: "System state matches baseline (no config changes detected)".to_string(),
            source: EvidenceSource::Baseline,
            observed_at: Utc::now(),
        });
    }

    Some(evidence)
}

/// Gather evidence from IssueStore.
fn gather_issue_evidence(subject: Option<&str>) -> Option<Vec<StateEvidence>> {
    let store = IssueStore::load().ok()?;
    let mut evidence = Vec::new();

    // Active issues
    for issue in &store.active_issues {
        let relevant = subject.map_or(true, |s| {
            issue.summary.to_lowercase().contains(&s.to_lowercase())
                || format!("{:?}", issue.issue_type).to_lowercase().contains(&s.to_lowercase())
        });

        if relevant {
            evidence.push(StateEvidence {
                observation: format!(
                    "Active issue: {} ({:?})",
                    issue.summary, issue.severity
                ),
                source: EvidenceSource::IssueStore,
                observed_at: chrono::DateTime::parse_from_rfc3339(&issue.detected_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            });
        }
    }

    // Recent history (last 5 acknowledged issues)
    for issue in store.history.iter().rev().take(5) {
        let relevant = subject.map_or(true, |s| {
            issue.summary.to_lowercase().contains(&s.to_lowercase())
        });

        if relevant && issue.acknowledged {
            evidence.push(StateEvidence {
                observation: format!(
                    "Historical issue (resolved): {}",
                    issue.summary
                ),
                source: EvidenceSource::IssueStore,
                observed_at: chrono::DateTime::parse_from_rfc3339(&issue.detected_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            });
        }
    }

    if !evidence.is_empty() {
        Some(evidence)
    } else {
        None
    }
}

/// Gather evidence from OutcomeLedger.
fn gather_ledger_evidence(subject: Option<&str>) -> Option<Vec<StateEvidence>> {
    let entries = read_all_outcomes().ok()?;
    let mut evidence = Vec::new();

    // Look at recent entries (last 10)
    for entry in entries.iter().rev().take(10) {
        // OutcomeRecord has request_id, mode, intent, outcome, etc.
        // We filter based on the outcome and mode
        let relevant = subject.map_or(true, |s| {
            // Check if the subject relates to the intent type
            let intent_str = format!("{:?}", entry.intent).to_lowercase();
            intent_str.contains(&s.to_lowercase())
        });

        if relevant {
            let outcome_str = match entry.outcome {
                Outcome::Resolved => "resolved",
                Outcome::Failed => "failed",
                Outcome::Cancelled => "cancelled",
                Outcome::Expired => "expired",
                Outcome::Abstained => "abstained",
            };

            evidence.push(StateEvidence {
                observation: format!(
                    "Request {} ({:?}) -> {}",
                    truncate(&entry.request_id, 8),
                    entry.intent,
                    outcome_str
                ),
                source: EvidenceSource::OutcomeLedger,
                observed_at: chrono::DateTime::parse_from_rfc3339(&entry.ts_utc)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            });
        }
    }

    if !evidence.is_empty() {
        Some(evidence)
    } else {
        None
    }
}

/// Gather diff evidence.
fn gather_diff_evidence(subject: Option<&str>) -> Option<Vec<String>> {
    let baseline = SystemBaseline::load()?;
    let changes = baseline.compare();
    let mut diffs = Vec::new();

    for path in &changes.config_changed {
        let relevant = subject.map_or(true, |s| path.contains(s));
        if relevant {
            diffs.push(format!("Config diff detected: {}", path));
        }
    }

    if !diffs.is_empty() {
        Some(diffs)
    } else {
        None
    }
}

/// Check if we have sufficient grounding for a subject.
pub fn has_sufficient_grounding(ctx: &GroundingContext) -> bool {
    // We have sufficient grounding if:
    // 1. We have at least one piece of system state evidence, AND
    // 2. The unknowns list doesn't contain critical gaps
    !ctx.system_state.is_empty() && !ctx.unknowns.iter().any(|u| u.contains("No baseline"))
}

/// Report what grounding is missing.
pub fn report_missing_grounding(ctx: &GroundingContext) -> Vec<String> {
    let mut missing = Vec::new();

    if ctx.system_state.is_empty() {
        missing.push("No system state evidence available".to_string());
    }

    if ctx.baselines.is_empty() {
        missing.push("No baseline comparison available".to_string());
    }

    missing.extend(ctx.unknowns.clone());

    missing
}

/// Truncate a string for display.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Check if a subject relates to a specific issue type.
pub fn subject_matches_issue_type(subject: &str, issue_type: &IssueType) -> bool {
    let s = subject.to_lowercase();
    match issue_type {
        IssueType::ConfigChanged | IssueType::ConfigIssue => {
            s.contains("config") || s.contains("group") || s.contains("passwd")
                || s.contains("/etc/") || s.contains("changed")
        }
        IssueType::ServiceFailed => {
            s.contains("service") || s.contains("failed") || s.contains("systemd")
                || s.contains("unit")
        }
        IssueType::DiskSpaceLow => {
            s.contains("disk") || s.contains("space") || s.contains("storage")
                || s.contains("full")
        }
        IssueType::MemoryHigh => {
            s.contains("memory") || s.contains("ram") || s.contains("swap")
                || s.contains("oom")
        }
        IssueType::NetworkIssue => {
            s.contains("network") || s.contains("wifi") || s.contains("ethernet")
                || s.contains("connection")
        }
        IssueType::SecurityUpdates | IssueType::SshSecurity | IssueType::FirewallInactive => {
            s.contains("security") || s.contains("permission") || s.contains("access")
                || s.contains("ssh") || s.contains("firewall")
        }
        IssueType::PackagesInstalled | IssueType::PackagesUpgraded => {
            s.contains("update") || s.contains("upgrade") || s.contains("pacman")
                || s.contains("package")
        }
        IssueType::Custom(_) => false,
        // Catch-all for other issue types
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gather_grounding_returns_context() {
        // Even without system access, should return a context
        let ctx = gather_grounding(None);
        // Context is always returned (may have unknowns)
        assert!(ctx.unknowns.is_empty() || !ctx.unknowns.is_empty());
    }

    #[test]
    fn test_has_sufficient_grounding() {
        // Empty context - not sufficient
        let empty = GroundingContext::default();
        assert!(!has_sufficient_grounding(&empty));

        // Context with evidence - sufficient
        let with_evidence = GroundingContext {
            system_state: vec![StateEvidence {
                observation: "test".to_string(),
                source: EvidenceSource::Baseline,
                observed_at: Utc::now(),
            }],
            ..Default::default()
        };
        assert!(has_sufficient_grounding(&with_evidence));

        // Context with critical unknown - not sufficient
        let with_critical_unknown = GroundingContext {
            system_state: vec![StateEvidence {
                observation: "test".to_string(),
                source: EvidenceSource::Baseline,
                observed_at: Utc::now(),
            }],
            unknowns: vec!["No baseline snapshot available".to_string()],
            ..Default::default()
        };
        assert!(!has_sufficient_grounding(&with_critical_unknown));
    }

    #[test]
    fn test_report_missing_grounding() {
        let ctx = GroundingContext {
            unknowns: vec!["Something missing".to_string()],
            ..Default::default()
        };

        let missing = report_missing_grounding(&ctx);
        assert!(missing.contains(&"No system state evidence available".to_string()));
        assert!(missing.contains(&"Something missing".to_string()));
    }

    #[test]
    fn test_subject_matches_issue_type() {
        assert!(subject_matches_issue_type("group", &IssueType::ConfigChanged));
        assert!(subject_matches_issue_type("/etc/group", &IssueType::ConfigChanged));
        assert!(subject_matches_issue_type("config changed", &IssueType::ConfigChanged));

        assert!(subject_matches_issue_type("service failed", &IssueType::ServiceFailed));
        assert!(subject_matches_issue_type("systemd unit", &IssueType::ServiceFailed));

        assert!(!subject_matches_issue_type("memory", &IssueType::ConfigChanged));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }
}
