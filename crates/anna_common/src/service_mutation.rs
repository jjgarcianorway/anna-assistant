//! Service Mutation Executor for Anna v0.0.81
//!
//! Executes systemd service control mutations:
//! - start/stop/restart
//! - enable/disable
//!
//! Only for whitelisted services in v0.0.80.
//! v0.0.81: Added service unit resolution for smarter previews.

use crate::mutation_engine_v1::{
    is_service_allowed, MutationCategory, MutationDetail, MutationPlanV1, MutationRiskLevel,
    MutationStep, RollbackStep, ServiceAction, StepExecutionResult, StepPreview,
    VerificationCheck,
};
use crate::mutation_verification::resolve_service_unit;
use crate::privilege::{check_privilege, generate_manual_commands, run_privileged};
use std::process::Command;

/// Get current state of a systemd service
pub fn get_service_state(service: &str) -> ServiceStateInfo {
    // Check if active
    let active_output = Command::new("systemctl")
        .args(["is-active", service])
        .output();

    let is_active = active_output
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let active_state = active_output
        .as_ref()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Check if enabled
    let enabled_output = Command::new("systemctl")
        .args(["is-enabled", service])
        .output();

    let is_enabled = enabled_output
        .as_ref()
        .map(|o| {
            let status = String::from_utf8_lossy(&o.stdout).trim().to_string();
            status == "enabled"
        })
        .unwrap_or(false);

    let enabled_state = enabled_output
        .as_ref()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    ServiceStateInfo {
        service: service.to_string(),
        is_active,
        active_state,
        is_enabled,
        enabled_state,
    }
}

/// Current state of a service
#[derive(Debug, Clone)]
pub struct ServiceStateInfo {
    pub service: String,
    pub is_active: bool,
    pub active_state: String,
    pub is_enabled: bool,
    pub enabled_state: String,
}

impl ServiceStateInfo {
    pub fn format_human(&self) -> String {
        format!(
            "{}: {} ({})",
            self.service,
            self.active_state,
            self.enabled_state
        )
    }
}

/// Generate preview for a service action
pub fn preview_service_action(service: &str, action: &ServiceAction) -> Result<StepPreview, String> {
    if !is_service_allowed(service) {
        return Err(format!(
            "Service '{}' is not in the allowed whitelist for v0.0.80",
            service
        ));
    }

    // v0.0.81: Resolve service unit name for smarter previews
    let resolution = resolve_service_unit(service);
    let actual_unit = &resolution.resolved_unit;

    let current = get_service_state(actual_unit);

    // Build current state with unit resolution info
    let current_state = if resolution.input != resolution.resolved_unit {
        format!(
            "{}\n(Note: '{}' -> {})",
            current.format_human(),
            resolution.input,
            resolution.resolved_unit
        )
    } else {
        current.format_human()
    };

    // Add warning if unit doesn't exist
    let current_state = if !resolution.exists {
        format!(
            "{}\nWarning: Unit '{}' may not exist on this system",
            current_state, resolution.resolved_unit
        )
    } else {
        current_state
    };

    let intended_state = match action {
        ServiceAction::Start => format!("{}: active (running)", actual_unit),
        ServiceAction::Stop => format!("{}: inactive (stopped)", actual_unit),
        ServiceAction::Restart => format!("{}: active (restarted)", actual_unit),
        ServiceAction::Enable => format!("{}: enabled", actual_unit),
        ServiceAction::Disable => format!("{}: disabled", actual_unit),
    };

    Ok(StepPreview {
        step_id: format!("service-{}-{}", service, action),
        description: format!("{} service {}", action, actual_unit),
        current_state,
        intended_state,
        diff: None, // No file diff for service actions
    })
}

/// Execute a service action
pub fn execute_service_action(
    service: &str,
    action: &ServiceAction,
) -> Result<StepExecutionResult, String> {
    if !is_service_allowed(service) {
        return Err(format!(
            "Service '{}' is not in the allowed whitelist",
            service
        ));
    }

    let priv_status = check_privilege();
    if !priv_status.available {
        return Err(priv_status.message);
    }

    let args: Vec<&str> = match action {
        ServiceAction::Start => vec!["start", service],
        ServiceAction::Stop => vec!["stop", service],
        ServiceAction::Restart => vec!["restart", service],
        ServiceAction::Enable => vec!["enable", service],
        ServiceAction::Disable => vec!["disable", service],
    };

    let output = run_privileged("systemctl", &args)?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let message = if success {
        format!("Successfully {} service {}", action, service)
    } else {
        format!("Failed to {} service {}: {}", action, service, stderr.trim())
    };

    Ok(StepExecutionResult {
        step_id: format!("service-{}-{}", service, action),
        success,
        message,
        stdout: if stdout.is_empty() {
            None
        } else {
            Some(stdout)
        },
        stderr: if stderr.is_empty() {
            None
        } else {
            Some(stderr)
        },
        exit_code: output.status.code(),
    })
}

/// Create verification checks for service action
pub fn create_service_verification(service: &str, action: &ServiceAction) -> Vec<VerificationCheck> {
    match action {
        ServiceAction::Start | ServiceAction::Restart => {
            vec![VerificationCheck {
                description: format!("Verify {} is running", service),
                command: Some(format!("systemctl is-active {}", service)),
                expected: "active".to_string(),
            }]
        }
        ServiceAction::Stop => {
            vec![VerificationCheck {
                description: format!("Verify {} is stopped", service),
                command: Some(format!("systemctl is-active {}", service)),
                expected: "inactive".to_string(),
            }]
        }
        ServiceAction::Enable => {
            vec![VerificationCheck {
                description: format!("Verify {} is enabled", service),
                command: Some(format!("systemctl is-enabled {}", service)),
                expected: "enabled".to_string(),
            }]
        }
        ServiceAction::Disable => {
            vec![VerificationCheck {
                description: format!("Verify {} is disabled", service),
                command: Some(format!("systemctl is-enabled {}", service)),
                expected: "disabled".to_string(),
            }]
        }
    }
}

/// Create rollback steps for service action
pub fn create_service_rollback(service: &str, action: &ServiceAction) -> Vec<RollbackStep> {
    match action {
        ServiceAction::Start => {
            vec![RollbackStep {
                description: format!("Stop {} to restore previous state", service),
                undo_action: format!("systemctl stop {}", service),
                backup_path: None,
            }]
        }
        ServiceAction::Stop => {
            vec![RollbackStep {
                description: format!("Start {} to restore previous state", service),
                undo_action: format!("systemctl start {}", service),
                backup_path: None,
            }]
        }
        ServiceAction::Restart => {
            // Restart doesn't have a meaningful rollback - service was already running
            vec![RollbackStep {
                description: "No rollback for restart - service was already running".to_string(),
                undo_action: "N/A".to_string(),
                backup_path: None,
            }]
        }
        ServiceAction::Enable => {
            vec![RollbackStep {
                description: format!("Disable {} to restore previous state", service),
                undo_action: format!("systemctl disable {}", service),
                backup_path: None,
            }]
        }
        ServiceAction::Disable => {
            vec![RollbackStep {
                description: format!("Enable {} to restore previous state", service),
                undo_action: format!("systemctl enable {}", service),
                backup_path: None,
            }]
        }
    }
}

/// Get risk level for service action
pub fn get_service_action_risk(service: &str, action: &ServiceAction) -> MutationRiskLevel {
    // NetworkManager and sshd are higher risk because they affect connectivity
    let is_critical_service = service == "NetworkManager" || service == "sshd";

    match action {
        ServiceAction::Start | ServiceAction::Enable => {
            if is_critical_service {
                MutationRiskLevel::Medium
            } else {
                MutationRiskLevel::Low
            }
        }
        ServiceAction::Stop | ServiceAction::Disable => {
            if is_critical_service {
                MutationRiskLevel::High
            } else {
                MutationRiskLevel::Medium
            }
        }
        ServiceAction::Restart => {
            if is_critical_service {
                MutationRiskLevel::Medium
            } else {
                MutationRiskLevel::Low
            }
        }
    }
}

/// Generate manual commands for user when privilege not available
pub fn generate_service_manual_commands(service: &str, action: &ServiceAction) -> Vec<String> {
    generate_manual_commands("systemctl", &[&action.to_string(), service])
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_service_action_risk() {
        // NetworkManager stop is high risk
        assert_eq!(
            get_service_action_risk("NetworkManager", &ServiceAction::Stop),
            MutationRiskLevel::High
        );
        // docker restart is low risk
        assert_eq!(
            get_service_action_risk("docker", &ServiceAction::Restart),
            MutationRiskLevel::Low
        );
        // sshd disable is high risk
        assert_eq!(
            get_service_action_risk("sshd", &ServiceAction::Disable),
            MutationRiskLevel::High
        );
    }

    #[test]
    fn test_create_service_verification() {
        let checks = create_service_verification("docker", &ServiceAction::Start);
        assert_eq!(checks.len(), 1);
        assert!(checks[0].expected.contains("active"));

        let checks = create_service_verification("docker", &ServiceAction::Stop);
        assert!(checks[0].expected.contains("inactive"));
    }

    #[test]
    fn test_create_service_rollback() {
        let rollback = create_service_rollback("docker", &ServiceAction::Start);
        assert_eq!(rollback.len(), 1);
        assert!(rollback[0].undo_action.contains("stop"));

        let rollback = create_service_rollback("docker", &ServiceAction::Stop);
        assert!(rollback[0].undo_action.contains("start"));
    }

    #[test]
    fn test_generate_service_manual_commands() {
        let cmds = generate_service_manual_commands("docker", &ServiceAction::Restart);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("systemctl"));
        assert!(cmds[0].contains("restart"));
        assert!(cmds[0].contains("docker"));
    }

    #[test]
    fn test_preview_service_not_allowed() {
        let result = preview_service_action("httpd", &ServiceAction::Start);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in the allowed whitelist"));
    }
}
