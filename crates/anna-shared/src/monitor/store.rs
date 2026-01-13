//! Issue store - persistent storage for monitoring issues.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::types::{Issue, MonitorResults, Severity};
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
    pub fn update(&mut self, results: MonitorResults) {
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

        // Keep only issues that are still present or new
        self.active_issues = results.issues;
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
