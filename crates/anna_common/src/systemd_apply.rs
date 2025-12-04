//! Systemd Service Preview and Apply v0.0.51
//!
//! Preview and apply service actions with evidence.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::generate_request_id;
use crate::service_state::{ActiveState, EnabledState};
use crate::systemd_action::{assess_risk, RiskLevel, ServiceAction, ServiceOperation};
use crate::systemd_probe::{probe_service, ServiceProbe};
use crate::systemd_rollback::ROLLBACK_BASE;

// =============================================================================
// Preview Result
// =============================================================================

/// Preview of what a service action would change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePreview {
    pub evidence_id: String,
    pub service: String,
    pub operation: ServiceOperation,
    pub exists: bool,
    pub current_active: ActiveState,
    pub current_enabled: EnabledState,
    pub will_change_active: bool,
    pub will_change_enabled: bool,
    pub expected_active: ActiveState,
    pub expected_enabled: EnabledState,
    pub risk_level: RiskLevel,
    pub confirmation_required: Option<String>,
    pub change_summary: String,
    pub probe: ServiceProbe,
}

/// Generate a preview of service action without executing
pub fn preview_service_action(action: &ServiceAction) -> Result<ServicePreview, String> {
    let evidence_id = format!(
        "PRV{}",
        generate_request_id().chars().take(8).collect::<String>()
    );

    // Probe current state
    let probe = probe_service(&action.service);

    if !probe.exists {
        return Err(format!("Service '{}' does not exist", action.service));
    }

    // Assess risk
    let risk_level = assess_risk(action);

    if risk_level == RiskLevel::Denied {
        return Err(format!(
            "Action denied: '{}' is a core system service. Manual intervention required.",
            action.service
        ));
    }

    // Determine expected state changes
    let (will_change_active, expected_active) = compute_active_changes(&action.operation, &probe);
    let (will_change_enabled, expected_enabled) =
        compute_enabled_changes(&action.operation, &probe);

    // Generate change summary
    let change_summary = generate_change_summary(
        &probe,
        will_change_active,
        will_change_enabled,
        &expected_active,
        &expected_enabled,
    );

    Ok(ServicePreview {
        evidence_id,
        service: action.service.clone(),
        operation: action.operation,
        exists: true,
        current_active: probe.active_state.clone(),
        current_enabled: probe.enabled_state.clone(),
        will_change_active,
        will_change_enabled,
        expected_active,
        expected_enabled,
        risk_level,
        confirmation_required: risk_level.confirmation_phrase().map(|s| s.to_string()),
        change_summary,
        probe,
    })
}

fn compute_active_changes(
    operation: &ServiceOperation,
    probe: &ServiceProbe,
) -> (bool, ActiveState) {
    match operation {
        ServiceOperation::Start => {
            let will_change = !probe.active_state.is_running();
            (will_change, ActiveState::Active)
        }
        ServiceOperation::Stop => {
            let will_change = probe.active_state.is_running();
            (will_change, ActiveState::Inactive)
        }
        ServiceOperation::Restart => (true, ActiveState::Active),
        ServiceOperation::Enable | ServiceOperation::Disable => (false, probe.active_state.clone()),
    }
}

fn compute_enabled_changes(
    operation: &ServiceOperation,
    probe: &ServiceProbe,
) -> (bool, EnabledState) {
    match operation {
        ServiceOperation::Enable => {
            let will_change = probe.enabled_state != EnabledState::Enabled;
            (will_change, EnabledState::Enabled)
        }
        ServiceOperation::Disable => {
            let will_change = probe.enabled_state != EnabledState::Disabled;
            (will_change, EnabledState::Disabled)
        }
        ServiceOperation::Start | ServiceOperation::Stop | ServiceOperation::Restart => {
            (false, probe.enabled_state.clone())
        }
    }
}

fn generate_change_summary(
    probe: &ServiceProbe,
    will_change_active: bool,
    will_change_enabled: bool,
    expected_active: &ActiveState,
    expected_enabled: &EnabledState,
) -> String {
    let mut parts = Vec::new();

    if will_change_active {
        parts.push(format!(
            "Active state: {} -> {}",
            probe.active_state.as_str(),
            expected_active.as_str()
        ));
    }

    if will_change_enabled {
        parts.push(format!(
            "Enabled state: {} -> {}",
            probe.enabled_state.as_str(),
            expected_enabled.as_str()
        ));
    }

    if parts.is_empty() {
        format!(
            "Service already in desired state (active={}, enabled={})",
            probe.active_state.as_str(),
            probe.enabled_state.as_str()
        )
    } else {
        parts.join("; ")
    }
}

// =============================================================================
// Apply Result
// =============================================================================

/// Result from applying a service action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceApplyResult {
    pub success: bool,
    pub case_id: String,
    pub evidence_id: String,
    pub service: String,
    pub operation: ServiceOperation,
    pub pre_state: ServiceStateSnapshot,
    pub post_state: ServiceStateSnapshot,
    pub verified: bool,
    pub verify_message: String,
    pub rollback_command: String,
    pub error: Option<String>,
}

/// Snapshot of service state for before/after comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStateSnapshot {
    pub active_state: ActiveState,
    pub enabled_state: EnabledState,
    pub main_pid: Option<u32>,
}

/// Apply a service action
pub fn apply_service_action(
    action: &ServiceAction,
    case_id: &str,
    confirmation: &str,
) -> Result<ServiceApplyResult, String> {
    let evidence_id = format!(
        "APL{}",
        generate_request_id().chars().take(8).collect::<String>()
    );

    // Preview first (validates service exists and action is allowed)
    let preview = preview_service_action(action)?;

    // Check confirmation
    if let Some(required) = &preview.confirmation_required {
        if confirmation != required {
            return Err(format!(
                "Invalid confirmation. Required: '{}', received: '{}'",
                required, confirmation
            ));
        }
    }

    // Capture pre-state
    let pre_state = ServiceStateSnapshot {
        active_state: preview.current_active.clone(),
        enabled_state: preview.current_enabled.clone(),
        main_pid: preview.probe.main_pid,
    };

    // Execute the action
    execute_systemctl_action(action)?;

    // Capture post-state
    let post_probe = probe_service(&action.service);
    let post_state = ServiceStateSnapshot {
        active_state: post_probe.active_state.clone(),
        enabled_state: post_probe.enabled_state.clone(),
        main_pid: post_probe.main_pid,
    };

    // Verify expected outcome
    let (verified, verify_message) = verify_action_result(action, &post_probe);

    // Write rollback metadata
    write_rollback_metadata(case_id, action, &pre_state, &post_state)?;

    // Generate rollback command
    let rollback_command = format!("annactl 'rollback {}'", case_id);

    Ok(ServiceApplyResult {
        success: true,
        case_id: case_id.to_string(),
        evidence_id,
        service: action.service.clone(),
        operation: action.operation,
        pre_state,
        post_state,
        verified,
        verify_message,
        rollback_command,
        error: None,
    })
}

fn execute_systemctl_action(action: &ServiceAction) -> Result<(), String> {
    let systemctl_args = match action.operation {
        ServiceOperation::Start => vec!["start", &action.service],
        ServiceOperation::Stop => vec!["stop", &action.service],
        ServiceOperation::Restart => vec!["restart", &action.service],
        ServiceOperation::Enable => vec!["enable", &action.service],
        ServiceOperation::Disable => vec!["disable", &action.service],
    };

    let output = Command::new("systemctl")
        .args(&systemctl_args)
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "systemctl {} failed: {}",
            action.operation,
            stderr.trim()
        ));
    }

    Ok(())
}

fn verify_action_result(action: &ServiceAction, post_probe: &ServiceProbe) -> (bool, String) {
    match action.operation {
        ServiceOperation::Start => {
            if post_probe.active_state.is_running() {
                (true, format!("Verified: {} is now active", action.service))
            } else {
                (
                    false,
                    format!("Warning: {} is not active after start", action.service),
                )
            }
        }
        ServiceOperation::Stop => {
            if !post_probe.active_state.is_running() {
                (
                    true,
                    format!("Verified: {} is now inactive", action.service),
                )
            } else {
                (
                    false,
                    format!("Warning: {} is still active after stop", action.service),
                )
            }
        }
        ServiceOperation::Restart => {
            if post_probe.active_state.is_running() {
                (
                    true,
                    format!("Verified: {} restarted successfully", action.service),
                )
            } else {
                (
                    false,
                    format!("Warning: {} is not active after restart", action.service),
                )
            }
        }
        ServiceOperation::Enable => {
            if post_probe.enabled_state == EnabledState::Enabled {
                (true, format!("Verified: {} is now enabled", action.service))
            } else {
                (
                    false,
                    format!("Warning: {} is not enabled after enable", action.service),
                )
            }
        }
        ServiceOperation::Disable => {
            if post_probe.enabled_state == EnabledState::Disabled {
                (
                    true,
                    format!("Verified: {} is now disabled", action.service),
                )
            } else {
                (
                    false,
                    format!("Warning: {} is not disabled after disable", action.service),
                )
            }
        }
    }
}

fn write_rollback_metadata(
    case_id: &str,
    action: &ServiceAction,
    pre_state: &ServiceStateSnapshot,
    post_state: &ServiceStateSnapshot,
) -> Result<(), String> {
    let rollback_dir = PathBuf::from(ROLLBACK_BASE).join(case_id);
    fs::create_dir_all(&rollback_dir).map_err(|e| format!("Cannot create rollback dir: {}", e))?;

    let metadata = serde_json::json!({
        "case_id": case_id,
        "service": action.service,
        "operation": action.operation.as_str(),
        "reason": action.reason,
        "pre_state": {
            "active_state": pre_state.active_state.as_str(),
            "enabled_state": pre_state.enabled_state.as_str(),
            "main_pid": pre_state.main_pid,
        },
        "post_state": {
            "active_state": post_state.active_state.as_str(),
            "enabled_state": post_state.enabled_state.as_str(),
            "main_pid": post_state.main_pid,
        },
        "timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    });

    let metadata_path = rollback_dir.join("service_rollback.json");
    let content = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Cannot serialize metadata: {}", e))?;

    fs::write(&metadata_path, content).map_err(|e| format!("Cannot write metadata: {}", e))?;

    Ok(())
}
