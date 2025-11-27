//! Anna Common - Shared types and schemas for Anna v0.13.0
//!
//! Zero hardcoded knowledge. Only evidence-based facts.
//! v0.3.0: Strict hallucination guardrails, stable repeated answers, LLM-orchestrated help/version.
//! v0.4.0: Dev auto-update every 10 minutes when enabled.
//! v0.5.0: Natural language configuration, hardware-aware model selection.
//! v0.6.0: ASCII-only sysadmin style, multi-round reliability refinement.
//! v0.7.0: Self-health monitoring and auto-repair engine.
//! v0.8.0: Observability and debug logging with JSONL output.
//! v0.9.0: Locked CLI surface, status command.
//! v0.10.0: Strict evidence discipline - LLM-A/LLM-B audit loop with proof.
//! v0.11.0: Persistent knowledge store, event-driven learning, user telemetry.
//! v0.12.0: Strict probe catalog, fix_and_accept verdict, partial answer fallback.
//! v0.12.2: Iteration-aware prompts, force answer when evidence exists.
//! v0.13.0: Strict Evidence Discipline - No more fabricated data.

// Allow dead code for features planned but not yet fully wired
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod answer_engine;
pub mod config;
pub mod config_mapper;
pub mod hardware;
pub mod knowledge;
pub mod logging;
pub mod presentation;
pub mod prompts;
pub mod reliability;
pub mod schemas;
pub mod self_health;
pub mod types;
pub mod updater;

pub use answer_engine::*;
pub use config::*;
pub use config_mapper::*;
pub use hardware::*;
pub use knowledge::*;
pub use logging::*;
pub use presentation::*;
pub use prompts::{
    generate_llm_a_prompt, generate_llm_a_prompt_with_iteration, generate_llm_b_prompt,
    LLM_A_SYSTEM_PROMPT, LLM_B_SYSTEM_PROMPT,
};
pub use reliability::*;
pub use schemas::*;
pub use self_health::*;
pub use types::*;
pub use updater::*;
