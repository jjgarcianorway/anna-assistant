//! Anna Common - Shared types and schemas for Anna v0.5.0
//!
//! Zero hardcoded knowledge. Only evidence-based facts.
//! v0.3.0: Strict hallucination guardrails, stable repeated answers, LLM-orchestrated help/version.
//! v0.4.0: Dev auto-update every 10 minutes when enabled.
//! v0.5.0: Natural language configuration, hardware-aware model selection.

pub mod config;
pub mod config_mapper;
pub mod hardware;
pub mod prompts;
pub mod schemas;
pub mod types;
pub mod updater;

pub use config::*;
pub use config_mapper::*;
pub use hardware::*;
pub use schemas::*;
pub use types::*;
pub use updater::*;
