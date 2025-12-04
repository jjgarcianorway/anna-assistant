//! File Edit Tool Executors v0.0.50
//!
//! Executor implementations for user file edit tools:
//! - file_edit_preview_v1: Preview changes without applying
//! - file_edit_apply_v1: Apply changes with backup
//! - file_edit_rollback_v1: Rollback changes from case_id

use crate::tools::ToolResult;
use crate::user_file_mutation::{
    UserFileEditAction, EditMode,
    generate_edit_preview, apply_edit, execute_rollback,
    generate_mutation_case_id,
};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Helper to extract string from Value
fn get_string_param(params: &HashMap<String, Value>, key: &str) -> Option<String> {
    params.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
}

// =============================================================================
// Preview Tool Executor
// =============================================================================

/// Execute file_edit_preview_v1 tool
pub fn execute_file_edit_preview_v1(
    parameters: &HashMap<String, Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    // Extract parameters
    let path = match get_string_param(parameters, "path") {
        Some(p) => p,
        None => {
            return ToolResult {
                tool_name: "file_edit_preview_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({ "error": "Missing required parameter: path" }),
                human_summary: "Error: missing 'path' parameter".to_string(),
                success: false,
                error: Some("Missing required parameter: path".to_string()),
                timestamp,
            };
        }
    };

    let mode_str = get_string_param(parameters, "mode").unwrap_or_else(|| "append_line".to_string());
    let mode = match mode_str.as_str() {
        "append_line" => EditMode::AppendLine,
        "set_key_value" => EditMode::SetKeyValue,
        _ => {
            let err = format!("Invalid mode: {}. Use append_line or set_key_value", mode_str);
            return ToolResult {
                tool_name: "file_edit_preview_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({ "error": &err }),
                human_summary: format!("Error: invalid mode '{}'", mode_str),
                success: false,
                error: Some(err),
                timestamp,
            };
        }
    };

    // Build action based on mode
    let action = match mode {
        EditMode::AppendLine => {
            let line = match get_string_param(parameters, "line") {
                Some(l) => l,
                None => {
                    return ToolResult {
                        tool_name: "file_edit_preview_v1".to_string(),
                        evidence_id: evidence_id.to_string(),
                        data: json!({ "error": "append_line mode requires 'line' parameter" }),
                        human_summary: "Error: missing 'line' parameter for append_line mode".to_string(),
                        success: false,
                        error: Some("append_line mode requires 'line' parameter".to_string()),
                        timestamp,
                    };
                }
            };
            let target_user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            UserFileEditAction::append_line(&path, &line, &target_user)
        }
        EditMode::SetKeyValue => {
            let key = match get_string_param(parameters, "key") {
                Some(k) => k,
                None => {
                    return ToolResult {
                        tool_name: "file_edit_preview_v1".to_string(),
                        evidence_id: evidence_id.to_string(),
                        data: json!({ "error": "set_key_value mode requires 'key' parameter" }),
                        human_summary: "Error: missing 'key' parameter for set_key_value mode".to_string(),
                        success: false,
                        error: Some("set_key_value mode requires 'key' parameter".to_string()),
                        timestamp,
                    };
                }
            };
            let value = match get_string_param(parameters, "value") {
                Some(v) => v,
                None => {
                    return ToolResult {
                        tool_name: "file_edit_preview_v1".to_string(),
                        evidence_id: evidence_id.to_string(),
                        data: json!({ "error": "set_key_value mode requires 'value' parameter" }),
                        human_summary: "Error: missing 'value' parameter for set_key_value mode".to_string(),
                        success: false,
                        error: Some("set_key_value mode requires 'value' parameter".to_string()),
                        timestamp,
                    };
                }
            };
            let separator = get_string_param(parameters, "separator").unwrap_or_else(|| "=".to_string());
            let target_user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            UserFileEditAction::set_key_value(&path, &key, &value, &separator, &target_user)
        }
    };

    // Generate preview
    match generate_edit_preview(&action) {
        Ok(preview) => {
            let human_summary = if preview.would_change {
                format!(
                    "Preview: {}\nPath: {}\nFile exists: {}\n\nDiff:\n{}",
                    preview.change_description,
                    preview.path,
                    preview.exists,
                    preview.diff_unified
                )
            } else {
                format!(
                    "No changes needed: {}\nPath: {}",
                    preview.change_description,
                    preview.path
                )
            };

            ToolResult {
                tool_name: "file_edit_preview_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "preview": {
                        "path": preview.path,
                        "exists": preview.exists,
                        "would_change": preview.would_change,
                        "change_description": preview.change_description,
                        "diff_unified": preview.diff_unified,
                        "current_line_count": preview.current_line_count,
                        "before_hash": preview.before_hash,
                        "policy_allowed": preview.policy_result.allowed,
                        "policy_reason": preview.policy_result.reason,
                    }
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => {
            ToolResult {
                tool_name: "file_edit_preview_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({ "error": &e }),
                human_summary: format!("Preview failed: {}", e),
                success: false,
                error: Some(e),
                timestamp,
            }
        }
    }
}

// =============================================================================
// Apply Tool Executor
// =============================================================================

/// Execute file_edit_apply_v1 tool
pub fn execute_file_edit_apply_v1(
    parameters: &HashMap<String, Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    // Extract parameters
    let path = match get_string_param(parameters, "path") {
        Some(p) => p,
        None => {
            return ToolResult {
                tool_name: "file_edit_apply_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({ "error": "Missing required parameter: path" }),
                human_summary: "Error: missing 'path' parameter".to_string(),
                success: false,
                error: Some("Missing required parameter: path".to_string()),
                timestamp,
            };
        }
    };

    let mode_str = get_string_param(parameters, "mode").unwrap_or_else(|| "append_line".to_string());
    let mode = match mode_str.as_str() {
        "append_line" => EditMode::AppendLine,
        "set_key_value" => EditMode::SetKeyValue,
        _ => {
            let err = format!("Invalid mode: {}", mode_str);
            return ToolResult {
                tool_name: "file_edit_apply_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({ "error": &err }),
                human_summary: format!("Error: invalid mode '{}'", mode_str),
                success: false,
                error: Some(err),
                timestamp,
            };
        }
    };

    // Check confirmation phrase
    let confirmation = get_string_param(parameters, "confirmation").unwrap_or_default();
    if confirmation != crate::user_file_mutation::USER_FILE_CONFIRMATION {
        let err = format!(
            "Invalid confirmation phrase. Required: '{}'",
            crate::user_file_mutation::USER_FILE_CONFIRMATION
        );
        return ToolResult {
            tool_name: "file_edit_apply_v1".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "error": "Invalid confirmation phrase",
                "required": crate::user_file_mutation::USER_FILE_CONFIRMATION,
                "received": confirmation,
            }),
            human_summary: format!(
                "Apply rejected: confirmation phrase must be '{}'",
                crate::user_file_mutation::USER_FILE_CONFIRMATION
            ),
            success: false,
            error: Some(err),
            timestamp,
        };
    }

    // Check that preview_id was provided (Junior will verify this exists)
    let preview_id = match get_string_param(parameters, "preview_id") {
        Some(id) => id,
        None => {
            return ToolResult {
                tool_name: "file_edit_apply_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({ "error": "Missing preview_id - must preview before apply" }),
                human_summary: "Error: must call file_edit_preview_v1 first".to_string(),
                success: false,
                error: Some("Missing preview_id - must preview before apply".to_string()),
                timestamp,
            };
        }
    };

    // Build action based on mode
    let action = match mode {
        EditMode::AppendLine => {
            let line = match get_string_param(parameters, "line") {
                Some(l) => l,
                None => {
                    return ToolResult {
                        tool_name: "file_edit_apply_v1".to_string(),
                        evidence_id: evidence_id.to_string(),
                        data: json!({ "error": "append_line mode requires 'line' parameter" }),
                        human_summary: "Error: missing 'line' parameter".to_string(),
                        success: false,
                        error: Some("append_line mode requires 'line' parameter".to_string()),
                        timestamp,
                    };
                }
            };
            let target_user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            UserFileEditAction::append_line(&path, &line, &target_user)
        }
        EditMode::SetKeyValue => {
            let key = match get_string_param(parameters, "key") {
                Some(k) => k,
                None => {
                    return ToolResult {
                        tool_name: "file_edit_apply_v1".to_string(),
                        evidence_id: evidence_id.to_string(),
                        data: json!({ "error": "set_key_value mode requires 'key' parameter" }),
                        human_summary: "Error: missing 'key' parameter".to_string(),
                        success: false,
                        error: Some("set_key_value mode requires 'key' parameter".to_string()),
                        timestamp,
                    };
                }
            };
            let value = match get_string_param(parameters, "value") {
                Some(v) => v,
                None => {
                    return ToolResult {
                        tool_name: "file_edit_apply_v1".to_string(),
                        evidence_id: evidence_id.to_string(),
                        data: json!({ "error": "set_key_value mode requires 'value' parameter" }),
                        human_summary: "Error: missing 'value' parameter".to_string(),
                        success: false,
                        error: Some("set_key_value mode requires 'value' parameter".to_string()),
                        timestamp,
                    };
                }
            };
            let separator = get_string_param(parameters, "separator").unwrap_or_else(|| "=".to_string());
            let target_user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            UserFileEditAction::set_key_value(&path, &key, &value, &separator, &target_user)
        }
    };

    // Generate case_id
    let case_id = generate_mutation_case_id();

    // Apply the edit
    match apply_edit(&action, &case_id) {
        Ok(result) => {
            let human_summary = if result.verified {
                format!(
                    "Applied successfully:\n  Path: {}\n  Case ID: {}\n  Verified: yes\n  {}\n\nRollback: {}",
                    result.path,
                    result.case_id,
                    result.verify_message,
                    result.rollback_command
                )
            } else {
                format!(
                    "Applied with verification warning:\n  Path: {}\n  Case ID: {}\n  Warning: {}\n\nRollback: {}",
                    result.path,
                    result.case_id,
                    result.verify_message,
                    result.rollback_command
                )
            };

            ToolResult {
                tool_name: "file_edit_apply_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "apply": {
                        "success": result.success,
                        "case_id": result.case_id,
                        "path": result.path,
                        "backup_path": result.backup_path.display().to_string(),
                        "before_hash": result.before_hash,
                        "after_hash": result.after_hash,
                        "verified": result.verified,
                        "verify_message": result.verify_message,
                        "rollback_command": result.rollback_command,
                        "preview_id": preview_id,
                    }
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => {
            ToolResult {
                tool_name: "file_edit_apply_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({ "error": &e }),
                human_summary: format!("Apply failed: {}", e),
                success: false,
                error: Some(e),
                timestamp,
            }
        }
    }
}

// =============================================================================
// Rollback Tool Executor
// =============================================================================

/// Execute file_edit_rollback_v1 tool
pub fn execute_file_edit_rollback_v1(
    parameters: &HashMap<String, Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    // Extract case_id
    let case_id = match get_string_param(parameters, "case_id") {
        Some(id) => id,
        None => {
            return ToolResult {
                tool_name: "file_edit_rollback_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({ "error": "Missing required parameter: case_id" }),
                human_summary: "Error: missing 'case_id' parameter".to_string(),
                success: false,
                error: Some("Missing required parameter: case_id".to_string()),
                timestamp,
            };
        }
    };

    // Execute rollback
    match execute_rollback(&case_id) {
        Ok(result) => {
            let human_summary = if result.success {
                format!(
                    "Rollback successful:\n  Case ID: {}\n  Path: {}\n  Hashes match: {}",
                    result.case_id,
                    result.path,
                    result.hashes_match
                )
            } else {
                format!(
                    "Rollback completed with warning:\n  Case ID: {}\n  Path: {}\n  Error: {}",
                    result.case_id,
                    result.path,
                    result.error.as_deref().unwrap_or("unknown")
                )
            };

            ToolResult {
                tool_name: "file_edit_rollback_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "rollback": {
                        "success": result.success,
                        "case_id": result.case_id,
                        "path": result.path,
                        "restored_hash": result.restored_hash,
                        "backup_hash": result.backup_hash,
                        "hashes_match": result.hashes_match,
                    }
                }),
                human_summary,
                success: result.success,
                error: result.error,
                timestamp,
            }
        }
        Err(e) => {
            ToolResult {
                tool_name: "file_edit_rollback_v1".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({ "error": &e }),
                human_summary: format!("Rollback failed: {}", e),
                success: false,
                error: Some(e),
                timestamp,
            }
        }
    }
}
