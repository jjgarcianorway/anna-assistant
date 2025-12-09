//! RelevantHealthSummary struct and methods (v0.0.210).

use serde::{Deserialize, Serialize};

use super::types::{HealthChange, HealthItem};

/// Relevant health summary - only actionable items
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelevantHealthSummary {
    /// Critical issues (sorted by category then severity)
    pub critical: Vec<HealthItem>,
    /// Warnings (sorted by category then severity)
    pub warnings: Vec<HealthItem>,
    /// Notes (only included if clearly relevant)
    pub notes: Vec<String>,
    /// Changes since last check (optional)
    pub changed_since_last: Vec<HealthChange>,
    /// True if there are no issues to report
    pub nothing_to_report: bool,
}

impl RelevantHealthSummary {
    /// Create empty summary (nothing to report)
    pub fn healthy() -> Self {
        Self {
            nothing_to_report: true,
            ..Default::default()
        }
    }

    /// Add a critical item
    pub fn add_critical(&mut self, item: HealthItem) {
        self.critical.push(item);
        self.nothing_to_report = false;
    }

    /// Add a warning item
    pub fn add_warning(&mut self, item: HealthItem) {
        self.warnings.push(item);
        self.nothing_to_report = false;
    }

    /// Add a note (sparingly)
    pub fn add_note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    /// Add a change item
    pub fn add_change(&mut self, change: HealthChange) {
        self.changed_since_last.push(change);
    }

    /// Sort items deterministically (category order, then by sort_key descending)
    pub fn sort(&mut self) {
        self.critical.sort_by(|a, b| {
            a.category
                .cmp(&b.category)
                .then_with(|| b.sort_key.cmp(&a.sort_key))
        });
        self.warnings.sort_by(|a, b| {
            a.category
                .cmp(&b.category)
                .then_with(|| b.sort_key.cmp(&a.sort_key))
        });
    }

    /// Total issue count
    pub fn issue_count(&self) -> usize {
        self.critical.len() + self.warnings.len()
    }

    /// Format as user-facing text
    pub fn format(&self) -> String {
        if self.nothing_to_report && self.changed_since_last.is_empty() {
            return "No critical issues detected. No warnings detected.".to_string();
        }

        let mut lines = Vec::new();

        // Critical first
        if !self.critical.is_empty() {
            for item in &self.critical {
                lines.push(item.format());
            }
        }

        // Then warnings
        if !self.warnings.is_empty() {
            for item in &self.warnings {
                lines.push(item.format());
            }
        }

        // Changes (if any)
        if !self.changed_since_last.is_empty() {
            if !lines.is_empty() {
                lines.push(String::new()); // blank line
            }
            lines.push("Changes since last check:".to_string());
            for change in &self.changed_since_last {
                // v0.0.265: ASCII icons instead of emojis
                let icon = if change.positive { "[+]" } else { "[*]" };
                lines.push(format!("  {} {}", icon, change.description));
            }
        }

        // Notes only if we have other content
        if !self.notes.is_empty() && !lines.is_empty() {
            for note in &self.notes {
                lines.push(format!("[i] {}", note));
            }
        }

        if lines.is_empty() {
            "No critical issues detected. No warnings detected.".to_string()
        } else {
            lines.join("\n")
        }
    }
}
