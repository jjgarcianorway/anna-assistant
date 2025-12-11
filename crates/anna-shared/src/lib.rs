//! Shared types and utilities for Anna components.
//! v0.0.73: Single source of truth for version via version module.
//! v0.0.74: Model selector with Qwen3-VL preference.
//! v0.0.75: UX realism, stats/RPG backend, recipes learned, citations.
//! v0.0.408: Research-first knowledge system with evidence-based answers.
//! v0.0.410: Evidence pipeline - real knowledge engine (probes, docs, wiki).
//! v0.0.414: Doc-first reasoning with citations and honesty tracking.
//! v0.0.415: Strict specialist contract - fast, honest, grounded answers.
//! v0.0.416: Knowledge engine and self-learning recipes.
//! v0.0.417: Strict reliability - direct answers only, no tutorials.
//! v0.0.418: Full recipe learning system - learn from tickets, execute without LLM.
//! v0.0.420: Recipe V2 - Clean learning engine with global/user recipes.
//! v0.0.421: Specialist V2 - Stable, schema-driven responses, no parse errors.
//! v0.0.422: Knowledge V2 - Research-first layer (Arch Wiki, man pages, help).
//! v0.0.423: Recipe V3 - Safe learning/execution engine with preconditions and risk levels.
//! v0.0.424: Knowledge V4 - Complete local knowledge engine with citations.
//! v0.0.425: Specialist V3 - Strict JSON contract, robust parser, no parse errors.
//! v0.0.426: Strict ticket lifecycle, honest metrics, reality-based stats.
//! v0.0.427: Self-learning recipe engine with evidence-based matching.
//! v0.0.428: Strict specialist protocol, no-bullshit policy, honest stats.
//! v0.0.429: Documentation brain - Arch Wiki, man pages, help as local knowledge graph.

pub mod advice;
pub mod answer_contract;
pub mod brief;
pub mod budget;
pub mod change;
pub mod change_history;
pub mod change_transaction;
pub mod claims;
pub mod clarify;
pub mod clarify_v2;
pub mod config_intent;
pub mod config_seed_recipes; // v0.0.264: Seed recipes for editor configs
pub mod config_types; // v0.0.264: Config types (ConfigTarget, ConfigIntent)
pub mod editor_recipe_data;
pub mod editor_recipes;
pub mod email; // v0.0.113
pub mod error;
pub mod facts;
pub mod facts_types;
pub mod fastpath;
pub mod git_recipes;
pub mod grounding;
pub mod doc_brain; // v0.0.406: Unified doc search (man pages, wiki, help)
pub mod guard;
pub mod health_brief;
pub mod health_delta;
pub mod health_view;
pub mod helpers;
pub mod intake;
pub mod inventory;
pub mod knowledge;
pub mod ledger;
pub mod model_registry;
pub mod model_selector;
pub mod narrator;
pub mod package_recipes;
pub mod parsers;
pub mod pending;
pub mod person_stats;
pub mod probe_spine;
pub mod progress;
pub mod recipe;
pub mod recipe_feedback;
pub mod recipe_file; // v0.0.406: TOML-based authored recipes
pub mod recipe_index;
pub mod recipe_learning;
pub mod recipe_matcher;
pub mod reliability;
pub mod report;
pub mod resource_limits;
pub mod review;
pub mod review_gate;
pub mod review_prompts;
pub mod revision;
pub mod roster;
pub mod rpc;
pub mod service_recipes;
pub mod shell_recipes;
pub mod snapshot;
pub mod specialists;
pub mod ssh_recipes;
pub mod systemd_recipes; // v0.0.233
pub mod cron_recipes; // v0.0.234
pub mod docker_recipes; // v0.0.235
pub mod config_parser; // v0.0.236
pub mod idle_tips; // v0.0.240
pub mod health_tips; // v0.0.244
pub mod greeting_insights; // v0.0.245
pub mod context_memory; // v0.0.246
pub mod distro_utils; // v0.0.383: Distro-aware package recommendations
pub mod followup_hints; // v0.0.384: Context-aware follow-up suggestions
pub mod stats;
pub mod status;
pub mod status_snapshot;
pub mod teams;
pub mod telemetry;
pub mod ticket;
pub mod ticket_log; // v0.0.406: Structured ticket logs for learning
pub mod ticket_state; // v0.0.407: Explicit ticket lifecycle and states
pub mod ticket_stats; // v0.0.407: Truthful ticket statistics
pub mod llm_parse; // v0.0.407: Strict LLM JSON parsing with error handling
pub mod comms_render; // v0.0.407: Internal comms rendering from ticket state
pub mod error_output; // v0.0.407: User-friendly error messages
pub mod knowledge_item; // v0.0.408: Knowledge item abstraction
pub mod doc_search; // v0.0.408: Local documentation search
pub mod solver_prompts; // v0.0.408: Evidence-focused solver prompts
pub mod recipe_candidate; // v0.0.408: Recipe candidate storage for learning
pub mod specialist_response; // v0.0.409: Unified specialist response schema
pub mod evidence_engine; // v0.0.410: Evidence engine core types
pub mod probe_registry; // v0.0.410: Composable probe definitions
pub mod doc_fetcher; // v0.0.410: Enhanced doc fetchers
pub mod evidence_gatherer; // v0.0.410: Evidence orchestration
pub mod knowledge_index; // v0.0.410: Compiled knowledge store
pub mod evidence_pipeline; // v0.0.410: Full evidence integration
pub mod recipe_engine; // v0.0.412: Self-learning recipe system
pub mod recipe_store_v2; // v0.0.412: Persistent recipe storage
pub mod doc_snippet; // v0.0.412: Documentation source integration
pub mod recipe_executor; // v0.0.412: Recipe execution engine
pub mod recipe_exec_helpers; // v0.0.412: Execution helper functions
pub mod recipe_templates; // v0.0.412: Generic parameterized recipes
pub mod recipe_converter; // v0.0.412: Ticket-to-recipe conversion
pub mod transcript_segment; // v0.0.413: Transcript segment data model
pub mod transcript_render; // v0.0.413: Cinematic/debug transcript renderer
pub mod ui_config; // v0.0.413: UI configuration (mode, spinner, etc.)
pub mod repl_greeting; // v0.0.413: Stats-based REPL greeting
pub mod knowledge_query; // v0.0.414: Doc-first knowledge query interface
pub mod knowledge_config; // v0.0.414: Knowledge source configuration
pub mod knowledge_executor; // v0.0.414: Knowledge query executor
pub mod intent_policy; // v0.0.414: Intent-based routing (no hardcoded NL)
pub mod doc_first_workflow; // v0.0.414: Doc-first specialist reasoning
pub mod knowledge_learning; // v0.0.414: Self-learning from docs and tickets
pub mod strict_contract; // v0.0.415: Strict specialist JSON contract
pub mod strict_prompts; // v0.0.415: Strict specialist prompts
pub mod translator_contract; // v0.0.415: Strict translator schema
pub mod answer_shaper; // v0.0.415: Shape answers for users
pub mod honest_stats; // v0.0.415: Honest stats tracking
pub mod regression_tests; // v0.0.415: Shape validation tests
pub mod knowledge_engine; // v0.0.416: Knowledge engine (man, help, wiki)
pub mod canonical_intents; // v0.0.416: Canonical intents and topics
pub mod learned_recipes; // v0.0.416: Self-learning recipe schema
pub mod recipe_learner; // v0.0.416: Recipe learning engine
pub mod recipe_fast_path; // v0.0.416: Recipe execution before specialists
pub mod recipe_stats; // v0.0.416: Recipe usage stats
pub mod intent_handlers; // v0.0.417: Deterministic intent handlers
pub mod recipe_schema; // v0.0.418: Recipe data model
pub mod recipe_storage; // v0.0.418: Recipe file storage and indexing
pub mod recipe_eligibility; // v0.0.418: Recipe learning eligibility checker
pub mod recipe_extractor; // v0.0.418: Extract recipes from tickets
pub mod recipe_matcher_v2; // v0.0.418: Runtime recipe matching
pub mod recipe_runtime; // v0.0.418: Recipe execution engine
pub mod recipe_telemetry; // v0.0.418: Recipe usage telemetry
pub mod seed_recipes; // v0.0.418: Initial seed recipes
pub mod recipe_v2; // v0.0.420: Clean learning engine with global/user recipes
pub mod specialist_v2; // v0.0.421: Stable, schema-driven specialist responses
pub mod knowledge_v2; // v0.0.422: Research-first knowledge layer
pub mod recipe_v3; // v0.0.423: Safe learning/execution engine with risk levels
pub mod knowledge_v4; // v0.0.424: Complete local knowledge engine with citations
pub mod specialist_v3; // v0.0.425: Strict JSON contract, robust parser, no parse errors
pub mod ticket_lifecycle; // v0.0.426: Strict ticket lifecycle state machine
pub mod honest_metrics; // v0.0.426: Reality-based stats (no fake 100%)
pub mod learning_engine; // v0.0.427: Self-learning recipe engine with evidence-based matching
pub mod specialist_protocol; // v0.0.428: Strict specialist protocol, no-bullshit policy, honest stats
pub mod doc_engine; // v0.0.429: Documentation brain - local knowledge graph
pub mod ticket_packet;
pub mod trace;
pub mod transcript;
pub mod transcript_ext;
pub mod ui;
pub mod update_ledger;
pub mod verify;
pub mod version;

// v0.0.67: Service desk narrative modules
pub mod citations;
pub mod render;
pub mod stats_store;

// v0.0.75: UX realism + stats/RPG + recipes + citations
pub mod citation;
pub mod event_log;
pub mod presentation;
pub mod recipe_store;
pub mod result_signals;

// v0.0.81: Service Desk Theatre - cinematic narrative
pub mod theatre;

// v0.0.86: Streak calculations for stats/RPG
pub mod streaks;

// v0.0.87: Dialogue variety for theatre
pub mod dialogue;

// v0.0.89: Personalized greetings and context-aware dialogue
pub mod greetings;

// v0.0.90: Achievement badges for stats/RPG
pub mod achievements;

// v0.0.105: Service Desk Foundation - tickets and user profiles
pub mod ticket_tracker;
pub mod user_profile;

// v0.0.107: Staff performance tracking
pub mod staff_stats;

// v0.0.256: Synonym expansion for recipe matching
pub mod synonyms;

// v0.0.257: Desktop configuration recipes (wallpaper, themes)
pub mod desktop_recipes;

// v0.0.258: Pending ticket retry queue
pub mod pending_queue;

// v0.0.268: Query scenario test corpus (100+ queries)
pub mod query_scenarios;

// v0.0.275: LLM-generated greeting context
pub mod greeting_context;

// v0.0.280: System telemetry tracking
pub mod system_telemetry;

// v0.0.281: Proactive health alerts
pub mod health_alerts;

// v0.0.282: LLM-based recipe similarity scoring
pub mod recipe_similarity;

// v0.0.282: Idle-time learning suggestions
pub mod learning_suggestions;

// v0.0.286: Proactive maintenance actions
pub mod maintenance_actions;

// v0.0.288: Learning progress tracking
pub mod learning_progress;

// v0.0.289: Interesting facts for greetings
pub mod interesting_facts;

// v0.0.322: Probe effectiveness learning
pub mod probe_learning;

// v0.0.401: Specialist knowledge capture
pub mod specialist_learning;
pub mod specialist_patterns; // v0.0.401: Generic pattern matching
pub mod specialist_recipes; // v0.0.401: Recipes from specialist lessons
pub mod learning_stats; // v0.0.401: Learning progress statistics
pub mod clarification_learning; // v0.0.401: Learning from user clarifications

// v0.0.404: JSON-only specialist contract
pub mod specialist_contract;

pub use error::AnnaError;
pub use ledger::{Ledger, LedgerEntry, LedgerEntryKind};
pub use rpc::{
    Capabilities, DaemonInfo, HardwareSummary, ProbeParams, ProbeType, RpcMethod, RpcRequest,
    RpcResponse, RuntimeContext,
};
pub use status::{
    BenchmarkResult, DaemonState, DaemonStatus, HardwareInfo, LlmState, LlmStatus, ModelInfo,
    OllamaStatus, ProgressInfo, UpdateStatus,
};
// v0.0.73: Re-export version constants for backward compatibility
pub use version::{VersionInfo, BUILD_DATE, GIT_SHA, PROTOCOL_VERSION, VERSION};

/// Socket path for annad (use socket_path() for env override support)
pub const SOCKET_PATH: &str = "/run/anna/anna.sock";

/// State directory for Anna (use state_dir() for env override support)
pub const STATE_DIR: &str = "/var/lib/anna";

/// Get socket path with env override support (ANNA_SOCKET)
pub fn socket_path() -> String {
    std::env::var("ANNA_SOCKET").unwrap_or_else(|_| SOCKET_PATH.to_string())
}

/// Get state directory with env override support (ANNA_STATE_DIR)
pub fn state_dir() -> String {
    std::env::var("ANNA_STATE_DIR").unwrap_or_else(|_| STATE_DIR.to_string())
}

/// Ledger file path
pub const LEDGER_PATH: &str = "/var/lib/anna/ledger.json";

/// Config file path
pub const CONFIG_PATH: &str = "/var/lib/anna/config.json";

/// Update check interval in seconds (default, can be overridden by config)
pub const DEFAULT_UPDATE_CHECK_INTERVAL: u64 = 60;

/// GitHub repository for version checks
pub const GITHUB_REPO: &str = "jjgarcianorway/anna-assistant";
