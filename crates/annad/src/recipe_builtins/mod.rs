//! Built-in recipe matchers for shell, git, SSH, systemd, cron, and Docker configurations.
//!
//! Extracted from recipe_fast_path.rs (v0.0.163) for modularization.
//! Split into separate modules for better maintainability.
//!
//! This module re-exports all the public functions to preserve the original API.

mod docker;
mod git;
mod shell;
mod ssh;
mod system_services;

// Re-export all public functions to preserve the original API
pub use docker::check_docker_recipes;
pub use git::check_git_recipes;
pub use shell::check_shell_recipes;
pub use ssh::check_ssh_recipes;
pub use system_services::{check_cron_recipes, check_systemd_recipes};
