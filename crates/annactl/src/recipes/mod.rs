//! Recipe Modules - Deterministic ActionPlan Generators
//!
//! Beta.151: Hard-coded, testable recipes for common user scenarios
//!
//! These modules generate predictable ActionPlans without relying on LLM
//! generation, reducing hallucination risk and ensuring consistent, safe
//! behavior for common tasks.
//!
//! Each recipe module:
//! - Detects if it matches a user request
//! - Uses telemetry to generate context-aware commands
//! - Provides proper checks, rollback, and risk classification
//! - Includes comprehensive tests

pub mod docker;
pub mod neovim;
pub mod packages;
pub mod wallpaper;

use anna_common::action_plan_v3::ActionPlan;
use anyhow::Result;
use std::collections::HashMap;

/// Try to match user request against known recipe patterns
///
/// Returns Some(ActionPlan) if a recipe matches, None if no match found
pub fn try_recipe_match(
    user_input: &str,
    telemetry: &HashMap<String, String>,
) -> Option<Result<ActionPlan>> {
    // Try each recipe in order of specificity
    if docker::DockerRecipe::matches_request(user_input) {
        return Some(docker::DockerRecipe::build_plan(telemetry));
    }

    if wallpaper::WallpaperRecipe::matches_request(user_input) {
        return Some(wallpaper::WallpaperRecipe::build_plan(telemetry));
    }

    if neovim::NeovimRecipe::matches_request(user_input) {
        return Some(neovim::NeovimRecipe::build_plan(telemetry));
    }

    if packages::PackagesRecipe::matches_request(user_input) {
        return Some(packages::PackagesRecipe::build_plan(telemetry));
    }

    // No recipe matched
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_matching() {
        let telemetry = HashMap::new();

        // Docker should match
        assert!(try_recipe_match("install docker", &telemetry).is_some());

        // Wallpaper should match
        assert!(try_recipe_match("change my wallpaper", &telemetry).is_some());

        // Neovim should match
        assert!(try_recipe_match("install neovim", &telemetry).is_some());

        // Packages should match
        assert!(try_recipe_match("fix broken packages", &telemetry).is_some());

        // Generic query should not match
        assert!(try_recipe_match("what is the weather", &telemetry).is_none());
    }
}
