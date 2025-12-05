//! Health delta tracking for "how is my computer" queries (v0.0.41).
//!
//! Provides fast (<1s) system health summaries from cached snapshots.
//! Compares current snapshot to previous to detect meaningful changes.
//!
//! Stores last 5 snapshots in memory for history/trending.

use crate::snapshot::{diff_snapshots, DeltaItem, SystemSnapshot};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Maximum snapshots to keep in history
const MAX_HISTORY_SIZE: usize = 5;

/// Health delta showing what changed (v0.0.41)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthDelta {
    /// Fields that changed
    pub changed_fields: Vec<String>,
    /// Previous values (field -> value string)
    pub prev_values: BTreeMap<String, String>,
    /// New values (field -> value string)
    pub new_values: BTreeMap<String, String>,
    /// Actionable items from diff
    pub delta_items: Vec<DeltaItem>,
    /// Summary sentence for quick display
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

/// In-memory snapshot history (v0.0.41)
/// Stores last N snapshots, rotated on each refresh.
#[derive(Debug, Clone, Default)]
pub struct SnapshotHistory {
    /// Snapshots ordered oldest-first
    snapshots: Vec<SystemSnapshot>,
}

impl SnapshotHistory {
    /// Create empty history
    pub fn new() -> Self {
        Self { snapshots: Vec::new() }
    }

    /// Add a new snapshot, rotating out old ones
    pub fn push(&mut self, snapshot: SystemSnapshot) {
        self.snapshots.push(snapshot);
        while self.snapshots.len() > MAX_HISTORY_SIZE {
            self.snapshots.remove(0);
        }
    }

    /// Get the most recent snapshot
    pub fn latest(&self) -> Option<&SystemSnapshot> {
        self.snapshots.last()
    }

    /// Get the previous snapshot (second most recent)
    pub fn previous(&self) -> Option<&SystemSnapshot> {
        if self.snapshots.len() >= 2 {
            Some(&self.snapshots[self.snapshots.len() - 2])
        } else {
            None
        }
    }

    /// Get snapshot N positions back (0 = latest)
    pub fn get_back(&self, n: usize) -> Option<&SystemSnapshot> {
        if n >= self.snapshots.len() {
            None
        } else {
            Some(&self.snapshots[self.snapshots.len() - 1 - n])
        }
    }

    /// Get all snapshots (oldest first)
    pub fn all(&self) -> &[SystemSnapshot] {
        &self.snapshots
    }

    /// Get count of snapshots
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Calculate delta between latest and previous
    pub fn latest_delta(&self) -> Option<HealthDelta> {
        let prev = self.previous()?;
        let curr = self.latest()?;
        Some(HealthDelta::from_snapshots(prev, curr))
    }

    /// Generate health summary from current state
    pub fn health_summary(&self) -> HealthSummary {
        let latest = self.latest();
        let delta = self.latest_delta();

        HealthSummary {
            snapshot: latest.cloned(),
            delta,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_history() {
        let history = SnapshotHistory::new();
        assert!(history.is_empty());
        assert!(history.latest().is_none());
        assert!(history.previous().is_none());
        assert!(history.latest_delta().is_none());
    }

    #[test]
    fn test_history_rotation() {
        let mut history = SnapshotHistory::new();

        // Add more than MAX_HISTORY_SIZE snapshots
        for i in 0..10 {
            let mut snap = SystemSnapshot::now();
            snap.set_memory(1000, (i * 100) as u64);
            history.push(snap);
        }

        assert_eq!(history.len(), MAX_HISTORY_SIZE);
        // Latest should be the last one added (i=9)
        let latest = history.latest().unwrap();
        assert_eq!(latest.memory_used_bytes, 900);
    }

    #[test]
    fn test_health_delta_no_changes() {
        let snap1 = SystemSnapshot::now();
        let snap2 = snap1.clone();

        let delta = HealthDelta::from_snapshots(&snap1, &snap2);
        assert!(!delta.has_changes());
        assert!(!delta.has_actionable());
    }

    #[test]
    fn test_health_delta_memory_change() {
        let mut snap1 = SystemSnapshot::now();
        snap1.set_memory(1000, 500);

        let mut snap2 = SystemSnapshot::now();
        snap2.set_memory(1000, 600);

        let delta = HealthDelta::from_snapshots(&snap1, &snap2);
        assert!(delta.has_changes());
        assert!(delta.changed_fields.contains(&"memory".to_string()));
    }

    #[test]
    fn test_health_summary_healthy() {
        let mut snap = SystemSnapshot::now();
        snap.set_memory(100, 50); // 50%
        snap.add_disk("/", 50);

        let mut history = SnapshotHistory::new();
        history.push(snap);

        let summary = history.health_summary();
        assert!(summary.is_healthy());
        assert_eq!(summary.status_emoji(), "✅");
    }

    #[test]
    fn test_health_summary_unhealthy() {
        let mut snap = SystemSnapshot::now();
        snap.set_memory(100, 50);
        snap.add_disk("/", 96); // Critical!

        let mut history = SnapshotHistory::new();
        history.push(snap);

        let summary = history.health_summary();
        assert!(!summary.is_healthy());
    }
}
