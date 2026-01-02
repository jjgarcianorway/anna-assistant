//! Precondition checking for recipe execution.

use crate::recipe_schema::{Precondition, Recipe};
use super::types::{ExecutionContext, PreconditionResult};

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
            let expanded = super::utils::expand_path(path, ctx);
            let exists = std::path::Path::new(&expanded).exists();
            (exists, format!("File '{}' exists", path))
        }
        Precondition::DirExists { path } => {
            let expanded = super::utils::expand_path(path, ctx);
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
            let met = ctx
                .probes
                .get(probe)
                .map(|r| {
                    regex::Regex::new(pattern)
                        .map(|re| re.is_match(r))
                        .unwrap_or(false)
                })
                .unwrap_or(false);
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
