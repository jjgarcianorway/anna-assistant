//! Health delta tracking for "how is my computer" queries.
//! v0.0.41: Fast (<1s) system health summaries from cached snapshots.
//! v0.0.42: IT-department style output, only warnings/deltas by default.

use crate::snapshot::{diff_snapshots, DeltaItem, SystemSnapshot};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Maximum snapshots to keep in history
const MAX_HISTORY_SIZE: usize = 5;

/// Health delta showing what changed
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthDelta {
    pub changed_fields: Vec<String>,
    pub prev_values: BTreeMap<String, String>,
    pub new_values: BTreeMap<String, String>,
    pub delta_items: Vec<DeltaItem>,
    pub summary: String,
}

impl HealthDelta {
    /// Create from comparing two snapshots
    pub fn from_snapshots(prev: &SystemSnapshot, curr: &SystemSnapshot) -> Self {
        let delta_items = diff_snapshots(prev, curr);
        let mut changed_fields = Vec::new();
        let mut prev_values = BTreeMap::new();
        let mut new_values = BTreeMap::new();

        // Track disk changes
        for (mount, &curr_pct) in &curr.disk {
            let prev_pct = prev.disk.get(mount).copied().unwrap_or(0);
            if curr_pct != prev_pct {
                let field = format!("disk:{}", mount);
                changed_fields.push(field.clone());
                prev_values.insert(field.clone(), format!("{}%", prev_pct));
                new_values.insert(field, format!("{}%", curr_pct));
            }
        }

        // Track memory changes
        let prev_mem = prev.memory_percent();
        let curr_mem = curr.memory_percent();
        if prev_mem != curr_mem {
            changed_fields.push("memory".to_string());
            prev_values.insert("memory".to_string(), format!("{}%", prev_mem));
            new_values.insert("memory".to_string(), format!("{}%", curr_mem));
        }

        // Track failed services changes
        let prev_failed: std::collections::BTreeSet<_> =
            prev.failed_services.iter().collect();
        let curr_failed: std::collections::BTreeSet<_> =
            curr.failed_services.iter().collect();

        let new_failed: Vec<_> = curr_failed.difference(&prev_failed).collect();
        let recovered: Vec<_> = prev_failed.difference(&curr_failed).collect();

        if !new_failed.is_empty() || !recovered.is_empty() {
            changed_fields.push("services".to_string());
            prev_values.insert(
                "services".to_string(),
                format!("{} failed", prev.failed_services.len()),
            );
            new_values.insert(
                "services".to_string(),
                format!("{} failed", curr.failed_services.len()),
            );
        }

        // Generate summary
        let summary = generate_summary(&delta_items, curr);

        Self {
            changed_fields,
            prev_values,
            new_values,
            delta_items,
            summary,
        }
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.changed_fields.is_empty()
    }

    /// Check if there are actionable items (errors/warnings)
    pub fn has_actionable(&self) -> bool {
        self.delta_items.iter().any(|d| d.is_error() || d.is_warning())
    }

    /// Get count of errors
    pub fn error_count(&self) -> usize {
        self.delta_items.iter().filter(|d| d.is_error()).count()
    }

    /// Get count of warnings
    pub fn warning_count(&self) -> usize {
        self.delta_items.iter().filter(|d| d.is_warning()).count()
    }
}

/// In-memory snapshot history - stores last N snapshots, rotated on refresh.
#[derive(Debug, Clone, Default)]
pub struct SnapshotHistory {
    snapshots: Vec<SystemSnapshot>,
}

impl SnapshotHistory {
    pub fn new() -> Self { Self { snapshots: Vec::new() } }

    pub fn push(&mut self, snapshot: SystemSnapshot) {
        self.snapshots.push(snapshot);
        while self.snapshots.len() > MAX_HISTORY_SIZE { self.snapshots.remove(0); }
    }

    pub fn latest(&self) -> Option<&SystemSnapshot> { self.snapshots.last() }

    pub fn previous(&self) -> Option<&SystemSnapshot> {
        (self.snapshots.len() >= 2).then(|| &self.snapshots[self.snapshots.len() - 2])
    }

    pub fn get_back(&self, n: usize) -> Option<&SystemSnapshot> {
        (n < self.snapshots.len()).then(|| &self.snapshots[self.snapshots.len() - 1 - n])
    }

    pub fn all(&self) -> &[SystemSnapshot] { &self.snapshots }
    pub fn len(&self) -> usize { self.snapshots.len() }
    pub fn is_empty(&self) -> bool { self.snapshots.is_empty() }

    pub fn latest_delta(&self) -> Option<HealthDelta> {
        Some(HealthDelta::from_snapshots(self.previous()?, self.latest()?))
    }

    pub fn health_summary(&self) -> HealthSummary {
        HealthSummary {
            snapshot: self.latest().cloned(),
            delta: self.latest_delta(),
            history_count: self.len(),
        }
    }
}

/// Complete health summary for "how is my computer" (v0.0.41)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthSummary {
    /// Current snapshot (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<SystemSnapshot>,
    /// Delta from previous (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<HealthDelta>,
    /// Number of snapshots in history
    pub history_count: usize,
}

impl HealthSummary {
    /// Format as user-facing text (brief)
    pub fn format_brief(&self) -> String {
        match (&self.snapshot, &self.delta) {
            (Some(snap), Some(delta)) => {
                let mem = snap.memory_percent();
                let disk_summary = format_disk_summary(&snap.disk);
                let failed = snap.failed_services.len();

                let mut parts = vec![
                    format!("Memory: {}%", mem),
                    disk_summary,
                ];

                if failed > 0 {
                    parts.push(format!("Failed services: {}", failed));
                }

                if delta.has_actionable() {
                    parts.push(format!("Changes: {}", delta.summary));
                }

                parts.join(" | ")
            }
            (Some(snap), None) => {
                let mem = snap.memory_percent();
                let disk_summary = format_disk_summary(&snap.disk);
                format!("Memory: {}% | {}", mem, disk_summary)
            }
            (None, _) => "No system data available yet.".to_string(),
        }
    }

    /// Check if system is healthy (no errors or warnings)
    pub fn is_healthy(&self) -> bool {
        if let Some(snap) = &self.snapshot {
            // Check for critical disk usage
            if snap.disk.values().any(|&pct| pct >= 95) {
                return false;
            }
            // Check for failed services
            if !snap.failed_services.is_empty() {
                return false;
            }
            // Check for high memory
            if snap.memory_percent() >= 90 {
                return false;
            }
        }
        true
    }

    /// Get status emoji
    pub fn status_emoji(&self) -> &'static str {
        if self.is_healthy() {
            "✅"
        } else if let Some(delta) = &self.delta {
            if delta.error_count() > 0 {
                "🔴"
            } else {
                "⚠️"
            }
        } else {
            "⚠️"
        }
    }

    /// v0.0.42: Format as IT-department style output (minimal noise)
    /// Only shows issues and changes - quiet when healthy.
    pub fn format_it_style(&self) -> String {
        let mut lines = Vec::new();

        // Header with status
        let status = if self.is_healthy() {
            "All systems operational"
        } else {
            "Attention needed"
        };
        lines.push(status.to_string());

        if let Some(snap) = &self.snapshot {
            // Only show metrics if they're concerning
            let mem = snap.memory_percent();
            if mem >= 80 {
                lines.push(format!("  Memory: {}% (high)", mem));
            }

            // Show disks over 80%
            for (mount, pct) in &snap.disk {
                if *pct >= 80 {
                    let severity = if *pct >= 95 { "critical" } else { "high" };
                    lines.push(format!("  Disk {}: {}% ({})", mount, pct, severity));
                }
            }

            // Show failed services
            if !snap.failed_services.is_empty() {
                lines.push(format!("  Failed services: {}", snap.failed_services.len()));
                for svc in snap.failed_services.iter().take(3) {
                    lines.push(format!("    - {}", svc));
                }
                if snap.failed_services.len() > 3 {
                    lines.push(format!("    ... and {} more", snap.failed_services.len() - 3));
                }
            }
        }

        // Show delta if there are actionable items
        if let Some(delta) = &self.delta {
            if delta.has_actionable() {
                lines.push(format!("  Changes: {}", delta.summary));
            }
        }

        if lines.len() == 1 && self.is_healthy() {
            // Just the status, no issues - keep it clean
            lines[0].clone()
        } else {
            lines.join("\n")
        }
    }

    /// v0.0.42: One-liner status for quick display
    pub fn one_liner(&self) -> String {
        if self.is_healthy() {
            return "Your computer is running smoothly.".to_string();
        }

        let mut issues = Vec::new();
        if let Some(snap) = &self.snapshot {
            if snap.memory_percent() >= 90 {
                issues.push(format!("high memory ({}%)", snap.memory_percent()));
            }
            let crit_disks: Vec<_> = snap.disk.iter()
                .filter(|(_, &pct)| pct >= 95)
                .map(|(m, p)| format!("{} {}%", m, p))
                .collect();
            if !crit_disks.is_empty() {
                issues.push(format!("disk critical: {}", crit_disks.join(", ")));
            }
            if !snap.failed_services.is_empty() {
                issues.push(format!("{} failed service(s)", snap.failed_services.len()));
            }
        }

        if issues.is_empty() {
            "Minor concerns detected, but overall healthy.".to_string()
        } else {
            format!("Issues: {}", issues.join("; "))
        }
    }

    /// v0.0.42: Get issue count
    pub fn issue_count(&self) -> usize {
        let mut count = 0;
        if let Some(snap) = &self.snapshot {
            if snap.memory_percent() >= 90 {
                count += 1;
            }
            count += snap.disk.values().filter(|&&pct| pct >= 95).count();
            count += snap.failed_services.len();
        }
        count
    }

    /// v0.0.42: Get warning count (less severe than issues)
    pub fn warning_count(&self) -> usize {
        let mut count = 0;
        if let Some(snap) = &self.snapshot {
            if (80..90).contains(&snap.memory_percent()) {
                count += 1;
            }
            count += snap.disk.values().filter(|&&pct| (80..95).contains(&pct)).count();
        }
        count
    }
}

/// Generate a summary sentence from delta items
fn generate_summary(items: &[DeltaItem], curr: &SystemSnapshot) -> String {
    if items.is_empty() {
        let mem = curr.memory_percent();
        let max_disk = curr.disk.values().copied().max().unwrap_or(0);
        return format!(
            "System healthy. Memory {}%, max disk {}%.",
            mem, max_disk
        );
    }

    let errors = items.iter().filter(|d| d.is_error()).count();
    let warnings = items.iter().filter(|d| d.is_warning()).count();

    if errors > 0 && warnings > 0 {
        format!("{} error(s), {} warning(s) detected.", errors, warnings)
    } else if errors > 0 {
        format!("{} error(s) detected.", errors)
    } else if warnings > 0 {
        format!("{} warning(s) detected.", warnings)
    } else {
        format!("{} change(s) since last check.", items.len())
    }
}

/// Format disk usage summary
fn format_disk_summary(disk: &BTreeMap<String, u8>) -> String {
    if disk.is_empty() {
        return "Disk: unknown".to_string();
    }

    // Show root partition first, then highest usage
    let root_pct = disk.get("/").copied();
    let max_pct = disk.values().copied().max().unwrap_or(0);

    match root_pct {
        Some(pct) if pct == max_pct => format!("Disk /: {}%", pct),
        Some(root) => format!("Disk /: {}% (max {}%)", root, max_pct),
        None => format!("Disk max: {}%", max_pct),
    }
}

// Tests are in tests/health_delta_tests.rs
