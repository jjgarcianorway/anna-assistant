//! Plan Executor - Execute ActionPlans with proper privilege handling.
//! Phase 16: Turn fallback into real execution.

use anna_shared::action_plan::{
    ActionPlan, ActionStep, PlanExecutionResult, StepResult, VerificationResult,
};
use chrono::Utc;
use std::collections::HashMap;
use std::process::Command;
use std::sync::RwLock;
use tracing::{debug, info, warn};

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
pub fn take_pending_plan(session_id: &str) -> Option<ActionPlan> {
    if let Ok(mut guard) = PENDING_PLANS.write() {
        if let Some(map) = guard.as_mut() {
            return map.remove(session_id);
        }
    }
    None
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
                StepResult {
                    step_index,
                    success: true,
                    output: stdout,
                    error: None,
                }
            } else {
                warn!("Step {} failed: {}", step_index + 1, stderr);
                StepResult {
                    step_index,
                    success: false,
                    output: stdout,
                    error: Some(stderr),
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
            }
        }
    }
}

/// Run verification check.
fn run_verification(plan: &ActionPlan) -> Option<VerificationResult> {
    let verification = plan.verification.as_ref()?;

    info!("Running verification: {}", verification.description);

    match Command::new("sh")
        .arg("-c")
        .arg(&verification.command)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let passed = stdout.contains(&verification.success_pattern);

            Some(VerificationResult {
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
            })
        }
        Err(e) => Some(VerificationResult {
            passed: false,
            actual_output: String::new(),
            explanation: format!("Verification command failed: {}", e),
        }),
    }
}

/// Execute an action plan.
pub fn execute_plan(plan: &ActionPlan) -> PlanExecutionResult {
    info!("Executing plan: {} ({})", plan.summary, plan.id);

    let mut step_results = Vec::new();
    let mut all_success = true;

    for (i, step) in plan.steps.iter().enumerate() {
        let result = execute_step(step, i);
        if !result.success {
            all_success = false;
            step_results.push(result);
            // Stop on first failure
            break;
        }
        step_results.push(result);
    }

    // Run verification if all steps succeeded
    let verification_result = if all_success {
        run_verification(plan)
    } else {
        None
    };

    // Update success based on verification
    let final_success = if let Some(ref v) = verification_result {
        all_success && v.passed
    } else {
        all_success
    };

    PlanExecutionResult {
        plan_id: plan.id.clone(),
        success: final_success,
        step_results,
        verification_result,
        completed_at: Utc::now(),
    }
}

/// Format execution result for display to user.
pub fn format_execution_result(result: &PlanExecutionResult, plan: &ActionPlan) -> String {
    let mut output = String::new();

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
        let step = ActionStep {
            description: "Echo test".to_string(),
            command: "echo 'hello world'".to_string(),
            needs_sudo: false,
            expected_output: None,
        };

        let result = execute_step(&step, 0);
        assert!(result.success);
        assert!(result.output.contains("hello"));
    }

    #[test]
    fn test_execute_failing_step() {
        let step = ActionStep {
            description: "Failing command".to_string(),
            command: "exit 1".to_string(),
            needs_sudo: false,
            expected_output: None,
        };

        let result = execute_step(&step, 0);
        assert!(!result.success);
    }
}
