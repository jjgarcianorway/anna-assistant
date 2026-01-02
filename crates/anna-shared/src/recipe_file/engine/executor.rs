//! Recipe execution logic (v0.0.406).

use super::types::{ExecutionResult, RecipeContext, StepResult};
use crate::recipe_file::format::{ConfirmLevel, FileRecipe, RecipeStep};
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Execute a recipe with the given context
pub fn execute_recipe(
    recipe: &FileRecipe,
    context: &RecipeContext,
    probe_lookup: impl Fn(&str) -> Option<String>,
) -> ExecutionResult {
    let start = std::time::Instant::now();
    let mut result = ExecutionResult::empty(recipe.full_id());
    result.dry_run = !context.execute;
    result.confirmation_required = recipe.requires_confirmation();

    // Create backups if needed
    if context.execute && !recipe.plan.backup_paths.is_empty() {
        for path in &recipe.plan.backup_paths {
            let expanded = expand_path(path, &context.home_dir);
            if std::path::Path::new(&expanded).exists() {
                let backup = format!("{}.anna-backup", expanded);
                if let Err(e) = std::fs::copy(&expanded, &backup) {
                    warn!("Failed to backup {}: {}", expanded, e);
                } else {
                    debug!("Created backup: {}", backup);
                }
            }
        }
    }

    // Execute steps
    for step in &recipe.plan.steps {
        // Check condition
        if let Some(ref cond) = step.condition {
            if !evaluate_condition(cond, &result.variables) {
                result.steps.push(StepResult {
                    id: step.id.clone(),
                    command: String::new(),
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                    duration_ms: 0,
                    skipped: true,
                    extracted: HashMap::new(),
                });
                continue;
            }
        }

        // Get command
        let command = match step.get_command(&probe_lookup) {
            Some(cmd) => expand_path(&cmd, &context.home_dir),
            None => {
                result.error = Some(format!("Step {} has no command", step.id));
                result.success = false;
                break;
            }
        };

        // Check if already have probe output
        if let Some(ref probe_id) = step.probe {
            if let Some(output) = context.probe_outputs.get(probe_id) {
                let step_result = StepResult {
                    id: step.id.clone(),
                    command: command.clone(),
                    stdout: output.clone(),
                    stderr: String::new(),
                    exit_code: 0,
                    duration_ms: 0,
                    skipped: false,
                    extracted: extract_variables(&step.extract, output),
                };
                result.variables.extend(step_result.extracted.clone());
                result.steps.push(step_result);
                continue;
            }
        }

        // Handle confirmation
        if step.needs_confirm != ConfirmLevel::None && context.execute {
            let desc = step.description.as_deref().unwrap_or(&command);
            let confirmed = context.confirm_callback.map(|cb| cb(desc)).unwrap_or(false);

            if !confirmed {
                result.steps.push(StepResult {
                    id: step.id.clone(),
                    command,
                    stdout: String::new(),
                    stderr: "User declined confirmation".to_string(),
                    exit_code: -1,
                    duration_ms: 0,
                    skipped: true,
                    extracted: HashMap::new(),
                });
                result.error = Some("User declined confirmation".to_string());
                result.success = false;
                break;
            }
        }

        // Execute command
        let step_result = if context.execute {
            execute_step(step, &command)
        } else {
            // Dry run
            StepResult {
                id: step.id.clone(),
                command,
                stdout: "[dry run]".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 0,
                skipped: false,
                extracted: HashMap::new(),
            }
        };

        // Check for errors
        if step_result.exit_code != 0 && recipe.plan.stop_on_error {
            result.error = Some(format!(
                "Step {} failed with exit code {}",
                step.id, step_result.exit_code
            ));
            result.success = false;
            result.steps.push(step_result);
            break;
        }

        // Accumulate variables
        result.variables.extend(step_result.extracted.clone());
        result.steps.push(step_result);
    }

    result.total_duration_ms = start.elapsed().as_millis() as u64;
    result
}

/// Execute a single step
fn execute_step(step: &RecipeStep, command: &str) -> StepResult {
    let start = std::time::Instant::now();

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output();

    let (stdout, stderr, exit_code) = match output {
        Ok(out) => (
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
            out.status.code().unwrap_or(-1),
        ),
        Err(e) => (String::new(), e.to_string(), -1),
    };

    let extracted = extract_variables(&step.extract, &stdout);

    StepResult {
        id: step.id.clone(),
        command: command.to_string(),
        stdout,
        stderr,
        exit_code,
        duration_ms: start.elapsed().as_millis() as u64,
        skipped: false,
        extracted,
    }
}

/// Extract variables from output using regex patterns
fn extract_variables(patterns: &HashMap<String, String>, output: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    for (var_name, pattern) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(output) {
                if let Some(m) = caps.get(1) {
                    vars.insert(var_name.clone(), m.as_str().to_string());
                } else if let Some(m) = caps.get(0) {
                    vars.insert(var_name.clone(), m.as_str().to_string());
                }
            }
        }
    }

    vars
}

/// Expand ~ in paths
pub(crate) fn expand_path(path: &str, home: &str) -> String {
    if path.starts_with("~/") {
        format!("{}/{}", home, &path[2..])
    } else if path == "~" {
        home.to_string()
    } else {
        path.to_string()
    }
}

/// Evaluate a simple condition
fn evaluate_condition(condition: &str, variables: &HashMap<String, String>) -> bool {
    // Simple conditions like "prev_exit_code == 0" or "var_name"
    if condition.contains("==") {
        let parts: Vec<&str> = condition.split("==").collect();
        if parts.len() == 2 {
            let var = parts[0].trim();
            let val = parts[1].trim();
            return variables.get(var).map(|v| v == val).unwrap_or(false);
        }
    }

    // Just check if variable exists and is non-empty
    variables
        .get(condition.trim())
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path() {
        assert_eq!(expand_path("~/test", "/home/user"), "/home/user/test");
        assert_eq!(expand_path("~", "/home/user"), "/home/user");
        assert_eq!(expand_path("/absolute", "/home/user"), "/absolute");
    }

    #[test]
    fn test_extract_variables() {
        let mut patterns = HashMap::new();
        patterns.insert("count".to_string(), r"(\d+) failed".to_string());

        let output = "There are 5 failed services";
        let vars = extract_variables(&patterns, output);
        assert_eq!(vars.get("count"), Some(&"5".to_string()));
    }

    #[test]
    fn test_evaluate_condition() {
        let mut vars = HashMap::new();
        vars.insert("exit_code".to_string(), "0".to_string());
        vars.insert("found".to_string(), "yes".to_string());

        assert!(evaluate_condition("exit_code == 0", &vars));
        assert!(!evaluate_condition("exit_code == 1", &vars));
        assert!(evaluate_condition("found", &vars));
        assert!(!evaluate_condition("missing", &vars));
    }
}
