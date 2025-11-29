//! Permissions Health Check Module v2.1.0
//!
//! Checks and auto-fixes filesystem permissions for Anna's data directories.
//! This ensures XP tracking, telemetry, and knowledge storage work correctly.
//!
//! ## Design Philosophy
//!
//! Anna uses several directories that need to be writable:
//! - `/var/lib/anna/xp/` - XP store and tracking
//! - `/var/lib/anna/knowledge/` - Learned facts and patterns
//! - `/var/lib/anna/llm/` - LLM autoprovision data
//! - `/var/log/anna/` - Telemetry and logs
//!
//! The daemon (annad) runs as root, but annactl runs as the current user.
//! Both need write access to persist data.
//!
//! This module:
//! 1. Checks if directories exist and are writable
//! 2. Creates missing directories
//! 3. Attempts to fix permissions if possible (when running as root)
//! 4. Reports issues clearly for manual intervention

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

// ============================================================================
// Configuration
// ============================================================================

/// Directories that Anna needs for operation
pub const ANNA_DATA_DIR: &str = "/var/lib/anna";
pub const ANNA_XP_DIR: &str = "/var/lib/anna/xp";
pub const ANNA_KNOWLEDGE_DIR: &str = "/var/lib/anna/knowledge";
pub const ANNA_STATS_DIR: &str = "/var/lib/anna/knowledge/stats";
pub const ANNA_LLM_DIR: &str = "/var/lib/anna/llm";
pub const ANNA_LOG_DIR: &str = "/var/log/anna";
pub const ANNA_TELEMETRY_FILE: &str = "/var/log/anna/telemetry.jsonl";

/// All required directories
pub const REQUIRED_DIRS: &[&str] = &[
    ANNA_DATA_DIR,
    ANNA_XP_DIR,
    ANNA_KNOWLEDGE_DIR,
    ANNA_STATS_DIR,
    ANNA_LLM_DIR,
    ANNA_LOG_DIR,
];

// ============================================================================
// Permission Check Result
// ============================================================================

/// Result of a permission check
#[derive(Debug, Clone)]
pub struct PermissionCheck {
    pub path: PathBuf,
    pub exists: bool,
    pub readable: bool,
    pub writable: bool,
    pub issue: Option<String>,
}

impl PermissionCheck {
    /// Check permissions for a path
    pub fn check(path: &Path) -> Self {
        let exists = path.exists();
        let readable = path.is_dir() && fs::read_dir(path).is_ok();
        let writable = if exists {
            // Try to check write permission
            let test_file = path.join(".anna_write_test");
            let can_write = fs::write(&test_file, "test").is_ok();
            if can_write {
                let _ = fs::remove_file(&test_file);
            }
            can_write
        } else {
            false
        };

        let issue = if !exists {
            Some(format!("Directory does not exist: {}", path.display()))
        } else if !readable {
            Some(format!("Directory not readable: {}", path.display()))
        } else if !writable {
            Some(format!("Directory not writable: {}", path.display()))
        } else {
            None
        };

        Self {
            path: path.to_path_buf(),
            exists,
            readable,
            writable,
            issue,
        }
    }

    /// Is this path OK?
    pub fn is_ok(&self) -> bool {
        self.exists && self.readable && self.writable
    }
}

// ============================================================================
// Health Check Result
// ============================================================================

/// Result of a full permissions health check
#[derive(Debug, Clone)]
pub struct PermissionsHealthCheck {
    pub checks: Vec<PermissionCheck>,
    pub all_ok: bool,
    pub issues_count: usize,
    pub fixed_count: usize,
    pub summary: String,
}

impl PermissionsHealthCheck {
    /// Run health check on all required directories
    pub fn run() -> Self {
        let checks: Vec<PermissionCheck> = REQUIRED_DIRS
            .iter()
            .map(|p| PermissionCheck::check(Path::new(p)))
            .collect();

        let issues_count = checks.iter().filter(|c| !c.is_ok()).count();
        let all_ok = issues_count == 0;

        let summary = if all_ok {
            "All permissions OK".to_string()
        } else {
            format!("{} permission issues found", issues_count)
        };

        Self {
            checks,
            all_ok,
            issues_count,
            fixed_count: 0,
            summary,
        }
    }

    /// Get list of issues
    pub fn issues(&self) -> Vec<&str> {
        self.checks
            .iter()
            .filter_map(|c| c.issue.as_deref())
            .collect()
    }

    /// Format for display
    pub fn format_status(&self) -> String {
        let mut lines = Vec::new();

        if self.all_ok {
            lines.push("  ✅  All directories OK".to_string());
        } else {
            lines.push(format!("  ⚠️   {} issues found:", self.issues_count));
            for check in &self.checks {
                if let Some(issue) = &check.issue {
                    lines.push(format!("      - {}", issue));
                }
            }
            if self.fixed_count > 0 {
                lines.push(format!("  ✅  {} issues auto-fixed", self.fixed_count));
            }
        }

        lines.join("\n")
    }
}

// ============================================================================
// Auto-Fix Functions
// ============================================================================

/// Result of an auto-fix attempt
#[derive(Debug, Clone)]
pub struct AutoFixResult {
    pub path: PathBuf,
    pub action: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Attempt to fix permissions issues
/// This should be called at daemon startup when running as root
pub fn auto_fix_permissions() -> Vec<AutoFixResult> {
    let mut results = Vec::new();

    for dir_path in REQUIRED_DIRS {
        let path = Path::new(dir_path);

        // Create directory if missing
        if !path.exists() {
            match fs::create_dir_all(path) {
                Ok(_) => {
                    results.push(AutoFixResult {
                        path: path.to_path_buf(),
                        action: "Created directory".to_string(),
                        success: true,
                        error: None,
                    });
                    // Set permissions
                    let _ = set_world_writable(path);
                }
                Err(e) => {
                    results.push(AutoFixResult {
                        path: path.to_path_buf(),
                        action: "Create directory".to_string(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
            continue;
        }

        // Check if writable
        let test_file = path.join(".anna_write_test");
        let is_writable = fs::write(&test_file, "test").is_ok();
        if is_writable {
            let _ = fs::remove_file(&test_file);
            continue; // Already OK
        }

        // Try to fix permissions (requires root)
        if set_world_writable(path) {
            results.push(AutoFixResult {
                path: path.to_path_buf(),
                action: "Fixed permissions (chmod 777)".to_string(),
                success: true,
                error: None,
            });
        } else {
            results.push(AutoFixResult {
                path: path.to_path_buf(),
                action: "Fix permissions".to_string(),
                success: false,
                error: Some("Failed to chmod - may need manual intervention".to_string()),
            });
        }
    }

    // Also ensure telemetry file is writable
    let telemetry_path = Path::new(ANNA_TELEMETRY_FILE);
    if !telemetry_path.exists() {
        if let Some(parent) = telemetry_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::write(telemetry_path, "").is_ok() {
            let _ = set_world_writable(telemetry_path);
            results.push(AutoFixResult {
                path: telemetry_path.to_path_buf(),
                action: "Created telemetry file".to_string(),
                success: true,
                error: None,
            });
        }
    } else if !is_file_writable(telemetry_path) {
        if set_world_writable(telemetry_path) {
            results.push(AutoFixResult {
                path: telemetry_path.to_path_buf(),
                action: "Fixed telemetry file permissions".to_string(),
                success: true,
                error: None,
            });
        }
    }

    results
}

/// Set a path to be world-writable (chmod 777 for dirs, 666 for files)
fn set_world_writable(path: &Path) -> bool {
    let mode = if path.is_dir() { 0o777 } else { 0o666 };

    fs::set_permissions(path, fs::Permissions::from_mode(mode)).is_ok()
}

/// Check if a file is writable
fn is_file_writable(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    // Try to open for append
    fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .is_ok()
}

// ============================================================================
// Ensure Write Access (for use before write operations)
// ============================================================================

/// Ensure a directory exists and is writable
/// Creates it if missing, returns error if not writable
pub fn ensure_writable_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory {}: {}", path.display(), e))?;

        // Try to make it world-writable
        let _ = set_world_writable(path);
    }

    // Verify writable
    let test_file = path.join(".anna_write_test");
    fs::write(&test_file, "test")
        .map_err(|e| format!("Directory {} not writable: {}", path.display(), e))?;
    let _ = fs::remove_file(&test_file);

    Ok(())
}

/// Ensure a file's parent directory exists and is writable
pub fn ensure_writable_file_path(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        ensure_writable_dir(parent)?;
    }
    Ok(())
}

// ============================================================================
// Atomic Write (for safe persistence)
// ============================================================================

/// Write data to a file atomically
/// Writes to a temp file first, then renames
pub fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    // Ensure parent directory exists
    ensure_writable_file_path(path)?;

    // Write to temp file
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Rename to target (atomic on most filesystems)
    fs::rename(&temp_path, path)
        .map_err(|e| format!("Failed to rename {} to {}: {}", temp_path.display(), path.display(), e))?;

    Ok(())
}

/// Append data to a file, creating it if it doesn't exist
pub fn safe_append(path: &Path, content: &str) -> Result<(), String> {
    // Ensure parent directory exists
    ensure_writable_file_path(path)?;

    // Open for append, create if missing
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Failed to open {} for append: {}", path.display(), e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to append to {}: {}", path.display(), e))?;

    file.write_all(b"\n")
        .map_err(|e| format!("Failed to write newline to {}: {}", path.display(), e))?;

    Ok(())
}

// ============================================================================
// CLI Helper for Manual Fix
// ============================================================================

/// Generate a shell command to fix permissions manually
pub fn fix_command() -> String {
    let dirs = REQUIRED_DIRS.join(" ");
    format!(
        r#"# Run these commands as root to fix Anna permissions:
sudo mkdir -p {}
sudo chmod -R 777 /var/lib/anna /var/log/anna
sudo chown -R $USER:$USER /var/lib/anna /var/log/anna"#,
        dirs
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_permission_check_existing_writable() {
        let temp = TempDir::new().unwrap();
        let check = PermissionCheck::check(temp.path());

        assert!(check.exists);
        assert!(check.readable);
        assert!(check.writable);
        assert!(check.is_ok());
        assert!(check.issue.is_none());
    }

    #[test]
    fn test_permission_check_non_existing() {
        let path = Path::new("/tmp/anna_test_nonexistent_12345");
        let check = PermissionCheck::check(path);

        assert!(!check.exists);
        assert!(!check.is_ok());
        assert!(check.issue.is_some());
    }

    #[test]
    fn test_ensure_writable_dir() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("test_subdir");

        // Should create and make writable
        let result = ensure_writable_dir(&subdir);
        assert!(result.is_ok());
        assert!(subdir.exists());
    }

    #[test]
    fn test_atomic_write() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test_file.json");

        let content = r#"{"test": "data"}"#;
        let result = atomic_write(&file_path, content);
        assert!(result.is_ok());

        let read_back = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_back, content);
    }

    #[test]
    fn test_safe_append() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test_append.jsonl");

        // First append creates file
        safe_append(&file_path, "line1").unwrap();
        safe_append(&file_path, "line2").unwrap();
        safe_append(&file_path, "line3").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("line1"));
        assert!(content.contains("line2"));
        assert!(content.contains("line3"));

        // Should have 3 lines
        assert_eq!(content.lines().count(), 3);
    }

    #[test]
    fn test_health_check() {
        // This will check real system paths
        let health = PermissionsHealthCheck::run();

        // Just ensure it doesn't panic
        assert!(!health.summary.is_empty());
    }

    #[test]
    fn test_fix_command_generation() {
        let cmd = fix_command();

        assert!(cmd.contains("sudo"));
        assert!(cmd.contains("/var/lib/anna"));
        assert!(cmd.contains("/var/log/anna"));
    }
}
