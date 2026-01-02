//! Utility functions for recipe execution.

use crate::recipe_schema::PlanStep;
use super::types::ExecutionContext;
use std::collections::HashMap;

/// Expand path variables like $HOME, {param}.
pub fn expand_path(path: &str, ctx: &ExecutionContext) -> String {
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

/// Describe a plan step in human-readable format.
pub fn describe_step(step: &PlanStep, ctx: &ExecutionContext) -> String {
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
        PlanStep::ReplaceLine {
            path,
            pattern,
            replacement,
        } => {
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
            format!(
                "Remove lines matching '{}' from {}",
                pattern,
                expand_path(path, ctx)
            )
        }
        PlanStep::VerifyCommand { command, .. } => {
            format!("Verify: {}", command)
        }
        PlanStep::RunCommand {
            description,
            command,
            ..
        } => {
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

/// Expand paths in a plan step.
pub fn expand_step_paths(step: &PlanStep, ctx: &ExecutionContext) -> HashMap<String, String> {
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
