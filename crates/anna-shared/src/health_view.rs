//! Relevant Health View for "how is my computer" queries (v0.0.40).
//!
//! Produces minimal, actionable health summaries - NOT full reports.
//! Only shows critical issues and warnings. Silent when healthy.

use crate::snapshot::{
    SystemSnapshot, DISK_CRITICAL_THRESHOLD, DISK_WARN_THRESHOLD, MEMORY_HIGH_THRESHOLD,
};
use serde::{Deserialize, Serialize};

/// Severity level for health items
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthSeverity {
    /// Critical issues requiring immediate attention
    Critical,
    /// Warnings that should be addressed soon
    Warning,
    /// Informational notes (rarely shown)
    Note,
}

impl std::fmt::Display for HealthSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::Warning => write!(f, "warning"),
            Self::Note => write!(f, "note"),
        }
    }
}

/// A single health item (issue or warning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthItem {
    /// Severity level
    pub severity: HealthSeverity,
    /// Short description (one line)
    pub message: String,
    /// Category for grouping
    pub category: HealthCategory,
    /// Raw value (e.g., percentage) for deterministic sorting
    pub sort_key: u32,
}

impl HealthItem {
    /// Create a critical item
    pub fn critical(category: HealthCategory, message: impl Into<String>, sort_key: u32) -> Self {
        Self {
            severity: HealthSeverity::Critical,
            message: message.into(),
            category,
            sort_key,
        }
    }

    /// Create a warning item
    pub fn warning(category: HealthCategory, message: impl Into<String>, sort_key: u32) -> Self {
        Self {
            severity: HealthSeverity::Warning,
            message: message.into(),
            category,
            sort_key,
        }
    }

    /// Create a note item
    pub fn note(category: HealthCategory, message: impl Into<String>) -> Self {
        Self {
            severity: HealthSeverity::Note,
            message: message.into(),
            category,
            sort_key: 0,
        }
    }

    /// Format for display
    pub fn format(&self) -> String {
        let icon = match self.severity {
            HealthSeverity::Critical => "🔴",
            HealthSeverity::Warning => "⚠",
            HealthSeverity::Note => "ℹ",
        };
        format!("{} {}", icon, self.message)
    }
}

/// Health item category
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthCategory {
    /// Disk/storage issues
    Disk,
    /// Memory issues
    Memory,
    /// Service failures
    Services,
    /// System changes
    Changes,
}

/// A change since last snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthChange {
    /// What changed
    pub description: String,
    /// Whether this is a positive change (e.g., service recovered)
    pub positive: bool,
}

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
            a.category.cmp(&b.category).then_with(|| b.sort_key.cmp(&a.sort_key))
        });
        self.warnings.sort_by(|a, b| {
            a.category.cmp(&b.category).then_with(|| b.sort_key.cmp(&a.sort_key))
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
                let icon = if change.positive { "✅" } else { "⚡" };
                lines.push(format!("  {} {}", icon, change.description));
            }
        }

        // Notes only if we have other content
        if !self.notes.is_empty() && !lines.is_empty() {
            for note in &self.notes {
                lines.push(format!("ℹ {}", note));
            }
        }

        if lines.is_empty() {
            "No critical issues detected. No warnings detected.".to_string()
        } else {
            lines.join("\n")
        }
    }
}

/// Build a relevant health summary from a snapshot
pub fn build_health_summary(
    snapshot: &SystemSnapshot,
    prev_snapshot: Option<&SystemSnapshot>,
) -> RelevantHealthSummary {
    let mut summary = RelevantHealthSummary::default();

    // Check disk usage
    for (mount, &pct) in &snapshot.disk {
        if pct >= DISK_CRITICAL_THRESHOLD {
            summary.add_critical(HealthItem::critical(
                HealthCategory::Disk,
                format!("Disk {} is CRITICAL at {}% used", mount, pct),
                pct as u32,
            ));
        } else if pct >= DISK_WARN_THRESHOLD {
            summary.add_warning(HealthItem::warning(
                HealthCategory::Disk,
                format!("Disk {} is at {}% used", mount, pct),
                pct as u32,
            ));
        }
    }

    // Check memory
    let mem_pct = snapshot.memory_percent();
    if mem_pct >= MEMORY_HIGH_THRESHOLD {
        summary.add_warning(HealthItem::warning(
            HealthCategory::Memory,
            format!("Memory usage is high at {}%", mem_pct),
            mem_pct as u32,
        ));
    }

    // Check failed services
    if !snapshot.failed_services.is_empty() {
        for svc in &snapshot.failed_services {
            summary.add_critical(HealthItem::critical(
                HealthCategory::Services,
                format!("Service {} is failed", svc),
                0, // services sorted alphabetically
            ));
        }
    }

    // Check for changes since last snapshot
    if let Some(prev) = prev_snapshot {
        // New failed services
        for svc in &snapshot.failed_services {
            if !prev.failed_services.contains(svc) {
                summary.add_change(HealthChange {
                    description: format!("Service {} started failing", svc),
                    positive: false,
                });
            }
        }
        // Recovered services
        for svc in &prev.failed_services {
            if !snapshot.failed_services.contains(svc) {
                summary.add_change(HealthChange {
                    description: format!("Service {} recovered", svc),
                    positive: true,
                });
            }
        }
        // Disk usage increased significantly
        for (mount, &curr_pct) in &snapshot.disk {
            if let Some(&prev_pct) = prev.disk.get(mount) {
                if curr_pct >= prev_pct + 5 && curr_pct >= DISK_WARN_THRESHOLD {
                    summary.add_change(HealthChange {
                        description: format!("Disk {} increased from {}% to {}%", mount, prev_pct, curr_pct),
                        positive: false,
                    });
                }
            }
        }
    }

    // Set nothing_to_report flag
    summary.nothing_to_report = summary.critical.is_empty() && summary.warnings.is_empty();

    // Sort deterministically
    summary.sort();

    summary
}

/// Quick check if snapshot has any issues worth reporting
pub fn has_health_issues(snapshot: &SystemSnapshot) -> bool {
    // Check disk
    for &pct in snapshot.disk.values() {
        if pct >= DISK_WARN_THRESHOLD {
            return true;
        }
    }
    // Check memory
    if snapshot.memory_percent() >= MEMORY_HIGH_THRESHOLD {
        return true;
    }
    // Check services
    !snapshot.failed_services.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_system() {
        let mut snapshot = SystemSnapshot::new();
        snapshot.add_disk("/", 50);
        snapshot.set_memory(16_000_000_000, 8_000_000_000); // 50%

        let summary = build_health_summary(&snapshot, None);
        assert!(summary.nothing_to_report);
        assert!(summary.critical.is_empty());
        assert!(summary.warnings.is_empty());
        assert_eq!(summary.format(), "No critical issues detected. No warnings detected.");
    }

    #[test]
    fn test_disk_warning_only() {
        let mut snapshot = SystemSnapshot::new();
        snapshot.add_disk("/", 87); // above 85% threshold
        snapshot.set_memory(16_000_000_000, 8_000_000_000);

        let summary = build_health_summary(&snapshot, None);
        assert!(!summary.nothing_to_report);
        assert!(summary.critical.is_empty());
        assert_eq!(summary.warnings.len(), 1);
        assert!(summary.format().contains("87%"));
    }

    #[test]
    fn test_critical_disk() {
        let mut snapshot = SystemSnapshot::new();
        snapshot.add_disk("/", 96); // above 95% critical

        let summary = build_health_summary(&snapshot, None);
        assert_eq!(summary.critical.len(), 1);
        assert!(summary.format().contains("CRITICAL"));
    }

    #[test]
    fn test_failed_services() {
        let mut snapshot = SystemSnapshot::new();
        snapshot.add_failed_service("nginx.service");
        snapshot.add_failed_service("docker.service");

        let summary = build_health_summary(&snapshot, None);
        assert_eq!(summary.critical.len(), 2);
        assert!(summary.format().contains("nginx.service"));
        assert!(summary.format().contains("docker.service"));
    }

    #[test]
    fn test_mixed_issues_sorted() {
        let mut snapshot = SystemSnapshot::new();
        snapshot.add_disk("/", 96); // critical
        snapshot.add_disk("/home", 87); // warning
        snapshot.add_failed_service("nginx.service");
        snapshot.set_memory(16_000_000_000, 14_000_000_000); // 87.5% - warning

        let summary = build_health_summary(&snapshot, None);

        // Should have disk critical, service critical, disk warning, memory warning
        assert_eq!(summary.critical.len(), 2); // disk critical + service
        assert_eq!(summary.warnings.len(), 2); // disk warning + memory

        // Format should show critical first
        let formatted = summary.format();
        let critical_pos = formatted.find("CRITICAL").unwrap();
        let warning_pos = formatted.find("⚠").unwrap();
        assert!(critical_pos < warning_pos);
    }

    #[test]
    fn test_change_detection() {
        let mut prev = SystemSnapshot::new();
        prev.add_failed_service("nginx.service");

        let mut curr = SystemSnapshot::new();
        // nginx recovered, but docker failed
        curr.add_failed_service("docker.service");

        let summary = build_health_summary(&curr, Some(&prev));
        assert_eq!(summary.changed_since_last.len(), 2);

        let recovered = summary.changed_since_last.iter().find(|c| c.positive);
        assert!(recovered.is_some());
        assert!(recovered.unwrap().description.contains("nginx"));
    }

    #[test]
    fn test_has_health_issues() {
        let mut healthy = SystemSnapshot::new();
        healthy.add_disk("/", 50);
        assert!(!has_health_issues(&healthy));

        let mut warning = SystemSnapshot::new();
        warning.add_disk("/", 87);
        assert!(has_health_issues(&warning));

        let mut failed = SystemSnapshot::new();
        failed.add_failed_service("test.service");
        assert!(has_health_issues(&failed));
    }
}
