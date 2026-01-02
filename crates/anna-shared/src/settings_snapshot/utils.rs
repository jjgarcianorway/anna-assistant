// v0.0.592: Snapshot Utilities (Phase 168)
// Formatting and utility functions for snapshots

use super::manager::SnapshotManager;
use super::snapshot::SettingsSnapshot;

/// Format snapshot
pub fn format_snapshot(snapshot: &SettingsSnapshot) -> String {
    let mut output = String::new();

    output.push_str(&format!("Snapshot: {}\n", snapshot.name));
    output.push_str(&format!("ID: {}\n", &snapshot.id[..8]));
    output.push_str(&format!("Type: {} | Status: {}\n", snapshot.snapshot_type, snapshot.status));
    output.push_str(&format!("Created: {}\n", snapshot.created_at.format("%Y-%m-%d %H:%M")));
    output.push_str(&format!("Categories: {} | Size: {} bytes\n", snapshot.category_count(), snapshot.size));

    output
}

/// Format snapshot manager
pub fn format_snapshots(manager: &SnapshotManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Snapshots ===\n\n");
    output.push_str(&format!(
        "Total: {} | Size: {} bytes\n\n",
        manager.count(),
        manager.total_size()
    ));

    for snapshot in manager.all().iter().rev().take(10) {
        output.push_str(&format!(
            "{} [{}] - {} categories\n",
            snapshot.name,
            snapshot.snapshot_type,
            snapshot.category_count()
        ));
    }

    output
}

/// Check if query is about snapshots
pub fn is_snapshot_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("snapshot")
        || lower.contains("point in time")
        || lower.contains("checkpoint")
}

/// Fun fact about snapshots
pub fn settings_snapshot_fun_fact() -> &'static str {
    "Anna can create point-in-time snapshots of your settings for easy recovery!"
}
