//! Rollback System for Anna v0.0.9
//!
//! Automatic backup and rollback support for mutations:
//! - Timestamped file backups
//! - Structured mutation logs
//! - Rollback instruction generation
//! - Package management with provenance tracking

use crate::mutation_tools::{FileEditOp, RollbackInfo, ServiceState};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Base directory for rollback data
pub const ROLLBACK_BASE_DIR: &str = "/var/lib/anna/rollback";
/// Directory for file backups
pub const ROLLBACK_FILES_DIR: &str = "/var/lib/anna/rollback/files";
/// Directory for mutation logs
pub const ROLLBACK_LOGS_DIR: &str = "/var/lib/anna/rollback/logs";

/// Mutation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationLogEntry {
    pub request_id: String,
    pub timestamp: u64,
    pub timestamp_human: String,
    pub tool_name: String,
    pub evidence_ids: Vec<String>,
    pub mutation_type: MutationType,
    pub target: String,
    pub details: MutationDetails,
    pub success: bool,
    pub error: Option<String>,
}

/// Type of mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationType {
    FileEdit,
    SystemdRestart,
    SystemdReload,
    SystemdEnableNow,
    SystemdDisableNow,
    SystemdDaemonReload,
    // v0.0.9: Package operations
    PackageInstall,
    PackageRemove,
}

impl std::fmt::Display for MutationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MutationType::FileEdit => write!(f, "file_edit"),
            MutationType::SystemdRestart => write!(f, "systemd_restart"),
            MutationType::SystemdReload => write!(f, "systemd_reload"),
            MutationType::SystemdEnableNow => write!(f, "systemd_enable_now"),
            MutationType::SystemdDisableNow => write!(f, "systemd_disable_now"),
            MutationType::SystemdDaemonReload => write!(f, "systemd_daemon_reload"),
            MutationType::PackageInstall => write!(f, "package_install"),
            MutationType::PackageRemove => write!(f, "package_remove"),
        }
    }
}

/// Mutation-specific details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationDetails {
    FileEdit {
        file_path: PathBuf,
        backup_path: PathBuf,
        before_hash: String,
        after_hash: String,
        diff_summary: String,
        operations: Vec<String>, // Human-readable operation descriptions
    },
    Systemd {
        service: String,
        prior_state: Option<ServiceState>,
        operation: String,
    },
    // v0.0.9: Package operations
    Package {
        package: String,
        version: Option<String>,
        reason: String,
        operation: String, // "install" or "remove"
    },
}

/// Rollback manager
pub struct RollbackManager {
    base_dir: PathBuf,
}

impl RollbackManager {
    /// Create a new rollback manager
    pub fn new() -> Self {
        Self {
            base_dir: PathBuf::from(ROLLBACK_BASE_DIR),
        }
    }

    /// Create a new rollback manager with custom base directory (for testing)
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Ensure rollback directories exist
    pub fn ensure_dirs(&self) -> io::Result<()> {
        fs::create_dir_all(self.files_dir())?;
        fs::create_dir_all(self.logs_dir())?;
        Ok(())
    }

    /// Get files backup directory
    pub fn files_dir(&self) -> PathBuf {
        self.base_dir.join("files")
    }

    /// Get logs directory
    pub fn logs_dir(&self) -> PathBuf {
        self.base_dir.join("logs")
    }

    /// Get current timestamp
    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Format timestamp for display
    fn format_timestamp(ts: u64) -> String {
        use chrono::{TimeZone, Utc};
        Utc.timestamp_opt(ts as i64, 0)
            .single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| format!("{}s", ts))
    }

    /// Create a file backup before editing
    pub fn backup_file(&self, file_path: &Path, request_id: &str) -> io::Result<PathBuf> {
        self.ensure_dirs()?;

        let ts = Self::timestamp();
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Create backup path: /var/lib/anna/rollback/files/<timestamp>_<request_id>/<filename>
        let backup_dir = self.files_dir().join(format!("{}_{}", ts, request_id));
        fs::create_dir_all(&backup_dir)?;

        let backup_path = backup_dir.join(&file_name);

        // Also store the original path for reference
        let meta_path = backup_dir.join("_original_path.txt");
        fs::write(&meta_path, file_path.to_string_lossy().as_bytes())?;

        // Copy the file
        fs::copy(file_path, &backup_path)?;

        Ok(backup_path)
    }

    /// Calculate SHA256 hash of file content
    pub fn hash_file(path: &Path) -> io::Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let content = fs::read(path)?;
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Ok(format!("{:016x}", hasher.finish()))
    }

    /// Generate diff summary for file edit operations
    pub fn generate_diff_summary(operations: &[FileEditOp]) -> String {
        let mut summary = Vec::new();

        let mut inserts = 0;
        let mut replaces = 0;
        let mut deletes = 0;
        let mut appends = 0;
        let mut text_replaces = 0;

        for op in operations {
            match op {
                FileEditOp::InsertLine { .. } => inserts += 1,
                FileEditOp::ReplaceLine { .. } => replaces += 1,
                FileEditOp::DeleteLine { .. } => deletes += 1,
                FileEditOp::AppendLine { .. } => appends += 1,
                FileEditOp::ReplaceText { .. } => text_replaces += 1,
            }
        }

        if inserts > 0 {
            summary.push(format!("+{} line(s) inserted", inserts));
        }
        if replaces > 0 {
            summary.push(format!("~{} line(s) replaced", replaces));
        }
        if deletes > 0 {
            summary.push(format!("-{} line(s) deleted", deletes));
        }
        if appends > 0 {
            summary.push(format!("+{} line(s) appended", appends));
        }
        if text_replaces > 0 {
            summary.push(format!("~{} text replacement(s)", text_replaces));
        }

        if summary.is_empty() {
            "No changes".to_string()
        } else {
            summary.join(", ")
        }
    }

    /// Log a file edit mutation
    pub fn log_file_edit(
        &self,
        request_id: &str,
        evidence_ids: &[String],
        file_path: &Path,
        backup_path: &Path,
        before_hash: &str,
        after_hash: &str,
        operations: &[FileEditOp],
        success: bool,
        error: Option<&str>,
    ) -> io::Result<()> {
        let ts = Self::timestamp();

        let op_descriptions: Vec<String> = operations
            .iter()
            .map(|op| match op {
                FileEditOp::InsertLine {
                    line_number,
                    content,
                } => {
                    format!(
                        "Insert at line {}: '{}'",
                        line_number + 1,
                        truncate_str(content, 50)
                    )
                }
                FileEditOp::ReplaceLine {
                    line_number,
                    content,
                } => {
                    format!(
                        "Replace line {}: '{}'",
                        line_number + 1,
                        truncate_str(content, 50)
                    )
                }
                FileEditOp::DeleteLine { line_number } => {
                    format!("Delete line {}", line_number + 1)
                }
                FileEditOp::AppendLine { content } => {
                    format!("Append: '{}'", truncate_str(content, 50))
                }
                FileEditOp::ReplaceText {
                    pattern,
                    replacement,
                } => {
                    format!(
                        "Replace '{}' with '{}'",
                        truncate_str(pattern, 30),
                        truncate_str(replacement, 30)
                    )
                }
            })
            .collect();

        let entry = MutationLogEntry {
            request_id: request_id.to_string(),
            timestamp: ts,
            timestamp_human: Self::format_timestamp(ts),
            tool_name: "edit_file_lines".to_string(),
            evidence_ids: evidence_ids.to_vec(),
            mutation_type: MutationType::FileEdit,
            target: file_path.to_string_lossy().to_string(),
            details: MutationDetails::FileEdit {
                file_path: file_path.to_path_buf(),
                backup_path: backup_path.to_path_buf(),
                before_hash: before_hash.to_string(),
                after_hash: after_hash.to_string(),
                diff_summary: Self::generate_diff_summary(operations),
                operations: op_descriptions,
            },
            success,
            error: error.map(String::from),
        };

        self.write_log_entry(&entry)
    }

    /// Log a systemd operation
    pub fn log_systemd_operation(
        &self,
        request_id: &str,
        evidence_ids: &[String],
        mutation_type: MutationType,
        service: &str,
        operation: &str,
        prior_state: Option<ServiceState>,
        success: bool,
        error: Option<&str>,
    ) -> io::Result<()> {
        let ts = Self::timestamp();

        let entry = MutationLogEntry {
            request_id: request_id.to_string(),
            timestamp: ts,
            timestamp_human: Self::format_timestamp(ts),
            tool_name: format!("systemd_{}", operation),
            evidence_ids: evidence_ids.to_vec(),
            mutation_type,
            target: service.to_string(),
            details: MutationDetails::Systemd {
                service: service.to_string(),
                prior_state,
                operation: operation.to_string(),
            },
            success,
            error: error.map(String::from),
        };

        self.write_log_entry(&entry)
    }

    /// Log a package operation (v0.0.9)
    pub fn log_package_operation(
        &self,
        request_id: &str,
        evidence_ids: &[String],
        mutation_type: MutationType,
        package: &str,
        version: Option<&str>,
        reason: &str,
        success: bool,
        error: Option<&str>,
    ) -> io::Result<()> {
        let ts = Self::timestamp();

        let operation = match mutation_type {
            MutationType::PackageInstall => "install",
            MutationType::PackageRemove => "remove",
            _ => "unknown",
        };

        let entry = MutationLogEntry {
            request_id: request_id.to_string(),
            timestamp: ts,
            timestamp_human: Self::format_timestamp(ts),
            tool_name: format!("package_{}", operation),
            evidence_ids: evidence_ids.to_vec(),
            mutation_type,
            target: package.to_string(),
            details: MutationDetails::Package {
                package: package.to_string(),
                version: version.map(String::from),
                reason: reason.to_string(),
                operation: operation.to_string(),
            },
            success,
            error: error.map(String::from),
        };

        self.write_log_entry(&entry)
    }

    /// Write a log entry to the logs directory
    fn write_log_entry(&self, entry: &MutationLogEntry) -> io::Result<()> {
        self.ensure_dirs()?;

        let log_file = self
            .logs_dir()
            .join(format!("{}_{}.json", entry.timestamp, entry.request_id));

        let json = serde_json::to_string_pretty(entry)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        fs::write(&log_file, json)?;

        // Also append to the main log file
        let main_log = self.logs_dir().join("mutations.jsonl");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&main_log)?;

        let json_line =
            serde_json::to_string(entry).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        writeln!(file, "{}", json_line)?;

        Ok(())
    }

    /// Generate rollback info for a file edit
    pub fn file_rollback_info(&self, file_path: &Path, backup_path: &Path) -> RollbackInfo {
        RollbackInfo {
            backup_path: Some(backup_path.to_path_buf()),
            rollback_command: Some(format!(
                "cp '{}' '{}'",
                backup_path.display(),
                file_path.display()
            )),
            rollback_instructions: format!(
                "To restore the original file:\n\
                 1. Run: sudo cp '{}' '{}'\n\
                 2. Verify the file contents\n\
                 Backup created at: {}",
                backup_path.display(),
                file_path.display(),
                backup_path.display()
            ),
            prior_state: None,
        }
    }

    /// Generate rollback info for a systemd operation
    pub fn systemd_rollback_info(
        &self,
        operation: &str,
        service: &str,
        prior_state: Option<&ServiceState>,
    ) -> RollbackInfo {
        let (rollback_cmd, instructions) = match operation {
            "restart" => {
                let cmd = format!("systemctl restart {}", service);
                let instr = format!(
                    "The service was restarted. To restart again:\n\
                     Run: sudo systemctl restart {}\n\
                     Note: Service state before: {}",
                    service,
                    prior_state
                        .map(|s| s.to_string())
                        .unwrap_or("unknown".to_string())
                );
                (cmd, instr)
            }
            "reload" => {
                let cmd = format!("systemctl reload {}", service);
                let instr = format!(
                    "The service configuration was reloaded. To reload again:\n\
                     Run: sudo systemctl reload {}",
                    service
                );
                (cmd, instr)
            }
            "enable_now" => {
                let cmd = format!("systemctl disable --now {}", service);
                let instr = format!(
                    "The service was enabled and started. To revert:\n\
                     Run: sudo systemctl disable --now {}\n\
                     State before: {}",
                    service,
                    prior_state
                        .map(|s| s.to_string())
                        .unwrap_or("unknown".to_string())
                );
                (cmd, instr)
            }
            "disable_now" => {
                let cmd = format!("systemctl enable --now {}", service);
                let instr = format!(
                    "The service was disabled and stopped. To revert:\n\
                     Run: sudo systemctl enable --now {}\n\
                     State before: {}",
                    service,
                    prior_state
                        .map(|s| s.to_string())
                        .unwrap_or("unknown".to_string())
                );
                (cmd, instr)
            }
            "daemon_reload" => {
                let cmd = "systemctl daemon-reload".to_string();
                let instr =
                    "Daemon reload completed. This operation has no direct rollback.".to_string();
                (cmd, instr)
            }
            _ => {
                let cmd = format!("# Manual rollback needed for {}", operation);
                let instr = format!("Manual rollback required for {} operation", operation);
                (cmd, instr)
            }
        };

        RollbackInfo {
            backup_path: None,
            rollback_command: Some(rollback_cmd),
            rollback_instructions: instructions,
            prior_state: prior_state.map(|s| s.to_string()),
        }
    }

    /// Get recent mutation logs
    pub fn recent_logs(&self, limit: usize) -> io::Result<Vec<MutationLogEntry>> {
        let log_file = self.logs_dir().join("mutations.jsonl");
        if !log_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&log_file)?;
        let mut entries: Vec<MutationLogEntry> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        // Sort by timestamp descending
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Limit results
        entries.truncate(limit);

        Ok(entries)
    }
}

impl Default for RollbackManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate string for display
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_diff_summary_generation() {
        let ops = vec![
            FileEditOp::InsertLine {
                line_number: 0,
                content: "new line".to_string(),
            },
            FileEditOp::DeleteLine { line_number: 5 },
            FileEditOp::ReplaceLine {
                line_number: 10,
                content: "replaced".to_string(),
            },
        ];

        let summary = RollbackManager::generate_diff_summary(&ops);
        assert!(summary.contains("+1 line(s) inserted"));
        assert!(summary.contains("-1 line(s) deleted"));
        assert!(summary.contains("~1 line(s) replaced"));
    }

    #[test]
    fn test_backup_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.conf");
        fs::write(&test_file, "test content").unwrap();

        let rollback_dir = temp_dir.path().join("rollback");
        let manager = RollbackManager::with_base_dir(rollback_dir.clone());

        let backup_path = manager.backup_file(&test_file, "req123").unwrap();

        assert!(backup_path.exists());
        assert_eq!(fs::read_to_string(&backup_path).unwrap(), "test content");
    }

    #[test]
    fn test_file_hash() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let hash1 = RollbackManager::hash_file(&test_file).unwrap();

        // Same content should produce same hash
        let test_file2 = temp_dir.path().join("test2.txt");
        fs::write(&test_file2, "test content").unwrap();
        let hash2 = RollbackManager::hash_file(&test_file2).unwrap();
        assert_eq!(hash1, hash2);

        // Different content should produce different hash
        let test_file3 = temp_dir.path().join("test3.txt");
        fs::write(&test_file3, "different content").unwrap();
        let hash3 = RollbackManager::hash_file(&test_file3).unwrap();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_rollback_info_generation() {
        let manager = RollbackManager::new();

        // File rollback info
        let file_info = manager.file_rollback_info(
            Path::new("/etc/test.conf"),
            Path::new("/var/lib/anna/rollback/files/123/test.conf"),
        );
        assert!(file_info.rollback_command.is_some());
        assert!(file_info.backup_path.is_some());

        // Systemd rollback info
        let systemd_info = manager.systemd_rollback_info(
            "enable_now",
            "nginx.service",
            Some(&ServiceState {
                is_active: false,
                is_enabled: false,
            }),
        );
        assert!(systemd_info
            .rollback_command
            .unwrap()
            .contains("disable --now"));
    }

    #[test]
    fn test_log_writing() {
        let temp_dir = TempDir::new().unwrap();
        let rollback_dir = temp_dir.path().join("rollback");
        let manager = RollbackManager::with_base_dir(rollback_dir.clone());

        manager
            .log_systemd_operation(
                "req123",
                &["E1".to_string()],
                MutationType::SystemdRestart,
                "nginx.service",
                "restart",
                Some(ServiceState {
                    is_active: true,
                    is_enabled: true,
                }),
                true,
                None,
            )
            .unwrap();

        let logs = manager.recent_logs(10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].request_id, "req123");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short");
        assert_eq!(
            truncate_str("this is a very long string", 10),
            "this is a ..."
        );
    }
}
