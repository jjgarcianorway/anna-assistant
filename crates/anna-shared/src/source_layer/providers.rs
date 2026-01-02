//! Source Providers (Part 1a) - v0.0.443.
//!
//! First-class source providers for evidence-based answers:
//! - ManProvider: man pages (`man -P cat <cmd>`)
//! - HelpProvider: command --help output
//! - ArchWikiProvider: Arch Wiki (offline-first)
//! - LocalConfigProvider: /etc, ~/.config files
//! - SystemProbeProvider: existing probes
//!
//! LLMs are for orchestration, not truth generation.
//!
//! This module re-exports all provider types and implementations from sibling modules.

// Re-export all public types and functions from sibling modules
pub use super::providers_archwiki::ArchWikiProvider;
pub use super::providers_config::{commands_for_intent, LocalConfigProvider};
pub use super::providers_help::HelpProvider;
pub use super::providers_man::ManProvider;
pub use super::providers_types::{IntentCommands, SourceContent, SourceRequest, SourceType};
