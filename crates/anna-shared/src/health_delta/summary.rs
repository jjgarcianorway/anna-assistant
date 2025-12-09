//! Health summary (v0.0.225).

use serde::{Deserialize, Serialize};

use crate::snapshot::SystemSnapshot;

use super::helpers::format_disk_summary;
use super::types::HealthDelta;

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

                let mut parts = vec![format!("Memory: {}%", mem), disk_summary];

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

    /// Get status icon (v0.0.265: ASCII instead of emoji)
    pub fn status_emoji(&self) -> &'static str {
        if self.is_healthy() {
            "[ok]"
        } else if let Some(delta) = &self.delta {
            if delta.error_count() > 0 {
                "[!!]"
            } else {
                "[!]"
            }
        } else {
            "[!]"
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
                    lines.push(format!(
                        "    ... and {} more",
                        snap.failed_services.len() - 3
                    ));
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
            let crit_disks: Vec<_> = snap
                .disk
                .iter()
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
            count += snap
                .disk
                .values()
                .filter(|&&pct| (80..95).contains(&pct))
                .count();
        }
        count
    }
}
