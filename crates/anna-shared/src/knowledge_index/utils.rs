//! Utility functions for the knowledge index.

use std::path::PathBuf;

/// Get index file path
pub fn index_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".anna")
        .join("knowledge_index.json")
}

/// Get current time in milliseconds since UNIX epoch
pub fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
