//! Mutation Executor for Anna v0.0.9
//!
//! Executes approved mutation operations with:
//! - Automatic backup before changes
//! - Structured logging
//! - Rollback support
//! - Package management with helper provenance tracking

use crate::mutation_tools::{
    MutationError, MutationPlan, MutationRequest, MutationResult, MutationToolCatalog,
    RollbackInfo, FileEditOp, ServiceState, get_service_state,
    validate_mutation_request, MEDIUM_RISK_CONFIRMATION,
};
use crate::rollback::{RollbackManager, MutationType};
use crate::helpers::{HelpersManifest, InstalledBy, is_package_present, get_package_version};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Execute a mutation plan (all mutations in sequence)
pub fn execute_mutation_plan(
    plan: &MutationPlan,
    catalog: &MutationToolCatalog,
    rollback_manager: &RollbackManager,
) -> Result<Vec<MutationResult>, MutationError> {
    // Verify plan is approved
    if !plan.is_approved_for_execution() {
        return Err(MutationError::JuniorReliabilityTooLow {
            score: plan.junior_reliability,
            required: 70,
        });
    }

    let mut results = Vec::new();

    for request in &plan.mutations {
        let result = execute_mutation(request, catalog, rollback_manager)?;
        let success = result.success;
        results.push(result);

        // Stop on first failure
        if !success {
            break;
        }
    }

    Ok(results)
}

/// Execute a single mutation
pub fn execute_mutation(
    request: &MutationRequest,
    catalog: &MutationToolCatalog,
    rollback_manager: &RollbackManager,
) -> Result<MutationResult, MutationError> {
    // Validate the request
    let tool = validate_mutation_request(request, catalog)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Execute based on tool type
    let result = match request.tool_name.as_str() {
        "edit_file_lines" => execute_file_edit(request, rollback_manager, timestamp),
        "systemd_restart" => execute_systemd_restart(request, rollback_manager, timestamp),
        "systemd_reload" => execute_systemd_reload(request, rollback_manager, timestamp),
        "systemd_enable_now" => execute_systemd_enable_now(request, rollback_manager, timestamp),
        "systemd_disable_now" => execute_systemd_disable_now(request, rollback_manager, timestamp),
        "systemd_daemon_reload" => execute_systemd_daemon_reload(request, rollback_manager, timestamp),
        // v0.0.9: Package management
        "package_install" => execute_package_install(request, rollback_manager, timestamp),
        "package_remove" => execute_package_remove(request, rollback_manager, timestamp),
        _ => Err(MutationError::ToolNotAllowed(request.tool_name.clone())),
    };

    result
}

// =============================================================================
// File Edit Execution
// =============================================================================

fn execute_file_edit(
    request: &MutationRequest,
    rollback_manager: &RollbackManager,
    timestamp: u64,
) -> Result<MutationResult, MutationError> {
    let path_str = request.parameters.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MutationError::Other("Missing 'path' parameter".to_string()))?;

    let path = Path::new(path_str);

    let operations: Vec<FileEditOp> = request.parameters.get("operations")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or_else(|| MutationError::Other("Missing or invalid 'operations' parameter".to_string()))?;

    // Create backup first
    let backup_path = rollback_manager
        .backup_file(path, &request.request_id)
        .map_err(|e| MutationError::Other(format!("Backup failed: {}", e)))?;

    // Hash before edit
    let before_hash = RollbackManager::hash_file(path)
        .map_err(|e| MutationError::Other(format!("Hash failed: {}", e)))?;

    // Read current content
    let content = fs::read_to_string(path)
        .map_err(|e| MutationError::Other(format!("Read failed: {}", e)))?;

    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    // Apply operations
    for op in &operations {
        apply_file_edit_op(&mut lines, op)?;
    }

    // Write new content
    let new_content = lines.join("\n");
    // Preserve trailing newline if original had one
    let new_content = if content.ends_with('\n') && !new_content.ends_with('\n') {
        format!("{}\n", new_content)
    } else {
        new_content
    };

    fs::write(path, &new_content)
        .map_err(|e| MutationError::Other(format!("Write failed: {}", e)))?;

    // Hash after edit
    let after_hash = RollbackManager::hash_file(path)
        .map_err(|e| MutationError::Other(format!("Hash failed: {}", e)))?;

    // Log the mutation
    rollback_manager
        .log_file_edit(
            &request.request_id,
            &request.evidence_ids,
            path,
            &backup_path,
            &before_hash,
            &after_hash,
            &operations,
            true,
            None,
        )
        .map_err(|e| MutationError::Other(format!("Log failed: {}", e)))?;

    let rollback_info = rollback_manager.file_rollback_info(path, &backup_path);

    Ok(MutationResult {
        tool_name: "edit_file_lines".to_string(),
        success: true,
        error: None,
        human_summary: format!(
            "Edited {} ({} operation(s)). Backup at {}",
            path.display(),
            operations.len(),
            backup_path.display()
        ),
        rollback_info: Some(rollback_info),
        request_id: request.request_id.clone(),
        timestamp,
    })
}

fn apply_file_edit_op(lines: &mut Vec<String>, op: &FileEditOp) -> Result<(), MutationError> {
    match op {
        FileEditOp::InsertLine { line_number, content } => {
            if *line_number > lines.len() {
                return Err(MutationError::Other(format!(
                    "Line number {} out of range (file has {} lines)",
                    line_number, lines.len()
                )));
            }
            lines.insert(*line_number, content.clone());
        }
        FileEditOp::ReplaceLine { line_number, content } => {
            if *line_number >= lines.len() {
                return Err(MutationError::Other(format!(
                    "Line number {} out of range (file has {} lines)",
                    line_number, lines.len()
                )));
            }
            lines[*line_number] = content.clone();
        }
        FileEditOp::DeleteLine { line_number } => {
            if *line_number >= lines.len() {
                return Err(MutationError::Other(format!(
                    "Line number {} out of range (file has {} lines)",
                    line_number, lines.len()
                )));
            }
            lines.remove(*line_number);
        }
        FileEditOp::AppendLine { content } => {
            lines.push(content.clone());
        }
        FileEditOp::ReplaceText { pattern, replacement } => {
            for line in lines.iter_mut() {
                *line = line.replace(pattern, replacement);
            }
        }
    }
    Ok(())
}

// =============================================================================
// Systemd Operations
// =============================================================================

fn execute_systemd_restart(
    request: &MutationRequest,
    rollback_manager: &RollbackManager,
    timestamp: u64,
) -> Result<MutationResult, MutationError> {
    let service = request.parameters.get("service")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MutationError::Other("Missing 'service' parameter".to_string()))?;

    // Get prior state
    let prior_state = get_service_state(service);

    // Execute restart
    let output = Command::new("systemctl")
        .args(["restart", service])
        .output()
        .map_err(|e| MutationError::Other(format!("systemctl failed: {}", e)))?;

    let success = output.status.success();
    let error = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).to_string())
    };

    // Log the operation
    rollback_manager
        .log_systemd_operation(
            &request.request_id,
            &request.evidence_ids,
            MutationType::SystemdRestart,
            service,
            "restart",
            prior_state.clone(),
            success,
            error.as_deref(),
        )
        .ok();

    let rollback_info = rollback_manager.systemd_rollback_info("restart", service, prior_state.as_ref());

    Ok(MutationResult {
        tool_name: "systemd_restart".to_string(),
        success,
        error,
        human_summary: if success {
            format!("Restarted service {}", service)
        } else {
            format!("Failed to restart service {}", service)
        },
        rollback_info: Some(rollback_info),
        request_id: request.request_id.clone(),
        timestamp,
    })
}

fn execute_systemd_reload(
    request: &MutationRequest,
    rollback_manager: &RollbackManager,
    timestamp: u64,
) -> Result<MutationResult, MutationError> {
    let service = request.parameters.get("service")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MutationError::Other("Missing 'service' parameter".to_string()))?;

    let prior_state = get_service_state(service);

    let output = Command::new("systemctl")
        .args(["reload", service])
        .output()
        .map_err(|e| MutationError::Other(format!("systemctl failed: {}", e)))?;

    let success = output.status.success();
    let error = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).to_string())
    };

    rollback_manager
        .log_systemd_operation(
            &request.request_id,
            &request.evidence_ids,
            MutationType::SystemdReload,
            service,
            "reload",
            prior_state.clone(),
            success,
            error.as_deref(),
        )
        .ok();

    let rollback_info = rollback_manager.systemd_rollback_info("reload", service, prior_state.as_ref());

    Ok(MutationResult {
        tool_name: "systemd_reload".to_string(),
        success,
        error,
        human_summary: if success {
            format!("Reloaded service {} configuration", service)
        } else {
            format!("Failed to reload service {}", service)
        },
        rollback_info: Some(rollback_info),
        request_id: request.request_id.clone(),
        timestamp,
    })
}

fn execute_systemd_enable_now(
    request: &MutationRequest,
    rollback_manager: &RollbackManager,
    timestamp: u64,
) -> Result<MutationResult, MutationError> {
    let service = request.parameters.get("service")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MutationError::Other("Missing 'service' parameter".to_string()))?;

    let prior_state = get_service_state(service);

    let output = Command::new("systemctl")
        .args(["enable", "--now", service])
        .output()
        .map_err(|e| MutationError::Other(format!("systemctl failed: {}", e)))?;

    let success = output.status.success();
    let error = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).to_string())
    };

    rollback_manager
        .log_systemd_operation(
            &request.request_id,
            &request.evidence_ids,
            MutationType::SystemdEnableNow,
            service,
            "enable_now",
            prior_state.clone(),
            success,
            error.as_deref(),
        )
        .ok();

    let rollback_info = rollback_manager.systemd_rollback_info("enable_now", service, prior_state.as_ref());

    Ok(MutationResult {
        tool_name: "systemd_enable_now".to_string(),
        success,
        error,
        human_summary: if success {
            format!("Enabled and started service {}", service)
        } else {
            format!("Failed to enable service {}", service)
        },
        rollback_info: Some(rollback_info),
        request_id: request.request_id.clone(),
        timestamp,
    })
}

fn execute_systemd_disable_now(
    request: &MutationRequest,
    rollback_manager: &RollbackManager,
    timestamp: u64,
) -> Result<MutationResult, MutationError> {
    let service = request.parameters.get("service")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MutationError::Other("Missing 'service' parameter".to_string()))?;

    let prior_state = get_service_state(service);

    let output = Command::new("systemctl")
        .args(["disable", "--now", service])
        .output()
        .map_err(|e| MutationError::Other(format!("systemctl failed: {}", e)))?;

    let success = output.status.success();
    let error = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).to_string())
    };

    rollback_manager
        .log_systemd_operation(
            &request.request_id,
            &request.evidence_ids,
            MutationType::SystemdDisableNow,
            service,
            "disable_now",
            prior_state.clone(),
            success,
            error.as_deref(),
        )
        .ok();

    let rollback_info = rollback_manager.systemd_rollback_info("disable_now", service, prior_state.as_ref());

    Ok(MutationResult {
        tool_name: "systemd_disable_now".to_string(),
        success,
        error,
        human_summary: if success {
            format!("Disabled and stopped service {}", service)
        } else {
            format!("Failed to disable service {}", service)
        },
        rollback_info: Some(rollback_info),
        request_id: request.request_id.clone(),
        timestamp,
    })
}

fn execute_systemd_daemon_reload(
    request: &MutationRequest,
    rollback_manager: &RollbackManager,
    timestamp: u64,
) -> Result<MutationResult, MutationError> {
    let output = Command::new("systemctl")
        .args(["daemon-reload"])
        .output()
        .map_err(|e| MutationError::Other(format!("systemctl failed: {}", e)))?;

    let success = output.status.success();
    let error = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).to_string())
    };

    rollback_manager
        .log_systemd_operation(
            &request.request_id,
            &request.evidence_ids,
            MutationType::SystemdDaemonReload,
            "systemd",
            "daemon_reload",
            None,
            success,
            error.as_deref(),
        )
        .ok();

    let rollback_info = rollback_manager.systemd_rollback_info("daemon_reload", "systemd", None);

    Ok(MutationResult {
        tool_name: "systemd_daemon_reload".to_string(),
        success,
        error,
        human_summary: if success {
            "Reloaded systemd daemon configuration".to_string()
        } else {
            "Failed to reload systemd daemon".to_string()
        },
        rollback_info: Some(rollback_info),
        request_id: request.request_id.clone(),
        timestamp,
    })
}

// =============================================================================
// Package Management Operations (v0.0.9)
// =============================================================================

fn execute_package_install(
    request: &MutationRequest,
    rollback_manager: &RollbackManager,
    timestamp: u64,
) -> Result<MutationResult, MutationError> {
    let package = request.parameters.get("package")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MutationError::Other("Missing 'package' parameter".to_string()))?;

    let reason = request.parameters.get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Anna helper");

    // Check if already installed
    if is_package_present(package) {
        return Err(MutationError::PackageAlreadyInstalled(package.to_string()));
    }

    // Execute pacman install
    let output = Command::new("pacman")
        .args(["-S", "--noconfirm", package])
        .output()
        .map_err(|e| MutationError::Other(format!("pacman failed to start: {}", e)))?;

    let success = output.status.success();
    let error = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).to_string())
    };

    if success {
        // Track as Anna-installed helper
        let mut manifest = HelpersManifest::load();
        let version = get_package_version(package);
        manifest.record_anna_install(package, reason, version.clone());
        let _ = manifest.save();

        // Log the operation
        rollback_manager
            .log_package_operation(
                &request.request_id,
                &request.evidence_ids,
                MutationType::PackageInstall,
                package,
                version.as_deref(),
                reason,
                true,
                None,
            )
            .ok();

        let rollback_info = RollbackInfo {
            backup_path: None,
            rollback_command: Some(format!("pacman -R {}", package)),
            rollback_instructions: format!(
                "To remove this package:\n  sudo pacman -R {}",
                package
            ),
            prior_state: Some("not installed".to_string()),
        };

        Ok(MutationResult {
            tool_name: "package_install".to_string(),
            success: true,
            error: None,
            human_summary: format!(
                "Installed {} (tracked as Anna-installed helper)",
                package
            ),
            rollback_info: Some(rollback_info),
            request_id: request.request_id.clone(),
            timestamp,
        })
    } else {
        rollback_manager
            .log_package_operation(
                &request.request_id,
                &request.evidence_ids,
                MutationType::PackageInstall,
                package,
                None,
                reason,
                false,
                error.as_deref(),
            )
            .ok();

        Err(MutationError::PackageInstallFailed {
            package: package.to_string(),
            reason: error.unwrap_or_else(|| "Unknown error".to_string()),
        })
    }
}

fn execute_package_remove(
    request: &MutationRequest,
    rollback_manager: &RollbackManager,
    timestamp: u64,
) -> Result<MutationResult, MutationError> {
    let package = request.parameters.get("package")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MutationError::Other("Missing 'package' parameter".to_string()))?;

    // Check if installed
    if !is_package_present(package) {
        return Err(MutationError::PackageNotInstalled(package.to_string()));
    }

    // Check provenance - only allow removal of Anna-installed packages
    let manifest = HelpersManifest::load();
    if let Some(helper) = manifest.get(package) {
        if helper.installed_by != InstalledBy::Anna {
            return Err(MutationError::PackageNotRemovable {
                package: package.to_string(),
                reason: format!(
                    "Package was installed by {} - only Anna-installed packages can be removed",
                    helper.installed_by
                ),
            });
        }
    } else {
        // Not tracked = assume user installed
        return Err(MutationError::PackageNotRemovable {
            package: package.to_string(),
            reason: "Package not tracked as Anna-installed".to_string(),
        });
    }

    let prior_version = get_package_version(package);

    // Execute pacman remove
    let output = Command::new("pacman")
        .args(["-R", "--noconfirm", package])
        .output()
        .map_err(|e| MutationError::Other(format!("pacman failed to start: {}", e)))?;

    let success = output.status.success();
    let error = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).to_string())
    };

    // Log the operation
    rollback_manager
        .log_package_operation(
            &request.request_id,
            &request.evidence_ids,
            MutationType::PackageRemove,
            package,
            prior_version.as_deref(),
            "Anna-initiated removal",
            success,
            error.as_deref(),
        )
        .ok();

    if success {
        // Update helper state (mark as not present but preserve history)
        let mut manifest = HelpersManifest::load();
        if let Some(helper) = manifest.helpers.get_mut(package) {
            helper.present = false;
            helper.version = None;
        }
        let _ = manifest.save();

        let rollback_info = RollbackInfo {
            backup_path: None,
            rollback_command: Some(format!("pacman -S {}", package)),
            rollback_instructions: format!(
                "To reinstall this package:\n  sudo pacman -S {}",
                package
            ),
            prior_state: prior_version.map(|v| format!("installed ({})", v)),
        };

        Ok(MutationResult {
            tool_name: "package_remove".to_string(),
            success: true,
            error: None,
            human_summary: format!("Removed {} (was Anna-installed)", package),
            rollback_info: Some(rollback_info),
            request_id: request.request_id.clone(),
            timestamp,
        })
    } else {
        Err(MutationError::PackageRemoveFailed {
            package: package.to_string(),
            reason: error.unwrap_or_else(|| "Unknown error".to_string()),
        })
    }
}

/// Create a mutation request for package install
pub fn create_package_install_request(
    package: &str,
    reason: &str,
    evidence_ids: Vec<String>,
    confirmation: Option<String>,
) -> MutationRequest {
    let mut parameters = HashMap::new();
    parameters.insert("package".to_string(), serde_json::json!(package));
    parameters.insert("reason".to_string(), serde_json::json!(reason));

    MutationRequest {
        tool_name: "package_install".to_string(),
        parameters,
        confirmation_token: confirmation,
        evidence_ids,
        request_id: generate_request_id(),
    }
}

/// Create a mutation request for package remove
pub fn create_package_remove_request(
    package: &str,
    evidence_ids: Vec<String>,
    confirmation: Option<String>,
) -> MutationRequest {
    let mut parameters = HashMap::new();
    parameters.insert("package".to_string(), serde_json::json!(package));

    MutationRequest {
        tool_name: "package_remove".to_string(),
        parameters,
        confirmation_token: confirmation,
        evidence_ids,
        request_id: generate_request_id(),
    }
}

// =============================================================================
// Helper: Generate request ID
// =============================================================================

/// Generate a unique request ID
pub fn generate_request_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("mut_{}", ts)
}

/// Create a mutation request for file edit
pub fn create_file_edit_request(
    path: &str,
    operations: Vec<FileEditOp>,
    evidence_ids: Vec<String>,
    confirmation: Option<String>,
) -> MutationRequest {
    let mut parameters = HashMap::new();
    parameters.insert("path".to_string(), serde_json::json!(path));
    parameters.insert("operations".to_string(), serde_json::to_value(operations).unwrap());

    MutationRequest {
        tool_name: "edit_file_lines".to_string(),
        parameters,
        confirmation_token: confirmation,
        evidence_ids,
        request_id: generate_request_id(),
    }
}

/// Create a mutation request for systemd operation
pub fn create_systemd_request(
    operation: &str,
    service: &str,
    evidence_ids: Vec<String>,
    confirmation: Option<String>,
) -> MutationRequest {
    let tool_name = format!("systemd_{}", operation);
    let mut parameters = HashMap::new();

    if operation != "daemon_reload" {
        parameters.insert("service".to_string(), serde_json::json!(service));
    }

    MutationRequest {
        tool_name,
        parameters,
        confirmation_token: confirmation,
        evidence_ids,
        request_id: generate_request_id(),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_edit_insert() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.conf");
        fs::write(&test_file, "line1\nline2\nline3\n").unwrap();

        let rollback_dir = temp_dir.path().join("rollback");
        let manager = RollbackManager::with_base_dir(rollback_dir);
        let catalog = MutationToolCatalog::new();

        let request = MutationRequest {
            tool_name: "edit_file_lines".to_string(),
            parameters: {
                let mut p = HashMap::new();
                p.insert("path".to_string(), serde_json::json!(test_file.to_string_lossy()));
                p.insert("operations".to_string(), serde_json::json!([
                    { "InsertLine": { "line_number": 1, "content": "inserted" } }
                ]));
                p
            },
            confirmation_token: Some(MEDIUM_RISK_CONFIRMATION.to_string()),
            evidence_ids: vec!["E1".to_string()],
            request_id: "test123".to_string(),
        };

        let result = execute_mutation(&request, &catalog, &manager).unwrap();
        assert!(result.success);

        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("inserted"));
    }

    #[test]
    fn test_file_edit_replace() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.conf");
        fs::write(&test_file, "line1\nline2\nline3\n").unwrap();

        let rollback_dir = temp_dir.path().join("rollback");
        let manager = RollbackManager::with_base_dir(rollback_dir);
        let catalog = MutationToolCatalog::new();

        let request = MutationRequest {
            tool_name: "edit_file_lines".to_string(),
            parameters: {
                let mut p = HashMap::new();
                p.insert("path".to_string(), serde_json::json!(test_file.to_string_lossy()));
                p.insert("operations".to_string(), serde_json::json!([
                    { "ReplaceLine": { "line_number": 1, "content": "replaced" } }
                ]));
                p
            },
            confirmation_token: Some(MEDIUM_RISK_CONFIRMATION.to_string()),
            evidence_ids: vec!["E1".to_string()],
            request_id: "test124".to_string(),
        };

        let result = execute_mutation(&request, &catalog, &manager).unwrap();
        assert!(result.success);

        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("replaced"));
        assert!(!content.contains("line2"));
    }

    #[test]
    fn test_file_edit_creates_backup() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.conf");
        fs::write(&test_file, "original content\n").unwrap();

        let rollback_dir = temp_dir.path().join("rollback");
        let manager = RollbackManager::with_base_dir(rollback_dir.clone());
        let catalog = MutationToolCatalog::new();

        let request = MutationRequest {
            tool_name: "edit_file_lines".to_string(),
            parameters: {
                let mut p = HashMap::new();
                p.insert("path".to_string(), serde_json::json!(test_file.to_string_lossy()));
                p.insert("operations".to_string(), serde_json::json!([
                    { "AppendLine": { "content": "new line" } }
                ]));
                p
            },
            confirmation_token: Some(MEDIUM_RISK_CONFIRMATION.to_string()),
            evidence_ids: vec!["E1".to_string()],
            request_id: "test125".to_string(),
        };

        let result = execute_mutation(&request, &catalog, &manager).unwrap();
        assert!(result.success);
        assert!(result.rollback_info.is_some());

        let rollback = result.rollback_info.unwrap();
        assert!(rollback.backup_path.is_some());

        // Verify backup exists and has original content
        let backup_path = rollback.backup_path.unwrap();
        assert!(backup_path.exists());
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "original content\n");
    }

    #[test]
    fn test_missing_confirmation_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.conf");
        fs::write(&test_file, "content\n").unwrap();

        let rollback_dir = temp_dir.path().join("rollback");
        let manager = RollbackManager::with_base_dir(rollback_dir);
        let catalog = MutationToolCatalog::new();

        let request = MutationRequest {
            tool_name: "edit_file_lines".to_string(),
            parameters: {
                let mut p = HashMap::new();
                p.insert("path".to_string(), serde_json::json!(test_file.to_string_lossy()));
                p.insert("operations".to_string(), serde_json::json!([]));
                p
            },
            confirmation_token: None, // Missing!
            evidence_ids: vec!["E1".to_string()],
            request_id: "test126".to_string(),
        };

        let result = execute_mutation(&request, &catalog, &manager);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MutationError::MissingConfirmation));
    }

    #[test]
    fn test_wrong_confirmation_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.conf");
        fs::write(&test_file, "content\n").unwrap();

        let rollback_dir = temp_dir.path().join("rollback");
        let manager = RollbackManager::with_base_dir(rollback_dir);
        let catalog = MutationToolCatalog::new();

        let request = MutationRequest {
            tool_name: "edit_file_lines".to_string(),
            parameters: {
                let mut p = HashMap::new();
                p.insert("path".to_string(), serde_json::json!(test_file.to_string_lossy()));
                p.insert("operations".to_string(), serde_json::json!([]));
                p
            },
            confirmation_token: Some("yes".to_string()), // Wrong!
            evidence_ids: vec!["E1".to_string()],
            request_id: "test127".to_string(),
        };

        let result = execute_mutation(&request, &catalog, &manager);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MutationError::WrongConfirmation { .. }));
    }

    #[test]
    fn test_mutation_plan_requires_approval() {
        let mut plan = MutationPlan::new();
        plan.junior_approved = false;
        plan.junior_reliability = 80;

        let catalog = MutationToolCatalog::new();
        let manager = RollbackManager::new();

        let result = execute_mutation_plan(&plan, &catalog, &manager);
        assert!(result.is_err());
    }

    #[test]
    fn test_mutation_plan_requires_reliability() {
        let mut plan = MutationPlan::new();
        plan.junior_approved = true;
        plan.junior_reliability = 50; // Too low

        let catalog = MutationToolCatalog::new();
        let manager = RollbackManager::new();

        let result = execute_mutation_plan(&plan, &catalog, &manager);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MutationError::JuniorReliabilityTooLow { .. }
        ));
    }
}
