//! Anna Common - Shared types and schemas for Anna v0.30.2
//!
//! v0.27.0: Qwen inference, reliability improvements.
//! v0.28.0: Auto-update improvements, installer fixes.
//! v0.28.1: Emoji removal, ASCII-only aesthetic.
//! v0.29.0: Fast-path for unsupported questions (no 100s LLM waits).
//! v0.30.0: Journalctl probe, question classifier, auto-update fix (ETXTBSY).
//! v0.30.1: Debug output limit increased from 2000 to 8000 chars.
//! v0.30.2: Fix update status display - use semver comparison, not string equality.
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
//! v0.15.0: Research Loop Engine - Junior/Senior LLM with command whitelist.
//! v0.16.0: Qwen3 default, granular hardware tiers.
//! v0.16.1: Dynamic model registry, on-demand LLM loading, fact validation.
//! v0.18.0: Step-by-step orchestration (one action per iteration).
//! v0.19.0: Subproblem decomposition, fact-aware planning, Senior as mentor.
//! v0.20.0: Background telemetry, warm-up learning, fact store integration.
//! v0.21.0: Hybrid answer pipeline (fast-first, selective probing, no loops).
//! v0.22.0: Fact Brain & Question Decomposition (TTLs, validated facts).
//! v0.23.0: System Brain, User Brain & Idle Learning.
//! v0.24.0: App Awareness, Stats & Faster Answers.
//! v0.25.0: Relevance First, Idle Learning, No Hard-Coding.
//! v0.26.0: Auto-update Reliability, Self-Healing, Logging.

// Allow dead code for features planned but not yet fully wired
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod answer_engine;
pub mod command_whitelist;
pub mod config;
pub mod config_mapper;
pub mod hardware;
pub mod knowledge;
pub mod logging;
pub mod model_registry;
pub mod presentation;
pub mod prompts;
pub mod question_classifier;
pub mod reliability;
pub mod safety;
pub mod schemas;
pub mod self_health;
pub mod types;
pub mod updater;

pub use answer_engine::*;
pub use command_whitelist::*;
pub use config::*;
pub use config_mapper::*;
pub use hardware::*;
pub use knowledge::*;
pub use logging::*;
pub use model_registry::*;
pub use presentation::*;
pub use prompts::{
    generate_llm_a_prompt, generate_llm_a_prompt_with_iteration, generate_llm_b_prompt,
    LLM_A_SYSTEM_PROMPT, LLM_B_SYSTEM_PROMPT,
    // v0.18.0 prompts
    generate_junior_prompt, generate_senior_prompt, LLM_A_SYSTEM_PROMPT_V18, LLM_B_SYSTEM_PROMPT_V18,
    // v0.19.0 prompts
    generate_junior_decomposition_prompt, generate_junior_post_mentor_prompt,
    generate_junior_work_prompt, generate_senior_mentor_prompt, generate_senior_review_prompt,
    LLM_A_SYSTEM_PROMPT_V19, LLM_B_SYSTEM_PROMPT_V19,
};
pub use question_classifier::*;
pub use reliability::*;
pub use safety::*;
pub use schemas::*;
pub use self_health::*;
pub use types::*;
pub use updater::*;
