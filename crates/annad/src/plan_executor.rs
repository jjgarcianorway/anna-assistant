//! Plan Executor - Execute ActionPlans with proper privilege handling.
//! Phase 16: Turn fallback into real execution.
//! Phase 17: State capture, verification, and rollback.
//! Phase 25: Verification strictness (Unknown = Failed).

use anna_shared::action_plan::{
    ActionPlan, ActionStep, PlanExecutionResult, StepResult, VerificationResult,
    VerificationStatus,
};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::process::Command;
use std::sync::RwLock;
use tracing::{debug, info, warn};

use crate::plan_stash::PlanStash;

/// TTL for pending plans (5 minutes).
const PLAN_TTL_MINUTES: i64 = 5;

/// Store pending plans by session ID.
static PENDING_PLANS: RwLock<Option<HashMap<String, ActionPlan>>> = RwLock::new(None);

/// Set a pending plan for a session.
pub fn set_pending_plan(session_id: &str, plan: ActionPlan) {
    if let Ok(mut guard) = PENDING_PLANS.write() {
        let map = guard.get_or_insert_with(HashMap::new);
        info!("Set pending plan {} for session {}", plan.id, session_id);
        map.insert(session_id.to_string(), plan);
    }
}

/// Get and remove a pending plan for a session.
/// Returns None if plan has expired (TTL exceeded).
pub fn take_pending_plan(session_id: &str) -> Option<ActionPlan> {
    if let Ok(mut guard) = PENDING_PLANS.write() {
        if let Some(map) = guard.as_mut() {
            if let Some(plan) = map.remove(session_id) {
                // Check TTL
                let age = Utc::now() - plan.created_at;
                if age > Duration::minutes(PLAN_TTL_MINUTES) {
                    info!("Plan {} expired ({}min old)", plan.id, age.num_minutes());
                    return None;
                }
                return Some(plan);
            }
        }
    }
    None
}

/// Check if a pending plan has expired.
pub fn is_plan_expired(session_id: &str) -> bool {
    if let Ok(guard) = PENDING_PLANS.read() {
        if let Some(map) = guard.as_ref() {
            if let Some(plan) = map.get(session_id) {
                let age = Utc::now() - plan.created_at;
                return age > Duration::minutes(PLAN_TTL_MINUTES);
            }
        }
    }
    false
}

/// Check if there's a pending plan for a session.
pub fn has_pending_plan(session_id: &str) -> bool {
    if let Ok(guard) = PENDING_PLANS.read() {
        if let Some(map) = guard.as_ref() {
            return map.contains_key(session_id);
        }
    }
    false
}

/// Execute a single step.
fn execute_step(step: &ActionStep, step_index: usize) -> StepResult {
    info!("Executing step {}: {}", step_index + 1, step.description);

    let cmd = if step.needs_sudo {
        // Use pkexec for privilege escalation (GUI prompt)
        format!("pkexec sh -c '{}'", step.command.replace('\'', "'\\''"))
    } else {
        step.command.clone()
    };

    debug!("Running: {}", cmd);

    match Command::new("sh").arg("-c").arg(&cmd).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                info!("Step {} succeeded", step_index + 1);
                // Run per-step verification if defined
                let verified = run_step_verification(step);
                StepResult {
                    step_index,
                    success: true,
                    output: stdout,
                    error: None,
                    verified,
                }
            } else {
                warn!("Step {} failed: {}", step_index + 1, stderr);
                StepResult {
                    step_index,
                    success: false,
                    output: stdout,
                    error: Some(stderr),
                    verified: None,
                }
            }
        }
        Err(e) => {
            warn!("Step {} execution error: {}", step_index + 1, e);
            StepResult {
                step_index,
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
                verified: None,
            }
        }
    }
}

/// Run per-step verification if defined.
fn run_step_verification(step: &ActionStep) -> Option<bool> {
    let verify_cmd = step.verify_command.as_ref()?;
    let verify_pattern = step.verify_pattern.as_ref()?;

    debug!("Running step verification: {}", verify_cmd);

    match Command::new("sh").arg("-c").arg(verify_cmd).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let passed = stdout.contains(verify_pattern) || verify_pattern.is_empty();
            debug!("Step verification passed: {}", passed);
            Some(passed)
        }
        Err(e) => {
            warn!("Step verification failed: {}", e);
            Some(false)
        }
    }
}

/// Run final verification check.
/// Phase 25: Returns (VerificationResult, VerificationStatus) for strictness.
fn run_verification(plan: &ActionPlan) -> (Option<VerificationResult>, VerificationStatus) {
    let verification = match plan.verification.as_ref() {
        Some(v) => v,
        None => return (None, VerificationStatus::Unknown),
    };

    info!("Running verification: {}", verification.description);

    match Command::new("sh")
        .arg("-c")
        .arg(&verification.command)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let passed = stdout.contains(&verification.success_pattern);

            // Phase 25: Determine verification status
            let status = if passed {
                VerificationStatus::Passed
            } else {
                VerificationStatus::Failed
            };

            let result = VerificationResult {
                passed,
                actual_output: stdout,
                explanation: if passed {
                    format!("Verified: {}", verification.description)
                } else {
                    format!(
                        "Verification failed: expected '{}' in output",
                        verification.success_pattern
                    )
                },
            };

            (Some(result), status)
        }
        Err(e) => {
            // Phase 25: Command execution error = Unknown
            let result = VerificationResult {
                passed: false,
                actual_output: String::new(),
                explanation: format!("Verification command failed: {}", e),
            };
            (Some(result), VerificationStatus::Unknown)
        }
    }
}

/// Execute rollback for failed steps.
fn execute_rollback(stash: &PlanStash, failed_step: usize) -> bool {
    info!("Executing rollback up to step {}", failed_step);

    // Rollback in reverse order
    for step_idx in (0..=failed_step).rev() {
        if let Some(step_state) = stash.get_step_state(step_idx) {
            // Restore files
            for backup in &step_state.file_backups {
                if let Err(e) = step_state.restore_file(backup) {
                    warn!("Failed to restore file: {}", e);
                }
            }
            // Restore unit states
            for unit_state in &step_state.unit_states {
                if let Err(e) = step_state.restore_unit_state(unit_state) {
                    warn!("Failed to restore unit state: {}", e);
                }
            }
        }
    }
    true
}

/// Execute an action plan with state capture and rollback support.
pub fn execute_plan(plan: &ActionPlan) -> PlanExecutionResult {
    info!("Executing plan: {} ({})", plan.summary, plan.id);

    // Handle no-changes case
    if !plan.changes_needed {
        // Phase 25: No verification when no changes needed
        return PlanExecutionResult {
            plan_id: plan.id.clone(),
            success: true,
            step_results: Vec::new(),
            verification_result: None,
            verification_status: VerificationStatus::Passed, // No changes = trivially verified
            rollback_performed: false,
            rollback_success: None,
            completed_at: Utc::now(),
        };
    }

    // Initialize stash for rollback
    let mut stash = PlanStash::new(&plan.id);
    let stash_initialized = stash.init().is_ok();

    let mut step_results = Vec::new();
    let mut all_success = true;
    let mut failed_step: Option<usize> = None;

    for (i, step) in plan.steps.iter().enumerate() {
        // Capture state before execution
        if stash_initialized {
            let step_state = stash.create_step_state(i);
            for file in &step.affects_files {
                let _ = step_state.backup_file(file);
            }
            for unit in &step.affects_units {
                let _ = step_state.capture_unit_state(unit);
            }
        }

        // Execute step
        let result = execute_step(step, i);
        let step_failed = !result.success || result.verified == Some(false);

        if step_failed {
            all_success = false;
            failed_step = Some(i);
            step_results.push(result);
            break;
        }
        step_results.push(result);
    }

    // Phase 25: Run verification with status tracking
    let (verification_result, verification_status) = if all_success {
        run_verification(plan)
    } else {
        (None, VerificationStatus::Unknown)
    };

    // Phase 25: Success requires verification_status == Passed (Unknown = Failed)
    let verification_success = verification_status == VerificationStatus::Passed;
    let overall_success = all_success && verification_success;

    let need_rollback = !overall_success;
    let mut rollback_performed = false;
    let mut rollback_success = None;

    if need_rollback && plan.rollback.possible && stash_initialized {
        let rollback_to = failed_step.unwrap_or(plan.steps.len().saturating_sub(1));
        rollback_performed = true;
        rollback_success = Some(execute_rollback(&stash, rollback_to));
    }

    // Cleanup stash on success
    if overall_success {
        let _ = stash.cleanup();
    }

    PlanExecutionResult {
        plan_id: plan.id.clone(),
        success: overall_success,
        step_results,
        verification_result,
        verification_status,
        rollback_performed,
        rollback_success,
        completed_at: Utc::now(),
    }
}

/// Format execution result for display to user.
pub fn format_execution_result(result: &PlanExecutionResult, plan: &ActionPlan) -> String {
    let mut output = String::new();

    // Handle no-changes case
    if !plan.changes_needed {
        return format!(
            "No changes needed. {}",
            plan.skip_reason.as_deref().unwrap_or("Already configured.")
        );
    }

    if result.success {
        output.push_str(&format!("Done. {}\n", plan.summary));
    } else {
        output.push_str(&format!("Failed to {}.\n", plan.summary.to_lowercase()));
    }

    // Show step results
    for (i, step_result) in result.step_results.iter().enumerate() {
        let status = if step_result.success { "OK" } else { "FAILED" };
        let step_desc = plan
            .steps
            .get(i)
            .map(|s| s.description.as_str())
            .unwrap_or("Unknown step");
        output.push_str(&format!("  [{}] {}\n", status, step_desc));

        if let Some(ref err) = step_result.error {
            output.push_str(&format!("       Error: {}\n", err.trim()));
        }
    }

    // Show verification result
    if let Some(ref v) = result.verification_result {
        let status = if v.passed { "VERIFIED" } else { "UNVERIFIED" };
        output.push_str(&format!("\n[{}] {}\n", status, v.explanation));
    }

    // Show rollback status
    if result.rollback_performed {
        let status = match result.rollback_success {
            Some(true) => "Changes rolled back successfully.",
            Some(false) => "Rollback attempted but had errors.",
            None => "Rollback status unknown.",
        };
        output.push_str(&format!("\n{}\n", status));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_plan_storage() {
        let plan = ActionPlan::new("test question", "Test Plan", "Testing");
        set_pending_plan("session-1", plan);

        assert!(has_pending_plan("session-1"));
        assert!(!has_pending_plan("session-2"));

        let retrieved = take_pending_plan("session-1");
        assert!(retrieved.is_some());
        assert!(!has_pending_plan("session-1"));
    }

    #[test]
    fn test_execute_simple_step() {
        let step = ActionStep::new("Echo test", "echo 'hello world'", false);
        let result = execute_step(&step, 0);
        assert!(result.success && result.output.contains("hello"));
    }

    #[test]
    fn test_execute_failing_step() {
        let step = ActionStep::new("Failing", "exit 1", false);
        assert!(!execute_step(&step, 0).success);
    }

    #[test]
    fn test_no_changes_plan() {
        let mut plan = ActionPlan::new("test", "Test", "Testing");
        plan.mark_no_changes("Already configured");
        let result = execute_plan(&plan);
        assert!(result.success && result.step_results.is_empty());
    }

    #[test]
    fn test_plan_ttl_and_expiry() {
        assert_eq!(PLAN_TTL_MINUTES, 5);
        // Fresh plan should not be expired
        let plan = ActionPlan::new("test", "Test", "Testing");
        set_pending_plan("ttl-test", plan);
        assert!(!is_plan_expired("ttl-test"));
        let _ = take_pending_plan("ttl-test");
        // Non-existent plan should return false
        assert!(!is_plan_expired("nonexistent-xyz"));
    }

    #[test]
    fn test_phase25_no_changes_verification_status() {
        let mut plan = ActionPlan::new("test", "Test", "Testing");
        plan.mark_no_changes("Already configured");
        let result = execute_plan(&plan);
        assert!(result.success);
        // Phase 25: No changes = trivially verified (Passed)
        assert_eq!(result.verification_status, VerificationStatus::Passed);
    }
}
