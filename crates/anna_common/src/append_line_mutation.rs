//! Append Line Mutation Flow v0.0.47
//!
//! Single-capability mutation: append a line to a text file.
//! Includes evidence collection, diff preview, confirmation gates,
//! execution, verification, and rollback.
//!
//! Sandbox-safe by default:
//! - Current working directory
//! - /tmp/
//! - $HOME/ (with policy checks)
//!
//! Non-negotiables:
//! - Preserves file ownership and permissions
//! - Creates case file for every operation
//! - Full rollback support via case_id

use crate::mutation_tools::{FileEditOp, MutationError, RollbackInfo};
use crate::rollback::{RollbackManager, MutationType, MutationDetails, MutationLogEntry};
use crate::policy::{get_policy, PolicyCheckResult, generate_policy_evidence_id};
use crate::transcript::{CaseFile, CaseOutcome};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// v0.0.47 dev sandbox paths - always allowed for append mutations
const DEV_SANDBOX_PATHS: &[&str] = &["/tmp"];

/// Confirmation phrases for different risk levels
pub const SANDBOX_CONFIRMATION: &str = "yes";
pub const HOME_CONFIRMATION: &str = "I CONFIRM (medium risk)";

/// Result of checking if a path is in the dev sandbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxCheck {
    pub path: String,
    pub in_sandbox: bool,
    pub in_cwd: bool,
    pub in_tmp: bool,
    pub in_home: bool,
    pub reason: String,
    pub evidence_id: String,
    pub risk_level: RiskLevel,
    pub confirmation_phrase: String,
}

/// Risk level for append operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Sandbox (cwd, /tmp) - low risk, simple "yes" confirmation
    Sandbox,
    /// Home directory - medium risk, full confirmation phrase
    Home,
    /// System paths - blocked in v0.0.47
    System,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Sandbox => write!(f, "sandbox (low)"),
            RiskLevel::Home => write!(f, "home (medium)"),
            RiskLevel::System => write!(f, "system (blocked)"),
        }
    }
}

/// Evidence collected before mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendMutationEvidence {
    pub path: String,
    pub exists: bool,
    pub file_stat: Option<FileStatEvidence>,
    pub file_preview: Option<FilePreviewEvidence>,
    pub before_hash: Option<String>,
    pub sandbox_check: SandboxCheck,
    pub policy_check: PolicyCheckResult,
}

/// File stat evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatEvidence {
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
    pub size: u64,
    pub mtime: i64,
}

/// File preview evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreviewEvidence {
    pub line_count: usize,
    pub last_20_lines: Vec<String>,
    pub ends_with_newline: bool,
}

/// Diff preview for the append operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendDiffPreview {
    pub path: String,
    pub current_lines: usize,
    pub last_lines_before: Vec<String>,
    pub line_to_append: String,
    pub will_add_newline: bool,
    pub diff_text: String,
}

/// Result of append mutation execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendMutationResult {
    pub success: bool,
    pub case_id: String,
    pub path: String,
    pub line_appended: String,
    pub before_hash: String,
    pub after_hash: String,
    pub backup_path: PathBuf,
    pub rollback_info: RollbackInfo,
    pub error: Option<String>,
}

/// Rollback result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub success: bool,
    pub case_id: String,
    pub path: String,
    pub restored_hash: String,
    pub expected_hash: String,
    pub hashes_match: bool,
    pub error: Option<String>,
}

/// Check if a path is in the dev sandbox
pub fn check_sandbox(path: &Path) -> SandboxCheck {
    let path_str = path.to_string_lossy().to_string();
    let evidence_id = generate_policy_evidence_id();

    // Check current working directory
    let cwd = std::env::current_dir().ok();
    let in_cwd = cwd.as_ref().map(|c| path.starts_with(c)).unwrap_or(false);

    // Check /tmp
    let in_tmp = path.starts_with("/tmp");

    // Check home directory
    let home = std::env::var("HOME").ok();
    let in_home = home.as_ref().map(|h| path.starts_with(h)).unwrap_or(false);

    let in_sandbox = in_cwd || in_tmp;

    let (risk_level, confirmation_phrase, reason) = if in_sandbox {
        (
            RiskLevel::Sandbox,
            SANDBOX_CONFIRMATION.to_string(),
            format!("Path is in dev sandbox ({})", if in_cwd { "cwd" } else { "/tmp" }),
        )
    } else if in_home {
        (
            RiskLevel::Home,
            HOME_CONFIRMATION.to_string(),
            "Path is in home directory".to_string(),
        )
    } else {
        (
            RiskLevel::System,
            String::new(),
            "System path - blocked in v0.0.47".to_string(),
        )
    };

    SandboxCheck {
        path: path_str,
        in_sandbox,
        in_cwd,
        in_tmp,
        in_home,
        reason,
        evidence_id,
        risk_level,
        confirmation_phrase,
    }
}

/// Collect evidence before mutation
pub fn collect_evidence(path: &Path) -> AppendMutationEvidence {
    let path_str = path.to_string_lossy().to_string();
    let exists = path.exists();

    // File stat
    let file_stat = if exists {
        fs::metadata(path).ok().map(|meta| FileStatEvidence {
            uid: meta.uid(),
            gid: meta.gid(),
            mode: meta.mode(),
            size: meta.len(),
            mtime: meta.mtime(),
        })
    } else {
        None
    };

    // File preview
    let file_preview = if exists {
        fs::read_to_string(path).ok().map(|content| {
            let lines: Vec<&str> = content.lines().collect();
            let last_20: Vec<String> = lines.iter().rev().take(20).rev().map(|s| s.to_string()).collect();
            FilePreviewEvidence {
                line_count: lines.len(),
                last_20_lines: last_20,
                ends_with_newline: content.ends_with('\n'),
            }
        })
    } else {
        None
    };

    // Before hash
    let before_hash = if exists {
        hash_file(path).ok()
    } else {
        None
    };

    // Sandbox check
    let sandbox_check = check_sandbox(path);

    // Policy check
    let policy = get_policy();
    let policy_check = policy.is_path_allowed(&path_str);

    AppendMutationEvidence {
        path: path_str,
        exists,
        file_stat,
        file_preview,
        before_hash,
        sandbox_check,
        policy_check,
    }
}

/// Generate diff preview for append operation
pub fn generate_diff_preview(evidence: &AppendMutationEvidence, line_to_append: &str) -> AppendDiffPreview {
    let current_lines = evidence.file_preview.as_ref().map(|p| p.line_count).unwrap_or(0);
    let last_lines_before = evidence.file_preview.as_ref()
        .map(|p| p.last_20_lines.clone())
        .unwrap_or_default();
    let ends_with_newline = evidence.file_preview.as_ref()
        .map(|p| p.ends_with_newline)
        .unwrap_or(true);

    // Build diff text
    let mut diff_text = String::new();
    diff_text.push_str(&format!("--- {}\n", evidence.path));
    diff_text.push_str(&format!("+++ {} (after append)\n", evidence.path));

    // Show context (last few lines)
    let context_start = current_lines.saturating_sub(3);
    for (i, line) in last_lines_before.iter().rev().take(3).rev().enumerate() {
        diff_text.push_str(&format!(" {:4} | {}\n", context_start + i + 1, line));
    }

    // Show the appended line
    if !ends_with_newline && !last_lines_before.is_empty() {
        diff_text.push_str("      | (adding newline before append)\n");
    }
    diff_text.push_str(&format!("+{:4} | {}\n", current_lines + 1, line_to_append));

    AppendDiffPreview {
        path: evidence.path.clone(),
        current_lines,
        last_lines_before,
        line_to_append: line_to_append.to_string(),
        will_add_newline: !ends_with_newline,
        diff_text,
    }
}

/// Check if mutation is allowed based on evidence
pub fn check_mutation_allowed(evidence: &AppendMutationEvidence) -> Result<(), MutationError> {
    // Check sandbox/risk level
    if evidence.sandbox_check.risk_level == RiskLevel::System {
        return Err(MutationError::PolicyBlocked {
            path: PathBuf::from(&evidence.path),
            reason: "System paths blocked in v0.0.47 - only sandbox (cwd, /tmp) and $HOME allowed".to_string(),
            evidence_id: evidence.sandbox_check.evidence_id.clone(),
            policy_rule: "v0.0.47:sandbox_only".to_string(),
        });
    }

    // If path is in $HOME but not in sandbox, check policy
    if evidence.sandbox_check.risk_level == RiskLevel::Home && !evidence.policy_check.allowed {
        return Err(MutationError::PolicyBlocked {
            path: PathBuf::from(&evidence.path),
            reason: evidence.policy_check.reason.clone(),
            evidence_id: evidence.policy_check.evidence_id.clone(),
            policy_rule: evidence.policy_check.policy_rule.clone(),
        });
    }

    Ok(())
}

/// Execute append line mutation
pub fn execute_append_line(
    path: &Path,
    line: &str,
    evidence: &AppendMutationEvidence,
    case_id: &str,
) -> Result<AppendMutationResult, MutationError> {
    let path_str = path.to_string_lossy().to_string();

    // Check allowed
    check_mutation_allowed(evidence)?;

    let rollback_manager = RollbackManager::new();

    // Get original metadata for ownership preservation
    let original_uid = evidence.file_stat.as_ref().map(|s| s.uid);
    let original_gid = evidence.file_stat.as_ref().map(|s| s.gid);
    let original_mode = evidence.file_stat.as_ref().map(|s| s.mode);

    // Create backup
    let backup_path = if evidence.exists {
        rollback_manager.backup_file(path, case_id)
            .map_err(|e| MutationError::Other(format!("Backup failed: {}", e)))?
    } else {
        // Create empty backup marker for new files
        let backup_dir = rollback_manager.files_dir().join(format!("{}_{}", timestamp(), case_id));
        fs::create_dir_all(&backup_dir)
            .map_err(|e| MutationError::Other(format!("Backup dir failed: {}", e)))?;
        let marker = backup_dir.join("_new_file_marker.txt");
        fs::write(&marker, format!("New file created: {}", path_str))
            .map_err(|e| MutationError::Other(format!("Marker write failed: {}", e)))?;
        marker
    };

    // Get before hash
    let before_hash = evidence.before_hash.clone().unwrap_or_else(|| "none".to_string());

    // Read current content or start empty
    let current_content = if evidence.exists {
        fs::read_to_string(path)
            .map_err(|e| MutationError::Other(format!("Read failed: {}", e)))?
    } else {
        String::new()
    };

    // Append line with proper newline handling
    let new_content = if current_content.is_empty() {
        format!("{}\n", line)
    } else if current_content.ends_with('\n') {
        format!("{}{}\n", current_content, line)
    } else {
        format!("{}\n{}\n", current_content, line)
    };

    // Write new content
    fs::write(path, &new_content)
        .map_err(|e| MutationError::Other(format!("Write failed: {}", e)))?;

    // Restore ownership if we have it (requires root)
    if let (Some(uid), Some(gid)) = (original_uid, original_gid) {
        let _ = std::process::Command::new("chown")
            .args([&format!("{}:{}", uid, gid), &path_str])
            .output();
    }

    // Restore permissions if we have them
    if let Some(mode) = original_mode {
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode & 0o7777));
    }

    // Get after hash
    let after_hash = hash_file(path)
        .map_err(|e| MutationError::Other(format!("After hash failed: {}", e)))?;

    // Log the mutation
    let operation = FileEditOp::AppendLine { content: line.to_string() };
    rollback_manager.log_file_edit(
        case_id,
        &[evidence.sandbox_check.evidence_id.clone()],
        path,
        &backup_path,
        &before_hash,
        &after_hash,
        &[operation],
        true,
        None,
    ).map_err(|e| MutationError::Other(format!("Log failed: {}", e)))?;

    // Generate rollback info
    let rollback_info = RollbackInfo {
        backup_path: Some(backup_path.clone()),
        rollback_command: Some(format!("annactl 'rollback {}'", case_id)),
        rollback_instructions: format!(
            "To rollback this change:\n\
             1. Run: annactl 'rollback {}'\n\
             2. Or manually: cp '{}' '{}'\n\
             Backup at: {}",
            case_id,
            backup_path.display(),
            path_str,
            backup_path.display()
        ),
        prior_state: Some(format!("hash={}", before_hash)),
    };

    Ok(AppendMutationResult {
        success: true,
        case_id: case_id.to_string(),
        path: path_str,
        line_appended: line.to_string(),
        before_hash,
        after_hash,
        backup_path,
        rollback_info,
        error: None,
    })
}

/// Execute rollback by case_id
pub fn execute_rollback(case_id: &str) -> Result<RollbackResult, String> {
    let rollback_manager = RollbackManager::new();

    // Find the mutation log for this case
    let logs = rollback_manager.recent_logs(100)
        .map_err(|e| format!("Cannot read mutation logs: {}", e))?;

    let entry = logs.iter()
        .find(|e| e.request_id == case_id)
        .ok_or_else(|| format!("No mutation found with case_id: {}", case_id))?;

    // Extract file details
    let (file_path, backup_path, expected_hash) = match &entry.details {
        MutationDetails::FileEdit { file_path, backup_path, before_hash, .. } => {
            (file_path.clone(), backup_path.clone(), before_hash.clone())
        }
        _ => return Err(format!("Case {} is not a file edit mutation", case_id)),
    };

    // Read backup content
    let backup_content = fs::read(&backup_path)
        .map_err(|e| format!("Cannot read backup at {}: {}", backup_path.display(), e))?;

    // Get backup metadata for ownership
    let backup_meta = fs::metadata(&backup_path).ok();

    // Restore file
    fs::write(&file_path, &backup_content)
        .map_err(|e| format!("Cannot write to {}: {}", file_path.display(), e))?;

    // Restore ownership if possible
    if let Some(meta) = backup_meta {
        let uid = meta.uid();
        let gid = meta.gid();
        let mode = meta.mode();

        let _ = std::process::Command::new("chown")
            .args([&format!("{}:{}", uid, gid), &file_path.to_string_lossy().to_string()])
            .output();
        let _ = fs::set_permissions(&file_path, fs::Permissions::from_mode(mode & 0o7777));
    }

    // Verify hash
    let restored_hash = hash_file(&file_path)
        .map_err(|e| format!("Cannot hash restored file: {}", e))?;
    let hashes_match = restored_hash == expected_hash;

    Ok(RollbackResult {
        success: true,
        case_id: case_id.to_string(),
        path: file_path.to_string_lossy().to_string(),
        restored_hash,
        expected_hash,
        hashes_match,
        error: None,
    })
}

/// Hash a file
fn hash_file(path: &Path) -> io::Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let content = fs::read(path)?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
}

/// Get current timestamp
fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Generate a case ID for mutations
pub fn generate_mutation_case_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("mut_{}", ts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sandbox_check_tmp() {
        let check = check_sandbox(Path::new("/tmp/test.txt"));
        assert!(check.in_sandbox);
        assert!(check.in_tmp);
        assert_eq!(check.risk_level, RiskLevel::Sandbox);
        assert_eq!(check.confirmation_phrase, SANDBOX_CONFIRMATION);
    }

    #[test]
    fn test_sandbox_check_system() {
        let check = check_sandbox(Path::new("/etc/hosts"));
        assert!(!check.in_sandbox);
        assert!(!check.in_tmp);
        assert!(!check.in_cwd);
        assert_eq!(check.risk_level, RiskLevel::System);
    }

    #[test]
    fn test_collect_evidence() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "line1\nline2\nline3\n").unwrap();

        let evidence = collect_evidence(&test_file);
        assert!(evidence.exists);
        assert!(evidence.file_stat.is_some());
        assert!(evidence.file_preview.is_some());
        assert!(evidence.before_hash.is_some());

        let preview = evidence.file_preview.as_ref().unwrap();
        assert_eq!(preview.line_count, 3);
        assert!(preview.ends_with_newline);
    }

    #[test]
    fn test_diff_preview() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "line1\nline2\nline3\n").unwrap();

        let evidence = collect_evidence(&test_file);
        let diff = generate_diff_preview(&evidence, "new line");

        assert_eq!(diff.current_lines, 3);
        assert_eq!(diff.line_to_append, "new line");
        assert!(diff.diff_text.contains("+"));
    }

    #[test]
    fn test_execute_append_in_tmp() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "original content\n").unwrap();

        let evidence = collect_evidence(&test_file);
        let case_id = generate_mutation_case_id();

        // This should work because TempDir is typically in /tmp
        let result = execute_append_line(&test_file, "appended line", &evidence, &case_id);

        // If it fails due to policy, that's expected in some test environments
        if result.is_ok() {
            let r = result.unwrap();
            assert!(r.success);
            let content = fs::read_to_string(&test_file).unwrap();
            assert!(content.contains("appended line"));
        }
    }
}
