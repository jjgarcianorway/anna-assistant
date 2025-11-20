//! ActionPlan Execution Engine
//!
//! Beta.147: Executes structured action plans with safety checks and rollback
//!
//! Features:
//! - Runs necessary checks before execution
//! - User confirmation for risky operations
//! - Step-by-step command execution
//! - Automatic rollback on failure
//! - Detailed execution logging

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::{Result, anyhow};
use std::process::Command;

/// Execution result for a single step
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Complete execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub checks_passed: Vec<String>,
    pub checks_failed: Vec<String>,
    pub steps_completed: Vec<StepResult>,
    pub steps_failed: Vec<StepResult>,
    pub rollback_performed: bool,
    pub rollback_results: Vec<StepResult>,
}

impl ExecutionResult {
    pub fn new() -> Self {
        Self {
            success: false,
            checks_passed: Vec::new(),
            checks_failed: Vec::new(),
            steps_completed: Vec::new(),
            steps_failed: Vec::new(),
            rollback_performed: false,
            rollback_results: Vec::new(),
        }
    }
}

/// ActionPlan executor
pub struct ActionPlanExecutor {
    pub plan: ActionPlan,
    pub auto_confirm: bool,  // Skip confirmation prompts (dangerous!)
}

impl ActionPlanExecutor {
    /// Create new executor for an action plan
    pub fn new(plan: ActionPlan) -> Self {
        Self {
            plan,
            auto_confirm: false,
        }
    }

    /// Execute the complete action plan
    pub async fn execute(&self) -> Result<ExecutionResult> {
        let mut result = ExecutionResult::new();

        // Step 1: Run necessary checks
        eprintln!("🔍 Running necessary checks...");
        for check in &self.plan.necessary_checks {
            match self.execute_check(check).await {
                Ok(check_result) => {
                    if check_result.success {
                        eprintln!("  ✅ {}", check.description);
                        result.checks_passed.push(check.id.clone());
                    } else {
                        eprintln!("  ❌ {}: {}", check.description, check_result.stderr);
                        result.checks_failed.push(check.id.clone());

                        if check.required {
                            return Err(anyhow!(
                                "Required check '{}' failed: {}",
                                check.id,
                                check_result.stderr
                            ));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  ⚠️ Check '{}' error: {}", check.description, e);
                    result.checks_failed.push(check.id.clone());

                    if check.required {
                        return Err(anyhow!("Required check '{}' failed: {}", check.id, e));
                    }
                }
            }
        }

        // Step 2: Get user confirmation if needed
        if self.plan.requires_confirmation() && !self.auto_confirm {
            let max_risk = self.plan.max_risk_level().unwrap_or(RiskLevel::Info);
            eprintln!();
            eprintln!("⚠️  This plan requires confirmation.");
            eprintln!("   Max Risk: {:?}", max_risk);
            eprintln!("   Steps: {}", self.plan.command_plan.len());
            eprintln!();

            if !self.ask_confirmation()? {
                eprintln!("❌ Execution cancelled by user.");
                return Ok(result);
            }
        }

        // Step 3: Execute command plan
        eprintln!();
        eprintln!("🚀 Executing command plan...");

        for (i, step) in self.plan.command_plan.iter().enumerate() {
            eprintln!("  {}. {} {}", i + 1, step.risk_level.emoji(), step.description);

            match self.execute_step(step).await {
                Ok(step_result) => {
                    if step_result.success {
                        eprintln!("     ✅ Success");
                        result.steps_completed.push(step_result);
                    } else {
                        eprintln!("     ❌ Failed (exit code: {})", step_result.exit_code);
                        if !step_result.stderr.is_empty() {
                            eprintln!("     Error: {}", step_result.stderr.trim());
                        }
                        result.steps_failed.push(step_result);

                        // Rollback on failure
                        eprintln!();
                        eprintln!("🔄 Command failed, initiating rollback...");
                        let completed = result.steps_completed.clone();
                        self.perform_rollback(&completed, &mut result).await?;
                        return Ok(result);
                    }
                }
                Err(e) => {
                    eprintln!("     ⚠️ Execution error: {}", e);
                    let failed_result = StepResult {
                        step_id: step.id.clone(),
                        success: false,
                        stdout: String::new(),
                        stderr: e.to_string(),
                        exit_code: -1,
                    };
                    result.steps_failed.push(failed_result);

                    // Rollback on error
                    eprintln!();
                    eprintln!("🔄 Error occurred, initiating rollback...");
                    let completed = result.steps_completed.clone();
                    self.perform_rollback(&completed, &mut result).await?;
                    return Ok(result);
                }
            }
        }

        eprintln!();
        eprintln!("✅ Action plan completed successfully!");
        result.success = true;
        Ok(result)
    }

    /// Execute a necessary check
    async fn execute_check(&self, check: &NecessaryCheck) -> Result<StepResult> {
        self.execute_command(&check.id, &check.command).await
    }

    /// Execute a command step
    async fn execute_step(&self, step: &CommandStep) -> Result<StepResult> {
        self.execute_command(&step.id, &step.command).await
    }

    /// Execute a shell command
    async fn execute_command(&self, id: &str, command: &str) -> Result<StepResult> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        Ok(StepResult {
            step_id: id.to_string(),
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    /// Perform rollback for completed steps
    async fn perform_rollback(
        &self,
        completed_steps: &[StepResult],
        result: &mut ExecutionResult,
    ) -> Result<()> {
        result.rollback_performed = true;

        // Rollback in reverse order
        for step_result in completed_steps.iter().rev() {
            // Find corresponding rollback
            if let Some(step) = self.plan.command_plan.iter().find(|s| s.id == step_result.step_id) {
                if let Some(rollback_id) = &step.rollback_id {
                    if let Some(rollback) = self.plan.rollback_plan.iter().find(|r| &r.id == rollback_id) {
                        eprintln!("  ↩ Rollback: {}", rollback.description);

                        match self.execute_rollback(rollback).await {
                            Ok(rollback_result) => {
                                if rollback_result.success {
                                    eprintln!("     ✅ Rollback successful");
                                } else {
                                    eprintln!("     ⚠️ Rollback failed (exit code: {})", rollback_result.exit_code);
                                }
                                result.rollback_results.push(rollback_result);
                            }
                            Err(e) => {
                                eprintln!("     ⚠️ Rollback error: {}", e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a rollback step
    async fn execute_rollback(&self, rollback: &RollbackStep) -> Result<StepResult> {
        self.execute_command(&rollback.id, &rollback.command).await
    }

    /// Ask user for confirmation
    fn ask_confirmation(&self) -> Result<bool> {
        use std::io::{self, Write};

        print!("Execute this plan? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().eq_ignore_ascii_case("y"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::action_plan_v3::PlanMeta;

    #[tokio::test]
    async fn test_simple_execution() {
        let plan = ActionPlan {
            analysis: "Test plan".to_string(),
            goals: vec!["Test goal".to_string()],
            necessary_checks: vec![],
            command_plan: vec![CommandStep {
                id: "step1".to_string(),
                description: "Echo test".to_string(),
                command: "echo 'Hello, World!'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            }],
            rollback_plan: vec![],
            notes_for_user: "Test notes".to_string(),
            meta: PlanMeta {
                detection_results: Default::default(),
                template_used: None,
                llm_version: "test".to_string(),
            },
        };

        let mut executor = ActionPlanExecutor::new(plan);
        executor.auto_confirm = true;

        let result = executor.execute().await.unwrap();
        assert!(result.success);
        assert_eq!(result.steps_completed.len(), 1);
        assert_eq!(result.steps_failed.len(), 0);
    }
}
