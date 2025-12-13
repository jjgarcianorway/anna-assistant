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
//! v0.0.430: Background workers, idle-time learning, alerts, and long-running tickets.
//! v0.0.431: Hollywood UX - Unified transcript and terminal renderer.
//! v0.0.432: Knowledge pipeline - Priority-ordered knowledge fetching and learning.
//! v0.0.433: Robustness layer - Timeouts, failure handling, and truthful stats.
//! v0.0.434: Hardware-aware model selection, local model management, and helper installation.
//! v0.0.435: Evidence-first knowledge engine - citations, probe primitives, recipes with promotion.
//! v0.0.436: Anna Protocol v1 - Unbreakable typed JSON communication, no more parse errors.
//! v0.0.437: Question Contract - Fix understanding and answer minimality.
//! v0.0.438: Fast Pipeline - Hard budgets, no streaming for parsed calls, reliability stats.
//! v0.0.439: Deterministic Routing - Fix "Sofia handles everything" with intent-to-department map.
//! v0.0.440: Specialist Contract v1 - Strict JSON schema, retries, fallback summarizer.
//! v0.0.441: ERA Pipeline - Universal Evidence → Reasoning → Answer architecture.
//! v0.0.442: Ticket Integrity - Honest stats, clarification-first, package/system separation.
//! v0.0.443: Source Layer - Citations, trace observability, clean inventories, honest stats/UI.
//! v0.0.444: Reliability Metrics - Canonical outcomes, honest stats, accurate inventories.
//! v0.0.444: Debug Mode - 3-level debug output, sanitization, reason codes, routing transparency.
//! v0.0.445: Hard Reliability Gate - No answer without evidence, no fake success.
//! v0.0.448: Deterministic Probes - Intent-specific probe mapping, no LLM guessing.

pub mod advice;
pub mod anna_proto; // v0.0.436: Unbreakable typed model communication protocol
pub mod answer_contract;
pub mod answer_shaper; // v0.0.415: Shape answers for users
pub mod background_worker; // v0.0.430: Background job system
pub mod brief;
pub mod budget;
pub mod canonical_intents; // v0.0.416: Canonical intents and topics
pub mod change;
pub mod change_history;
pub mod change_transaction;
pub mod claims;
pub mod clarify;
pub mod clarify_v2;
pub mod comms_render; // v0.0.407: Internal comms rendering from ticket state
pub mod config_intent;
pub mod config_parser; // v0.0.236
pub mod config_seed_recipes; // v0.0.264: Seed recipes for editor configs
pub mod config_types; // v0.0.264: Config types (ConfigTarget, ConfigIntent)
pub mod context_memory; // v0.0.246
pub mod cron_recipes; // v0.0.234
pub mod database_recipes; // v0.0.461: Database backup/restore recipes
pub mod debug_config; // v0.0.473: Debug configuration via natural language
pub mod debug_mode; // v0.0.444: Debug levels, sanitization, reason codes
pub mod kubernetes_recipes; // v0.0.459: Kubernetes pod/deployment recipes
pub mod deterministic_probes; // v0.0.448: Intent → probes deterministic mapping
pub mod deterministic_routing; // v0.0.439: Deterministic routing with intent-to-department map
pub mod distro_utils; // v0.0.383: Distro-aware package recommendations
pub mod doc_brain; // v0.0.406: Unified doc search (man pages, wiki, help)
pub mod doc_engine; // v0.0.429: Documentation brain - local knowledge graph
pub mod doc_fetcher; // v0.0.410: Enhanced doc fetchers
pub mod doc_first_workflow; // v0.0.414: Doc-first specialist reasoning
pub mod doc_search; // v0.0.408: Local documentation search
pub mod doc_snippet; // v0.0.412: Documentation source integration
pub mod docker_recipes; // v0.0.235
pub mod editor_recipe_data;
pub mod editor_recipes;
pub mod email; // v0.0.113
pub mod era_pipeline; // v0.0.441: ERA Pipeline - Evidence → Reasoning → Answer
pub mod error;
pub mod error_output; // v0.0.407: User-friendly error messages
pub mod evidence_engine; // v0.0.410: Evidence engine core types
pub mod evidence_first; // v0.0.435: Evidence-first knowledge engine
pub mod evidence_gatherer; // v0.0.410: Evidence orchestration
pub mod evidence_pipeline; // v0.0.410: Full evidence integration
pub mod facts;
pub mod facts_types;
pub mod facts_maintenance; // v0.0.471: Facts lifecycle maintenance
pub mod fast_pipeline; // v0.0.438: Fast Pipeline - hard budgets, no streaming, reliability stats
pub mod fastpath;
pub mod followup_hints; // v0.0.384: Context-aware follow-up suggestions
pub mod git_recipes;
pub mod greeting_insights; // v0.0.245
pub mod grounding;
pub mod guard;
pub mod hardware_aware; // v0.0.434: Hardware-aware model selection and helper management
pub mod health_brief;
pub mod health_delta;
pub mod health_tips; // v0.0.244
pub mod health_view;
pub mod helpers;
pub mod hollywood_ux; // v0.0.431: Unified transcript and Hollywood terminal renderer
pub mod honest_metrics; // v0.0.426: Reality-based stats (no fake 100%)
pub mod honest_stats; // v0.0.415: Honest stats tracking
pub mod idle_tips; // v0.0.240
pub mod intake;
pub mod intent_handlers; // v0.0.417: Deterministic intent handlers
pub mod intent_policy; // v0.0.414: Intent-based routing (no hardcoded NL)
pub mod inventory;
pub mod knowledge;
pub mod knowledge_config; // v0.0.414: Knowledge source configuration
pub mod knowledge_engine; // v0.0.416: Knowledge engine (man, help, wiki)
pub mod knowledge_executor; // v0.0.414: Knowledge query executor
pub mod knowledge_index; // v0.0.410: Compiled knowledge store
pub mod knowledge_item; // v0.0.408: Knowledge item abstraction
pub mod knowledge_learning; // v0.0.414: Self-learning from docs and tickets
pub mod knowledge_pipeline; // v0.0.432: Priority-ordered knowledge fetching and learning
pub mod knowledge_query; // v0.0.414: Doc-first knowledge query interface
pub mod knowledge_v2; // v0.0.422: Research-first knowledge layer
pub mod knowledge_v4; // v0.0.424: Complete local knowledge engine with citations
pub mod learned_recipes; // v0.0.416: Self-learning recipe schema
pub mod learning_engine; // v0.0.427: Self-learning recipe engine with evidence-based matching
pub mod learning_explanations; // v0.0.457: Learning mode command explanations
pub mod ledger;
pub mod llm_parse; // v0.0.407: Strict LLM JSON parsing with error handling
pub mod long_task; // v0.0.455: Long-running task detection and handling
pub mod model_registry;
pub mod model_selector;
pub mod narrator;
pub mod network_recipes; // v0.0.462: Network troubleshooting recipes
pub mod notification_config; // v0.0.470: Notification config via natural language
pub mod package_recipes;
pub mod parsers;
pub mod pending;
pub mod person_stats;
pub mod probe_registry; // v0.0.410: Composable probe definitions
pub mod probe_spine;
pub mod progress;
pub mod question_contract; // v0.0.437: Question Contract - typed intent and answer shape enforcement
pub mod recipe;
pub mod recipe_candidate; // v0.0.408: Recipe candidate storage for learning
pub mod recipe_converter; // v0.0.412: Ticket-to-recipe conversion
pub mod recipe_eligibility; // v0.0.418: Recipe learning eligibility checker
pub mod recipe_engine; // v0.0.412: Self-learning recipe system
pub mod recipe_exec_helpers; // v0.0.412: Execution helper functions
pub mod recipe_executor; // v0.0.412: Recipe execution engine
pub mod recipe_extractor; // v0.0.418: Extract recipes from tickets
pub mod recipe_fast_path; // v0.0.416: Recipe execution before specialists
pub mod recipe_feedback;
pub mod recipe_file; // v0.0.406: TOML-based authored recipes
pub mod recipe_index;
pub mod recipe_learner; // v0.0.416: Recipe learning engine
pub mod recipe_learning;
pub mod recipe_matcher;
pub mod recipe_matcher_v2; // v0.0.418: Runtime recipe matching
pub mod recipe_runtime; // v0.0.418: Recipe execution engine
pub mod recipe_schema; // v0.0.418: Recipe data model
pub mod recipe_stats; // v0.0.416: Recipe usage stats
pub mod recipe_storage; // v0.0.418: Recipe file storage and indexing
pub mod recipe_store_v2; // v0.0.412: Persistent recipe storage
pub mod recipe_telemetry; // v0.0.418: Recipe usage telemetry
pub mod recipe_templates; // v0.0.412: Generic parameterized recipes
pub mod recipe_v2; // v0.0.420: Clean learning engine with global/user recipes
pub mod recipe_v3; // v0.0.423: Safe learning/execution engine with risk levels
pub mod regression_tests; // v0.0.415: Shape validation tests
pub mod reliability;
pub mod reliability_gate; // v0.0.445: Hard reliability gate, claim/evidence model, deterministic-first
pub mod reliability_metrics; // v0.0.444: Canonical outcomes, real reliability stats
pub mod repl_greeting; // v0.0.413: Stats-based REPL greeting
pub mod report;
pub mod resource_limits;
pub mod risk_config; // v0.0.474: Risk level configuration via natural language
pub mod review;
pub mod review_gate;
pub mod review_prompts;
pub mod revision;
pub mod robustness; // v0.0.433: Timeouts, failure handling, and truthful stats
pub mod roster;
pub mod rpc;
pub mod seed_recipes; // v0.0.418: Initial seed recipes
pub mod senior_strategic; // v0.0.458: Senior idle-time strategic thinking
pub mod service_recipes;
pub mod session_display; // v0.0.475: Session summary display
pub mod settings_display; // v0.0.476: Unified settings display
pub mod shell_recipes;
pub mod snapshot;
pub mod solver_prompts; // v0.0.408: Evidence-focused solver prompts
pub mod source_layer; // v0.0.443: Source providers, citations, trace, inventories
pub mod specialist_contract_v1; // v0.0.440: Specialist Contract v1 - strict JSON, retries, fallback
pub mod specialist_protocol; // v0.0.428: Strict specialist protocol, no-bullshit policy, honest stats
pub mod specialist_response; // v0.0.409: Unified specialist response schema
pub mod specialist_v2; // v0.0.421: Stable, schema-driven specialist responses
pub mod specialist_v3; // v0.0.425: Strict JSON contract, robust parser, no parse errors
pub mod specialists;
pub mod ssh_recipes;
pub mod stats;
pub mod status;
pub mod status_snapshot;
pub mod strict_contract; // v0.0.415: Strict specialist JSON contract
pub mod strict_prompts; // v0.0.415: Strict specialist prompts
pub mod systemd_recipes; // v0.0.233
pub mod teams;
pub mod telemetry;
pub mod ticket;
pub mod ticket_integrity; // v0.0.442: Honest stats, clarification-first, package/system separation
pub mod ticket_lifecycle; // v0.0.426: Strict ticket lifecycle state machine
pub mod ticket_log; // v0.0.406: Structured ticket logs for learning
pub mod ticket_packet;
pub mod ticket_state; // v0.0.407: Explicit ticket lifecycle and states
pub mod ticket_stats; // v0.0.407: Truthful ticket statistics
pub mod trace;
pub mod transcript;
pub mod transcript_ext;
pub mod transcript_render; // v0.0.413: Cinematic/debug transcript renderer
pub mod transcript_segment; // v0.0.413: Transcript segment data model
pub mod translator_contract; // v0.0.415: Strict translator schema
pub mod truth_ledger;
pub mod ui;
pub mod ui_config; // v0.0.413: UI configuration (mode, spinner, etc.)
pub mod update_ledger;
pub mod user_alarms; // v0.0.456: Natural language alarms and reminders
pub mod preference_config; // v0.0.467: Natural language preference configuration
pub mod verify;
pub mod webserver_recipes; // v0.0.460: Nginx/Apache configuration recipes
pub mod wiki_cache; // v0.0.472: Arch Wiki local caching
pub mod xp_display; // v0.0.478: XP/Level RPG-style progression display
pub mod fun_stats_display; // v0.0.479: Fun statistics display
pub mod capabilities_display; // v0.0.480: Capabilities display
pub mod query_type_router; // v0.0.481: Query type router
pub mod contextual_tips; // v0.0.482: Contextual tips system
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

// v0.0.454: Dynamic team availability based on hardware
pub mod team_availability;

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
pub mod greeting_tips; // v0.0.468: Configuration tips in greetings

// v0.0.280: System telemetry tracking
pub mod system_telemetry;
pub mod system_monitors; // v0.0.469: Proactive system monitoring

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
pub mod clarification_learning;
pub mod learning_stats; // v0.0.401: Learning progress statistics
pub mod specialist_learning;
pub mod specialist_patterns; // v0.0.401: Generic pattern matching
pub mod specialist_recipes; // v0.0.401: Recipes from specialist lessons // v0.0.401: Learning from user clarifications

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
