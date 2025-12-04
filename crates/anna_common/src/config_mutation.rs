//! Config File Mutation Executor for Anna v0.0.80
//!
//! Safe config file editing with:
//! - Allowlist of safe files
//! - Backup before modification
//! - Diff preview
//! - Rollback support
//!
//! Supported operations:
//! - Add line
//! - Replace exact line
//! - Comment/uncomment line

use crate::mutation_engine_v1::{
    is_config_allowed, ConfigEditOp, MutationRiskLevel, RollbackStep, StepExecutionResult,
    StepPreview, VerificationCheck,
};
use crate::privilege::{check_privilege, run_privileged};
use std::fs;
use std::path::{Path, PathBuf};

/// Rollback directory for config backups
const ROLLBACK_BASE: &str = "/var/lib/anna/rollback/files";

/// Maximum file size for editing (1 MB)
const MAX_FILE_SIZE: u64 = 1024 * 1024;

/// Generate unified diff between two strings
fn generate_diff(original: &str, modified: &str, path: &str) -> String {
    let mut diff_lines = Vec::new();
    diff_lines.push(format!("--- a{}", path));
    diff_lines.push(format!("+++ b{}", path));

    let orig_lines: Vec<&str> = original.lines().collect();
    let mod_lines: Vec<&str> = modified.lines().collect();

    // Simple diff - just show what changed
    let mut i = 0;
    let mut j = 0;

    while i < orig_lines.len() || j < mod_lines.len() {
        if i < orig_lines.len() && j < mod_lines.len() && orig_lines[i] == mod_lines[j] {
            // Lines match, skip
            i += 1;
            j += 1;
        } else if i < orig_lines.len()
            && (j >= mod_lines.len() || !mod_lines.contains(&orig_lines[i]))
        {
            // Line removed
            diff_lines.push(format!("-{}", orig_lines[i]));
            i += 1;
        } else if j < mod_lines.len() {
            // Line added
            diff_lines.push(format!("+{}", mod_lines[j]));
            j += 1;
        } else {
            i += 1;
            j += 1;
        }
    }

    diff_lines.join("\n")
}

/// Apply config edit operation to content
fn apply_edit(content: &str, op: &ConfigEditOp) -> Result<String, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

    match op {
        ConfigEditOp::AddLine { line } => {
            // Add line at end (if not already present)
            if !lines.iter().any(|l| l.trim() == line.trim()) {
                result_lines.push(line.clone());
            }
        }
        ConfigEditOp::ReplaceLine { old, new } => {
            let mut found = false;
            for line in &mut result_lines {
                if line.trim() == old.trim() {
                    *line = new.clone();
                    found = true;
                }
            }
            if !found {
                return Err(format!("Line not found: {}", old));
            }
        }
        ConfigEditOp::CommentLine { pattern } => {
            let mut found = false;
            for line in &mut result_lines {
                if line.trim() == pattern.trim() && !line.trim().starts_with('#') {
                    *line = format!("#{}", line);
                    found = true;
                }
            }
            if !found {
                return Err(format!("Uncommented line not found: {}", pattern));
            }
        }
        ConfigEditOp::UncommentLine { pattern } => {
            let mut found = false;
            let pattern_trimmed = pattern.trim().trim_start_matches('#');
            for line in &mut result_lines {
                let line_trimmed = line.trim();
                if line_trimmed.starts_with('#') {
                    let uncommented = line_trimmed.trim_start_matches('#').trim();
                    if uncommented == pattern_trimmed {
                        *line = uncommented.to_string();
                        found = true;
                    }
                }
            }
            if !found {
                return Err(format!("Commented line not found: {}", pattern));
            }
        }
    }

    Ok(result_lines.join("\n") + "\n")
}

/// Generate preview for config edit
pub fn preview_config_edit(path: &str, op: &ConfigEditOp) -> Result<StepPreview, String> {
    if !is_config_allowed(path) {
        return Err(format!(
            "Config file '{}' is not in the allowed whitelist for v0.0.80",
            path
        ));
    }

    let path_obj = Path::new(path);

    // Check file exists
    if !path_obj.exists() {
        return Err(format!("Config file does not exist: {}", path));
    }

    // Check file size
    let metadata = fs::metadata(path_obj).map_err(|e| format!("Cannot read file: {}", e))?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!(
            "File too large: {} bytes (max {} bytes)",
            metadata.len(),
            MAX_FILE_SIZE
        ));
    }

    // Read current content
    let content = fs::read_to_string(path_obj).map_err(|e| format!("Cannot read file: {}", e))?;

    // Apply edit to get modified content
    let modified = apply_edit(&content, op)?;

    // Generate diff
    let diff = if content == modified {
        None
    } else {
        Some(generate_diff(&content, &modified, path))
    };

    let op_desc = match op {
        ConfigEditOp::AddLine { line } => format!("Add line: {}", line),
        ConfigEditOp::ReplaceLine { old, new } => {
            format!("Replace '{}' with '{}'", old, new)
        }
        ConfigEditOp::CommentLine { pattern } => format!("Comment out: {}", pattern),
        ConfigEditOp::UncommentLine { pattern } => format!("Uncomment: {}", pattern),
    };

    Ok(StepPreview {
        step_id: format!("config-edit-{}", path.replace('/', "-")),
        description: format!("Edit {}: {}", path, op_desc),
        current_state: format!("File: {} ({} lines)", path, content.lines().count()),
        intended_state: format!("File: {} ({} lines)", path, modified.lines().count()),
        diff,
    })
}

/// Create backup of config file before modification
fn create_backup(path: &str, case_id: &str) -> Result<PathBuf, String> {
    let source = Path::new(path);
    if !source.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    // Create backup directory
    let backup_dir = PathBuf::from(ROLLBACK_BASE).join(case_id);
    fs::create_dir_all(&backup_dir).map_err(|e| format!("Cannot create backup dir: {}", e))?;

    // Create backup filename (preserve directory structure)
    let filename = path.replace('/', "_").trim_start_matches('_').to_string();
    let backup_path = backup_dir.join(&filename);

    // Copy file
    fs::copy(source, &backup_path).map_err(|e| format!("Cannot create backup: {}", e))?;

    Ok(backup_path)
}

/// Execute config edit
pub fn execute_config_edit(
    path: &str,
    op: &ConfigEditOp,
    case_id: &str,
) -> Result<StepExecutionResult, String> {
    if !is_config_allowed(path) {
        return Err(format!(
            "Config file '{}' is not in the allowed whitelist",
            path
        ));
    }

    let priv_status = check_privilege();
    if !priv_status.available {
        return Err(priv_status.message);
    }

    // Create backup first
    let backup_path = create_backup(path, case_id)?;

    // Read current content
    let content = fs::read_to_string(path).map_err(|e| format!("Cannot read file: {}", e))?;

    // Apply edit
    let modified = apply_edit(&content, op)?;

    // Write modified content
    // We need to use privileged write since config files are typically root-owned
    let temp_file = format!("/tmp/anna_config_edit_{}", case_id);
    fs::write(&temp_file, &modified).map_err(|e| format!("Cannot write temp file: {}", e))?;

    // Move temp file to target (with sudo)
    let output = run_privileged("cp", &[&temp_file, path])?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_file);

    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let message = if success {
        format!("Successfully edited {}. Backup at {:?}", path, backup_path)
    } else {
        format!("Failed to edit {}: {}", path, stderr.trim())
    };

    Ok(StepExecutionResult {
        step_id: format!("config-edit-{}", path.replace('/', "-")),
        success,
        message,
        stdout: None,
        stderr: if stderr.is_empty() {
            None
        } else {
            Some(stderr)
        },
        exit_code: output.status.code(),
    })
}

/// Restore config from backup
pub fn rollback_config_edit(path: &str, case_id: &str) -> Result<(), String> {
    let filename = path.replace('/', "_").trim_start_matches('_').to_string();
    let backup_path = PathBuf::from(ROLLBACK_BASE).join(case_id).join(&filename);

    if !backup_path.exists() {
        return Err(format!("Backup not found: {:?}", backup_path));
    }

    let priv_status = check_privilege();
    if !priv_status.available {
        return Err(priv_status.message);
    }

    let output = run_privileged("cp", &[backup_path.to_str().unwrap(), path])?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to restore backup: {}", stderr.trim()))
    }
}

/// Create verification checks for config edit
pub fn create_config_verification(path: &str, op: &ConfigEditOp) -> Vec<VerificationCheck> {
    let desc = match op {
        ConfigEditOp::AddLine { line } => {
            format!("Verify line '{}' exists in {}", line.trim(), path)
        }
        ConfigEditOp::ReplaceLine { new, .. } => {
            format!("Verify line '{}' exists in {}", new.trim(), path)
        }
        ConfigEditOp::CommentLine { pattern } => {
            format!("Verify line '{}' is commented in {}", pattern.trim(), path)
        }
        ConfigEditOp::UncommentLine { pattern } => {
            format!(
                "Verify line '{}' is uncommented in {}",
                pattern.trim(),
                path
            )
        }
    };

    vec![VerificationCheck {
        description: desc,
        command: Some(format!("cat {}", path)),
        expected: "Contains expected changes".to_string(),
    }]
}

/// Create rollback steps for config edit
pub fn create_config_rollback(path: &str, case_id: &str) -> Vec<RollbackStep> {
    let filename = path.replace('/', "_").trim_start_matches('_').to_string();
    let backup_path = PathBuf::from(ROLLBACK_BASE).join(case_id).join(&filename);

    vec![RollbackStep {
        description: format!("Restore {} from backup", path),
        undo_action: format!("cp {:?} {}", backup_path, path),
        backup_path: Some(backup_path),
    }]
}

/// Get risk level for config edit
pub fn get_config_edit_risk(path: &str, _op: &ConfigEditOp) -> MutationRiskLevel {
    // SSH config is higher risk
    if path.contains("sshd") {
        MutationRiskLevel::High
    } else if path.contains("pacman") {
        MutationRiskLevel::Medium
    } else {
        MutationRiskLevel::Medium
    }
}

/// Generate manual commands for user when privilege not available
pub fn generate_config_manual_commands(
    path: &str,
    op: &ConfigEditOp,
) -> Vec<String> {
    match op {
        ConfigEditOp::AddLine { line } => {
            vec![format!("echo '{}' | sudo tee -a {}", line, path)]
        }
        ConfigEditOp::ReplaceLine { old, new } => {
            vec![format!(
                "sudo sed -i 's/^{}$/{}/' {}",
                old.replace('/', "\\/"),
                new.replace('/', "\\/"),
                path
            )]
        }
        ConfigEditOp::CommentLine { pattern } => {
            vec![format!(
                "sudo sed -i 's/^{}$/#&/' {}",
                pattern.replace('/', "\\/"),
                path
            )]
        }
        ConfigEditOp::UncommentLine { pattern } => {
            let pat = pattern.trim().trim_start_matches('#');
            vec![format!(
                "sudo sed -i 's/^#\\s*{}$/{}/' {}",
                pat.replace('/', "\\/"),
                pat.replace('/', "\\/"),
                path
            )]
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_edit_add_line() {
        let content = "line1\nline2\n";
        let op = ConfigEditOp::AddLine {
            line: "line3".to_string(),
        };
        let result = apply_edit(content, &op).unwrap();
        assert!(result.contains("line3"));
    }

    #[test]
    fn test_apply_edit_add_line_idempotent() {
        let content = "line1\nline2\n";
        let op = ConfigEditOp::AddLine {
            line: "line1".to_string(),
        };
        let result = apply_edit(content, &op).unwrap();
        // Should not duplicate
        assert_eq!(result.matches("line1").count(), 1);
    }

    #[test]
    fn test_apply_edit_replace_line() {
        let content = "foo=bar\nbaz=qux\n";
        let op = ConfigEditOp::ReplaceLine {
            old: "foo=bar".to_string(),
            new: "foo=newvalue".to_string(),
        };
        let result = apply_edit(content, &op).unwrap();
        assert!(result.contains("foo=newvalue"));
        assert!(!result.contains("foo=bar"));
    }

    #[test]
    fn test_apply_edit_comment_line() {
        let content = "option=value\n";
        let op = ConfigEditOp::CommentLine {
            pattern: "option=value".to_string(),
        };
        let result = apply_edit(content, &op).unwrap();
        assert!(result.contains("#option=value"));
    }

    #[test]
    fn test_apply_edit_uncomment_line() {
        let content = "#option=value\n";
        let op = ConfigEditOp::UncommentLine {
            pattern: "option=value".to_string(),
        };
        let result = apply_edit(content, &op).unwrap();
        assert!(result.contains("option=value"));
        assert!(!result.contains("#option=value"));
    }

    #[test]
    fn test_generate_diff() {
        let original = "line1\nline2\n";
        let modified = "line1\nline3\n";
        let diff = generate_diff(original, modified, "/etc/test.conf");
        assert!(diff.contains("-line2"));
        assert!(diff.contains("+line3"));
    }

    #[test]
    fn test_get_config_edit_risk() {
        assert_eq!(
            get_config_edit_risk("/etc/ssh/sshd_config", &ConfigEditOp::AddLine {
                line: "test".to_string()
            }),
            MutationRiskLevel::High
        );
        assert_eq!(
            get_config_edit_risk("/etc/pacman.conf", &ConfigEditOp::AddLine {
                line: "test".to_string()
            }),
            MutationRiskLevel::Medium
        );
    }

    #[test]
    fn test_preview_config_not_allowed() {
        let result = preview_config_edit(
            "/etc/passwd",
            &ConfigEditOp::AddLine {
                line: "test".to_string(),
            },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in the allowed whitelist"));
    }
}
