//! Safe change engine for reversible config edits.
//!
//! Provides backup-first, idempotent operations for config file modifications.
//! All changes create backups before modification and support rollback.
//!
//! v0.0.27: Initial implementation with ensure_line operation.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

/// Risk level for a planned change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeRisk {
    /// Low risk - easily reversible config change
    Low,
    /// Medium risk - may affect application behavior
    Medium,
    /// High risk - system-wide impact
    High,
}

impl Default for ChangeRisk {
    fn default() -> Self {
        Self::Low
    }
}

/// A planned change before execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePlan {
    /// Human-readable description
    pub description: String,
    /// Target file path
    pub target_path: PathBuf,
    /// Path for backup file
    pub backup_path: PathBuf,
    /// The change operation
    pub operation: ChangeOperation,
    /// Risk level
    pub risk: ChangeRisk,
    /// Whether the target file currently exists
    pub target_exists: bool,
    /// Whether change would be a no-op (line already exists)
    pub is_noop: bool,
}

/// Change operation to perform
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChangeOperation {
    /// Ensure a line exists in the file (idempotent)
    EnsureLine { line: String },
    /// Append a line to the file
    AppendLine { line: String },
}

impl ChangePlan {
    /// Get a user-friendly summary of what will change
    pub fn summary(&self) -> String {
        if self.is_noop {
            format!("No change needed: {}", self.description)
        } else {
            format!(
                "{} (backup: {})",
                self.description,
                self.backup_path.display()
            )
        }
    }
}

/// Result of applying a change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeResult {
    /// Whether the change was applied
    pub applied: bool,
    /// Whether it was a no-op (already in desired state)
    pub was_noop: bool,
    /// Path to backup if created
    pub backup_path: Option<PathBuf>,
    /// Error message if failed
    pub error: Option<String>,
    /// Diagnostic information
    pub diagnostics: Vec<String>,
}

impl ChangeResult {
    /// Create a success result
    pub fn success(backup_path: PathBuf) -> Self {
        Self {
            applied: true,
            was_noop: false,
            backup_path: Some(backup_path),
            error: None,
            diagnostics: vec![],
        }
    }

    /// Create a no-op result (already in desired state)
    pub fn noop() -> Self {
        Self {
            applied: false,
            was_noop: true,
            backup_path: None,
            error: None,
            diagnostics: vec!["Line already present, no change needed".to_string()],
        }
    }

    /// Create a failure result
    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            applied: false,
            was_noop: false,
            backup_path: None,
            error: Some(error.into()),
            diagnostics: vec![],
        }
    }

    /// Add diagnostic info
    pub fn with_diagnostic(mut self, msg: impl Into<String>) -> Self {
        self.diagnostics.push(msg.into());
        self
    }
}

/// Plan an ensure_line operation
pub fn plan_ensure_line(config_path: &Path, line: &str) -> io::Result<ChangePlan> {
    let target_exists = config_path.exists();
    let is_noop = if target_exists {
        line_exists_in_file(config_path, line)?
    } else {
        false
    };

    let backup_path = compute_backup_path(config_path);

    Ok(ChangePlan {
        description: format!(
            "Ensure line '{}' in {}",
            truncate_line(line, 30),
            config_path.display()
        ),
        target_path: config_path.to_path_buf(),
        backup_path,
        operation: ChangeOperation::EnsureLine {
            line: line.to_string(),
        },
        risk: ChangeRisk::Low,
        target_exists,
        is_noop,
    })
}

/// Apply a change plan
pub fn apply_change(plan: &ChangePlan) -> ChangeResult {
    // If no-op, return early
    if plan.is_noop {
        return ChangeResult::noop();
    }

    // Create backup if target exists
    if plan.target_exists {
        if let Err(e) = create_backup(&plan.target_path, &plan.backup_path) {
            return ChangeResult::failed(format!("Failed to create backup: {}", e));
        }
    }

    // Apply the operation
    let result = match &plan.operation {
        ChangeOperation::EnsureLine { line } => {
            apply_ensure_line(&plan.target_path, line, plan.target_exists)
        }
        ChangeOperation::AppendLine { line } => append_line(&plan.target_path, line),
    };

    match result {
        Ok(()) => ChangeResult::success(plan.backup_path.clone()),
        Err(e) => ChangeResult::failed(format!("Failed to apply change: {}", e)),
    }
}

/// Rollback a change using the backup
pub fn rollback(plan: &ChangePlan) -> ChangeResult {
    if !plan.backup_path.exists() {
        return ChangeResult::failed("Backup file not found");
    }

    match fs::copy(&plan.backup_path, &plan.target_path) {
        Ok(_) => {
            // Optionally remove backup after successful rollback
            let _ = fs::remove_file(&plan.backup_path);
            ChangeResult::success(plan.backup_path.clone())
                .with_diagnostic("Restored from backup")
        }
        Err(e) => ChangeResult::failed(format!("Failed to restore from backup: {}", e)),
    }
}

/// Check if a line exists in a file
fn line_exists_in_file(path: &Path, line: &str) -> io::Result<bool> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let normalized = line.trim();

    for file_line in reader.lines() {
        if file_line?.trim() == normalized {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Create a backup of a file
fn create_backup(source: &Path, backup: &Path) -> io::Result<()> {
    // Ensure backup directory exists
    if let Some(parent) = backup.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, backup)?;
    Ok(())
}

/// Apply ensure_line operation (idempotent)
fn apply_ensure_line(path: &Path, line: &str, target_exists: bool) -> io::Result<()> {
    if target_exists {
        // Check if line already exists
        if line_exists_in_file(path, line)? {
            return Ok(()); // Already present, no-op
        }
    }

    // Append the line
    append_line(path, line)
}

/// Append a line to a file
fn append_line(path: &Path, line: &str) -> io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    // Add newline before if file exists and doesn't end with newline
    if path.exists() {
        let content = fs::read_to_string(path)?;
        if !content.is_empty() && !content.ends_with('\n') {
            writeln!(file)?;
        }
    }

    writeln!(file, "{}", line)
}

/// Compute deterministic backup path
/// Uses content hash for stable naming
fn compute_backup_path(original: &Path) -> PathBuf {
    let backup_dir = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("backups");

    let filename = original
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Use path hash for uniqueness
    let path_hash = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        original.hash(&mut hasher);
        hasher.finish()
    };

    backup_dir.join(format!("{}.{:08x}.bak", filename, path_hash as u32))
}

/// Truncate a line for display
fn truncate_line(line: &str, max_len: usize) -> String {
    if line.len() <= max_len {
        line.to_string()
    } else {
        format!("{}...", &line[..max_len - 3])
    }
}

// Tests moved to tests/change_tests.rs to keep this file under 400 lines
