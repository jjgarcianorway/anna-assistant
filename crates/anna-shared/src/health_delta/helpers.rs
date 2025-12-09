//! Health delta helpers (v0.0.225).

use std::collections::BTreeMap;

use crate::snapshot::{DeltaItem, SystemSnapshot};

/// Generate a summary sentence from delta items
pub fn generate_summary(items: &[DeltaItem], curr: &SystemSnapshot) -> String {
    if items.is_empty() {
        let mem = curr.memory_percent();
        let max_disk = curr.disk.values().copied().max().unwrap_or(0);
        return format!("System healthy. Memory {}%, max disk {}%.", mem, max_disk);
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
pub fn format_disk_summary(disk: &BTreeMap<String, u8>) -> String {
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
