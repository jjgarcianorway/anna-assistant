//! Anna Common - Shared types and utilities
//!
//! This crate contains data models and utilities shared between
//! the daemon (annad) and CLI client (annactl).

pub mod advice_cache;
pub mod audio; // Audio system detection (PipeWire, Pulse, ALSA)
pub mod beautiful;
pub mod boot; // Boot system detection (UEFI/BIOS, Secure Boot, bootloader)
pub mod caretaker_brain; // Core analysis engine - ties everything together
pub mod categories;
pub mod change_log; // Phase 5.1: Change logging and rollback
pub mod change_log_db; // Phase 5.1: SQLite persistence for change logs
pub mod change_recipe; // Phase 7: Safe change recipes with strict guardrails
pub mod change_recipe_display; // Phase 7: UI display for change recipes
pub mod file_backup; // File backup system with SHA256 verification
pub mod command_meta;
pub mod config;
pub mod config_file; // Desktop config file parsing (Hyprland, i3, Sway)
pub mod config_parser;
pub mod context;
pub mod desktop; // Desktop environment detection (Hyprland, i3, KDE, etc.)
pub mod desktop_automation; // Desktop automation helpers (wallpaper, config updates, reload)
pub mod disk_analysis;
pub mod display;
pub mod github_releases;
pub mod hardware_capability; // Hardware capability detection for local LLM
pub mod ignore_filters;
pub mod insights; // Phase 5.2: Behavioral insights engine
pub mod installation_source;
pub mod ipc;
pub mod language; // Language system with natural configuration
pub mod learning;
pub mod llm; // Task 12: LLM abstraction layer
pub mod llm_upgrade; // Step 3: Hardware upgrade detection for brain improvements
pub mod model_profiles; // Data-driven model selection with upgrade paths
pub mod ollama_installer; // Automatic local LLM bootstrap
pub mod paths;
pub mod personality; // Phase 5.1: Conversational personality controls
pub mod prediction;
pub mod profile;
pub mod prediction_actions;
pub mod prompt_builder; // Phase 9: LLM prompt construction with safety
pub mod recipe_validator; // Phase 9: LLM response parsing and validation
pub mod rollback;
pub mod self_healing;
pub mod suggestions; // Phase 5.1: Suggestion engine with Arch Wiki integration
pub mod suggestion_engine; // Task 8: Deep Caretaker v0.1 - Rule-based suggestion generation
pub mod telemetry; // Telemetry structures from annad
pub mod terminal_format; // Phase 8: Beautiful terminal formatting
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
