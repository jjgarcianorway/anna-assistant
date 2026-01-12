//! Anna daemon - LLM-driven Linux assistant.
//!
//! Core functionality:
//! - Pure LLM intelligence (no pattern matching) - v0.2.0
//! - Multi-stage investigation
//! - Grounded answers based on actual command output
//! - Smart fix suggestions based on findings
//! - Auto-update from GitHub
//! - Unix socket server for client communication

pub mod autofix;  // TODO: Remove after full migration to llm_core
pub mod binary_watcher;
pub mod changes;
pub mod core_loop;
mod core_loop_old;  // Legacy - being replaced by llm_core
pub mod department;
pub mod intent;
pub mod llm_core;  // NEW: Pure LLM-driven core loop
pub mod ollama;
pub mod patterns;  // TODO: Remove after full migration to llm_core
pub mod ralph;
pub mod recipes;
pub mod server;
pub mod state;
pub mod team_speak;
pub mod translator;
pub mod update;
pub mod update_loop;
pub mod update_ops;
pub mod validation;
