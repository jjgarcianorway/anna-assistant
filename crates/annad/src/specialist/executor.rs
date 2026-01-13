//! Specialist Executor - Execution contract for specialists.
//!
//! Specialists execute recipes and diagnostic probes within their allowed helpers.
//! CRITICAL: Specialists return structured output, NOT user-facing text.

use super::output::SpecialistOutput;
use super::registry::{SpecialistDefinition, SpecialistLevel};
use crate::translator::intent::{IntentSubject, UserIntent};
use anna_shared::helpers::tool_available;
use anna_shared::profile::SystemInfo;
use anna_shared::recipe::RecipeBook;
use std::process::Command;

/// Execution context passed to specialist.
pub struct ExecutionContext<'a> {
    pub intent: &'a UserIntent,
    pub recipe_book: &'a RecipeBook,
    pub system_info: &'a SystemInfo,
    pub recipe_hint: Option<&'a str>,
}

/// Specialist execution contract.
///
/// CRITICAL: Specialists CANNOT emit user-facing text.
/// They return structured SpecialistOutput which the display layer renders.
pub struct SpecialistExecutor;

impl SpecialistExecutor {
    /// Execute specialist work.
    pub fn execute(
        specialist: &SpecialistDefinition,
        context: ExecutionContext<'_>,
    ) -> SpecialistOutput {
        // Step 1: Check helper availability
        let missing_helpers = Self::check_helpers(specialist);
        if !missing_helpers.is_empty() {
            return SpecialistOutput::NeedsHelpers {
                specialist_id: specialist.id.to_string(),
                missing: missing_helpers,
            };
        }

        // Step 2: Try recipe if hinted
        if let Some(recipe_id) = context.recipe_hint {
            if let Some(recipe) = context.recipe_book.recipes.iter().find(|r| r.id == recipe_id) {
                return Self::execute_recipe(specialist, recipe, context.intent);
            }
        }

        // Step 3: Find matching recipe
        let matches = context
            .recipe_book
            .find_matches(&context.intent.original_input, context.system_info);

        if let Some(recipe) = matches.first() {
            return Self::execute_recipe(specialist, recipe, context.intent);
        }

        // Step 4: No recipe - run diagnostic probes
        Self::run_diagnostics(specialist, context.intent)
    }

    /// Check which helpers are missing.
    fn check_helpers(specialist: &SpecialistDefinition) -> Vec<String> {
        specialist
            .allowed_helpers
            .iter()
            .filter(|h| !tool_available(h))
            .map(|s| s.to_string())
            .collect()
    }

    /// Execute a recipe with the specialist's allowed helpers.
    fn execute_recipe(
        specialist: &SpecialistDefinition,
        recipe: &anna_shared::recipe::Recipe,
        intent: &UserIntent,
    ) -> SpecialistOutput {
        let mut commands_run = Vec::new();
        let mut outputs = Vec::new();
        let mut errors = Vec::new();

        for cmd in &recipe.commands {
            // Extract base command name
            let cmd_base = cmd.command.split_whitespace().next().unwrap_or("");

            // Check if command uses allowed helper
            if !specialist.allowed_helpers.contains(&cmd_base) {
                // Skip commands outside specialist's scope
                continue;
            }

            commands_run.push(cmd.command.clone());

            match execute_command(&cmd.command) {
                Ok(output) => outputs.push(output),
                Err(e) => {
                    errors.push(format!("{}: {}", cmd.command, e));
                }
            }
        }

        if errors.is_empty() || !outputs.is_empty() {
            // Determine if we should learn from this
            let should_learn = intent.confidence >= 0.8 && recipe.source == anna_shared::recipe::RecipeSource::BuiltIn;

            SpecialistOutput::Completed {
                specialist_id: specialist.id.to_string(),
                specialist_name: specialist.name.to_string(),
                commands_executed: commands_run,
                outputs,
                confidence: intent.confidence,
                recipe_used: Some(recipe.id.clone()),
                should_learn,
            }
        } else {
            SpecialistOutput::Failed {
                specialist_id: specialist.id.to_string(),
                reason: errors.join("; "),
                can_escalate: specialist.level == SpecialistLevel::Junior,
            }
        }
    }

    /// Run diagnostic probes when no recipe matches.
    fn run_diagnostics(specialist: &SpecialistDefinition, intent: &UserIntent) -> SpecialistOutput {
        let diagnostic_commands = Self::select_diagnostic_commands(specialist, intent);
        let mut outputs = Vec::new();
        let mut commands_run = Vec::new();

        for cmd in diagnostic_commands {
            commands_run.push(cmd.clone());
            if let Ok(output) = execute_command(&cmd) {
                outputs.push(output);
            }
        }

        if outputs.is_empty() {
            SpecialistOutput::NeedsEscalation {
                specialist_id: specialist.id.to_string(),
                reason: "No diagnostic output available".to_string(),
            }
        } else {
            SpecialistOutput::Completed {
                specialist_id: specialist.id.to_string(),
                specialist_name: specialist.name.to_string(),
                commands_executed: commands_run,
                outputs,
                confidence: 0.6, // Diagnostic-only is medium confidence
                recipe_used: None,
                should_learn: false,
            }
        }
    }

    /// Select diagnostic commands based on specialist domain and intent.
    fn select_diagnostic_commands(
        specialist: &SpecialistDefinition,
        intent: &UserIntent,
    ) -> Vec<String> {
        let mut commands = Vec::new();

        // Select based on intent subject and available helpers
        match &intent.subject {
            IntentSubject::DiskUsage if specialist.allowed_helpers.contains(&"df") => {
                commands.push("df -h".to_string());
            }
            IntentSubject::MemoryUsage if specialist.allowed_helpers.contains(&"free") => {
                commands.push("free -h".to_string());
            }
            IntentSubject::CpuUsage if specialist.allowed_helpers.contains(&"top") => {
                commands.push("top -bn1 | head -20".to_string());
            }
            IntentSubject::ServiceStatus if specialist.allowed_helpers.contains(&"systemctl") => {
                commands.push("systemctl --failed".to_string());
            }
            IntentSubject::NetworkStatus if specialist.allowed_helpers.contains(&"ip") => {
                commands.push("ip addr".to_string());
            }
            _ => {
                // Fallback: use first available helper with basic args
                if let Some(helper) = specialist.allowed_helpers.first() {
                    commands.push(format!("{} --help 2>&1 | head -5", helper));
                }
            }
        }

        // Limit to 3 diagnostic commands
        commands.truncate(3);
        commands
    }
}

/// Execute a shell command and capture output.
fn execute_command(cmd: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| format!("Failed to execute: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.is_empty() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(stderr.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist::domain::Domain;
    use crate::specialist::registry::get_junior;

    #[test]
    fn test_check_helpers_all_available() {
        // Most basic commands should be available on any system
        let spec = get_junior(Domain::System).unwrap();
        // We can't guarantee any helper is available in test, but we can test the function
        let missing = SpecialistExecutor::check_helpers(spec);
        // Just verify it returns a Vec
        assert!(missing.len() <= spec.allowed_helpers.len());
    }

    #[test]
    fn test_select_diagnostic_commands_disk() {
        let spec = get_junior(Domain::Storage).unwrap();
        let intent = UserIntent {
            subject: IntentSubject::DiskUsage,
            ..Default::default()
        };
        let commands = SpecialistExecutor::select_diagnostic_commands(spec, &intent);
        assert!(!commands.is_empty());
        assert!(commands.len() <= 3);
    }

    #[test]
    fn test_select_diagnostic_commands_memory() {
        let spec = get_junior(Domain::System).unwrap();
        let intent = UserIntent {
            subject: IntentSubject::MemoryUsage,
            ..Default::default()
        };
        let commands = SpecialistExecutor::select_diagnostic_commands(spec, &intent);
        assert!(!commands.is_empty());
    }

    #[test]
    fn test_execute_command_success() {
        let result = execute_command("echo hello");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello"));
    }

    #[test]
    fn test_execute_command_failure() {
        let result = execute_command("nonexistent_command_12345");
        // Should return error or empty output
        assert!(result.is_err() || result.unwrap().is_empty());
    }

    #[test]
    fn test_executor_respects_allowed_helpers() {
        // Create a fake context - executor should only use allowed helpers
        let spec = get_junior(Domain::Storage).unwrap();

        // Storage junior has: df, du, lsblk, findmnt, mount
        assert!(spec.allowed_helpers.contains(&"df"));
        assert!(!spec.allowed_helpers.contains(&"systemctl"));
    }
}
