//! Recipe Modules - Deterministic ActionPlan Generators
//!
//! Beta.151: Hard-coded, testable recipes for common user scenarios
//! Beta.152: Expanded with systemd, network, system_update, and AUR recipes
//! Beta.153: Added SSH, firewall (UFW), and user management recipes
//! Beta.154: Added development environment recipes (Rust, Python, Node.js)
//! Beta.155: Added GPU driver management recipes (NVIDIA, AMD, Intel)
//! Beta.156: Added infrastructure recipes (Docker Compose, PostgreSQL, Nginx)
//! Beta.157: Added system management recipes (monitoring, backup, performance)
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

// Beta.151 recipes
pub mod docker;
pub mod neovim;
pub mod packages;
pub mod wallpaper;

// Beta.152 recipes
pub mod aur;
pub mod network;
pub mod systemd;
pub mod system_update;

// Beta.153 recipes
pub mod firewall;
pub mod ssh;
pub mod users;

// Beta.154 recipes
pub mod nodejs;
pub mod python;
pub mod rust;

// Beta.155 recipes
pub mod amd;
pub mod intel;
pub mod nvidia;

// Beta.156 recipes
pub mod docker_compose;
pub mod postgresql;
pub mod webserver;

// Beta.157 recipes
pub mod backup;
pub mod monitoring;
pub mod performance;

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
    // Beta.152: Enhanced telemetry with user_request for sub-recipe routing
    let mut telemetry_with_request = telemetry.clone();
    telemetry_with_request.insert("user_request".to_string(), user_input.to_string());

    // Try each recipe in order of specificity
    // Beta.152: More specific recipes first to avoid false matches

    // AUR recipes (very specific)
    if aur::AurRecipe::matches_request(user_input) {
        return Some(aur::AurRecipe::build_plan(&telemetry_with_request));
    }

    // System update recipes (specific)
    if system_update::SystemUpdateRecipe::matches_request(user_input) {
        return Some(system_update::SystemUpdateRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.154 recipes - Development environments (specific)
    if rust::RustRecipe::matches_request(user_input) {
        return Some(rust::RustRecipe::build_plan(&telemetry_with_request));
    }

    if python::PythonRecipe::matches_request(user_input) {
        return Some(python::PythonRecipe::build_plan(&telemetry_with_request));
    }

    if nodejs::NodeJsRecipe::matches_request(user_input) {
        return Some(nodejs::NodeJsRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.157 recipes - System management (specific)
    if monitoring::MonitoringRecipe::matches_request(user_input) {
        return Some(monitoring::MonitoringRecipe::build_plan(&telemetry_with_request));
    }

    if backup::BackupRecipe::matches_request(user_input) {
        return Some(backup::BackupRecipe::build_plan(&telemetry_with_request));
    }

    if performance::PerformanceRecipe::matches_request(user_input) {
        return Some(performance::PerformanceRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.156 recipes - Infrastructure (specific)
    if docker_compose::DockerComposeRecipe::matches_request(user_input) {
        return Some(docker_compose::DockerComposeRecipe::build_plan(&telemetry_with_request));
    }

    if postgresql::PostgresqlRecipe::matches_request(user_input) {
        return Some(postgresql::PostgresqlRecipe::build_plan(&telemetry_with_request));
    }

    if webserver::WebServerRecipe::matches_request(user_input) {
        return Some(webserver::WebServerRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.155 recipes - GPU drivers (specific)
    if nvidia::NvidiaRecipe::matches_request(user_input) {
        return Some(nvidia::NvidiaRecipe::build_plan(&telemetry_with_request));
    }

    if amd::AmdRecipe::matches_request(user_input) {
        return Some(amd::AmdRecipe::build_plan(&telemetry_with_request));
    }

    if intel::IntelRecipe::matches_request(user_input) {
        return Some(intel::IntelRecipe::build_plan(&telemetry_with_request));
    }

    // Systemd service management (specific)
    if systemd::SystemdRecipe::matches_request(user_input) {
        return Some(systemd::SystemdRecipe::build_plan(&telemetry_with_request));
    }

    // Network diagnostics (specific)
    if network::NetworkRecipe::matches_request(user_input) {
        return Some(network::NetworkRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.153 recipes
    // SSH management (specific)
    if ssh::SshRecipe::matches_request(user_input) {
        return Some(ssh::SshRecipe::build_plan(&telemetry_with_request));
    }

    // Firewall management (specific)
    if firewall::FirewallRecipe::matches_request(user_input) {
        return Some(firewall::FirewallRecipe::build_plan(&telemetry_with_request));
    }

    // User and group management (specific)
    if users::UsersRecipe::matches_request(user_input) {
        return Some(users::UsersRecipe::build_plan(&telemetry_with_request));
    }

    // Beta.151 recipes
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

        // Beta.151 recipes
        assert!(try_recipe_match("install docker", &telemetry).is_some());
        assert!(try_recipe_match("change my wallpaper", &telemetry).is_some());
        assert!(try_recipe_match("install neovim", &telemetry).is_some());
        assert!(try_recipe_match("fix broken packages", &telemetry).is_some());

        // Beta.152 recipes
        assert!(try_recipe_match("enable NetworkManager service", &telemetry).is_some());
        assert!(try_recipe_match("restart bluetooth", &telemetry).is_some());
        assert!(try_recipe_match("check internet connection", &telemetry).is_some());
        assert!(try_recipe_match("show available wifi networks", &telemetry).is_some());
        assert!(try_recipe_match("check for system updates", &telemetry).is_some());
        assert!(try_recipe_match("update system", &telemetry).is_some());
        assert!(try_recipe_match("install package from AUR", &telemetry).is_some());
        assert!(try_recipe_match("do I have yay installed", &telemetry).is_some());

        // Beta.153 recipes
        assert!(try_recipe_match("install SSH server", &telemetry).is_some());
        assert!(try_recipe_match("generate SSH keys", &telemetry).is_some());
        assert!(try_recipe_match("install firewall", &telemetry).is_some());
        assert!(try_recipe_match("enable ufw", &telemetry).is_some());
        assert!(try_recipe_match("allow SSH through firewall", &telemetry).is_some());
        assert!(try_recipe_match("add user john", &telemetry).is_some());
        assert!(try_recipe_match("remove user testaccount", &telemetry).is_some());
        assert!(try_recipe_match("add user to docker group", &telemetry).is_some());
        assert!(try_recipe_match("list users", &telemetry).is_some());

        // Beta.154 recipes
        assert!(try_recipe_match("install Rust", &telemetry).is_some());
        assert!(try_recipe_match("install cargo and rustup", &telemetry).is_some());
        assert!(try_recipe_match("check Rust status", &telemetry).is_some());
        assert!(try_recipe_match("install Python", &telemetry).is_some());
        assert!(try_recipe_match("setup Python development environment", &telemetry).is_some());
        assert!(try_recipe_match("create Python venv", &telemetry).is_some());
        assert!(try_recipe_match("install Node.js", &telemetry).is_some());
        assert!(try_recipe_match("setup npm", &telemetry).is_some());
        assert!(try_recipe_match("initialize new npm project", &telemetry).is_some());

        // Beta.155 recipes
        assert!(try_recipe_match("install NVIDIA drivers", &telemetry).is_some());
        assert!(try_recipe_match("setup nvidia GPU", &telemetry).is_some());
        assert!(try_recipe_match("install CUDA", &telemetry).is_some());
        assert!(try_recipe_match("install AMD drivers", &telemetry).is_some());
        assert!(try_recipe_match("setup AMD GPU", &telemetry).is_some());
        assert!(try_recipe_match("install ROCm", &telemetry).is_some());
        assert!(try_recipe_match("install Intel drivers", &telemetry).is_some());
        assert!(try_recipe_match("setup Intel GPU", &telemetry).is_some());
        assert!(try_recipe_match("check nvidia status", &telemetry).is_some());

        // Beta.156 recipes
        assert!(try_recipe_match("install docker-compose", &telemetry).is_some());
        assert!(try_recipe_match("init docker compose project", &telemetry).is_some());
        assert!(try_recipe_match("validate docker-compose.yml", &telemetry).is_some());
        assert!(try_recipe_match("install postgresql", &telemetry).is_some());
        assert!(try_recipe_match("create postgres database", &telemetry).is_some());
        assert!(try_recipe_match("configure postgresql security", &telemetry).is_some());
        assert!(try_recipe_match("install nginx", &telemetry).is_some());
        assert!(try_recipe_match("create nginx site", &telemetry).is_some());
        assert!(try_recipe_match("enable nginx SSL", &telemetry).is_some());

        // Beta.157 recipes
        assert!(try_recipe_match("install monitoring tools", &telemetry).is_some());
        assert!(try_recipe_match("install htop", &telemetry).is_some());
        assert!(try_recipe_match("setup btop", &telemetry).is_some());
        assert!(try_recipe_match("install backup tools", &telemetry).is_some());
        assert!(try_recipe_match("setup rsync", &telemetry).is_some());
        assert!(try_recipe_match("install borg", &telemetry).is_some());
        assert!(try_recipe_match("install cpupower", &telemetry).is_some());
        assert!(try_recipe_match("set cpu governor", &telemetry).is_some());
        assert!(try_recipe_match("tune swappiness", &telemetry).is_some());

        // Generic query should not match
        assert!(try_recipe_match("what is the weather", &telemetry).is_none());
        assert!(try_recipe_match("tell me a joke", &telemetry).is_none());
    }

    #[test]
    fn test_recipe_priority() {
        let telemetry = HashMap::new();

        // AUR-specific queries should match AUR recipe, not generic package recipe
        let aur_match = try_recipe_match("install yay from AUR", &telemetry);
        assert!(aur_match.is_some());
        let plan = aur_match.unwrap().unwrap();
        assert!(plan.meta.detection_results.other.get("recipe_module")
            .and_then(|v| v.as_str())
            .unwrap_or("").contains("aur.rs"));

        // System update queries should match system_update recipe
        let update_match = try_recipe_match("update all packages", &telemetry);
        assert!(update_match.is_some());
        let plan = update_match.unwrap().unwrap();
        assert!(plan.meta.detection_results.other.get("recipe_module")
            .and_then(|v| v.as_str())
            .unwrap_or("").contains("system_update.rs"));
    }
}
