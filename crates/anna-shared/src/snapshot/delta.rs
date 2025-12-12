//! Delta detection between snapshots (v0.0.219).

use serde::{Deserialize, Serialize};

use super::types::{
    SystemSnapshot, DISK_CHANGE_THRESHOLD, DISK_CRITICAL_THRESHOLD, DISK_WARN_THRESHOLD,
    MEMORY_CHANGE_THRESHOLD, MEMORY_HIGH_THRESHOLD,
};

/// A single delta item between snapshots
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaItem {
    /// Disk usage crossed warning threshold
    DiskWarning { mount: String, prev: u8, curr: u8 },
    /// Disk usage crossed critical threshold
    DiskCritical { mount: String, prev: u8, curr: u8 },
    /// Disk usage increased significantly
    DiskIncreased { mount: String, prev: u8, curr: u8 },
    /// New failed service appeared
    NewFailedService { unit: String },
    /// Service recovered (was failed, now ok)
    ServiceRecovered { unit: String },
    /// Memory crossed high threshold
    MemoryHigh { prev_percent: u8, curr_percent: u8 },
    /// Memory increased significantly
    MemoryIncreased { prev_percent: u8, curr_percent: u8 },
}

impl DeltaItem {
    /// Format as single line for display (v0.0.265: ASCII instead of emojis)
    pub fn format(&self) -> String {
        match self {
            Self::DiskWarning { mount, prev, curr } => {
                format!("[!] Disk {} at {}% (was {}%)", mount, curr, prev)
            }
            Self::DiskCritical { mount, prev, curr } => {
                format!("[!!] Disk {} CRITICAL at {}% (was {}%)", mount, curr, prev)
            }
            Self::DiskIncreased { mount, prev, curr } => {
                format!("[^] Disk {} increased to {}% (was {}%)", mount, curr, prev)
            }
            Self::NewFailedService { unit } => {
                format!("[!!] Service {} failed", unit)
            }
            Self::ServiceRecovered { unit } => {
                format!("[ok] Service {} recovered", unit)
            }
            Self::MemoryHigh {
                prev_percent,
                curr_percent,
            } => {
                format!(
                    "[!] Memory high at {}% (was {}%)",
                    curr_percent, prev_percent
                )
            }
            Self::MemoryIncreased {
                prev_percent,
                curr_percent,
            } => {
                format!(
                    "[^] Memory increased to {}% (was {}%)",
                    curr_percent, prev_percent
                )
            }
        }
    }

    /// Check if this is an error-level delta
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::DiskCritical { .. } | Self::NewFailedService { .. }
        )
    }

    /// Check if this is a warning-level delta
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::DiskWarning { .. } | Self::MemoryHigh { .. })
    }
}

/// Compare two snapshots and return meaningful deltas only
pub fn diff_snapshots(prev: &SystemSnapshot, curr: &SystemSnapshot) -> Vec<DeltaItem> {
    let mut deltas = Vec::new();

    // Disk deltas (deterministic order via BTreeMap)
    for (mount, &curr_pct) in &curr.disk {
        let prev_pct = prev.disk.get(mount).copied().unwrap_or(0);

        // Check critical threshold crossing
        if curr_pct >= DISK_CRITICAL_THRESHOLD && prev_pct < DISK_CRITICAL_THRESHOLD {
            deltas.push(DeltaItem::DiskCritical {
                mount: mount.clone(),
                prev: prev_pct,
                curr: curr_pct,
            });
        }
        // Check warning threshold crossing
        else if curr_pct >= DISK_WARN_THRESHOLD && prev_pct < DISK_WARN_THRESHOLD {
            deltas.push(DeltaItem::DiskWarning {
                mount: mount.clone(),
                prev: prev_pct,
                curr: curr_pct,
            });
        }
        // Check significant increase
        else if curr_pct >= prev_pct + DISK_CHANGE_THRESHOLD {
            deltas.push(DeltaItem::DiskIncreased {
                mount: mount.clone(),
                prev: prev_pct,
                curr: curr_pct,
            });
        }
    }

    // Failed services deltas
    for unit in &curr.failed_services {
        if !prev.failed_services.contains(unit) {
            deltas.push(DeltaItem::NewFailedService { unit: unit.clone() });
        }
    }
    for unit in &prev.failed_services {
        if !curr.failed_services.contains(unit) {
            deltas.push(DeltaItem::ServiceRecovered { unit: unit.clone() });
        }
    }

    // Memory deltas
    let prev_mem = prev.memory_percent();
    let curr_mem = curr.memory_percent();

    if curr_mem >= MEMORY_HIGH_THRESHOLD && prev_mem < MEMORY_HIGH_THRESHOLD {
        deltas.push(DeltaItem::MemoryHigh {
            prev_percent: prev_mem,
            curr_percent: curr_mem,
        });
    } else if curr_mem >= prev_mem + MEMORY_CHANGE_THRESHOLD {
        deltas.push(DeltaItem::MemoryIncreased {
            prev_percent: prev_mem,
            curr_percent: curr_mem,
        });
    }

    deltas
}

/// Format deltas as text for display (deterministic, no walls of text)
pub fn format_deltas_text(deltas: &[DeltaItem]) -> String {
    if deltas.is_empty() {
        return "No new warnings since last check.".to_string();
    }

    let mut lines: Vec<String> = deltas.iter().map(|d| d.format()).collect();

    // Cap at 5 lines to avoid spam
    if lines.len() > 5 {
        let omitted = lines.len() - 4;
        lines.truncate(4);
        lines.push(format!("... and {} more changes", omitted));
    }

    lines.join("\n")
}

/// Check if deltas contain any errors or warnings worth showing
pub fn has_actionable_deltas(deltas: &[DeltaItem]) -> bool {
    deltas.iter().any(|d| d.is_error() || d.is_warning())
}
