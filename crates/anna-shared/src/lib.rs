//! Shared types and utilities for Anna components.
//!
//! This crate contains shared modules used across the Anna assistant project.
//! For detailed version history, see the main CHANGELOG.md in the repository root.
//!
//! Module organization:
//! - modules_core: Core functionality (tickets, specialists, protocols, etc.)
//! - modules_config: Configuration modules
//! - modules_settings: Settings system (v0.0.554+)
//! - modules_display: Display, stats, UI modules
//! - modules_recipes: Recipe learning and execution
//! - modules_knowledge: Knowledge engine and documentation
//! - modules_misc: Tracking, monitoring, and utilities
//! - constants: Constants and configuration paths
//! - exports: Public re-exports for common types

// Module organization - Split into separate files to keep under 400 lines
mod modules_core;
mod modules_config;
mod modules_settings;
mod modules_display;
mod modules_recipes;
mod modules_knowledge;
mod modules_misc;

// Constants and exports
pub mod constants;
mod exports;

// Re-export all modules from the organized files
pub use modules_core::*;
pub use modules_config::*;
pub use modules_settings::*;
pub use modules_display::*;
pub use modules_recipes::*;
pub use modules_knowledge::*;
pub use modules_misc::*;

// Re-export constants and common types
pub use constants::*;
pub use exports::*;
