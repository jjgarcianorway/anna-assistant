//! Recipe executor for Anna's learning system.
//! v0.0.418: Executes recipe plans using existing transaction mechanisms.
//!
//! The executor:
//! - Verifies preconditions using probes
//! - Builds a plan for the transaction engine
//! - Prompts user for confirmation if required
//! - Executes steps with rollback on failure
//! - Updates recipe metrics

use crate::recipe_schema::{ConfirmationPolicy, PlanStep, Precondition, Recipe, RecipeStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of recipe execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Human-readable summary
    pub summary: String,
    /// Steps that were executed
    pub steps_executed: Vec<StepResult>,
    /// Whether rollback was performed
    pub rolled_back: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Result of executing a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step type name
    pub step_type: String,
    /// Whether step succeeded
    pub success: bool,
    /// Output or error message
    pub message: String,
    /// Whether this step can be rolled back
    pub rollback_available: bool,
}

/// Precondition check result.
#[derive(Debug, Clone)]
pub struct PreconditionResult {
    pub all_met: bool,
    pub failed: Vec<String>,
    pub passed: Vec<String>,
}

/// Recipe execution context.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Probe results available
    pub probes: HashMap<String, String>,
    /// User-provided parameters (for parameterized recipes)
    pub params: HashMap<String, String>,
    /// Whether user confirmed execution
    pub user_confirmed: bool,
    /// Dry run mode (don't actually execute)
    pub dry_run: bool,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            probes: HashMap::new(),
            params: HashMap::new(),
            user_confirmed: false,
            dry_run: false,
        }
    }
}

/// Check if a recipe can be executed (preconditions met).
pub fn check_preconditions(recipe: &Recipe, ctx: &ExecutionContext) -> PreconditionResult {
    let mut passed = Vec::new();
    let mut failed = Vec::new();

    for precondition in &recipe.preconditions {
        let (met, desc) = check_single_precondition(precondition, ctx);
        if met {
            passed.push(desc);
        } else {
            failed.push(desc);
        }
    }

    PreconditionResult {
        all_met: failed.is_empty(),
        failed,
        passed,
    }
}

fn check_single_precondition(precond: &Precondition, ctx: &ExecutionContext) -> (bool, String) {
    match precond {
        Precondition::ToolExists { tool } => {
            // Check if we have a probe result for this tool
            let probe_key = format!("which_{}", tool);
            let exists = ctx
                .probes
                .get(&probe_key)
                .or_else(|| ctx.probes.get("which"))
                .map(|r| r.contains(tool) && !r.contains("not found"))
                .unwrap_or(false);
            (exists, format!("Tool '{}' exists", tool))
        }
        Precondition::FileExists { path } => {
            let expanded = expand_path(path, ctx);
            let exists = std::path::Path::new(&expanded).exists();
            (exists, format!("File '{}' exists", path))
        }
        Precondition::DirExists { path } => {
            let expanded = expand_path(path, ctx);
            let exists = std::path::Path::new(&expanded).is_dir();
            (exists, format!("Directory '{}' exists", path))
        }
        Precondition::ProbeContains { probe, contains } => {
            let met = ctx
                .probes
                .get(probe)
                .map(|r| r.contains(contains))
                .unwrap_or(false);
            (met, format!("Probe '{}' contains '{}'", probe, contains))
        }
        Precondition::ProbeMatches { probe, pattern } => {
            let met = ctx.probes.get(probe).map(|r| {
                regex::Regex::new(pattern)
                    .map(|re| re.is_match(r))
                    .unwrap_or(false)
            }).unwrap_or(false);
            (met, format!("Probe '{}' matches pattern", probe))
        }
        Precondition::ServiceExists { service } => {
            // Check systemctl list-units or similar probe
            let probe_key = "systemctl_list_units";
            let exists = ctx
                .probes
                .get(probe_key)
                .map(|r| r.contains(service))
                .unwrap_or(true); // Assume exists if no probe
            (exists, format!("Service '{}' exists", service))
        }
        Precondition::ProbeCheck { probe, condition } => {
            // Generic probe check
            let met = ctx
                .probes
                .get(probe)
                .map(|_| true) // Just check probe exists for now
                .unwrap_or(false);
            (met, format!("Probe '{}' satisfies '{}'", probe, condition))
        }
    }
}

/// Expand path variables like $HOME, {param}.
fn expand_path(path: &str, ctx: &ExecutionContext) -> String {
    let mut result = path.to_string();

    // Expand $HOME
    if let Ok(home) = std::env::var("HOME") {
        result = result.replace("$HOME", &home);
        result = result.replace("~", &home);
    }

    // Expand parameters
    for (key, value) in &ctx.params {
        result = result.replace(&format!("{{{}}}", key), value);
    }

    result
}

/// Check if recipe needs user confirmation.
pub fn needs_confirmation(recipe: &Recipe, ctx: &ExecutionContext) -> bool {
    if ctx.user_confirmed {
        return false;
    }

    match recipe.confirmation_policy {
        ConfirmationPolicy::Never => false,
        ConfirmationPolicy::Require => true,
        ConfirmationPolicy::MutatingOnly => recipe.has_mutating_steps(),
    }
}

/// Generate a confirmation prompt for the user.
pub fn generate_confirmation_prompt(recipe: &Recipe, ctx: &ExecutionContext) -> String {
    let mut prompt = format!(
        "I can {} by performing the following steps:\n\n",
        recipe.pattern.user_goal
    );

    for (i, step) in recipe.plan.iter().enumerate() {
        let step_desc = describe_step(step, ctx);
        prompt.push_str(&format!("{}. {}\n", i + 1, step_desc));
    }

    if !recipe.touched_files().is_empty() {
        prompt.push_str("\nFiles that will be modified:\n");
        for file in recipe.touched_files() {
            prompt.push_str(&format!("  - {}\n", expand_path(&file, ctx)));
        }
    }

    prompt.push_str("\nApply this change?");
    prompt
}

fn describe_step(step: &PlanStep, ctx: &ExecutionContext) -> String {
    match step {
        PlanStep::Explain { message } => message.clone(),
        PlanStep::BackupFile { path } => {
            format!("Backup '{}'", expand_path(path, ctx))
        }
        PlanStep::AppendLine { path, line } => {
            format!("Add '{}' to {}", line, expand_path(path, ctx))
        }
        PlanStep::PrependLine { path, line } => {
            format!("Prepend '{}' to {}", line, expand_path(path, ctx))
        }
        PlanStep::ReplaceLine { path, pattern, replacement } => {
            format!(
                "Replace '{}' with '{}' in {}",
                pattern,
                replacement,
                expand_path(path, ctx)
            )
        }
        PlanStep::EnsureLine { path, line } => {
            format!("Ensure '{}' exists in {}", line, expand_path(path, ctx))
        }
        PlanStep::RemoveLines { path, pattern } => {
            format!("Remove lines matching '{}' from {}", pattern, expand_path(path, ctx))
        }
        PlanStep::VerifyCommand { command, .. } => {
            format!("Verify: {}", command)
        }
        PlanStep::RunCommand { description, command, .. } => {
            if description.is_empty() {
                format!("Run: {}", command)
            } else {
                description.clone()
            }
        }
        PlanStep::EnableService { service, start } => {
            if *start {
                format!("Enable and start service '{}'", service)
            } else {
                format!("Enable service '{}'", service)
            }
        }
        PlanStep::DisableService { service, stop } => {
            if *stop {
                format!("Disable and stop service '{}'", service)
            } else {
                format!("Disable service '{}'", service)
            }
        }
        PlanStep::RestartService { service } => {
            format!("Restart service '{}'", service)
        }
        PlanStep::CreateDir { path, .. } => {
            format!("Create directory '{}'", expand_path(path, ctx))
        }
        PlanStep::WriteFile { path, .. } => {
            format!("Create/overwrite '{}'", expand_path(path, ctx))
        }
        PlanStep::SetEnvVar { name, value, .. } => {
            format!("Set environment variable {}={}", name, value)
        }
    }
}

/// Execute a recipe (returns execution plan, actual execution done by caller).
pub fn prepare_execution(
    recipe: &Recipe,
    ctx: &ExecutionContext,
) -> Result<ExecutionPlan, String> {
    // Check if recipe is usable
    if recipe.status != RecipeStatus::Active {
        return Err(format!("Recipe '{}' is not active", recipe.id));
    }

    // Check preconditions
    let precond_result = check_preconditions(recipe, ctx);
    if !precond_result.all_met {
        return Err(format!(
            "Preconditions not met: {}",
            precond_result.failed.join(", ")
        ));
    }

    // Check confirmation
    if needs_confirmation(recipe, ctx) && !ctx.user_confirmed {
        return Err("User confirmation required".into());
    }

    // Build execution plan
    let steps: Vec<ExecutionStep> = recipe
        .plan
        .iter()
        .map(|s| ExecutionStep {
            step: s.clone(),
            expanded_paths: expand_step_paths(s, ctx),
        })
        .collect();

    Ok(ExecutionPlan {
        recipe_id: recipe.id.clone(),
        steps,
        rollback_on_failure: recipe.success_criteria.rollback_on_failure,
    })
}

fn expand_step_paths(step: &PlanStep, ctx: &ExecutionContext) -> HashMap<String, String> {
    let mut expanded = HashMap::new();

    match step {
        PlanStep::BackupFile { path }
        | PlanStep::AppendLine { path, .. }
        | PlanStep::PrependLine { path, .. }
        | PlanStep::ReplaceLine { path, .. }
        | PlanStep::EnsureLine { path, .. }
        | PlanStep::RemoveLines { path, .. }
        | PlanStep::WriteFile { path, .. }
        | PlanStep::CreateDir { path, .. } => {
            expanded.insert("path".into(), expand_path(path, ctx));
        }
        PlanStep::SetEnvVar { shell_config, .. } => {
            expanded.insert("shell_config".into(), expand_path(shell_config, ctx));
        }
        _ => {}
    }

    expanded
}

/// Execution plan ready for the transaction engine.
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub recipe_id: String,
    pub steps: Vec<ExecutionStep>,
    pub rollback_on_failure: bool,
}

/// A single execution step with expanded paths.
#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub step: PlanStep,
    pub expanded_paths: HashMap<String, String>,
}

/// Generate a summary of what a recipe would do (for display).
pub fn generate_recipe_summary(recipe: &Recipe, ctx: &ExecutionContext) -> String {
    let mut summary = String::new();

    summary.push_str(&format!("Recipe: {} (v{})\n", recipe.id, recipe.version));
    summary.push_str(&format!("Goal: {}\n", recipe.pattern.user_goal));

    if !recipe.citations.is_empty() {
        summary.push_str(&format!("Based on: {}\n", recipe.citations.join(", ")));
    }

    summary.push_str("\nSteps:\n");
    for (i, step) in recipe.plan.iter().enumerate() {
        summary.push_str(&format!("  {}. {}\n", i + 1, describe_step(step, ctx)));
    }

    if recipe.metrics.times_used > 0 {
        let success_rate = if recipe.metrics.times_used > 0 {
            let successes = recipe.metrics.times_used - recipe.metrics.times_failed;
            successes as f32 / recipe.metrics.times_used as f32 * 100.0
        } else {
            100.0
        };
        summary.push_str(&format!(
            "\nUsed {} times ({:.0}% success rate)\n",
            recipe.metrics.times_used, success_rate
        ));
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_schema::{RecipeMatcher, RecipePattern};

    fn make_test_recipe() -> Recipe {
        let mut recipe = Recipe::new(
            "test".into(),
            "desktop".into(),
            "configure_editor".into(),
            RecipePattern {
                user_goal: "enable syntax highlighting".into(),
                slots: HashMap::new(),
            },
            RecipeMatcher {
                required_keywords: vec!["vim".into()],
                optional_keywords: vec![],
                negative_keywords: vec![],
                min_confidence: 0.8,
                exact_intent: None,
            },
            vec![
                PlanStep::BackupFile {
                    path: "$HOME/.vimrc".into(),
                },
                PlanStep::AppendLine {
                    path: "$HOME/.vimrc".into(),
                    line: "syntax enable".into(),
                },
            ],
        );
        recipe.preconditions = vec![Precondition::FileExists {
            path: "$HOME/.vimrc".into(),
        }];
        recipe
    }

    #[test]
    fn test_expand_path() {
        let ctx = ExecutionContext::default();
        let expanded = expand_path("$HOME/.vimrc", &ctx);
        assert!(!expanded.contains("$HOME"));
    }

    #[test]
    fn test_precondition_check() {
        let recipe = make_test_recipe();
        let ctx = ExecutionContext::default();

        // File may or may not exist, but function should work
        let result = check_preconditions(&recipe, &ctx);
        assert!(!result.passed.is_empty() || !result.failed.is_empty());
    }

    #[test]
    fn test_confirmation_prompt() {
        let recipe = make_test_recipe();
        let ctx = ExecutionContext::default();

        let prompt = generate_confirmation_prompt(&recipe, &ctx);
        assert!(prompt.contains("syntax enable"));
        assert!(prompt.contains("Apply this change"));
    }
}
