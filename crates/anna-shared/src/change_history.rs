//! Change history tracking for undo/redo support.
//!
//! v0.0.97: Records applied changes for audit and rollback.

use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use crate::change::{ChangePlan, ChangeResult};

/// A recorded change entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEntry {
    /// Unique ID for this change
    pub id: String,
    /// When the change was applied
    pub timestamp: String,
    /// Description of the change
    pub description: String,
    /// Target file path
    pub target_path: PathBuf,
    /// Backup file path (for undo)
    pub backup_path: PathBuf,
    /// Whether backup exists (can undo)
    pub can_undo: bool,
    /// Whether this change has been undone
    pub undone: bool,
}

impl ChangeEntry {
    /// Create a new entry from a plan and result
    pub fn from_applied(plan: &ChangePlan, result: &ChangeResult) -> Option<Self> {
        if !result.applied {
            return None;
        }

        let id = format!("{:08x}", rand_id());
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

        Some(Self {
            id,
            timestamp,
            description: plan.description.clone(),
            target_path: plan.target_path.clone(),
            backup_path: result.backup_path.clone().unwrap_or_else(|| plan.backup_path.clone()),
            can_undo: result.backup_path.is_some(),
            undone: false,
        })
    }

    /// Format for display
    pub fn format_display(&self) -> String {
        let status = if self.undone {
            "[undone]"
        } else if self.can_undo {
            "[can undo]"
        } else {
            "[no backup]"
        };

        format!(
            "{} {} {} - {}",
            self.id, self.timestamp, status, self.description
        )
    }
}

/// Generate a random ID
fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (duration.as_nanos() % u32::MAX as u128) as u32
}

/// Get the change history file path
pub fn history_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".anna").join("change_history.jsonl")
}

/// Record a change to history
pub fn record_change(plan: &ChangePlan, result: &ChangeResult) -> std::io::Result<Option<String>> {
    let entry = match ChangeEntry::from_applied(plan, result) {
        Some(e) => e,
        None => return Ok(None),
    };

    let history_path = history_file();

    // Ensure directory exists
    if let Some(parent) = history_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Append to history file
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_path)?;

    let mut writer = BufWriter::new(file);
    let line = serde_json::to_string(&entry)?;
    writeln!(writer, "{}", line)?;

    Ok(Some(entry.id))
}

/// Read change history (most recent first)
pub fn read_history(limit: usize) -> std::io::Result<Vec<ChangeEntry>> {
    let history_path = history_file();

    if !history_path.exists() {
        return Ok(vec![]);
    }

    let file = File::open(&history_path)?;
    let reader = BufReader::new(file);

    let mut entries: Vec<ChangeEntry> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| serde_json::from_str(&line).ok())
        .collect();

    // Reverse to get most recent first
    entries.reverse();

    // Limit results
    entries.truncate(limit);

    // Check if backups still exist
    for entry in &mut entries {
        if !entry.undone {
            entry.can_undo = entry.backup_path.exists();
        }
    }

    Ok(entries)
}

/// Find a change entry by ID
pub fn find_change(id: &str) -> std::io::Result<Option<ChangeEntry>> {
    let history_path = history_file();

    if !history_path.exists() {
        return Ok(None);
    }

    let file = File::open(&history_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if let Ok(entry) = serde_json::from_str::<ChangeEntry>(&line) {
            if entry.id == id {
                let mut entry = entry;
                if !entry.undone {
                    entry.can_undo = entry.backup_path.exists();
                }
                return Ok(Some(entry));
            }
        }
    }

    Ok(None)
}

/// Undo a change by restoring from backup
pub fn undo_change(id: &str) -> std::io::Result<UndoResult> {
    let entry = match find_change(id)? {
        Some(e) => e,
        None => return Ok(UndoResult::NotFound),
    };

    if entry.undone {
        return Ok(UndoResult::AlreadyUndone);
    }

    if !entry.backup_path.exists() {
        return Ok(UndoResult::NoBackup);
    }

    // Restore from backup
    fs::copy(&entry.backup_path, &entry.target_path)?;

    // Mark as undone in history
    mark_undone(id)?;

    Ok(UndoResult::Success {
        restored_from: entry.backup_path,
        restored_to: entry.target_path,
    })
}

/// Mark a change as undone in history
fn mark_undone(id: &str) -> std::io::Result<()> {
    let history_path = history_file();

    if !history_path.exists() {
        return Ok(());
    }

    // Read all entries
    let file = File::open(&history_path)?;
    let reader = BufReader::new(file);

    let mut entries: Vec<ChangeEntry> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| serde_json::from_str(&line).ok())
        .collect();

    // Mark matching entry as undone
    for entry in &mut entries {
        if entry.id == id {
            entry.undone = true;
            entry.can_undo = false;
        }
    }

    // Rewrite history file
    let file = File::create(&history_path)?;
    let mut writer = BufWriter::new(file);

    for entry in &entries {
        let line = serde_json::to_string(entry)?;
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}

/// Result of an undo operation
#[derive(Debug)]
pub enum UndoResult {
    Success {
        restored_from: PathBuf,
        restored_to: PathBuf,
    },
    NotFound,
    AlreadyUndone,
    NoBackup,
}

impl UndoResult {
    pub fn is_success(&self) -> bool {
        matches!(self, UndoResult::Success { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_entry_format() {
        let entry = ChangeEntry {
            id: "abcd1234".to_string(),
            timestamp: "2025-12-06 12:00:00 UTC".to_string(),
            description: "Add syntax on to ~/.vimrc".to_string(),
            target_path: PathBuf::from("/home/user/.vimrc"),
            backup_path: PathBuf::from("/home/user/.vimrc.anna-backup"),
            can_undo: true,
            undone: false,
        };

        let display = entry.format_display();
        assert!(display.contains("abcd1234"));
        assert!(display.contains("[can undo]"));
        assert!(display.contains("syntax on"));
    }
}
