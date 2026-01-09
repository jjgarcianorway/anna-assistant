//! Anna daemon - simplified version.
//!
//! Core functionality:
//! - Auto-update from GitHub
//! - LLM-powered command execution loop
//! - Unix socket server for client communication

pub mod core_loop;
pub mod intent;
pub mod ollama;
pub mod server;
pub mod state;
pub mod update;
pub mod update_loop;
pub mod update_ops;
