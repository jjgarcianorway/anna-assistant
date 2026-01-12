//! Safe Change Engine - Anna backs up before any modification.
//! v0.0.998: Initial implementation
//!
//! Every change Anna makes:
//! - Creates a backup first
//! - Logs the change with timestamp
//! - Can be undone via natural language

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};

/// A recorded change that can be undone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRecord {
    /// Unique ID for this change
    pub id: String,
    /// What was changed (e.g., "vim-dark-mode")
    pub name: String,
    /// Human description
    pub description: String,
    /// File that was modified
    pub file_path: String,
    /// Path to backup file
    pub backup_path: String,
    /// When the change was made
    pub created_at: DateTime<Utc>,
    /// Whether this change has been undone
    pub undone: bool,
    /// Category (vim, git, bash, service, etc.)
    pub category: String,
}

/// Change history storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeHistory {
    pub changes: Vec<ChangeRecord>,
}

impl ChangeHistory {
    /// Get data directory
    fn data_dir() -> PathBuf {
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("anna");
        fs::create_dir_all(&dir).ok();
        dir
    }

    /// Get backup directory
    fn backup_dir() -> PathBuf {
        let dir = Self::data_dir().join("backups");
        fs::create_dir_all(&dir).ok();
        dir
    }

    /// Get history file path
    fn history_path() -> PathBuf {
        Self::data_dir().join("change_history.json")
    }

    /// Load change history
    pub fn load() -> Self {
        let path = Self::history_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save change history
    pub fn save(&self) -> Result<(), String> {
        let path = Self::history_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&path, content).map_err(|e| format!("Failed to write: {}", e))?;
        Ok(())
    }

    /// Add a change record
    pub fn add(&mut self, record: ChangeRecord) {
        self.changes.push(record);
        // Keep last 100 changes
        if self.changes.len() > 100 {
            // Remove oldest undone changes first
            if let Some(idx) = self.changes.iter().position(|c| c.undone) {
                self.changes.remove(idx);
            } else {
                self.changes.remove(0);
            }
        }
        if let Err(e) = self.save() {
            warn!("Failed to save change history: {}", e);
        }
    }

    /// Get recent changes that can be undone
    pub fn undoable(&self) -> Vec<&ChangeRecord> {
        self.changes.iter().filter(|c| !c.undone).rev().collect()
    }

    /// Find a change by ID or name
    pub fn find(&self, query: &str) -> Option<&ChangeRecord> {
        let q = query.to_lowercase();
        self.changes
            .iter()
            .filter(|c| !c.undone)
            .rev()
            .find(|c| c.id.contains(&q) || c.name.to_lowercase().contains(&q) || c.category.to_lowercase().contains(&q))
    }

    /// Mark a change as undone
    pub fn mark_undone(&mut self, id: &str) -> bool {
        if let Some(change) = self.changes.iter_mut().find(|c| c.id == id) {
            change.undone = true;
            self.save().ok();
            true
        } else {
            false
        }
    }
}

/// Create a backup of a file before modifying it
pub fn backup_file(file_path: &str, change_name: &str) -> Result<String, String> {
    let path = PathBuf::from(file_path);
    if !path.exists() {
        // File doesn't exist yet, no backup needed
        return Ok(String::new());
    }

    let timestamp = Utc::now().timestamp();
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let backup_name = format!("{}.anna.{}.{}", file_name, change_name, timestamp);
    let backup_path = ChangeHistory::backup_dir().join(&backup_name);

    fs::copy(&path, &backup_path)
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    info!("Created backup: {:?}", backup_path);
    Ok(backup_path.to_string_lossy().to_string())
}

/// Apply a change to a file (with backup)
pub fn apply_change(
    file_path: &str,
    content: &str,
    name: &str,
    description: &str,
    category: &str,
) -> Result<ChangeRecord, String> {
    // Create backup first
    let backup_path = backup_file(file_path, name)?;

    // Expand ~ to home directory
    let expanded_path = if file_path.starts_with("~/") {
        dirs::home_dir()
            .map(|h| h.join(&file_path[2..]))
            .unwrap_or_else(|| PathBuf::from(file_path))
    } else {
        PathBuf::from(file_path)
    };

    // Create parent directories if needed
    if let Some(parent) = expanded_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // Write the new content
    fs::write(&expanded_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    // Create change record
    let id = format!("{}-{}", name, Utc::now().timestamp());
    let record = ChangeRecord {
        id: id.clone(),
        name: name.to_string(),
        description: description.to_string(),
        file_path: file_path.to_string(),
        backup_path,
        created_at: Utc::now(),
        undone: false,
        category: category.to_string(),
    };

    // Save to history
    let mut history = ChangeHistory::load();
    history.add(record.clone());

    info!("Applied change: {} ({})", name, description);
    Ok(record)
}

/// Append content to a file (with backup)
pub fn append_to_file(
    file_path: &str,
    content: &str,
    name: &str,
    description: &str,
    category: &str,
) -> Result<ChangeRecord, String> {
    // Create backup first
    let backup_path = backup_file(file_path, name)?;

    // Expand ~ to home directory
    let expanded_path = if file_path.starts_with("~/") {
        dirs::home_dir()
            .map(|h| h.join(&file_path[2..]))
            .unwrap_or_else(|| PathBuf::from(file_path))
    } else {
        PathBuf::from(file_path)
    };

    // Read existing content
    let existing = fs::read_to_string(&expanded_path).unwrap_or_default();

    // Append new content
    let new_content = if existing.is_empty() {
        content.to_string()
    } else if existing.ends_with('\n') {
        format!("{}{}\n", existing, content)
    } else {
        format!("{}\n{}\n", existing, content)
    };

    // Write back
    fs::write(&expanded_path, &new_content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    // Create change record
    let id = format!("{}-{}", name, Utc::now().timestamp());
    let record = ChangeRecord {
        id: id.clone(),
        name: name.to_string(),
        description: description.to_string(),
        file_path: file_path.to_string(),
        backup_path,
        created_at: Utc::now(),
        undone: false,
        category: category.to_string(),
    };

    // Save to history
    let mut history = ChangeHistory::load();
    history.add(record.clone());

    info!("Appended to file: {} ({})", name, description);
    Ok(record)
}

/// Undo a change by restoring from backup
pub fn undo_change(change: &ChangeRecord) -> Result<String, String> {
    if change.backup_path.is_empty() {
        // No backup means file was created new - delete it
        let path = PathBuf::from(&change.file_path);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove file: {}", e))?;
        }

        let mut history = ChangeHistory::load();
        history.mark_undone(&change.id);

        return Ok(format!("Removed {} (it was newly created)", change.file_path));
    }

    let backup = PathBuf::from(&change.backup_path);
    if !backup.exists() {
        return Err("Backup file not found - cannot undo".to_string());
    }

    // Expand ~ in file path
    let file_path = if change.file_path.starts_with("~/") {
        dirs::home_dir()
            .map(|h| h.join(&change.file_path[2..]))
            .unwrap_or_else(|| PathBuf::from(&change.file_path))
    } else {
        PathBuf::from(&change.file_path)
    };

    // Restore from backup
    fs::copy(&backup, &file_path)
        .map_err(|e| format!("Failed to restore from backup: {}", e))?;

    // Mark as undone
    let mut history = ChangeHistory::load();
    history.mark_undone(&change.id);

    info!("Undone change: {}", change.name);
    Ok(format!("Restored {} from backup", change.file_path))
}

/// Run a system command (for service management, etc.)
pub fn run_command(cmd: &str) -> Result<String, String> {
    match Command::new("sh").arg("-c").arg(cmd).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                Ok(stdout.trim().to_string())
            } else {
                Err(stderr.trim().to_string())
            }
        }
        Err(e) => Err(format!("Failed to run command: {}", e)),
    }
}

/// Run a sudo command (for service management)
pub fn run_sudo_command(cmd: &str) -> Result<String, String> {
    run_command(&format!("sudo {}", cmd))
}

/// Get undo summary for display
pub fn get_undo_summary() -> String {
    let history = ChangeHistory::load();
    let undoable: Vec<_> = history.undoable();

    if undoable.is_empty() {
        return "No changes to undo. I haven't modified anything yet.".to_string();
    }

    let mut summary = String::from("Changes that can be undone:\n");
    for (i, change) in undoable.iter().take(5).enumerate() {
        let time = change.created_at.format("%H:%M");
        summary.push_str(&format!(
            "  {}. [{}] {} - {} ({})\n",
            i + 1,
            change.category,
            change.name,
            change.description,
            time
        ));
    }

    if undoable.len() > 5 {
        summary.push_str(&format!("  ... and {} more\n", undoable.len() - 5));
    }

    summary.push_str("\nSay 'undo [name]' or 'undo last change' to restore.");
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_history_load_save() {
        let mut history = ChangeHistory::default();
        let record = ChangeRecord {
            id: "test-123".to_string(),
            name: "test".to_string(),
            description: "Test change".to_string(),
            file_path: "/tmp/test".to_string(),
            backup_path: "/tmp/test.bak".to_string(),
            created_at: Utc::now(),
            undone: false,
            category: "test".to_string(),
        };
        history.add(record);
        assert_eq!(history.changes.len(), 1);
    }
}
