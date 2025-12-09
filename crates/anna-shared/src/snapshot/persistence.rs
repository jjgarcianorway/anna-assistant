//! Snapshot persistence (v0.0.219).

use std::path::PathBuf;

use super::types::SystemSnapshot;

/// Get snapshots directory path
pub fn snapshots_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("snapshots")
}

/// Get last snapshot file path
pub fn last_snapshot_path() -> PathBuf {
    snapshots_dir().join("last.json")
}

/// Load the last saved snapshot (if any)
pub fn load_last_snapshot() -> Option<SystemSnapshot> {
    let path = last_snapshot_path();
    if !path.exists() {
        return None;
    }
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Save snapshot as the "last" snapshot
pub fn save_snapshot(snapshot: &SystemSnapshot) -> std::io::Result<()> {
    let dir = snapshots_dir();
    std::fs::create_dir_all(&dir)?;
    let path = last_snapshot_path();
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Clear all snapshots (for reset)
pub fn clear_snapshots() -> std::io::Result<()> {
    let dir = snapshots_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}
