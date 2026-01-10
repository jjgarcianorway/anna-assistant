//! Anna daemon - simplified version.
//!
//! Core functionality:
//! - Auto-update from GitHub
//! - LLM-powered command execution loop
//! - Unix socket server for client communication
//! - Streaming answer validation (v0.0.889)

pub mod core_loop;
mod core_loop_old;
pub mod intent;
pub mod ollama;
pub mod patterns;
pub mod server;
pub mod state;
pub mod update;
pub mod update_loop;
pub mod update_ops;
pub mod validation;
