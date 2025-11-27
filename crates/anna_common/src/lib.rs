//! Anna Common - Shared types and schemas for Anna v0.8.0
//!
//! Zero hardcoded knowledge. Only evidence-based facts.
//! v0.3.0: Strict hallucination guardrails, stable repeated answers, LLM-orchestrated help/version.
//! v0.4.0: Dev auto-update every 10 minutes when enabled.
//! v0.5.0: Natural language configuration, hardware-aware model selection.
//! v0.6.0: ASCII-only sysadmin style, multi-round reliability refinement.
//! v0.7.0: Self-health monitoring and auto-repair engine.
//! v0.8.0: Observability and debug logging with JSONL output.

pub mod config;
pub mod config_mapper;
pub mod hardware;
pub mod logging;
pub mod presentation;
pub mod prompts;
pub mod reliability;
pub mod schemas;
pub mod self_health;
pub mod types;
pub mod updater;

pub use config::*;
pub use config_mapper::*;
pub use hardware::*;
pub use logging::*;
pub use presentation::*;
pub use reliability::*;
pub use schemas::*;
pub use self_health::*;
pub use types::*;
pub use updater::*;
