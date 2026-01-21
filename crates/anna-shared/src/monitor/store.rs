//! Issue store - persistent storage for monitoring issues.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::types::{Issue, IssueType, MonitorResults, Severity};
use crate::config::anna_data_dir;

/// Store for persistent issues tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueStore {
    /// Currently active issues
    pub active_issues: Vec<Issue>,

    /// Historical issues (resolved)
    pub history: Vec<Issue>,

    /// When the store was last updated
    pub last_updated: Option<String>,
}

impl IssueStore {
    /// Load from disk
    pub fn load() -> Result<Self> {
        let path = issues_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let store: IssueStore = serde_json::from_str(&content)?;
            Ok(store)
        } else {
            Ok(IssueStore::default())
        }
    }

    /// Save to disk
    pub fn save(&self) -> Result<()> {
        let path = issues_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Update with new check results
    /// Phase 34A: Preserves notified/acknowledged flags for matching issues
    pub fn update(&mut self, results: MonitorResults) {
        // Build lookup map of existing issues by (type, summary) to preserve flags
        let existing_flags: std::collections::HashMap<(IssueType, String), (bool, bool)> =
            self.active_issues
                .iter()
                .map(|i| ((i.issue_type.clone(), i.summary.clone()), (i.notified, i.acknowledged)))
                .collect();

        // Mark old issues that are no longer present as resolved
        for old_issue in &mut self.active_issues {
            let still_present = results.issues.iter().any(|new| {
                new.issue_type == old_issue.issue_type && new.summary == old_issue.summary
            });

            if !still_present {
                old_issue.acknowledged = true;
                self.history.push(old_issue.clone());
            }
        }

        // Update active issues, preserving flags for issues that already existed
        let mut new_issues = results.issues;
        for issue in &mut new_issues {
            let key = (issue.issue_type.clone(), issue.summary.clone());
            if let Some(&(notified, acknowledged)) = existing_flags.get(&key) {
                // Phase 34A: Preserve flags - issue already existed, keep its state
                issue.notified = notified;
                issue.acknowledged = acknowledged;
            }
        }
        self.active_issues = new_issues;
        self.last_updated = Some(chrono::Utc::now().to_rfc3339());

        // Limit history size
        if self.history.len() > 100 {
            self.history = self.history.split_off(self.history.len() - 100);
        }
    }

    /// Get issues that haven't been notified yet
    pub fn get_unnotified(&self) -> Vec<&Issue> {
        self.active_issues.iter().filter(|i| !i.notified).collect()
    }

    /// Mark issues as notified
    pub fn mark_notified(&mut self) {
        for issue in &mut self.active_issues {
            issue.notified = true;
        }
    }

    /// Get critical issues
    pub fn get_critical(&self) -> Vec<&Issue> {
        self.active_issues
            .iter()
            .filter(|i| i.severity == Severity::Critical && !i.acknowledged)
            .collect()
    }

    /// Acknowledge an issue
    pub fn acknowledge(&mut self, summary: &str) {
        if let Some(issue) = self.active_issues.iter_mut().find(|i| i.summary == summary) {
            issue.acknowledged = true;
        }
    }
}

/// Get issues storage path
pub fn issues_path() -> PathBuf {
    anna_data_dir().join("issues.json")
}

/// v0.3.70: Find an issue matching a warning inquiry subject
/// Returns the matching issue if found
pub fn find_matching_issue(subject: &str) -> Option<Issue> {
    let store = IssueStore::load().ok()?;
    let subject_lower = subject.to_lowercase();

    // Search active issues for a match
    for issue in &store.active_issues {
        let summary_lower = issue.summary.to_lowercase();
        let details_lower = issue.details.to_lowercase();

        // Match on subject appearing in summary or details
        if summary_lower.contains(&subject_lower) || details_lower.contains(&subject_lower) {
            return Some(issue.clone());
        }

        // Match specific file patterns
        if subject_lower == "group" && details_lower.contains("/etc/group") {
            return Some(issue.clone());
        }
        if subject_lower == "passwd" && details_lower.contains("/etc/passwd") {
            return Some(issue.clone());
        }
        if subject_lower == "shadow" && details_lower.contains("/etc/shadow") {
            return Some(issue.clone());
        }
        if subject_lower == "sudoers" && details_lower.contains("/etc/sudoers") {
            return Some(issue.clone());
        }
        if (subject_lower == "sshd" || subject_lower == "ssh")
            && details_lower.contains("sshd_config")
        {
            return Some(issue.clone());
        }
    }

    None
}

/// v0.3.70: Format an issue as pure evidence - NO explanation, NO interpretation
/// This is the DATA template for Observation Phase compliance
pub fn format_issue_evidence(issue: &Issue) -> String {
    let mut output = String::new();

    // File/Entity affected
    output.push_str("EVIDENCE REPORT\n");
    output.push_str("---------------\n\n");

    // Extract file path from details if present
    if let Some(file_path) = extract_file_path(&issue.details) {
        output.push_str(&format!("File: {}\n", file_path));
    }

    // Condition detected
    output.push_str(&format!(
        "Condition: {:?}\n",
        issue.issue_type
    ));

    // Severity
    output.push_str(&format!("Severity: {:?}\n", issue.severity));

    // Evidence (raw data)
    output.push_str(&format!("Evidence: {}\n", issue.details));

    // Detection time
    output.push_str(&format!("Detected: {}\n", issue.detected_at));

    // Hash comparison note if config change
    if matches!(issue.issue_type, IssueType::ConfigChanged) {
        output.push_str("\nBaseline: Hash mismatch against stored baseline\n");
        output.push_str("Method: SHA-256 comparison of file contents\n");
    }

    output.push_str("\n[END OF EVIDENCE - No interpretation provided]\n");

    output
}

/// Helper to extract file path from issue details
fn extract_file_path(details: &str) -> Option<String> {
    // Look for common file path patterns
    let patterns = [
        "/etc/group",
        "/etc/passwd",
        "/etc/shadow",
        "/etc/sudoers",
        "/etc/ssh/sshd_config",
        "/etc/fstab",
        "/etc/hosts",
        "/etc/hostname",
        "/etc/resolv.conf",
        "/etc/pacman.conf",
        "/etc/mkinitcpio.conf",
        "/etc/default/grub",
        "/boot/loader/loader.conf",
        "/etc/systemd/system.conf",
        "/etc/security/limits.conf",
        "/etc/pam.d/system-auth",
        "/etc/firewalld/firewalld.conf",
        "/etc/nftables.conf",
    ];

    for pattern in patterns {
        if details.contains(pattern) {
            return Some(pattern.to_string());
        }
    }

    None
}

// =============================================================================
// Phase 34A: Regression Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_issue(issue_type: IssueType, summary: &str) -> Issue {
        Issue {
            issue_type,
            severity: Severity::Warning,
            summary: summary.to_string(),
            details: String::new(),
            detected_at: chrono::Utc::now().to_rfc3339(),
            suggested_fix: None,
            acknowledged: false,
            notified: false,
        }
    }

    fn make_results(issues: Vec<Issue>) -> MonitorResults {
        MonitorResults {
            issues,
            checked_at: chrono::Utc::now().to_rfc3339(),
            duration_ms: 100,
        }
    }

    #[test]
    fn test_config_banner_printed_once_per_session() {
        // Phase 34A: Simulate two update cycles with the same issue
        // The notified flag must be preserved so the alert shows only once

        let mut store = IssueStore::default();

        // First update: New issue detected
        let issue1 = make_issue(IssueType::ConfigChanged, "Config changed: group");
        store.update(make_results(vec![issue1]));

        // Issue is fresh, should be unnotified
        assert_eq!(store.get_unnotified().len(), 1, "First update: issue should be unnotified");

        // Mark as notified (simulates what get_pending_alerts does)
        store.mark_notified();
        assert_eq!(store.get_unnotified().len(), 0, "After marking: no unnotified issues");

        // Second update: Same issue still present (monitoring re-runs)
        let issue2 = make_issue(IssueType::ConfigChanged, "Config changed: group");
        store.update(make_results(vec![issue2]));

        // KEY ASSERTION: notified flag must be preserved
        // The banner should NOT appear again
        assert_eq!(
            store.get_unnotified().len(), 0,
            "Phase 34A: After second update with same issue, notified flag must be preserved"
        );
    }

    #[test]
    fn test_new_issues_are_not_notified() {
        // Verify new issues (different type/summary) are shown
        let mut store = IssueStore::default();

        // First issue
        let issue1 = make_issue(IssueType::ConfigChanged, "Config changed: group");
        store.update(make_results(vec![issue1]));
        store.mark_notified();

        // Different issue appears
        let issue2 = make_issue(IssueType::ServiceFailed, "sshd.service failed");
        store.update(make_results(vec![issue2]));

        // New issue should be unnotified
        assert_eq!(store.get_unnotified().len(), 1);
        assert_eq!(store.get_unnotified()[0].summary, "sshd.service failed");
    }
}

/// Format issues for display
pub fn format_issues_summary(issues: &[Issue]) -> String {
    if issues.is_empty() {
        return "No issues detected.".to_string();
    }

    let mut output = String::new();
    let critical: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Critical).collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Warning).collect();
    let info: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Info).collect();

    // v0.3.30: Use plain text instead of emojis
    if !critical.is_empty() {
        output.push_str("CRITICAL:\n");
        for issue in critical {
            output.push_str(&format!("  - {}\n", issue.summary));
        }
    }

    if !warnings.is_empty() {
        output.push_str("WARNINGS:\n");
        for issue in warnings {
            output.push_str(&format!("  - {}\n", issue.summary));
        }
    }

    if !info.is_empty() {
        output.push_str("INFO:\n");
        for issue in info {
            output.push_str(&format!("  - {}\n", issue.summary));
        }
    }

    output
}
