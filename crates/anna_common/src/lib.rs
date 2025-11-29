//! Anna Common - Shared types and schemas for Anna v1.0.0
//!
//! v1.0.0: Anna the Movie - Fly-on-the-wall conversation traces, unified answer UX
//! v0.95.0: RPG Display System - expanded titles, reliability-scaled XP, mood text
//! v0.92.0: Decision Policy - central routing logic, circuit breaker, per-path metrics
//! v0.91.0: Natural Language Debug Mode Control - persistent toggle via natural language
//! v0.89.0: Conversational Debug Mode - persistent toggle via natural language, Brain fast path
//! v0.88.0: Dynamic Probe Catalog & XP Wiring - single source of truth for probes, Junior/Senior XP events
//! v0.87.0: Latency Cuts & Brain Fast Path - <3s simple questions, hard fallback, always visible answer
//! v0.86.0: XP Reinforcement - Anna/Junior/Senior XP tracking, trust, ranks, behaviour bias
//! v0.85.1: XP Log Command - xp-log command, 24h metrics in status, completing v0.84.0 tasks
//! v0.85.0: Architecture Optimisation - Brain layer, LLM reduction, self-sufficiency
//! v0.84.0: Hard Test Harness - benchmarks, metrics, reliability validation
//! v0.83.0: Performance Focus - compact prompts, 15s target latency
//! v0.81.0: Structured Answers - headline/details/evidence format, latency budgets
//! v0.80.0: Razorback Fast Path - <5s response for simple questions
//! v0.79.0: CPU semantics and evidence scoring fix - probe-backed = Green
//! v0.78.0: Senior JSON Fix - minimal prompt, robust parsing, fallback scoring
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
pub mod bench;
pub mod brain;
pub mod brain_fast;
pub mod command_whitelist;
pub mod debug_state;
pub mod decision_policy;
pub mod llm_validator;
pub mod performance;
pub mod rpg_display;
pub mod xp_events;
pub mod xp_log;
pub mod xp_track;
pub mod cpu_summary;
pub mod mem_summary;
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
pub mod structured_answer;
pub mod trace;
pub mod types;
pub mod updater;
pub mod conversation_trace;
pub mod ui_colors;

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
    // v0.78.0 prompts - Minimal Senior Auditor (legacy)
    generate_senior_prompt_v78, LLM_B_SYSTEM_PROMPT_V78,
    // v0.79.0 prompts - CPU semantics and scoring fix
    generate_senior_prompt_v79, LLM_B_SYSTEM_PROMPT_V79,
    // v0.80.0 prompts - Razorback Fast Path
    generate_junior_prompt_v80, generate_senior_prompt_v80,
    LLM_A_SYSTEM_PROMPT_V80, LLM_B_SYSTEM_PROMPT_V80, ProbeSummary,
    // v0.83.0 prompts - Performance Focus (compact, decisive, 15s target)
    generate_junior_prompt_v83, generate_senior_prompt_v83,
    LLM_A_SYSTEM_PROMPT_V83, LLM_B_SYSTEM_PROMPT_V83,
};
// CPU Summary helper (v0.79.0)
pub use cpu_summary::{CpuSummary, summarize_cpu, summarize_cpu_from_text};
// Memory Summary helper (v0.80.0)
pub use mem_summary::{MemSummary, summarize_mem_from_text};
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
// Structured Answer exports (v0.81.0)
pub use structured_answer::{
    StructuredAnswer, DialogTrace, QaOutput, LatencyBudget, is_simple_question,
};
// XP Log exports (v0.84.0)
pub use xp_log::{XpLog, StoredXpEvent, Metrics24h, XP_LOG_DIR, XP_LOG_FILE};
// XP Events exports (v0.88.0)
pub use xp_events::{XpEvent, XpEventType};
// XP Track exports (v0.86.0)
pub use xp_track::{
    XpTrack, XpStore, JuniorStats as XpJuniorStats, SeniorStats as XpSeniorStats,
    AnnaStats as XpAnnaStats, get_title, xp_for_level, XP_DIR,
};
// Brain Fast Path exports (v0.87.0)
pub use brain_fast::{
    FastQuestionType, FastAnswer, try_fast_answer, create_fallback_answer, create_partial_answer,
    TimingSummary, BRAIN_BUDGET_MS, LLM_A_BUDGET_MS, LLM_B_BUDGET_MS,
    GLOBAL_SOFT_LIMIT_MS, GLOBAL_HARD_LIMIT_MS,
};
// Debug State exports (v0.89.0)
pub use debug_state::{
    DebugState, DebugIntent, debug_is_enabled, debug_set_enabled, debug_get_state,
    DEBUG_STATE_DIR, DEBUG_STATE_FILE,
};
// Decision Policy exports (v0.92.0)
pub use decision_policy::{
    BrainDomain, DecisionPlan, DecisionPolicy, AgentHealth, PathMetrics,
};
// RPG Display exports (v0.95.0)
pub use rpg_display::{
    get_rpg_title, get_title_color, get_mood_text, get_streak_text,
    progress_bar, progress_bar_with_text,
    TrustLevel, ReliabilityScale, RPG_TITLE_BANDS,
    AnnaXpEvent, JuniorXpEvent, SeniorXpEvent,
};
// Conversation Trace exports (v1.0.0)
pub use conversation_trace::{
    AnswerOrigin, OrchestrationTrace, ProbeExecTrace, ProbeStatus,
    JuniorPlanTrace, SeniorReviewTrace, SeniorVerdictType,
    ReliabilityLevel, FinalAnswerDisplay,
    store_last_answer, get_last_answer, clear_last_answer, has_last_answer,
    is_explain_request, explain_last_answer,
};
// UI Colors exports (v1.0.0) - Canonical color definitions
pub use ui_colors::{
    // Thresholds (canonical from docs/architecture.md)
    THRESHOLD_GREEN, THRESHOLD_YELLOW, THRESHOLD_RED,
    // Actor colors
    COLOR_ANNA, COLOR_JUNIOR, COLOR_SENIOR, COLOR_SYSTEM,
    // Reliability colors
    COLOR_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_REFUSED,
    // Status colors
    COLOR_OK, COLOR_ERROR, COLOR_WARNING, COLOR_MUTED,
    // Types (use UiReliabilityLevel to avoid conflict with conversation_trace::ReliabilityLevel)
    ReliabilityLevel as UiReliabilityLevel,
    Actor as UiActor,
    // Helper functions
    format_score_colored, format_score_with_label,
    reliability_display, reliability_display_f32,
};
