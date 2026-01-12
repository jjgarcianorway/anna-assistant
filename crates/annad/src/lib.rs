//! Anna daemon - simplified version.
//!
//! Core functionality:
//! - Auto-update from GitHub
//! - LLM-powered command execution loop
//! - Unix socket server for client communication
//! - Streaming answer validation (v0.0.889)
//! - Automatic problem fixing (v0.0.993)
//! - Safe change engine with undo (v0.0.998)
//! - Configuration recipes (v0.0.998)
//! - Hollywood IT teams experience (v0.0.998)
//! - Full IT Department with specialists (v0.0.999)
//! - Ticket system and RPG progression (v0.0.999)

pub mod autofix;
pub mod changes;
pub mod core_loop;
mod core_loop_old;
pub mod department;
pub mod intent;
pub mod ollama;
pub mod patterns;
pub mod recipes;
pub mod server;
pub mod state;
pub mod team_speak;
pub mod update;
pub mod update_loop;
pub mod update_ops;
pub mod validation;
