//! Pending clarification persistence (v0.0.227).

use super::types::PendingClarification;
use std::path::PathBuf;

/// Get pending clarification file path
pub fn pending_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("pending.json")
}

/// Load pending clarification (if any)
pub fn load_pending() -> Option<PendingClarification> {
    let path = pending_path();
    if !path.exists() {
        return None;
    }
    let data = std::fs::read_to_string(&path).ok()?;
    let pending: PendingClarification = serde_json::from_str(&data).ok()?;

    // Check if stale
    if pending.is_stale() {
        let _ = clear_pending();
        return None;
    }

    Some(pending)
}

/// Save pending clarification
pub fn save_pending(pending: &PendingClarification) -> std::io::Result<()> {
    let path = pending_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(pending)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Clear pending clarification
pub fn clear_pending() -> std::io::Result<()> {
    let path = pending_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Check if there's a pending clarification
pub fn has_pending() -> bool {
    pending_path().exists()
}
