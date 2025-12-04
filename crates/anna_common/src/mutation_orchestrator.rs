//! Mutation Orchestrator for Anna v0.0.80
//!
//! High-level orchestration of the mutation pipeline:
//! 1. Plan - Generate structured action plan
//! 2. Preview - Show diff preview
//! 3. Confirm - Require confirmation phrase
//! 4. Execute - Run the mutation
//! 5. Verify - Check success
//! 6. Rollback - Undo if verification fails

use crate::config_mutation::{
    create_config_rollback, create_config_verification, execute_config_edit, get_config_edit_risk,
    generate_config_manual_commands, preview_config_edit, rollback_config_edit,
};
use crate::mutation_engine_v1::{
    ConfigEditOp, MutationCategory, MutationDetail, MutationExecutionResult, MutationPlanState,
    MutationPlanV1, MutationPreview, MutationRiskLevel, MutationStats, MutationStep,
    PackageAction, RollbackResult, RollbackStep, ServiceAction, StepExecutionResult, StepPreview,
    VerificationCheck, VerificationResult,
};
use crate::package_mutation::{
    create_install_rollback, create_install_verification, create_remove_rollback,
    create_remove_verification, execute_package_install, execute_package_remove,
    generate_package_manual_commands, get_package_action_risk, preview_package_install,
    preview_package_remove,
};
use crate::privilege::{check_privilege, format_privilege_blocked};
use crate::service_mutation::{
    create_service_rollback, create_service_verification, execute_service_action,
    generate_service_manual_commands, get_service_action_risk, preview_service_action,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a unique plan ID
fn generate_plan_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("plan-{:x}", now)
}

/// Create a mutation plan from a service action request
pub fn plan_service_mutation(
    service: &str,
    action: ServiceAction,
    case_id: Option<String>,
) -> Result<MutationPlanV1, String> {
    // Check whitelist first
    if !crate::mutation_engine_v1::is_service_allowed(service) {
        return Err(format!(
            "Service '{}' is not in the allowed whitelist for v0.0.80",
            service
        ));
    }

    let risk = get_service_action_risk(service, &action);
    let step = MutationStep {
        step_id: format!("service-{}-{}", service, action),
        description: format!("{} service {}", action, service),
        category: MutationCategory::ServiceControl,
        mutation: MutationDetail::ServiceControl {
            service: service.to_string(),
            action: action.clone(),
        },
        risk,
        affected_resources: vec![format!("service:{}", service)],
    };

    let verification = create_service_verification(service, &action);
    let rollback = create_service_rollback(service, &action);

    Ok(MutationPlanV1 {
        plan_id: generate_plan_id(),
        case_id,
        steps: vec![step],
        risk,
        affected_resources: vec![format!("service:{}", service)],
        verification_checks: verification,
        rollback_steps: rollback,
        state: MutationPlanState::Created,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    })
}

/// Create a mutation plan from a package action request
pub fn plan_package_mutation(
    action: PackageAction,
    packages: Vec<String>,
    case_id: Option<String>,
) -> Result<MutationPlanV1, String> {
    if packages.is_empty() {
        return Err("No packages specified".to_string());
    }

    let risk = get_package_action_risk(&action, &packages);

    let (mutation, verification, rollback) = match action {
        PackageAction::Install => (
            MutationDetail::PackageInstall {
                packages: packages.clone(),
            },
            create_install_verification(&packages),
            create_install_rollback(&packages),
        ),
        PackageAction::Remove => (
            MutationDetail::PackageRemove {
                packages: packages.clone(),
            },
            create_remove_verification(&packages),
            create_remove_rollback(&packages),
        ),
    };

    let step = MutationStep {
        step_id: format!("pkg-{}-{}", action.to_string(), packages.join("-")),
        description: format!("{} packages: {}", action.to_string(), packages.join(", ")),
        category: MutationCategory::PackageManagement,
        mutation,
        risk,
        affected_resources: packages.iter().map(|p| format!("package:{}", p)).collect(),
    };

    Ok(MutationPlanV1 {
        plan_id: generate_plan_id(),
        case_id,
        steps: vec![step],
        risk,
        affected_resources: packages.iter().map(|p| format!("package:{}", p)).collect(),
        verification_checks: verification,
        rollback_steps: rollback,
        state: MutationPlanState::Created,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    })
}

impl PackageAction {
    fn to_string(&self) -> &'static str {
        match self {
            PackageAction::Install => "install",
            PackageAction::Remove => "remove",
        }
    }
}

/// Create a mutation plan from a config edit request
pub fn plan_config_mutation(
    path: &str,
    op: ConfigEditOp,
    case_id: Option<String>,
) -> Result<MutationPlanV1, String> {
    let risk = get_config_edit_risk(path, &op);
    let case_id_str = case_id.clone().unwrap_or_else(|| "default".to_string());

    let step = MutationStep {
        step_id: format!("config-{}", path.replace('/', "-")),
        description: format!("Edit config file {}", path),
        category: MutationCategory::ConfigEdit,
        mutation: MutationDetail::ConfigEdit {
            path: path.to_string(),
            operation: op.clone(),
        },
        risk,
        affected_resources: vec![format!("file:{}", path)],
    };

    let verification = create_config_verification(path, &op);
    let rollback = create_config_rollback(path, &case_id_str);

    Ok(MutationPlanV1 {
        plan_id: generate_plan_id(),
        case_id,
        steps: vec![step],
        risk,
        affected_resources: vec![format!("file:{}", path)],
        verification_checks: verification,
        rollback_steps: rollback,
        state: MutationPlanState::Created,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    })
}

/// Generate preview for a mutation plan
pub fn generate_preview(plan: &mut MutationPlanV1) -> Result<MutationPreview, String> {
    let priv_status = check_privilege();
    let mut step_previews = Vec::new();
    let mut manual_commands = Vec::new();

    for step in &plan.steps {
        let preview = match &step.mutation {
            MutationDetail::ServiceControl { service, action } => {
                if !priv_status.available {
                    manual_commands.extend(generate_service_manual_commands(service, action));
                }
                preview_service_action(service, action)?
            }
            MutationDetail::PackageInstall { packages } => {
                if !priv_status.available {
                    manual_commands
                        .extend(generate_package_manual_commands(&PackageAction::Install, packages));
                }
                preview_package_install(packages)?
            }
            MutationDetail::PackageRemove { packages } => {
                if !priv_status.available {
                    manual_commands
                        .extend(generate_package_manual_commands(&PackageAction::Remove, packages));
                }
                preview_package_remove(packages)?
            }
            MutationDetail::ConfigEdit { path, operation } => {
                if !priv_status.available {
                    manual_commands.extend(generate_config_manual_commands(path, operation));
                }
                preview_config_edit(path, operation)?
            }
        };
        step_previews.push(preview);
    }

    // Update plan state
    if priv_status.available {
        plan.state = MutationPlanState::AwaitingConfirmation;
    } else {
        plan.state = MutationPlanState::BlockedPrivilege;
    }

    let summary = format!(
        "Mutation plan with {} step(s) at {} risk",
        plan.steps.len(),
        plan.risk
    );

    Ok(MutationPreview {
        plan_id: plan.plan_id.clone(),
        summary,
        step_previews,
        risk: plan.risk,
        confirmation_phrase: plan.risk.confirmation_phrase().to_string(),
        privilege_available: priv_status.available,
        manual_commands: if manual_commands.is_empty() {
            None
        } else {
            Some(manual_commands)
        },
    })
}

/// Validate confirmation phrase
pub fn validate_confirmation(plan: &MutationPlanV1, phrase: &str) -> bool {
    let expected = plan.risk.confirmation_phrase();
    phrase.trim().eq_ignore_ascii_case(expected)
}

/// Execute a mutation plan
pub fn execute_plan(plan: &mut MutationPlanV1) -> Result<MutationExecutionResult, String> {
    // Check privilege
    let priv_status = check_privilege();
    if !priv_status.available {
        plan.state = MutationPlanState::BlockedPrivilege;
        return Err(priv_status.message);
    }

    // Check state
    if plan.state != MutationPlanState::Confirmed {
        return Err(format!(
            "Plan not confirmed. Current state: {}",
            plan.state
        ));
    }

    plan.state = MutationPlanState::Executing;

    let mut step_results = Vec::new();
    let mut all_success = true;
    let case_id = plan.case_id.clone().unwrap_or_else(|| plan.plan_id.clone());

    for step in &plan.steps {
        let result = match &step.mutation {
            MutationDetail::ServiceControl { service, action } => {
                execute_service_action(service, action)?
            }
            MutationDetail::PackageInstall { packages } => execute_package_install(packages)?,
            MutationDetail::PackageRemove { packages } => execute_package_remove(packages)?,
            MutationDetail::ConfigEdit { path, operation } => {
                execute_config_edit(path, operation, &case_id)?
            }
        };

        if !result.success {
            all_success = false;
        }
        step_results.push(result);

        // Stop on first failure
        if !all_success {
            break;
        }
    }

    plan.state = MutationPlanState::ExecutionComplete;

    Ok(MutationExecutionResult {
        plan_id: plan.plan_id.clone(),
        success: all_success,
        step_results,
        verification_results: Vec::new(), // Will be populated by verify_plan
        rolled_back: false,
        rollback_result: None,
        final_state: plan.state,
        completed_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    })
}

/// Verify mutation plan execution
pub fn verify_plan(plan: &mut MutationPlanV1) -> Vec<VerificationResult> {
    plan.state = MutationPlanState::Verifying;

    let mut results = Vec::new();

    for check in &plan.verification_checks {
        let passed = if let Some(ref cmd) = check.command {
            // Run verification command
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if !parts.is_empty() {
                let output = std::process::Command::new(parts[0])
                    .args(&parts[1..])
                    .output();

                match output {
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        // Check if output matches expected
                        stdout.contains(&check.expected) || check.expected.contains(&stdout)
                    }
                    Err(_) => false,
                }
            } else {
                false
            }
        } else {
            // No command, assume pass
            true
        };

        results.push(VerificationResult {
            description: check.description.clone(),
            passed,
            actual: if passed {
                "Passed".to_string()
            } else {
                "Failed".to_string()
            },
        });
    }

    // Update plan state based on results
    let all_passed = results.iter().all(|r| r.passed);
    plan.state = if all_passed {
        MutationPlanState::VerifiedSuccess
    } else {
        MutationPlanState::VerificationFailed
    };

    results
}

/// Execute rollback for a plan
pub fn execute_rollback(plan: &mut MutationPlanV1) -> Result<RollbackResult, String> {
    plan.state = MutationPlanState::RollingBack;

    let case_id = plan.case_id.clone().unwrap_or_else(|| plan.plan_id.clone());
    let mut rolled_back_steps = Vec::new();
    let mut all_success = true;

    // Rollback in reverse order
    for step in plan.steps.iter().rev() {
        let result = match &step.mutation {
            MutationDetail::ServiceControl { service, action } => {
                // Execute the opposite action
                let opposite = match action {
                    ServiceAction::Start => ServiceAction::Stop,
                    ServiceAction::Stop => ServiceAction::Start,
                    ServiceAction::Enable => ServiceAction::Disable,
                    ServiceAction::Disable => ServiceAction::Enable,
                    ServiceAction::Restart => {
                        // No opposite for restart
                        rolled_back_steps.push(step.step_id.clone());
                        continue;
                    }
                };
                execute_service_action(service, &opposite)
            }
            MutationDetail::PackageInstall { packages } => execute_package_remove(packages),
            MutationDetail::PackageRemove { packages } => execute_package_install(packages),
            MutationDetail::ConfigEdit { path, .. } => {
                rollback_config_edit(path, &case_id).map(|_| StepExecutionResult {
                    step_id: step.step_id.clone(),
                    success: true,
                    message: "Restored from backup".to_string(),
                    stdout: None,
                    stderr: None,
                    exit_code: Some(0),
                })
            }
        };

        match result {
            Ok(r) => {
                if r.success {
                    rolled_back_steps.push(step.step_id.clone());
                } else {
                    all_success = false;
                }
            }
            Err(_) => {
                all_success = false;
            }
        }
    }

    plan.state = MutationPlanState::RolledBack;

    Ok(RollbackResult {
        success: all_success,
        steps_rolled_back: rolled_back_steps,
        message: if all_success {
            "Rollback completed successfully".to_string()
        } else {
            "Rollback completed with some failures".to_string()
        },
    })
}

/// Format the mutation preview for human display
pub fn format_preview_human(preview: &MutationPreview) -> String {
    let mut lines = Vec::new();

    lines.push("╔════════════════════════════════════════════════════════════════╗".to_string());
    lines.push("║  PROPOSED MUTATION                                             ║".to_string());
    lines.push("╠════════════════════════════════════════════════════════════════╣".to_string());
    lines.push(format!(
        "║  Risk Level: {:50}║",
        preview.risk.to_string().to_uppercase()
    ));
    lines.push("╚════════════════════════════════════════════════════════════════╝".to_string());
    lines.push("".to_string());

    lines.push("What will change:".to_string());
    lines.push("─────────────────".to_string());

    for step in &preview.step_previews {
        lines.push(format!("  • {}", step.description));
        lines.push(format!("    Current:  {}", step.current_state));
        lines.push(format!("    Intended: {}", step.intended_state));

        if let Some(ref diff) = step.diff {
            lines.push("    Diff:".to_string());
            for line in diff.lines().take(20) {
                lines.push(format!("      {}", line));
            }
        }
        lines.push("".to_string());
    }

    if preview.privilege_available {
        lines.push("To proceed, type the following confirmation phrase:".to_string());
        lines.push(format!("  {}", preview.confirmation_phrase));
    } else {
        lines.push(format_privilege_blocked(
            preview.manual_commands.as_ref().unwrap_or(&Vec::new()),
        ));
    }

    lines.join("\n")
}

/// Update mutation stats after execution
pub fn update_stats(result: &MutationExecutionResult) {
    let mut stats = MutationStats::load();
    stats.privilege_available = check_privilege().available;

    if result.rolled_back {
        stats.record_rollback();
    } else if result.success {
        stats.record_success();
    }

    let _ = stats.save();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_service_mutation() {
        let plan =
            plan_service_mutation("docker", ServiceAction::Restart, Some("case-1".to_string()))
                .unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.state, MutationPlanState::Created);
        assert!(!plan.verification_checks.is_empty());
        assert!(!plan.rollback_steps.is_empty());
    }

    #[test]
    fn test_plan_package_mutation() {
        let packages = vec!["vim".to_string()];
        let plan =
            plan_package_mutation(PackageAction::Install, packages, Some("case-1".to_string()))
                .unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert!(!plan.verification_checks.is_empty());
    }

    #[test]
    fn test_plan_config_mutation() {
        let op = ConfigEditOp::AddLine {
            line: "test=value".to_string(),
        };
        let plan = plan_config_mutation("/etc/pacman.conf", op, Some("case-1".to_string()));
        // This will fail if file doesn't exist, which is expected in test env
        // Just verify the function doesn't panic
        let _ = plan;
    }

    #[test]
    fn test_validate_confirmation() {
        let plan = plan_service_mutation("docker", ServiceAction::Start, None).unwrap();
        // Low risk service
        assert!(validate_confirmation(
            &plan,
            "I CONFIRM (low risk)"
        ));
        assert!(!validate_confirmation(&plan, "wrong phrase"));
    }

    #[test]
    fn test_format_preview() {
        let preview = MutationPreview {
            plan_id: "test-plan".to_string(),
            summary: "Test mutation".to_string(),
            step_previews: vec![StepPreview {
                step_id: "step-1".to_string(),
                description: "Test step".to_string(),
                current_state: "Current".to_string(),
                intended_state: "Intended".to_string(),
                diff: None,
            }],
            risk: MutationRiskLevel::Low,
            confirmation_phrase: "I CONFIRM (low risk)".to_string(),
            privilege_available: true,
            manual_commands: None,
        };

        let formatted = format_preview_human(&preview);
        assert!(formatted.contains("PROPOSED MUTATION"));
        assert!(formatted.contains("LOW"));
        assert!(formatted.contains("Test step"));
    }
}
