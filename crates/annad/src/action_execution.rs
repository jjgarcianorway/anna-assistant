//! Action Execution Framework - Safely execute actions with rollback support.
//!
//! Philosophy: Execute safely. Verify success. Rollback on failure.
//! NO HARDCODING: Context-aware execution, not scripted sequences.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// An executable action with safety guarantees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub action_type: ActionType,
    pub description: String,
    pub commands: Vec<String>,
    pub risk_level: crate::trust_calibration::RiskLevel,
    pub rollback_commands: Option<Vec<String>>,
    pub verification: Option<VerificationStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Cleanup,
    ServiceRestart,
    ConfigChange,
    PackageOperation,
    SystemOptimization,
}

/// Verification step to ensure action succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStep {
    pub description: String,
    pub command: String,
    pub expected_pattern: String,
}

/// Action execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub action_id: String,
    pub success: bool,
    pub executed_at: DateTime<Utc>,
    pub command_results: Vec<CommandResult>,
    pub verification_passed: Option<bool>,
    pub rollback_executed: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,
    pub exit_code: i32,
    pub output: String,
    pub success: bool,
}

/// Execute an action safely with rollback support.
pub async fn execute_action(action: &Action, trust_state: &mut crate::trust_calibration::TrustState) -> Result<ExecutionResult> {
    info!("Executing action: {} ({})", action.description, action.id);

    let mut execution_result = ExecutionResult {
        action_id: action.id.clone(),
        success: false,
        executed_at: Utc::now(),
        command_results: Vec::new(),
        verification_passed: None,
        rollback_executed: false,
        error_message: None,
    };

    // Execute commands sequentially
    for cmd in &action.commands {
        info!("Running command: {}", cmd);

        match crate::core_loop::execute_command(cmd) {
            Ok(output) => {
                execution_result.command_results.push(CommandResult {
                    command: cmd.clone(),
                    exit_code: 0,
                    output: output.clone(),
                    success: true,
                });
            }
            Err(e) => {
                warn!("Command failed: {}", e);
                execution_result.command_results.push(CommandResult {
                    command: cmd.clone(),
                    exit_code: 1,
                    output: format!("Error: {}", e),
                    success: false,
                });

                execution_result.error_message = Some(format!("Command failed: {}", e));

                // Attempt rollback
                if let Some(rollback_cmds) = &action.rollback_commands {
                    info!("Attempting rollback...");
                    execution_result.rollback_executed = true;

                    for rollback_cmd in rollback_cmds {
                        if let Err(e) = crate::core_loop::execute_command(rollback_cmd) {
                            warn!("Rollback command failed: {}", e);
                        }
                    }
                }

                trust_state.record_failure(&format!("{:?}", action.action_type));
                return Ok(execution_result);
            }
        }
    }

    // Verify action succeeded
    if let Some(verification) = &action.verification {
        info!("Verifying action: {}", verification.description);

        match crate::core_loop::execute_command(&verification.command) {
            Ok(output) => {
                let passed = output.contains(&verification.expected_pattern);
                execution_result.verification_passed = Some(passed);

                if !passed {
                    warn!("Verification failed: expected pattern not found");
                    execution_result.error_message = Some("Verification failed".to_string());

                    // Rollback
                    if let Some(rollback_cmds) = &action.rollback_commands {
                        info!("Verification failed, attempting rollback...");
                        execution_result.rollback_executed = true;

                        for rollback_cmd in rollback_cmds {
                            if let Err(e) = crate::core_loop::execute_command(rollback_cmd) {
                                warn!("Rollback command failed: {}", e);
                            }
                        }
                    }

                    trust_state.record_failure(&format!("{:?}", action.action_type));
                    return Ok(execution_result);
                }
            }
            Err(e) => {
                warn!("Verification command failed: {}", e);
                execution_result.verification_passed = Some(false);
                execution_result.error_message = Some(format!("Verification error: {}", e));

                trust_state.record_failure(&format!("{:?}", action.action_type));
                return Ok(execution_result);
            }
        }
    }

    execution_result.success = true;
    trust_state.record_success(&format!("{:?}", action.action_type));

    info!("Action completed successfully: {}", action.description);

    Ok(execution_result)
}

/// Build a cleanup action.
pub fn build_cleanup_action(cleanable_items: &[crate::cleanup_detector::CleanableItem]) -> Vec<Action> {
    let mut actions = Vec::new();

    for item in cleanable_items {
        if item.safety == crate::cleanup_detector::SafetyLevel::Safe {
            // Only auto-build actions for safe items
            let action = Action {
                id: format!("cleanup-{}", uuid::Uuid::new_v4()),
                action_type: ActionType::Cleanup,
                description: format!("Clean {}", item.description),
                commands: vec![item.cleanup_method.clone()],
                risk_level: crate::trust_calibration::RiskLevel::Safe,
                rollback_commands: None, // Cleanup is not reversible
                verification: None,
            };

            actions.push(action);
        }
    }

    actions
}

/// Build a service restart action.
pub fn build_service_restart_action(service_name: &str) -> Action {
    Action {
        id: format!("restart-{}-{}", service_name, uuid::Uuid::new_v4()),
        action_type: ActionType::ServiceRestart,
        description: format!("Restart {} service", service_name),
        commands: vec![format!("systemctl restart {}", service_name)],
        risk_level: crate::trust_calibration::RiskLevel::Low,
        rollback_commands: None,
        verification: Some(VerificationStep {
            description: format!("Verify {} is running", service_name),
            command: format!("systemctl is-active {}", service_name),
            expected_pattern: "active".to_string(),
        }),
    }
}

/// Format execution result for display.
pub fn format_execution_result(result: &ExecutionResult) -> String {
    let mut output = String::new();

    if result.success {
        output.push_str(&format!("✓ Action completed successfully\n"));
    } else {
        output.push_str(&format!("✗ Action failed\n"));
        if let Some(err) = &result.error_message {
            output.push_str(&format!("  Error: {}\n", err));
        }
    }

    output.push_str(&format!("\nCommands executed:\n"));
    for cmd_result in &result.command_results {
        let status = if cmd_result.success { "✓" } else { "✗" };
        output.push_str(&format!("  {} {}\n", status, cmd_result.command));

        if !cmd_result.success {
            output.push_str(&format!("    Output: {}\n", cmd_result.output.lines().take(3).collect::<Vec<_>>().join("\n    ")));
        }
    }

    if let Some(verified) = result.verification_passed {
        output.push_str(&format!("\nVerification: {}\n", if verified { "Passed" } else { "Failed" }));
    }

    if result.rollback_executed {
        output.push_str("\nRollback was executed due to failure.\n");
    }

    output
}

/// Execution history storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHistory {
    pub executions: Vec<ExecutionResult>,
}

impl ExecutionHistory {
    pub fn load() -> Self {
        let path = PathBuf::from("/var/lib/anna/execution_history.json");

        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(history) = serde_json::from_str(&contents) {
                return history;
            }
        }

        Self { executions: Vec::new() }
    }

    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from("/var/lib/anna/execution_history.json");

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;

        Ok(())
    }

    pub fn record(&mut self, result: ExecutionResult) {
        // Keep last 100 executions
        if self.executions.len() >= 100 {
            self.executions.remove(0);
        }

        self.executions.push(result);
        let _ = self.save();
    }

    pub fn get_success_rate(&self, action_type: &ActionType) -> f32 {
        let relevant: Vec<_> = self
            .executions
            .iter()
            .filter(|e| {
                // Match by action ID prefix
                match action_type {
                    ActionType::Cleanup => e.action_id.starts_with("cleanup"),
                    ActionType::ServiceRestart => e.action_id.starts_with("restart"),
                    _ => false,
                }
            })
            .collect();

        if relevant.is_empty() {
            return 0.5; // No data
        }

        let successful = relevant.iter().filter(|e| e.success).count();
        successful as f32 / relevant.len() as f32
    }
}
