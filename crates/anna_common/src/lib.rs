//! Anna Common - Shared types and utilities
//!
//! This crate contains data models and utilities shared between
//! the daemon (annad) and CLI client (annactl).

pub mod advice_cache;
pub mod beautiful;
pub mod categories;
pub mod command_meta; // Phase 3.1: Command classification
pub mod config;
pub mod config_parser;
pub mod context; // Phase 3.6: Persistent context layer
pub mod ignore_filters;
pub mod ipc;
pub mod paths; // rc.13.2: Dual-mode socket support
pub mod rollback;
pub mod types;
pub mod updater; // Beta.89: Rollback command generation

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
