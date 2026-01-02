//! Built-in systemd recipes (v0.0.233).
//!
//! This module contains all the built-in systemd recipes organized by category.

mod debug;
mod hardening;
mod logs;
mod service;
mod socket;
mod timer;
mod user_service;

use crate::systemd_recipes::types::SystemdRecipe;

// Re-export individual recipe functions
pub use debug::debug_service_recipe;
pub use hardening::harden_service_recipe;
pub use logs::view_logs_recipe;
pub use service::{create_service_recipe, enable_service_recipe};
pub use socket::socket_activation_recipe;
pub use timer::create_timer_recipe;
pub use user_service::create_user_service_recipe;

/// Get built-in systemd recipes
pub fn builtin_recipes() -> Vec<SystemdRecipe> {
    vec![
        create_service_recipe(),
        create_timer_recipe(),
        create_user_service_recipe(),
        enable_service_recipe(),
        view_logs_recipe(),
        debug_service_recipe(),
        socket_activation_recipe(),
        harden_service_recipe(),
    ]
}
