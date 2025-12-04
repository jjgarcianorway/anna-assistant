//! Systemd Tool Executors v0.0.51
//!
//! Executor implementations for systemd service action tools:
//! - systemd_service_probe_v1: Probe service state
//! - systemd_service_preview_v1: Preview action without executing
//! - systemd_service_apply_v1: Apply action with rollback
//! - systemd_service_rollback_v1: Rollback by case_id

use crate::tools::ToolResult;
use crate::systemd_action::{ServiceAction, ServiceOperation};
use crate::systemd_probe::probe_service;
use crate::systemd_apply::{preview_service_action, apply_service_action};
use crate::systemd_rollback::{rollback_service_action, generate_service_case_id};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Helper to extract string from Value
fn get_string_param(params: &HashMap<String, Value>, key: &str) -> Option<String> {
    params.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
}

/// Parse operation string to ServiceOperation enum
fn parse_operation(op_str: &str) -> Option<ServiceOperation> {
    match op_str.to_lowercase().as_str() {
        "start" => Some(ServiceOperation::Start),
        "stop" => Some(ServiceOperation::Stop),
        "restart" => Some(ServiceOperation::Restart),
        "enable" => Some(ServiceOperation::Enable),
        "disable" => Some(ServiceOperation::Disable),
        _ => None,
    }
}

/// Create an error ToolResult
fn error_result(tool: &str, evidence_id: &str, error: &str, timestamp: u64) -> ToolResult {
    ToolResult {
        tool_name: tool.to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({ "error": error }),
        human_summary: format!("Error: {}", error),
        success: false,
        error: Some(error.to_string()),
        timestamp,
    }
}

/// Require a string parameter, returning error result if missing
macro_rules! require_param {
    ($params:expr, $key:expr, $tool:expr, $eid:expr, $ts:expr) => {
        match get_string_param($params, $key) {
            Some(v) => v,
            None => return error_result($tool, $eid, &format!("Missing required parameter: {}", $key), $ts),
        }
    };
}

// =============================================================================
// Probe Tool Executor
// =============================================================================

/// Execute systemd_service_probe_v1 tool
pub fn execute_systemd_service_probe_v1(
    parameters: &HashMap<String, Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let service = require_param!(parameters, "service", "systemd_service_probe_v1", evidence_id, timestamp);
    let probe = probe_service(&service);

    let human_summary = if probe.exists {
        format!(
            "Service: {}\n  Active: {}\n  Enabled: {}\n  Description: {}",
            probe.service, probe.active_state.as_str(), probe.enabled_state.as_str(),
            probe.description.as_deref().unwrap_or("N/A")
        )
    } else {
        format!("Service '{}' does not exist", service)
    };

    ToolResult {
        tool_name: "systemd_service_probe_v1".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "probe": {
                "service": probe.service, "exists": probe.exists,
                "active_state": probe.active_state.as_str(),
                "enabled_state": probe.enabled_state.as_str(),
                "description": probe.description, "main_pid": probe.main_pid,
                "unit_file_path": probe.unit_file_path, "last_failure": probe.last_failure,
                "evidence_id": probe.evidence_id,
            }
        }),
        human_summary, success: true, error: None, timestamp,
    }
}

// =============================================================================
// Preview Tool Executor
// =============================================================================

/// Execute systemd_service_preview_v1 tool
pub fn execute_systemd_service_preview_v1(
    parameters: &HashMap<String, Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    const TOOL: &str = "systemd_service_preview_v1";
    let service = require_param!(parameters, "service", TOOL, evidence_id, timestamp);
    let operation_str = require_param!(parameters, "operation", TOOL, evidence_id, timestamp);

    let operation = match parse_operation(&operation_str) {
        Some(op) => op,
        None => return error_result(TOOL, evidence_id,
            &format!("Invalid operation: '{}'. Use: start, stop, restart, enable, disable", operation_str), timestamp),
    };

    let action = ServiceAction::new(&service, operation);

    match preview_service_action(&action) {
        Ok(preview) => {
            let human_summary = format!(
                "Preview: {} {}\nCurrent: active={}, enabled={}\nChange: {}\nRisk: {}\nConfirmation required: {}",
                operation.as_str(), preview.service, preview.current_active.as_str(),
                preview.current_enabled.as_str(), preview.change_summary, preview.risk_level.as_str(),
                preview.confirmation_required.as_deref().unwrap_or("none")
            );

            ToolResult {
                tool_name: TOOL.to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "preview": {
                        "service": preview.service, "operation": preview.operation.as_str(),
                        "exists": preview.exists, "current_active": preview.current_active.as_str(),
                        "current_enabled": preview.current_enabled.as_str(),
                        "will_change_active": preview.will_change_active,
                        "will_change_enabled": preview.will_change_enabled,
                        "expected_active": preview.expected_active.as_str(),
                        "expected_enabled": preview.expected_enabled.as_str(),
                        "risk_level": preview.risk_level.as_str(),
                        "confirmation_required": preview.confirmation_required,
                        "change_summary": preview.change_summary, "evidence_id": preview.evidence_id,
                    }
                }),
                human_summary, success: true, error: None, timestamp,
            }
        }
        Err(e) => error_result(TOOL, evidence_id, &format!("Preview failed: {}", e), timestamp),
    }
}

// =============================================================================
// Apply Tool Executor
// =============================================================================

/// Execute systemd_service_apply_v1 tool
pub fn execute_systemd_service_apply_v1(
    parameters: &HashMap<String, Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    const TOOL: &str = "systemd_service_apply_v1";
    let service = require_param!(parameters, "service", TOOL, evidence_id, timestamp);
    let operation_str = require_param!(parameters, "operation", TOOL, evidence_id, timestamp);
    let _preview_id = require_param!(parameters, "preview_id", TOOL, evidence_id, timestamp);
    let confirmation = require_param!(parameters, "confirmation", TOOL, evidence_id, timestamp);

    let operation = match parse_operation(&operation_str) {
        Some(op) => op,
        None => return error_result(TOOL, evidence_id, &format!("Invalid operation: '{}'", operation_str), timestamp),
    };

    let action = ServiceAction::new(&service, operation);
    let case_id = generate_service_case_id();

    match apply_service_action(&action, &case_id, &confirmation) {
        Ok(result) => {
            let human_summary = format!(
                "Result: {} {}\nCase ID: {}\nPre-state: active={}, enabled={}\n\
                 Post-state: active={}, enabled={}\nVerified: {}\n{}\n\nRollback: {}",
                operation.past_tense(), result.service, result.case_id,
                result.pre_state.active_state.as_str(), result.pre_state.enabled_state.as_str(),
                result.post_state.active_state.as_str(), result.post_state.enabled_state.as_str(),
                if result.verified { "yes" } else { "NO" }, result.verify_message, result.rollback_command
            );

            ToolResult {
                tool_name: TOOL.to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "apply": {
                        "success": result.success, "case_id": result.case_id, "service": result.service,
                        "operation": result.operation.as_str(),
                        "pre_state": { "active": result.pre_state.active_state.as_str(),
                                       "enabled": result.pre_state.enabled_state.as_str() },
                        "post_state": { "active": result.post_state.active_state.as_str(),
                                        "enabled": result.post_state.enabled_state.as_str() },
                        "verified": result.verified, "verify_message": result.verify_message,
                        "rollback_command": result.rollback_command, "evidence_id": result.evidence_id,
                    }
                }),
                human_summary, success: true, error: None, timestamp,
            }
        }
        Err(e) => error_result(TOOL, evidence_id, &format!("Apply failed: {}", e), timestamp),
    }
}

// =============================================================================
// Rollback Tool Executor
// =============================================================================

/// Execute systemd_service_rollback_v1 tool
pub fn execute_systemd_service_rollback_v1(
    parameters: &HashMap<String, Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    const TOOL: &str = "systemd_service_rollback_v1";
    let case_id = require_param!(parameters, "case_id", TOOL, evidence_id, timestamp);

    match rollback_service_action(&case_id) {
        Ok(result) => {
            let human_summary = format!(
                "Rollback: {}\nCase ID: {}\nService: {}\nRestored: active={}, enabled={}\n{}",
                if result.success { "successful" } else { "completed with warnings" },
                result.case_id, result.service, result.restored_active.as_str(),
                result.restored_enabled.as_str(), result.message
            );

            ToolResult {
                tool_name: TOOL.to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "rollback": {
                        "success": result.success, "case_id": result.case_id, "service": result.service,
                        "restored_active": result.restored_active.as_str(),
                        "restored_enabled": result.restored_enabled.as_str(),
                        "original_active": result.original_active.as_str(),
                        "original_enabled": result.original_enabled.as_str(),
                        "message": result.message,
                    }
                }),
                human_summary, success: result.success, error: result.error, timestamp,
            }
        }
        Err(e) => error_result(TOOL, evidence_id, &format!("Rollback failed: {}", e), timestamp),
    }
}
