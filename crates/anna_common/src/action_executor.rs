//! Action Executor v0.0.54
//!
//! Executes action plans with:
//! - Diff preview generation for file edits
//! - Confirmation validation
//! - Step-by-step execution via annad
//! - Rollback record creation

use std::fs;
use std::io::{self, Write as IoWrite};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::action_engine::{
    ActionDiffLine, ActionDiffPreview, ActionPlan, ActionResult, ActionStep, ActionType,
    BackupRecord, DeleteFileAction, DiffLineType, EditFileAction, EditIntent, MutationRiskLevel,
    PackageReason, PacmanAction, PacmanOperation, PlanStatus, RollbackHint, RollbackRecord,
    RollbackStepRecord, StepResult, StepStatus, SystemdAction, SystemdOperation,
    VerificationRecord, WriteFileAction, CONFIRM_DESTRUCTIVE, CONFIRM_HIGH, CONFIRM_LOW,
    CONFIRM_MEDIUM,
};
use crate::action_risk::score_action_risk;

// =============================================================================
// Constants
// =============================================================================

/// Base directory for case files
pub const CASES_DIR: &str = "/var/lib/anna/cases";
/// Base directory for rollback backups
pub const ROLLBACK_DIR: &str = "/var/lib/anna/rollback";
/// Maximum lines in diff preview
pub const MAX_DIFF_LINES: usize = 50;

// =============================================================================
// Diff Preview Generation
// =============================================================================

/// Generate a diff preview for a file edit action
pub fn generate_action_diff_preview(
    action: &EditFileAction,
    case_id: &str,
) -> io::Result<ActionDiffPreview> {
    let path = &action.path;
    let file_exists = path.exists();

    let current_content = if file_exists {
        fs::read_to_string(path)?
    } else {
        String::new()
    };

    let current_hash = if file_exists {
        Some(compute_hash(&current_content))
    } else {
        None
    };

    // Generate new content based on intent
    let new_content = apply_edit_intent(&current_content, &action.intent)?;

    // Generate unified diff
    let diff_lines = generate_unified_diff(&current_content, &new_content, path);

    let additions = diff_lines
        .iter()
        .filter(|l| l.line_type == DiffLineType::Addition)
        .count();
    let deletions = diff_lines
        .iter()
        .filter(|l| l.line_type == DiffLineType::Deletion)
        .count();
    let truncated = diff_lines.len() > MAX_DIFF_LINES;

    // Determine backup path
    let backup_path = get_backup_path(path, case_id);

    Ok(ActionDiffPreview {
        path: path.clone(),
        file_exists,
        current_hash,
        diff_lines: diff_lines.into_iter().take(MAX_DIFF_LINES).collect(),
        additions,
        deletions,
        truncated,
        backup_path,
    })
}

/// Apply an edit intent to content and return new content
fn apply_edit_intent(content: &str, intent: &EditIntent) -> io::Result<String> {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    match intent {
        EditIntent::Append { lines: new_lines } => {
            for line in new_lines {
                lines.push(line.clone());
            }
        }
        EditIntent::Replace {
            content: new_content,
        } => {
            return Ok(new_content.clone());
        }
        EditIntent::Patch => {
            // Patch is applied separately
            return Ok(content.to_string());
        }
        EditIntent::SetKeyValue {
            key,
            value,
            separator,
        } => {
            let pattern = format!("{}{}*", key, separator);
            let new_line = format!("{}{}{}", key, separator, value);
            let mut found = false;

            for line in &mut lines {
                if line.starts_with(&format!("{}{}", key, separator)) {
                    *line = new_line.clone();
                    found = true;
                    break;
                }
            }

            if !found {
                lines.push(new_line);
            }
        }
        EditIntent::InsertLine {
            line_number,
            content: line_content,
        } => {
            let idx = (*line_number).min(lines.len());
            lines.insert(idx, line_content.clone());
        }
        EditIntent::DeleteLine { line_number } => {
            if *line_number < lines.len() {
                lines.remove(*line_number);
            }
        }
    }

    Ok(lines.join("\n") + if content.ends_with('\n') { "\n" } else { "" })
}

/// Generate unified diff between two strings
fn generate_unified_diff(old: &str, new: &str, path: &Path) -> Vec<ActionDiffLine> {
    let mut lines = Vec::new();

    // Header
    lines.push(ActionDiffLine {
        line_type: DiffLineType::Header,
        content: format!("--- a/{}", path.display()),
        old_line: None,
        new_line: None,
    });
    lines.push(ActionDiffLine {
        line_type: DiffLineType::Header,
        content: format!("+++ b/{}", path.display()),
        old_line: None,
        new_line: None,
    });

    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // Simple line-by-line diff (not optimal but readable)
    let max_len = old_lines.len().max(new_lines.len());
    let mut old_idx = 0;
    let mut new_idx = 0;

    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        if old_idx < old_lines.len() && new_idx < new_lines.len() {
            if old_lines[old_idx] == new_lines[new_idx] {
                lines.push(ActionDiffLine {
                    line_type: DiffLineType::Context,
                    content: old_lines[old_idx].to_string(),
                    old_line: Some(old_idx + 1),
                    new_line: Some(new_idx + 1),
                });
                old_idx += 1;
                new_idx += 1;
            } else {
                // Check if it's a modification or insertion/deletion
                lines.push(ActionDiffLine {
                    line_type: DiffLineType::Deletion,
                    content: old_lines[old_idx].to_string(),
                    old_line: Some(old_idx + 1),
                    new_line: None,
                });
                lines.push(ActionDiffLine {
                    line_type: DiffLineType::Addition,
                    content: new_lines[new_idx].to_string(),
                    old_line: None,
                    new_line: Some(new_idx + 1),
                });
                old_idx += 1;
                new_idx += 1;
            }
        } else if old_idx < old_lines.len() {
            lines.push(ActionDiffLine {
                line_type: DiffLineType::Deletion,
                content: old_lines[old_idx].to_string(),
                old_line: Some(old_idx + 1),
                new_line: None,
            });
            old_idx += 1;
        } else {
            lines.push(ActionDiffLine {
                line_type: DiffLineType::Addition,
                content: new_lines[new_idx].to_string(),
                old_line: None,
                new_line: Some(new_idx + 1),
            });
            new_idx += 1;
        }
    }

    lines
}

// =============================================================================
// Backup and Hash Utilities
// =============================================================================

/// Compute SHA256 hash of content
pub fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get backup path for a file
pub fn get_backup_path(original_path: &Path, case_id: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let file_name = original_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    PathBuf::from(ROLLBACK_DIR)
        .join(case_id)
        .join("files")
        .join(format!("{}_{}", timestamp, file_name))
}

/// Create backup of a file
pub fn backup_file(path: &Path, backup_path: &Path) -> io::Result<BackupRecord> {
    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = fs::read_to_string(path)?;
    let hash = compute_hash(&content);
    fs::copy(path, backup_path)?;

    Ok(BackupRecord {
        original_path: path.to_path_buf(),
        backup_path: backup_path.to_path_buf(),
        hash,
        created_at: Utc::now(),
    })
}

// =============================================================================
// Confirmation Validation
// =============================================================================

/// Validate confirmation phrase for a plan
pub fn validate_confirmation(plan: &ActionPlan, phrase: &str) -> Result<(), String> {
    let expected = &plan.confirmation_phrase;
    let phrase = phrase.trim();

    if phrase == expected {
        Ok(())
    } else {
        Err(format!(
            "Invalid confirmation. Expected: '{}', got: '{}'",
            expected, phrase
        ))
    }
}

// =============================================================================
// Step Execution
// =============================================================================

/// Execute a single action step
pub fn execute_step(step: &ActionStep, case_id: &str) -> StepResult {
    let start = Instant::now();

    let result = match &step.action {
        ActionType::EditFile(edit) => execute_edit_file(edit, case_id),
        ActionType::WriteFile(write) => execute_write_file(write, case_id),
        ActionType::DeleteFile(delete) => execute_delete_file(delete, case_id),
        ActionType::Systemd(systemd) => execute_systemd(systemd),
        ActionType::Pacman(pacman) => execute_pacman(pacman),
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok((summary, rollback_hint, evidence_ids)) => StepResult {
            step_id: step.step_id.clone(),
            success: true,
            error: None,
            summary,
            evidence_ids,
            rollback_hint,
            duration_ms,
        },
        Err(e) => StepResult {
            step_id: step.step_id.clone(),
            success: false,
            error: Some(e),
            summary: "Step failed".to_string(),
            evidence_ids: vec![],
            rollback_hint: step.rollback_hint.clone(),
            duration_ms,
        },
    }
}

/// Execute a file edit
fn execute_edit_file(
    edit: &EditFileAction,
    case_id: &str,
) -> Result<(String, RollbackHint, Vec<String>), String> {
    let path = &edit.path;

    // Create backup if file exists
    let backup = if path.exists() {
        let backup_path = get_backup_path(path, case_id);
        Some(backup_file(path, &backup_path).map_err(|e| format!("Backup failed: {}", e))?)
    } else {
        None
    };

    // Read current content
    let current_content = if path.exists() {
        fs::read_to_string(path).map_err(|e| format!("Read failed: {}", e))?
    } else {
        String::new()
    };

    // Apply edit
    let new_content = apply_edit_intent(&current_content, &edit.intent)
        .map_err(|e| format!("Edit failed: {}", e))?;

    // Write atomically (write to temp, fsync, rename)
    let temp_path = path.with_extension("anna_tmp");
    {
        let mut file =
            fs::File::create(&temp_path).map_err(|e| format!("Create temp file failed: {}", e))?;
        file.write_all(new_content.as_bytes())
            .map_err(|e| format!("Write failed: {}", e))?;
        file.sync_all().map_err(|e| format!("Sync failed: {}", e))?;
    }
    fs::rename(&temp_path, path).map_err(|e| format!("Rename failed: {}", e))?;

    // Set permissions if specified
    if let Some(mode) = &edit.mode {
        if let Ok(mode_int) =
            u32::from_str_radix(mode.trim_start_matches("0o").trim_start_matches("0"), 8)
        {
            fs::set_permissions(path, fs::Permissions::from_mode(mode_int))
                .map_err(|e| format!("chmod failed: {}", e))?;
        }
    }

    let rollback_hint = RollbackHint {
        instructions: format!(
            "To restore: cp {} {}",
            backup
                .as_ref()
                .map(|b| b.backup_path.display().to_string())
                .unwrap_or_default(),
            path.display()
        ),
        command: backup
            .as_ref()
            .map(|b| format!("cp '{}' '{}'", b.backup_path.display(), path.display())),
        backup_path: backup.as_ref().map(|b| b.backup_path.clone()),
        prior_state: Some(compute_hash(&current_content)),
    };

    let summary = format!("Edited {}", path.display());
    let evidence_id = format!(
        "E{}",
        crate::generate_request_id()
            .chars()
            .take(8)
            .collect::<String>()
    );

    Ok((summary, rollback_hint, vec![evidence_id]))
}

/// Execute a file write (new file)
fn execute_write_file(
    write: &WriteFileAction,
    case_id: &str,
) -> Result<(String, RollbackHint, Vec<String>), String> {
    let path = &write.path;

    // Create parent directories
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Create dirs failed: {}", e))?;
    }

    // Write atomically
    let temp_path = path.with_extension("anna_tmp");
    {
        let mut file = fs::File::create(&temp_path).map_err(|e| format!("Create failed: {}", e))?;
        file.write_all(write.content.as_bytes())
            .map_err(|e| format!("Write failed: {}", e))?;
        file.sync_all().map_err(|e| format!("Sync failed: {}", e))?;
    }
    fs::rename(&temp_path, path).map_err(|e| format!("Rename failed: {}", e))?;

    // Set permissions
    if let Ok(mode_int) = u32::from_str_radix(
        write.mode.trim_start_matches("0o").trim_start_matches("0"),
        8,
    ) {
        fs::set_permissions(path, fs::Permissions::from_mode(mode_int))
            .map_err(|e| format!("chmod failed: {}", e))?;
    }

    let rollback_hint = RollbackHint {
        instructions: format!("To restore: rm '{}'", path.display()),
        command: Some(format!("rm '{}'", path.display())),
        backup_path: None,
        prior_state: None,
    };

    let summary = format!("Created {}", path.display());
    let evidence_id = format!(
        "E{}",
        crate::generate_request_id()
            .chars()
            .take(8)
            .collect::<String>()
    );

    Ok((summary, rollback_hint, vec![evidence_id]))
}

/// Execute a file delete
fn execute_delete_file(
    delete: &DeleteFileAction,
    case_id: &str,
) -> Result<(String, RollbackHint, Vec<String>), String> {
    let path = &delete.path;

    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    // Verify hash if required
    if let Some(required_hash) = &delete.require_hash {
        let content = fs::read_to_string(path).map_err(|e| format!("Read failed: {}", e))?;
        let actual_hash = compute_hash(&content);
        if actual_hash != *required_hash {
            return Err(format!(
                "Hash mismatch: expected {}, got {}",
                required_hash, actual_hash
            ));
        }
    }

    // Create backup before deletion
    let backup_path = get_backup_path(path, case_id);
    let backup = backup_file(path, &backup_path).map_err(|e| format!("Backup failed: {}", e))?;

    // Delete file
    fs::remove_file(path).map_err(|e| format!("Delete failed: {}", e))?;

    let rollback_hint = RollbackHint {
        instructions: format!(
            "To restore: cp '{}' '{}'",
            backup.backup_path.display(),
            path.display()
        ),
        command: Some(format!(
            "cp '{}' '{}'",
            backup.backup_path.display(),
            path.display()
        )),
        backup_path: Some(backup.backup_path.clone()),
        prior_state: Some(backup.hash),
    };

    let summary = format!("Deleted {}", path.display());
    let evidence_id = format!(
        "E{}",
        crate::generate_request_id()
            .chars()
            .take(8)
            .collect::<String>()
    );

    Ok((summary, rollback_hint, vec![evidence_id]))
}

/// Execute a systemd operation
fn execute_systemd(action: &SystemdAction) -> Result<(String, RollbackHint, Vec<String>), String> {
    let unit = &action.unit;
    let op = action.operation.as_str();

    // Execute systemctl
    let output = Command::new("systemctl")
        .arg(op)
        .arg(unit)
        .output()
        .map_err(|e| format!("systemctl failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("systemctl {} {} failed: {}", op, unit, stderr));
    }

    let rollback_hint = if let Some(inverse) = action.operation.inverse() {
        RollbackHint {
            instructions: format!("To undo: systemctl {} {}", inverse.as_str(), unit),
            command: Some(format!("systemctl {} {}", inverse.as_str(), unit)),
            backup_path: None,
            prior_state: None,
        }
    } else {
        RollbackHint::default()
    };

    let summary = format!("{}ed {}", op, unit);
    let evidence_id = format!(
        "S{}",
        crate::generate_request_id()
            .chars()
            .take(8)
            .collect::<String>()
    );

    Ok((summary, rollback_hint, vec![evidence_id]))
}

/// Execute a pacman operation
fn execute_pacman(action: &PacmanAction) -> Result<(String, RollbackHint, Vec<String>), String> {
    let packages = action.packages.join(" ");

    let (args, verb) = match action.operation {
        PacmanOperation::Install => (vec!["-S", "--noconfirm", "--needed"], "installed"),
        PacmanOperation::Remove => (vec!["-Rs", "--noconfirm"], "removed"),
    };

    let mut cmd = Command::new("pacman");
    for arg in &args {
        cmd.arg(arg);
    }
    for pkg in &action.packages {
        cmd.arg(pkg);
    }

    let output = cmd.output().map_err(|e| format!("pacman failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pacman failed: {}", stderr));
    }

    let inverse_op = match action.operation {
        PacmanOperation::Install => "pacman -Rs",
        PacmanOperation::Remove => "pacman -S",
    };

    let rollback_hint = RollbackHint {
        instructions: format!("To undo: {} {}", inverse_op, packages),
        command: Some(format!("{} {}", inverse_op, packages)),
        backup_path: None,
        prior_state: None,
    };

    let summary = format!("{} {}", verb, packages);
    let evidence_id = format!(
        "P{}",
        crate::generate_request_id()
            .chars()
            .take(8)
            .collect::<String>()
    );

    Ok((summary, rollback_hint, vec![evidence_id]))
}

// =============================================================================
// Plan Execution
// =============================================================================

/// Execute an entire action plan
pub fn execute_plan(plan: &mut ActionPlan, confirmation: &str) -> Result<ActionResult, String> {
    // Validate confirmation
    validate_confirmation(plan, confirmation)?;

    let started_at = Utc::now();
    let start = Instant::now();
    let mut step_results = Vec::new();
    let mut all_success = true;
    let mut backups = Vec::new();
    let mut verifications = Vec::new();

    plan.status = PlanStatus::Executing;

    for step in &mut plan.steps {
        step.status = StepStatus::Executing;
        let result = execute_step(step, &plan.case_id);

        if result.success {
            step.status = StepStatus::Succeeded;
            if let Some(ref backup_path) = result.rollback_hint.backup_path {
                backups.push(BackupRecord {
                    original_path: match &step.action {
                        ActionType::EditFile(e) => e.path.clone(),
                        ActionType::WriteFile(w) => w.path.clone(),
                        ActionType::DeleteFile(d) => d.path.clone(),
                        _ => PathBuf::new(),
                    },
                    backup_path: backup_path.clone(),
                    hash: result.rollback_hint.prior_state.clone().unwrap_or_default(),
                    created_at: Utc::now(),
                });
            }
            verifications.push(VerificationRecord {
                step_id: step.step_id.clone(),
                check_type: "post_execution".to_string(),
                passed: true,
                details: result.summary.clone(),
            });
        } else {
            step.status = StepStatus::Failed;
            all_success = false;
            verifications.push(VerificationRecord {
                step_id: step.step_id.clone(),
                check_type: "post_execution".to_string(),
                passed: false,
                details: result.error.clone().unwrap_or_default(),
            });
        }

        step_results.push(result);
    }

    plan.status = if all_success {
        PlanStatus::Completed
    } else {
        PlanStatus::PartiallyCompleted
    };

    let completed_at = Utc::now();
    let duration_ms = start.elapsed().as_millis() as u64;

    // Build rollback record
    let rollback_steps: Vec<RollbackStepRecord> = plan
        .steps
        .iter()
        .zip(step_results.iter())
        .map(|(step, result)| RollbackStepRecord {
            step_id: step.step_id.clone(),
            action_type: match &step.action {
                ActionType::EditFile(_) => "edit_file".to_string(),
                ActionType::WriteFile(_) => "write_file".to_string(),
                ActionType::DeleteFile(_) => "delete_file".to_string(),
                ActionType::Systemd(_) => "systemd".to_string(),
                ActionType::Pacman(_) => "pacman".to_string(),
            },
            target: match &step.action {
                ActionType::EditFile(e) => e.path.display().to_string(),
                ActionType::WriteFile(w) => w.path.display().to_string(),
                ActionType::DeleteFile(d) => d.path.display().to_string(),
                ActionType::Systemd(s) => s.unit.clone(),
                ActionType::Pacman(p) => p.packages.join(", "),
            },
            rollback_command: result.rollback_hint.command.clone(),
            backup_path: result.rollback_hint.backup_path.clone(),
            prior_state: result.rollback_hint.prior_state.clone(),
        })
        .collect();

    let restore_instructions: Vec<String> = step_results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.rollback_hint.instructions.clone())
        .collect();

    let rollback_record = RollbackRecord {
        case_id: plan.case_id.clone(),
        plan_id: plan.plan_id.clone(),
        steps: rollback_steps,
        created_at: Utc::now(),
        backups,
        restore_instructions,
        verifications,
    };

    // Save rollback record
    let cases_dir = Path::new(CASES_DIR);
    if let Err(e) = rollback_record.save(cases_dir) {
        eprintln!("Warning: Failed to save rollback record: {}", e);
    }

    Ok(ActionResult {
        plan_id: plan.plan_id.clone(),
        case_id: plan.case_id.clone(),
        success: all_success,
        step_results,
        rollback_record: Some(rollback_record),
        started_at,
        completed_at,
        duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash("hello world");
        assert_eq!(hash.len(), 64); // SHA256 hex is 64 chars
    }

    #[test]
    fn test_apply_edit_intent_append() {
        let content = "line1\nline2\n";
        let intent = EditIntent::Append {
            lines: vec!["line3".to_string()],
        };
        let result = apply_edit_intent(content, &intent).unwrap();
        assert!(result.contains("line3"));
    }

    #[test]
    fn test_apply_edit_intent_set_key_value() {
        let content = "key1=value1\nkey2=value2\n";
        let intent = EditIntent::SetKeyValue {
            key: "key1".to_string(),
            value: "newvalue".to_string(),
            separator: "=".to_string(),
        };
        let result = apply_edit_intent(content, &intent).unwrap();
        assert!(result.contains("key1=newvalue"));
        assert!(!result.contains("key1=value1"));
    }

    #[test]
    fn test_validate_confirmation() {
        let mut plan = ActionPlan::new("test", "Test plan");
        plan.confirmation_phrase = CONFIRM_MEDIUM.to_string();

        assert!(validate_confirmation(&plan, CONFIRM_MEDIUM).is_ok());
        assert!(validate_confirmation(&plan, "wrong").is_err());
    }
}
