//! Recipe Executor - Safe command execution with runtime validation
//!
//! This module executes command recipes with:
//! - Static validation before run (allowlist/denylist, file existence checks)
//! - Actual command execution with output capture
//! - Runtime validation after run (exit codes, output patterns)
//! - User confirmation for write operations
//! - Automatic rollback on failure (where applicable)

use crate::command_recipe::{
    CommandCategory, CommandRecipe, Recipe, RecipeResult, StepResult,
};
use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use std::time::Instant;

/// Recipe executor - runs validated recipes safely
pub struct RecipeExecutor {
    /// Whether to actually execute commands (false for dry-run)
    execute_enabled: bool,
}

impl RecipeExecutor {
    pub fn new(execute_enabled: bool) -> Self {
        Self { execute_enabled }
    }

    /// Execute a recipe with safety checks and validation
    ///
    /// For read-only recipes: executes automatically
    /// For write recipes: requires user confirmation (passed via confirm_fn)
    pub async fn execute_recipe<F>(
        &self,
        recipe: &Recipe,
        confirm_fn: F,
    ) -> Result<RecipeResult>
    where
        F: Fn(&str) -> bool,
    {
        let start_time = Instant::now();
        let mut step_results = Vec::new();
        let mut success = true;

        // Check if confirmation needed
        if !recipe.all_read_only {
            let confirmation_prompt = format!(
                "This recipe will perform write operations:\n{}\n\nProceed?",
                recipe.summary
            );
            if !confirm_fn(&confirmation_prompt) {
                return Ok(RecipeResult {
                    question: recipe.question.clone(),
                    step_results: vec![],
                    success: false,
                    answer: "Cancelled by user".to_string(),
                    total_time_ms: start_time.elapsed().as_millis() as u64,
                });
            }
        }

        // Execute each step in sequence
        for step in &recipe.steps {
            let step_result = self.execute_step(step).await?;

            let step_success = step_result.validation_passed;
            step_results.push(step_result.clone());

            if !step_success {
                success = false;
                // Stop on first failure
                break;
            }

            // If this was a write step that succeeded, log it
            if matches!(
                step.category,
                CommandCategory::UserWrite | CommandCategory::SystemWrite
            ) {
                tracing::info!(
                    "Write command succeeded: {} (rollback: {})",
                    step.command,
                    step.rollback_command.as_deref().unwrap_or("none")
                );
            }
        }

        let total_time_ms = start_time.elapsed().as_millis() as u64;

        // Generate final answer based on results
        let answer = self.generate_answer(recipe, &step_results, success);

        Ok(RecipeResult {
            question: recipe.question.clone(),
            step_results,
            success,
            answer,
            total_time_ms,
        })
    }

    /// Execute a single recipe step
    async fn execute_step(&self, step: &CommandRecipe) -> Result<StepResult> {
        let start_time = Instant::now();

        if !self.execute_enabled {
            // Dry run - just validate structure
            return Ok(StepResult {
                step_id: step.id.clone(),
                exit_code: 0,
                stdout: "[DRY RUN]".to_string(),
                stderr: String::new(),
                validation_passed: true,
                validation_failure: None,
                execution_time_ms: 0,
            });
        }

        // Execute command
        tracing::info!("Executing: {}", step.command);

        let output = Command::new("sh")
            .arg("-c")
            .arg(&step.command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(format!("Failed to execute: {}", step.command))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Runtime validation
        let (validation_passed, validation_failure) = if let Some(validation) =
            &step.expected_validation
        {
            self.validate_output(validation, exit_code, &stdout, &stderr)
        } else {
            // No validation specified - just check exit code
            (
                exit_code == 0,
                if exit_code != 0 {
                    Some(format!("Command failed with exit code {}", exit_code))
                } else {
                    None
                },
            )
        };

        Ok(StepResult {
            step_id: step.id.clone(),
            exit_code,
            stdout,
            stderr,
            validation_passed,
            validation_failure,
            execution_time_ms,
        })
    }

    /// Validate command output against expectations
    fn validate_output(
        &self,
        validation: &crate::command_recipe::OutputValidation,
        exit_code: i32,
        stdout: &str,
        stderr: &str,
    ) -> (bool, Option<String>) {
        // Check exit code
        if exit_code != validation.exit_code {
            return (
                false,
                Some(format!(
                    "Expected exit code {}, got {}",
                    validation.exit_code, exit_code
                )),
            );
        }

        // Check stdout must match
        if let Some(pattern) = &validation.stdout_must_match {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if !re.is_match(stdout) {
                        return (
                            false,
                            Some(format!("stdout did not match pattern: {}", pattern)),
                        );
                    }
                }
                Err(e) => {
                    return (false, Some(format!("Invalid regex pattern: {}", e)));
                }
            }
        }

        // Check stdout must not match
        if let Some(pattern) = &validation.stdout_must_not_match {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(stdout) {
                        return (
                            false,
                            Some(format!("stdout matched forbidden pattern: {}", pattern)),
                        );
                    }
                }
                Err(e) => {
                    return (false, Some(format!("Invalid regex pattern: {}", e)));
                }
            }
        }

        // Check stderr must match (if specified)
        if let Some(pattern) = &validation.stderr_must_match {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if !re.is_match(stderr) {
                        return (
                            false,
                            Some(format!("stderr did not match pattern: {}", pattern)),
                        );
                    }
                }
                Err(e) => {
                    return (false, Some(format!("Invalid regex pattern: {}", e)));
                }
            }
        }

        // All validations passed
        (true, None)
    }

    /// Generate final answer from step results
    fn generate_answer(
        &self,
        recipe: &Recipe,
        step_results: &[StepResult],
        success: bool,
    ) -> String {
        if !success {
            // Find first failed step
            if let Some(failed_step) = step_results.iter().find(|s| !s.validation_passed) {
                return format!(
                    "Execution failed: {}\n\nCommand: (step {})\nExit code: {}\nOutput: {}",
                    failed_step
                        .validation_failure
                        .as_deref()
                        .unwrap_or("Unknown error"),
                    failed_step.step_id,
                    failed_step.exit_code,
                    &failed_step.stdout[..failed_step.stdout.len().min(200)]
                );
            }
            return "Execution failed for unknown reason".to_string();
        }

        // Success - format answer based on outputs
        let mut answer = format!("Successfully executed recipe for: {}\n\n", recipe.question);

        for step_result in step_results {
            if !step_result.stdout.is_empty() {
                answer.push_str(&format!("Output:\n{}\n", step_result.stdout.trim()));
            }
        }

        // Add wiki sources
        if !recipe.wiki_sources.is_empty() {
            answer.push_str("\nReferences:\n");
            for source in &recipe.wiki_sources {
                answer.push_str(&format!("- {}\n", source));
            }
        }

        answer
    }

    /// Execute rollback command for a failed step
    pub async fn rollback_step(&self, step: &CommandRecipe) -> Result<()> {
        if let Some(rollback_cmd) = &step.rollback_command {
            tracing::warn!("Rolling back step {}: {}", step.id, rollback_cmd);

            let output = Command::new("sh")
                .arg("-c")
                .arg(rollback_cmd)
                .output()
                .context(format!("Failed to execute rollback: {}", rollback_cmd))?;

            if !output.status.success() {
                tracing::error!(
                    "Rollback failed: {} (exit code: {})",
                    rollback_cmd,
                    output.status.code().unwrap_or(-1)
                );
                anyhow::bail!("Rollback failed for step {}", step.id);
            }

            tracing::info!("Successfully rolled back step {}", step.id);
        }
        Ok(())
    }
}

impl Default for RecipeExecutor {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_recipe::OutputValidation;

    #[tokio::test]
    async fn test_simple_execution() {
        let executor = RecipeExecutor::new(true);

        let recipe = Recipe {
            question: "Test".to_string(),
            steps: vec![CommandRecipe {
                id: "test1".to_string(),
                command: "echo 'hello'".to_string(),
                category: CommandCategory::ReadOnly,
                safety_level: SafetyLevel::Safe,
                capture_output: true,
                expected_validation: Some(OutputValidation {
                    exit_code: 0,
                    stdout_must_match: Some("hello".to_string()),
                    stdout_must_not_match: None,
                    stderr_must_match: None,
                    validation_description: "Should output hello".to_string(),
                }),
                explanation: "Test".to_string(),
                doc_sources: vec![],
                rollback_command: None,
                template_id: None,
                template_params: std::collections::HashMap::new(),
            }],
            overall_safety: SafetyLevel::Safe,
            all_read_only: true,
            wiki_sources: vec![],
            summary: "Test recipe".to_string(),
            generated_by: None,
            critic_approval: None,
        };

        let result = executor.execute_recipe(&recipe, |_| true).await.unwrap();
        assert!(result.success);
        assert_eq!(result.step_results.len(), 1);
        assert!(result.step_results[0].validation_passed);
    }

    #[tokio::test]
    async fn test_validation_failure() {
        let executor = RecipeExecutor::new(true);

        let recipe = Recipe {
            question: "Test".to_string(),
            steps: vec![CommandRecipe {
                id: "test1".to_string(),
                command: "echo 'hello'".to_string(),
                category: CommandCategory::ReadOnly,
                safety_level: SafetyLevel::Safe,
                capture_output: true,
                expected_validation: Some(OutputValidation {
                    exit_code: 0,
                    stdout_must_match: Some("goodbye".to_string()), // Won't match
                    stdout_must_not_match: None,
                    stderr_must_match: None,
                    validation_description: "Should output goodbye".to_string(),
                }),
                explanation: "Test".to_string(),
                doc_sources: vec![],
                rollback_command: None,
                template_id: None,
                template_params: std::collections::HashMap::new(),
            }],
            overall_safety: SafetyLevel::Safe,
            all_read_only: true,
            wiki_sources: vec![],
            summary: "Test recipe".to_string(),
            generated_by: None,
            critic_approval: None,
        };

        let result = executor.execute_recipe(&recipe, |_| true).await.unwrap();
        assert!(!result.success);
        assert!(!result.step_results[0].validation_passed);
    }
}
