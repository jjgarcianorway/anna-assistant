//! Backup utility functions.

use crate::config::anna_data_dir;
use anyhow::Result;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use tracing::debug;

/// Backup directory path.
pub fn backup_dir() -> PathBuf {
    anna_data_dir().join("backups")
}

/// Create a timestamped backup directory.
pub fn create_backup_dir(prefix: &str) -> Result<PathBuf> {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = backup_dir().join(format!("{}_{}", prefix, timestamp));
    fs::create_dir_all(&backup_path)?;
    Ok(backup_path)
}

/// Backup a file if it exists.
pub fn backup_file(source: &PathBuf, backup_dir: &PathBuf, name: &str) -> Result<bool> {
    if source.exists() {
        let dest = backup_dir.join(name);
        fs::copy(source, &dest)?;
        debug!("Backed up {:?} to {:?}", source, dest);
        Ok(true)
    } else {
        Ok(false)
    }
}
