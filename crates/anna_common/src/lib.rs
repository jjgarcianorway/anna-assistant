//! Anna Common - Shared types and utilities
//!
//! This crate contains data models and utilities shared between
//! the daemon (annad) and CLI client (annactl).

pub mod advice_cache;
pub mod beautiful;
pub mod caretaker_brain; // Core analysis engine - ties everything together
pub mod categories;
pub mod change_log; // Phase 5.1: Change logging and rollback
pub mod change_log_db; // Phase 5.1: SQLite persistence for change logs
pub mod file_backup; // File backup system with SHA256 verification
pub mod command_meta;
pub mod config;
pub mod config_parser;
pub mod context;
pub mod disk_analysis;
pub mod display;
pub mod github_releases;
pub mod ignore_filters;
pub mod insights; // Phase 5.2: Behavioral insights engine
pub mod installation_source;
pub mod ipc;
pub mod language; // Language system with natural configuration
pub mod learning;
pub mod llm; // Task 12: LLM abstraction layer
pub mod paths;
pub mod personality; // Phase 5.1: Conversational personality controls
pub mod prediction;
pub mod profile;
pub mod prediction_actions;
pub mod rollback;
pub mod self_healing;
pub mod suggestions; // Phase 5.1: Suggestion engine with Arch Wiki integration
pub mod suggestion_engine; // Task 8: Deep Caretaker v0.1 - Rule-based suggestion generation
pub mod telemetry; // Telemetry structures from annad
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
pub use profile::*;
pub use rollback::*;
pub use types::*;
pub use updater::*;
