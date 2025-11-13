//! Anna Common - Shared types and utilities
//!
//! This crate contains data models and utilities shared between
//! the daemon (annad) and CLI client (annactl).

pub mod advice_cache;
pub mod beautiful;
pub mod caretaker_brain; // Core analysis engine - ties everything together
pub mod categories;
pub mod command_meta;
pub mod config;
pub mod config_parser;
pub mod context;
pub mod disk_analysis;
pub mod display;
pub mod github_releases;
pub mod ignore_filters;
pub mod installation_source;
pub mod ipc;
pub mod learning;
pub mod paths;
pub mod prediction;
pub mod prediction_actions;
pub mod rollback;
pub mod self_healing;
pub mod types;
pub mod updater;

pub use advice_cache::*;
pub use beautiful::*;
pub use categories::*;
pub use config::*;
pub use config_parser::*;
pub use ignore_filters::*;
pub use ipc::*;
pub use paths::*;
pub use rollback::*;
pub use types::*;
pub use updater::*;
