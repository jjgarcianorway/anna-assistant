//! Step execution handlers for different recipe step types.

use crate::recipe_engine::{Recipe, RecipeStep, RecipeStepType};
use tracing::debug;

use super::file_ops::{append_to_file, create_backup, prepend_to_file};
use super::types::{ExecutionContext, StepOutput};
use super::utils::{evaluate_condition, probe_id_to_command, run_shell_command, substitute_params};

/// Recipe executor
pub struct RecipeExecutor {
    /// Confirmation callback (returns true if user confirms)
    confirm_fn: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl Default for RecipeExecutor {
    fn default() -> Self {
        Self { confirm_fn: None }
    }
}

impl RecipeExecutor {
    /// Create new executor
    pub fn new() -> Self {
        Self::default()
    }

    /// Set confirmation callback
    pub fn with_confirm<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.confirm_fn = Some(Box::new(f));
        self
    }

    /// Execute a single step
    pub fn execute_step(
        &self,
        step: &RecipeStep,
        recipe: &Recipe,
        ctx: &mut ExecutionContext,
    ) -> Result<StepOutput, String> {
        let start = std::time::Instant::now();

        match step.kind {
            RecipeStepType::RunProbe => self.run_probe(step, ctx),
            RecipeStepType::RunCommand => self.run_command(step, recipe, ctx),
            RecipeStepType::CheckCondition => self.check_condition(step, ctx),
            RecipeStepType::EditFile => self.edit_file(step, recipe, ctx),
            RecipeStepType::RenderAnswer => self.render_answer(step, recipe, ctx),
            RecipeStepType::Subrecipe => Err("Subrecipe execution not yet implemented".to_string()),
        }
        .map(|mut output| {
            output.duration_ms = start.elapsed().as_millis() as u64;
            output
        })
    }

    /// Run a probe
    pub fn run_probe(
        &self,
        step: &RecipeStep,
        ctx: &ExecutionContext,
    ) -> Result<StepOutput, String> {
        let probe_id = step.params.get("probe_id").ok_or("Missing probe_id")?;
        let command = probe_id_to_command(probe_id);
        let command = substitute_params(&command, &ctx.params);

        debug!("Running probe: {}", command);
        run_shell_command(&command, &step.id)
    }

    /// Run a shell command with safety checks
    pub fn run_command(
        &self,
        step: &RecipeStep,
        recipe: &Recipe,
        ctx: &mut ExecutionContext,
    ) -> Result<StepOutput, String> {
        let command = step.params.get("command").ok_or("Missing command")?;
        let command = substitute_params(command, &ctx.params);

        // Check if confirmation needed
        if step.requires_confirmation() && !ctx.auto_confirm {
            let msg = format!(
                "Recipe '{}' wants to run:\n  {}\n\nAllow this command?",
                recipe.name, command
            );

            if let Some(ref confirm_fn) = self.confirm_fn {
                if !confirm_fn(&msg) {
                    return Ok(StepOutput {
                        step_id: step.id.clone(),
                        success: false,
                        stdout: String::new(),
                        stderr: "User declined".to_string(),
                        exit_code: -1,
                        duration_ms: 0,
                    });
                }
            }
        }

        debug!("Running command: {}", command);
        run_shell_command(&command, &step.id)
    }

    /// Check a condition
    pub fn check_condition(
        &self,
        step: &RecipeStep,
        ctx: &ExecutionContext,
    ) -> Result<StepOutput, String> {
        let condition = step.params.get("condition").ok_or("Missing condition")?;
        let target_step = step.params.get("target_step");

        // Get output from target step if specified
        let check_text = if let Some(target) = target_step {
            ctx.outputs
                .get(target)
                .map(|o| o.stdout.as_str())
                .unwrap_or("")
        } else {
            ""
        };

        // Simple condition checks
        let success = evaluate_condition(condition, check_text);

        Ok(StepOutput {
            step_id: step.id.clone(),
            success,
            stdout: if success {
                "condition met"
            } else {
                "condition not met"
            }
            .to_string(),
            stderr: String::new(),
            exit_code: if success { 0 } else { 1 },
            duration_ms: 0,
        })
    }

    /// Edit a file with safety
    pub fn edit_file(
        &self,
        step: &RecipeStep,
        recipe: &Recipe,
        ctx: &mut ExecutionContext,
    ) -> Result<StepOutput, String> {
        let file_path = step.params.get("file").ok_or("Missing file path")?;
        let file_path = substitute_params(file_path, &ctx.params);
        let content = step.params.get("content").ok_or("Missing content")?;
        let content = substitute_params(content, &ctx.params);
        let mode = step
            .params
            .get("mode")
            .map(|s| s.as_str())
            .unwrap_or("append");

        // Always ask for confirmation on file edits
        if !ctx.auto_confirm {
            let msg =
                format!(
                "Recipe '{}' wants to {} file:\n  {}\nWith content:\n  {}\n\nAllow this change?",
                recipe.name, mode, file_path, content.chars().take(100).collect::<String>()
            );

            if let Some(ref confirm_fn) = self.confirm_fn {
                if !confirm_fn(&msg) {
                    return Ok(StepOutput {
                        step_id: step.id.clone(),
                        success: false,
                        stdout: String::new(),
                        stderr: "User declined file edit".to_string(),
                        exit_code: -1,
                        duration_ms: 0,
                    });
                }
            }
        }

        // Create backup
        let backup_path = create_backup(&file_path)?;

        // Perform edit
        let result = match mode {
            "append" => append_to_file(&file_path, &content),
            "prepend" => prepend_to_file(&file_path, &content),
            "replace" => std::fs::write(&file_path, &content).map_err(|e| e.to_string()),
            _ => Err(format!("Unknown edit mode: {}", mode)),
        };

        match result {
            Ok(()) => Ok(StepOutput {
                step_id: step.id.clone(),
                success: true,
                stdout: format!("Edited {} (backup: {})", file_path, backup_path),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 0,
            }),
            Err(e) => Ok(StepOutput {
                step_id: step.id.clone(),
                success: false,
                stdout: String::new(),
                stderr: e,
                exit_code: 1,
                duration_ms: 0,
            }),
        }
    }

    /// Render the final answer
    pub fn render_answer(
        &self,
        step: &RecipeStep,
        _recipe: &Recipe,
        ctx: &ExecutionContext,
    ) -> Result<StepOutput, String> {
        let template = step.params.get("template").ok_or("Missing template")?;

        // Substitute parameters
        let mut answer = substitute_params(template, &ctx.params);

        // Substitute step outputs ({{step_id.stdout}}, {{step_id.exit_code}})
        for (step_id, output) in &ctx.outputs {
            answer = answer.replace(&format!("{{{{{}.stdout}}}}", step_id), &output.stdout);
            answer = answer.replace(&format!("{{{{{}.stderr}}}}", step_id), &output.stderr);
            answer = answer.replace(
                &format!("{{{{{}.exit_code}}}}", step_id),
                &output.exit_code.to_string(),
            );
        }

        Ok(StepOutput {
            step_id: step.id.clone(),
            success: true,
            stdout: answer,
            stderr: String::new(),
            exit_code: 0,
            duration_ms: 0,
        })
    }
}
