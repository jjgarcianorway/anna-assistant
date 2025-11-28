//! Anna Common - Shared types and schemas for Anna v0.77.0
//!
//! v0.77.0: Dialog View - LLM prompts/responses streamed to annactl (not logs)
//! v0.76.2: Dialog View events - LLM prompt/response events for real-time streaming
//! v0.76.1: Full Debug Output - no truncation, show system prompts, show timing
//! v0.76.0: Minimal Junior Planner - radically reduced prompt for 4B model performance
//!
//! v0.27.0: Qwen inference, reliability improvements.
//! v0.28.0: Auto-update improvements, installer fixes.
//! v0.28.1: Emoji removal, ASCII-only aesthetic.
//! v0.29.0: Fast-path for unsupported questions (no 100s LLM waits).
//! v0.30.0: Journalctl probe, question classifier, auto-update fix (ETXTBSY).
//! v0.30.1: Debug output limit increased from 2000 to 8000 chars.
//! v0.30.2: Fix update status display - use semver comparison, not string equality.
//! v0.40.0: Generic skills, parameterized commands, skill learning, no probe zoo.
//! v0.40.1: RPG progression system - levels, XP, titles, performance statistics.
//! v0.42.0: Negative Feedback, Skill Pain, Remediation Engine.
//! v0.43.0: Live Debug Streaming View.
//! v0.50.0: Brain Upgrade - 5-type classification, safe command policy, generic probes.
//! v0.60.0: Conversational UX - live progress events, conversation logging, persona messaging.
//! v0.65.0: Reliability Patch - confidence gating (60% min), stats tracking, daemon robustness.
//! v0.70.0: Evidence Oracle - structured LLM protocol, difficulty routing, knowledge-first.
//! v0.74.0: Structured Trace Pipeline - JSON traces, debug output, canonical questions.
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
pub mod events;
pub mod config;
pub mod config_mapper;
pub mod first_run;
pub mod hardware;
pub mod knowledge;
pub mod llm_protocol;
pub mod logging;
pub mod model_registry;
pub mod pain;
pub mod presentation;
pub mod progression;
pub mod prompts;
pub mod question_classifier;
pub mod reliability;
pub mod roles;
pub mod safety;
pub mod schemas;
pub mod self_health;
pub mod skills;
pub mod trace;
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
    // v0.76.0 prompts - Minimal Junior Planner
    generate_junior_prompt_v76, LLM_A_SYSTEM_PROMPT_V76, PROBE_LIST_V76,
};
pub use question_classifier::*;
pub use reliability::*;
pub use safety::*;
pub use schemas::*;
pub use self_health::*;
// Explicit skills exports to avoid naming conflicts
pub use skills::{
    builtin_skills, execute_safe_command, execute_safe_command_async, execute_skill,
    init_builtin_skills, validate_params, AnnaLevel, ParamSchema, ParamType, Skill, SkillExecutionResult,
    SkillQuery, SkillStats, SkillStore, SKILLS_DIR, SKILL_SCHEMA_VERSION,
};
// Alias the skills SystemStats to avoid conflict with knowledge::SystemStats
pub use skills::SystemStats as SkillSystemStats;
pub use types::*;
pub use updater::*;
// First-run detection (v0.72.0)
pub use first_run::{is_first_run, is_initialized, mark_initialized, MARKER_FILE};
// Explicit progression exports to avoid conflicts
pub use progression::{
    AnnaProgression, Level, Title, TITLE_BANDS,
    GlobalStats as ProgressionGlobalStats, PatternStats, PerformanceSnapshot,
    QuestionPattern, StatsEngine, STATS_DIR,
    XpCalculator, XpGain, XpInput, XpPenalty, XP_CONFIG,
};
// Pain module exports
pub use pain::{PainEvent, PainLog, PainType, PAIN_DIR};
// Roles module exports
pub use roles::{
    JuniorRank, JuniorStats, RoleStats, SeniorRank, SeniorStats, ROLES_DIR,
};
// Events module exports (v0.60.0)
pub use events::{
    Actor, EventKind, AnnaEvent, ConversationLog, ConversationSummary,
    question_received, classification_started, classification_done,
    probes_planned, command_running, command_done,
    senior_review_started, senior_review_done,
    user_clarification_needed, answer_synthesizing, answer_ready, error_event,
};
// LLM Protocol exports (v0.70.0) - only non-conflicting types
// Note: ProbeRequest, Citation, LlmAPlan, Verdict, ReliabilityScores are already
// defined in answer_engine and types modules - use those instead.
pub use llm_protocol::{
    Difficulty, QuestionIntent, LatencyExpectation,
    SafeShellRequest, DocsRequest, KnowledgeQuery,
    LlmADraftAnswer, LlmAOutput,
    KnowledgeUpdate, LlmBOutput,
    CommandTemplate,
};
// Trace module exports (v0.74.0 + v0.75.0)
pub use trace::{
    // v0.74.0 types
    QuestionTrace, JuniorPlan, ProbeTrace, SeniorVerdict,
    generate_correlation_id, is_debug_mode as trace_is_debug_mode,
    // v0.75.0 DebugBlock types
    DebugBlock, InputSection, JuniorPlanSection, ProbesSection,
    ProbeExecution, ProbeFailure, SeniorVerdictSection, VerdictScores,
    FinalAnswerSection, RawLlmMessages,
};
