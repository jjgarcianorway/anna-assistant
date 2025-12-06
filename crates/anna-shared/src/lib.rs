//! Shared types and utilities for Anna components.
//! v0.0.73: Single source of truth for version via version module.
//! v0.0.74: Model selector with Qwen3-VL preference.
//! v0.0.75: UX realism, stats/RPG backend, recipes learned, citations.

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
pub mod editor_recipe_data;
pub mod editor_recipes;
pub mod git_recipes;
pub mod package_recipes;
pub mod service_recipes;
pub mod shell_recipes;
pub mod error;
pub mod facts;
pub mod facts_types;
pub mod fastpath;
pub mod grounding;
pub mod guard;
pub mod health_brief;
pub mod health_delta;
pub mod health_view;
pub mod helpers;
pub mod inventory;
pub mod knowledge;
pub mod intake;
pub mod ledger;
pub mod model_registry;
pub mod model_selector;
pub mod narrator;
pub mod parsers;
pub mod pending;
pub mod person_stats;
pub mod probe_spine;
pub mod progress;
pub mod recipe;
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
pub mod snapshot;
pub mod specialists;
pub mod stats;
pub mod status;
pub mod status_snapshot;
pub mod teams;
pub mod telemetry;
pub mod ticket;
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
pub use version::{VERSION, GIT_SHA, BUILD_DATE, PROTOCOL_VERSION, VersionInfo};

/// Socket path for annad
pub const SOCKET_PATH: &str = "/run/anna/anna.sock";

/// State directory for Anna
pub const STATE_DIR: &str = "/var/lib/anna";

/// Ledger file path
pub const LEDGER_PATH: &str = "/var/lib/anna/ledger.json";

/// Config file path
pub const CONFIG_PATH: &str = "/var/lib/anna/config.json";

/// Update check interval in seconds (default, can be overridden by config)
pub const DEFAULT_UPDATE_CHECK_INTERVAL: u64 = 60;

/// GitHub repository for version checks
pub const GITHUB_REPO: &str = "jjgarcianorway/anna-assistant";
