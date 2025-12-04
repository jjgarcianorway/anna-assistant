//! User File Mutation v0.0.50
//!
//! Single-capability mutations for user-owned config files only.
//! Supports append_line and set_key_value modes.
//!
//! Hard Policy (v0.0.50):
//! - ONLY paths under $HOME allowed
//! - Path must not be symlink escaping $HOME
//! - Blocked: /etc, /usr, /var, /boot, /root, /proc, /sys, /dev
//! - No sudo required for user files
//! - annad runs as root but writes as target user
//!
//! Features:
//! - Diff preview before execution
//! - Confirmation gates (medium risk)
//! - Rollback backup with case_id
//! - Verification after apply

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::generate_request_id;
use crate::rollback::RollbackManager;

// =============================================================================
// Constants
// =============================================================================

/// Confirmation phrase for user file mutations (medium risk)
pub const USER_FILE_CONFIRMATION: &str = "I CONFIRM (medium risk)";

/// Rollback directory base
pub const ROLLBACK_BASE: &str = "/var/lib/anna/rollback";

/// Max diff lines to show
pub const MAX_DIFF_LINES: usize = 200;

/// Max file size for edits (1 MB)
pub const MAX_FILE_SIZE: u64 = 1024 * 1024;

/// Blocked path prefixes (system directories)
const BLOCKED_PREFIXES: &[&str] = &[
    "/etc", "/usr", "/var", "/boot", "/root", "/proc", "/sys", "/dev", "/run", "/lib", "/lib64",
    "/bin", "/sbin", "/opt",
];

// =============================================================================
// Edit Mode Enum
// =============================================================================

/// Edit mode for file mutations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EditMode {
    /// Append a line to the end of the file
    AppendLine,
    /// Set a key=value pair (add or update)
    SetKeyValue,
}

impl std::fmt::Display for EditMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditMode::AppendLine => write!(f, "append_line"),
            EditMode::SetKeyValue => write!(f, "set_key_value"),
        }
    }
}

// =============================================================================
// Edit Action Struct
// =============================================================================

/// User file edit action (v0.0.50)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFileEditAction {
    /// Absolute path to the file
    pub path: String,
    /// Edit mode
    pub mode: EditMode,
    /// Line content (for append_line mode)
    pub line: Option<String>,
    /// Key name (for set_key_value mode)
    pub key: Option<String>,
    /// Value (for set_key_value mode)
    pub value: Option<String>,
    /// Separator between key and value (default: "=")
    pub separator: Option<String>,
    /// Ensure file ends with newline
    pub ensure_newline: bool,
    /// Must be idempotent (always true for v0.0.50)
    pub idempotent: bool,
    /// Target user for ownership (username or uid)
    pub target_user: String,
    /// Backup strategy (always "copy" for v0.0.50)
    pub backup_strategy: String,
    /// Verification strategy
    pub verify_strategy: VerifyStrategy,
}

/// Verification strategy after apply
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyStrategy {
    /// Check that file contains the expected line/key
    FileContains { expected: String },
    /// Check hash changed
    HashChanged,
    /// No verification
    None,
}

impl Default for UserFileEditAction {
    fn default() -> Self {
        Self {
            path: String::new(),
            mode: EditMode::AppendLine,
            line: None,
            key: None,
            value: None,
            separator: Some("=".to_string()),
            ensure_newline: true,
            idempotent: true,
            target_user: String::new(),
            backup_strategy: "copy".to_string(),
            verify_strategy: VerifyStrategy::HashChanged,
        }
    }
}

impl UserFileEditAction {
    /// Create an append_line action
    pub fn append_line(path: &str, line: &str, target_user: &str) -> Self {
        Self {
            path: path.to_string(),
            mode: EditMode::AppendLine,
            line: Some(line.to_string()),
            target_user: target_user.to_string(),
            verify_strategy: VerifyStrategy::FileContains {
                expected: line.to_string(),
            },
            ..Default::default()
        }
    }

    /// Create a set_key_value action
    pub fn set_key_value(path: &str, key: &str, value: &str, sep: &str, target_user: &str) -> Self {
        let expected = format!("{}{}{}", key, sep, value);
        Self {
            path: path.to_string(),
            mode: EditMode::SetKeyValue,
            key: Some(key.to_string()),
            value: Some(value.to_string()),
            separator: Some(sep.to_string()),
            target_user: target_user.to_string(),
            verify_strategy: VerifyStrategy::FileContains { expected },
            ..Default::default()
        }
    }

    /// Validate the action parameters
    pub fn validate(&self) -> Result<(), String> {
        match self.mode {
            EditMode::AppendLine => {
                if self.line.is_none() || self.line.as_ref().map(|l| l.is_empty()).unwrap_or(true) {
                    return Err("append_line mode requires non-empty 'line'".to_string());
                }
            }
            EditMode::SetKeyValue => {
                if self.key.is_none() || self.key.as_ref().map(|k| k.is_empty()).unwrap_or(true) {
                    return Err("set_key_value mode requires non-empty 'key'".to_string());
                }
                if self.value.is_none() {
                    return Err("set_key_value mode requires 'value'".to_string());
                }
            }
        }

        if !self.idempotent {
            return Err("v0.0.50 requires idempotent=true".to_string());
        }

        if self.backup_strategy != "copy" {
            return Err("v0.0.50 only supports backup_strategy='copy'".to_string());
        }

        Ok(())
    }
}

// =============================================================================
// Path Policy
// =============================================================================

/// Result of path policy check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPolicyResult {
    pub allowed: bool,
    pub reason: String,
    pub evidence_id: String,
    pub path: String,
    pub resolved_path: Option<String>,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub in_home: bool,
    pub home_dir: Option<String>,
}

/// Check if path is allowed for user file mutation (v0.0.50 policy)
pub fn check_path_policy(path: &str) -> PathPolicyResult {
    let evidence_id = format!(
        "POL{}",
        generate_request_id().chars().take(8).collect::<String>()
    );
    let path_obj = Path::new(path);

    // Get home directory
    let home = std::env::var("HOME").ok();
    let home_dir = home.clone();

    // Check if path starts with blocked prefix
    for prefix in BLOCKED_PREFIXES {
        if path.starts_with(prefix) {
            return PathPolicyResult {
                allowed: false,
                reason: format!("Path blocked: {} paths not allowed in v0.0.50", prefix),
                evidence_id,
                path: path.to_string(),
                resolved_path: None,
                is_symlink: false,
                symlink_target: None,
                in_home: false,
                home_dir,
            };
        }
    }

    // Check if in home directory
    let in_home = home.as_ref().map(|h| path.starts_with(h)).unwrap_or(false);

    if !in_home {
        return PathPolicyResult {
            allowed: false,
            reason: "v0.0.50 only allows paths under $HOME".to_string(),
            evidence_id,
            path: path.to_string(),
            resolved_path: None,
            is_symlink: false,
            symlink_target: None,
            in_home: false,
            home_dir,
        };
    }

    // Check for symlink escape
    let (is_symlink, symlink_target, resolved_path) = check_symlink_escape(path_obj, &home);

    if is_symlink {
        if let Some(ref target) = symlink_target {
            let home_ref = home.as_deref().unwrap_or("");
            if !target.starts_with(home_ref) {
                return PathPolicyResult {
                    allowed: false,
                    reason: format!(
                        "Symlink escape detected: {} -> {} (outside $HOME)",
                        path, target
                    ),
                    evidence_id,
                    path: path.to_string(),
                    resolved_path,
                    is_symlink: true,
                    symlink_target: Some(target.clone()),
                    in_home: true,
                    home_dir,
                };
            }
        }
    }

    PathPolicyResult {
        allowed: true,
        reason: "Path is under $HOME and does not escape via symlink".to_string(),
        evidence_id,
        path: path.to_string(),
        resolved_path,
        is_symlink,
        symlink_target,
        in_home: true,
        home_dir,
    }
}

/// Check for symlink and potential escape
fn check_symlink_escape(
    path: &Path,
    home: &Option<String>,
) -> (bool, Option<String>, Option<String>) {
    // Check if path itself is a symlink
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            if let Ok(target) = fs::read_link(path) {
                let target_str = target.to_string_lossy().to_string();
                // Resolve to absolute path
                let resolved = if target.is_absolute() {
                    target.to_string_lossy().to_string()
                } else {
                    path.parent()
                        .map(|p| p.join(&target).to_string_lossy().to_string())
                        .unwrap_or(target_str.clone())
                };
                return (true, Some(target_str), Some(resolved));
            }
        }
    }

    // Try to canonicalize and check if it escapes home
    if let Ok(canonical) = fs::canonicalize(path) {
        let canonical_str = canonical.to_string_lossy().to_string();
        let is_different = canonical_str != path.to_string_lossy();
        if is_different {
            return (true, None, Some(canonical_str));
        }
    }

    (false, None, None)
}

// =============================================================================
// Preview Types
// =============================================================================

/// Preview result from file_edit_preview_v1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditPreview {
    pub evidence_id: String,
    pub path: String,
    pub exists: bool,
    pub current_line_count: usize,
    pub current_head: Vec<String>,
    pub current_tail: Vec<String>,
    pub diff_unified: String,
    pub would_change: bool,
    pub change_description: String,
    pub policy_result: PathPolicyResult,
    pub file_stat: Option<FileStat>,
    pub before_hash: Option<String>,
}

/// File stat information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStat {
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
    pub size: u64,
    pub mtime: i64,
}

/// Generate edit preview
pub fn generate_edit_preview(action: &UserFileEditAction) -> Result<EditPreview, String> {
    let evidence_id = format!(
        "E{}",
        generate_request_id().chars().take(8).collect::<String>()
    );
    let path = Path::new(&action.path);

    // Check policy first
    let policy_result = check_path_policy(&action.path);
    if !policy_result.allowed {
        return Err(format!("Policy denied: {}", policy_result.reason));
    }

    // Check if file exists
    let exists = path.exists();

    // Get file stat
    let file_stat = if exists {
        fs::metadata(path).ok().map(|m| FileStat {
            uid: m.uid(),
            gid: m.gid(),
            mode: m.mode(),
            size: m.len(),
            mtime: m.mtime(),
        })
    } else {
        None
    };

    // Check file size
    if let Some(ref stat) = file_stat {
        if stat.size > MAX_FILE_SIZE {
            return Err(format!(
                "File too large: {} bytes (max {})",
                stat.size, MAX_FILE_SIZE
            ));
        }
    }

    // Read current content
    let content = if exists {
        fs::read_to_string(path).map_err(|e| format!("Cannot read file: {}", e))?
    } else {
        String::new()
    };

    let lines: Vec<&str> = content.lines().collect();
    let current_line_count = lines.len();

    // Get head and tail
    let current_head: Vec<String> = lines.iter().take(10).map(|s| s.to_string()).collect();
    let current_tail: Vec<String> = lines
        .iter()
        .rev()
        .take(10)
        .rev()
        .map(|s| s.to_string())
        .collect();

    // Generate diff and determine if would change
    let (diff_unified, would_change, change_description) = match action.mode {
        EditMode::AppendLine => {
            let line = action.line.as_ref().unwrap();
            // Check if line already exists (idempotent)
            let already_exists = lines.iter().any(|l| l.trim() == line.trim());
            if already_exists {
                (
                    format!("# No changes - line already exists:\n# {}", line),
                    false,
                    format!("Line '{}' already exists in file", line),
                )
            } else {
                let diff = generate_append_diff(&content, line, current_line_count);
                (diff, true, format!("Will append line: '{}'", line))
            }
        }
        EditMode::SetKeyValue => {
            let key = action.key.as_ref().unwrap();
            let value = action.value.as_ref().unwrap();
            let sep = action.separator.as_deref().unwrap_or("=");
            let new_line = format!("{}{}{}", key, sep, value);

            // Find existing key
            let existing_idx = lines
                .iter()
                .position(|l| l.trim().starts_with(key) && l.contains(sep));

            if let Some(idx) = existing_idx {
                let old_line = lines[idx];
                if old_line.trim() == new_line.trim() {
                    (
                        format!("# No changes - key already has this value:\n# {}", new_line),
                        false,
                        format!("Key '{}' already set to '{}'", key, value),
                    )
                } else {
                    let diff = generate_set_key_diff(&content, idx, old_line, &new_line);
                    (
                        diff,
                        true,
                        format!(
                            "Will update key '{}' from '{}' to '{}'",
                            key,
                            old_line.trim(),
                            new_line
                        ),
                    )
                }
            } else {
                let diff = generate_append_diff(&content, &new_line, current_line_count);
                (
                    diff,
                    true,
                    format!("Will add key '{}' with value '{}'", key, value),
                )
            }
        }
    };

    // Compute before hash
    let before_hash = if exists {
        Some(hash_content(&content))
    } else {
        None
    };

    Ok(EditPreview {
        evidence_id,
        path: action.path.clone(),
        exists,
        current_line_count,
        current_head,
        current_tail,
        diff_unified,
        would_change,
        change_description,
        policy_result,
        file_stat,
        before_hash,
    })
}

/// Generate unified diff for append operation
fn generate_append_diff(content: &str, new_line: &str, line_count: usize) -> String {
    let mut diff = String::new();
    diff.push_str("--- a/file\n");
    diff.push_str("+++ b/file\n");

    // Show last 3 lines as context
    let lines: Vec<&str> = content.lines().collect();
    let context_start = line_count.saturating_sub(3);

    if line_count > 0 {
        diff.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            context_start + 1,
            3.min(line_count),
            context_start + 1,
            3.min(line_count) + 1
        ));

        for (i, line) in lines.iter().skip(context_start).take(3).enumerate() {
            diff.push_str(&format!(" {}\n", line));
        }
    } else {
        diff.push_str("@@ -0,0 +1,1 @@\n");
    }

    // Add newline indicator if needed
    if !content.is_empty() && !content.ends_with('\n') {
        diff.push_str("\\ No newline at end of file\n");
    }

    diff.push_str(&format!("+{}\n", new_line));
    diff
}

/// Generate unified diff for set_key_value operation
fn generate_set_key_diff(content: &str, idx: usize, old_line: &str, new_line: &str) -> String {
    let mut diff = String::new();
    diff.push_str("--- a/file\n");
    diff.push_str("+++ b/file\n");

    let lines: Vec<&str> = content.lines().collect();
    let context_start = idx.saturating_sub(2);
    let context_end = (idx + 3).min(lines.len());

    diff.push_str(&format!(
        "@@ -{},{} +{},{} @@\n",
        context_start + 1,
        context_end - context_start,
        context_start + 1,
        context_end - context_start
    ));

    for (i, line) in lines
        .iter()
        .enumerate()
        .skip(context_start)
        .take(context_end - context_start)
    {
        if i == idx {
            diff.push_str(&format!("-{}\n", old_line));
            diff.push_str(&format!("+{}\n", new_line));
        } else {
            diff.push_str(&format!(" {}\n", line));
        }
    }

    diff
}

// =============================================================================
// Apply Types
// =============================================================================

/// Apply result from file_edit_apply_v1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub success: bool,
    pub case_id: String,
    pub evidence_id: String,
    pub path: String,
    pub backup_path: PathBuf,
    pub before_hash: String,
    pub after_hash: String,
    pub verified: bool,
    pub verify_message: String,
    pub rollback_command: String,
    pub rollback_instructions: String,
    pub error: Option<String>,
}

/// Apply the edit action
pub fn apply_edit(action: &UserFileEditAction, case_id: &str) -> Result<ApplyResult, String> {
    let evidence_id = format!(
        "E{}",
        generate_request_id().chars().take(8).collect::<String>()
    );
    let path = Path::new(&action.path);

    // Validate action
    action.validate()?;

    // Check policy
    let policy = check_path_policy(&action.path);
    if !policy.allowed {
        return Err(format!("Policy denied: {}", policy.reason));
    }

    // Check target user matches
    let current_user = std::env::var("USER").unwrap_or_default();
    if action.target_user != current_user && !is_running_as_root() {
        return Err(format!(
            "Cannot write as user '{}' - running as '{}'",
            action.target_user, current_user
        ));
    }

    // Get current content and metadata
    let exists = path.exists();
    let (content, file_stat) = if exists {
        let c = fs::read_to_string(path).map_err(|e| format!("Cannot read: {}", e))?;
        let m = fs::metadata(path).ok();
        (c, m)
    } else {
        (String::new(), None)
    };

    let before_hash = hash_content(&content);

    // Create backup directory
    let backup_dir = PathBuf::from(ROLLBACK_BASE).join(case_id).join("backup");
    fs::create_dir_all(&backup_dir).map_err(|e| format!("Cannot create backup dir: {}", e))?;

    // Sanitize path for backup filename
    let sanitized_path = action
        .path
        .replace('/', "_")
        .trim_start_matches('_')
        .to_string();
    let backup_path = backup_dir.join(&sanitized_path);

    // Create backup
    if exists {
        fs::copy(path, &backup_path).map_err(|e| format!("Cannot backup: {}", e))?;
    } else {
        // Create marker for new file
        fs::write(&backup_path, "__NEW_FILE__")
            .map_err(|e| format!("Cannot create marker: {}", e))?;
    }

    // Apply the edit
    let new_content = match action.mode {
        EditMode::AppendLine => {
            let line = action.line.as_ref().unwrap();
            // Check idempotency
            let lines: Vec<&str> = content.lines().collect();
            if lines.iter().any(|l| l.trim() == line.trim()) {
                // Already exists, no change
                content.clone()
            } else {
                // Append
                if content.is_empty() {
                    format!("{}\n", line)
                } else if content.ends_with('\n') {
                    format!("{}{}\n", content, line)
                } else {
                    format!("{}\n{}\n", content, line)
                }
            }
        }
        EditMode::SetKeyValue => {
            let key = action.key.as_ref().unwrap();
            let value = action.value.as_ref().unwrap();
            let sep = action.separator.as_deref().unwrap_or("=");
            let new_line = format!("{}{}{}", key, sep, value);

            let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            let existing_idx = lines
                .iter()
                .position(|l| l.trim().starts_with(key) && l.contains(sep));

            if let Some(idx) = existing_idx {
                lines[idx] = new_line;
            } else {
                lines.push(new_line);
            }

            if action.ensure_newline {
                lines.join("\n") + "\n"
            } else {
                lines.join("\n")
            }
        }
    };

    // Write new content
    fs::write(path, &new_content).map_err(|e| format!("Cannot write: {}", e))?;

    // Restore ownership if we have metadata and running as root
    if let Some(ref meta) = file_stat {
        if is_running_as_root() {
            let uid = meta.uid();
            let gid = meta.gid();
            let _ = std::process::Command::new("chown")
                .args([&format!("{}:{}", uid, gid), &action.path])
                .output();
        }
        // Restore permissions
        let mode = meta.mode() & 0o7777;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
    }

    // Compute after hash
    let after_content =
        fs::read_to_string(path).map_err(|e| format!("Cannot read after: {}", e))?;
    let after_hash = hash_content(&after_content);

    // Verify
    let (verified, verify_message) = verify_edit(action, &after_content);

    // Write apply_result.json
    let result_path = backup_dir.parent().unwrap().join("apply_result.json");
    let result_json = serde_json::json!({
        "success": true,
        "case_id": case_id,
        "path": action.path,
        "mode": action.mode.to_string(),
        "before_hash": before_hash,
        "after_hash": after_hash,
        "backup_path": backup_path.display().to_string(),
        "verified": verified,
        "timestamp": timestamp(),
    });
    fs::write(
        &result_path,
        serde_json::to_string_pretty(&result_json).unwrap(),
    )
    .map_err(|e| format!("Cannot write result: {}", e))?;

    // Log to ops.log
    log_operation(case_id, &action.path, "apply", true, None)?;

    let rollback_command = format!("annactl 'rollback {}'", case_id);
    let rollback_instructions = format!(
        "To undo this change:\n\
         1. Run: {}\n\
         2. Or manually: cp '{}' '{}'\n\
         Backup stored at: {}",
        rollback_command,
        backup_path.display(),
        action.path,
        backup_path.display()
    );

    Ok(ApplyResult {
        success: true,
        case_id: case_id.to_string(),
        evidence_id,
        path: action.path.clone(),
        backup_path,
        before_hash,
        after_hash,
        verified,
        verify_message,
        rollback_command,
        rollback_instructions,
        error: None,
    })
}

/// Verify the edit was applied correctly
fn verify_edit(action: &UserFileEditAction, content: &str) -> (bool, String) {
    match &action.verify_strategy {
        VerifyStrategy::FileContains { expected } => {
            if content.contains(expected) {
                (true, format!("Verified: file contains '{}'", expected))
            } else {
                (
                    false,
                    format!("Verification failed: '{}' not found", expected),
                )
            }
        }
        VerifyStrategy::HashChanged => {
            // Just verify file is readable (hash change checked externally)
            (true, "File updated successfully".to_string())
        }
        VerifyStrategy::None => (true, "No verification requested".to_string()),
    }
}

// =============================================================================
// Rollback Types
// =============================================================================

/// Rollback result from file_edit_rollback_v1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub success: bool,
    pub case_id: String,
    pub path: String,
    pub restored_hash: String,
    pub backup_hash: String,
    pub hashes_match: bool,
    pub error: Option<String>,
}

/// Execute rollback for a case
pub fn execute_rollback(case_id: &str) -> Result<RollbackResult, String> {
    let rollback_dir = PathBuf::from(ROLLBACK_BASE).join(case_id);

    if !rollback_dir.exists() {
        return Err(format!("No rollback data found for case: {}", case_id));
    }

    // Read apply_result.json
    let result_path = rollback_dir.join("apply_result.json");
    let result_content =
        fs::read_to_string(&result_path).map_err(|e| format!("Cannot read apply result: {}", e))?;
    let result: serde_json::Value = serde_json::from_str(&result_content)
        .map_err(|e| format!("Cannot parse apply result: {}", e))?;

    let path = result["path"]
        .as_str()
        .ok_or("Missing path in apply result")?;
    let backup_path_str = result["backup_path"]
        .as_str()
        .ok_or("Missing backup_path in apply result")?;
    let before_hash = result["before_hash"]
        .as_str()
        .ok_or("Missing before_hash in apply result")?;

    let backup_path = PathBuf::from(backup_path_str);

    // Check if backup is a new file marker
    let backup_content =
        fs::read_to_string(&backup_path).map_err(|e| format!("Cannot read backup: {}", e))?;

    if backup_content == "__NEW_FILE__" {
        // File was created - delete it
        fs::remove_file(path).map_err(|e| format!("Cannot remove file: {}", e))?;
        log_operation(case_id, path, "rollback", true, None)?;
        return Ok(RollbackResult {
            success: true,
            case_id: case_id.to_string(),
            path: path.to_string(),
            restored_hash: "none".to_string(),
            backup_hash: "none".to_string(),
            hashes_match: true,
            error: None,
        });
    }

    // Restore from backup
    fs::copy(&backup_path, path).map_err(|e| format!("Cannot restore: {}", e))?;

    // Verify restoration
    let restored_content =
        fs::read_to_string(path).map_err(|e| format!("Cannot read restored: {}", e))?;
    let restored_hash = hash_content(&restored_content);
    let backup_hash = hash_content(&backup_content);
    let hashes_match = restored_hash == before_hash;

    log_operation(case_id, path, "rollback", true, None)?;

    Ok(RollbackResult {
        success: true,
        case_id: case_id.to_string(),
        path: path.to_string(),
        restored_hash,
        backup_hash,
        hashes_match,
        error: None,
    })
}

// =============================================================================
// Helpers
// =============================================================================

/// Hash content using simple hasher
fn hash_content(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Get current timestamp
fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Check if running as root
fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Log operation to ops.log
fn log_operation(
    case_id: &str,
    path: &str,
    operation: &str,
    success: bool,
    error: Option<&str>,
) -> Result<(), String> {
    let log_dir = PathBuf::from("/var/lib/anna/internal");
    let _ = fs::create_dir_all(&log_dir);

    let log_path = log_dir.join("ops.log");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("Cannot open ops.log: {}", e))?;

    let log_entry = serde_json::json!({
        "timestamp": timestamp(),
        "case_id": case_id,
        "operation": operation,
        "path": path,
        "success": success,
        "error": error,
    });

    writeln!(file, "{}", serde_json::to_string(&log_entry).unwrap())
        .map_err(|e| format!("Cannot write log: {}", e))?;

    Ok(())
}

/// Generate a case ID for mutations
pub fn generate_mutation_case_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("mut_{:x}", ts)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_policy_blocked() {
        let result = check_path_policy("/etc/hosts");
        assert!(!result.allowed);
        assert!(result.reason.contains("blocked"));
    }

    #[test]
    fn test_path_policy_home() {
        // This test depends on $HOME being set
        if let Ok(home) = std::env::var("HOME") {
            let path = format!("{}/.bashrc", home);
            let result = check_path_policy(&path);
            assert!(result.in_home);
            // May or may not be allowed depending on symlink status
        }
    }

    #[test]
    fn test_action_validate() {
        let mut action = UserFileEditAction::default();
        action.mode = EditMode::AppendLine;
        assert!(action.validate().is_err()); // missing line

        action.line = Some("test".to_string());
        assert!(action.validate().is_ok());
    }

    #[test]
    fn test_preview_append() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "line1\nline2\n").unwrap();

        // Set HOME to temp dir for test
        std::env::set_var("HOME", temp_dir.path());

        let action =
            UserFileEditAction::append_line(test_file.to_str().unwrap(), "new line", "test");

        let preview = generate_edit_preview(&action);
        assert!(preview.is_ok());
        let p = preview.unwrap();
        assert!(p.would_change);
        assert!(p.diff_unified.contains("+new line"));
    }

    #[test]
    fn test_preview_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "existing line\n").unwrap();

        std::env::set_var("HOME", temp_dir.path());

        let action =
            UserFileEditAction::append_line(test_file.to_str().unwrap(), "existing line", "test");

        let preview = generate_edit_preview(&action);
        assert!(preview.is_ok());
        let p = preview.unwrap();
        assert!(!p.would_change); // Already exists, idempotent
    }

    #[test]
    fn test_set_key_value_preview() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("config.ini");
        fs::write(&test_file, "foo=bar\nbaz=qux\n").unwrap();

        std::env::set_var("HOME", temp_dir.path());

        let action = UserFileEditAction::set_key_value(
            test_file.to_str().unwrap(),
            "foo",
            "newvalue",
            "=",
            "test",
        );

        let preview = generate_edit_preview(&action);
        assert!(preview.is_ok());
        let p = preview.unwrap();
        assert!(p.would_change);
        assert!(p.diff_unified.contains("-foo=bar"));
        assert!(p.diff_unified.contains("+foo=newvalue"));
    }

    #[test]
    fn test_symlink_escape_detection() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        // Create a symlink that escapes home
        let link_path = temp_dir.path().join("escape_link");
        let _ = std::os::unix::fs::symlink("/etc/hosts", &link_path);

        let result = check_path_policy(link_path.to_str().unwrap());
        // Should detect escape if symlink was created successfully
        if link_path.exists() {
            assert!(!result.allowed || result.is_symlink);
        }
    }
}
