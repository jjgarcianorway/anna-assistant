//! Shared types and utilities for Anna components.

pub mod advice;
pub mod brief;
pub mod budget;
pub mod change;
pub mod claims;
pub mod clarify;
pub mod clarify_v2;
pub mod config_intent;
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
pub mod narrator;
pub mod parsers;
pub mod pending;
pub mod person_stats;
pub mod probe_spine;
pub mod progress;
pub mod recipe;
pub mod recipe_index;
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

pub use error::AnnaError;
pub use ledger::{Ledger, LedgerEntry, LedgerEntryKind};
pub use rpc::{
    Capabilities, HardwareSummary, ProbeParams, ProbeType, RpcMethod, RpcRequest, RpcResponse,
    RuntimeContext,
};
pub use status::{
    BenchmarkResult, DaemonState, DaemonStatus, HardwareInfo, LlmState, LlmStatus, ModelInfo,
    OllamaStatus, ProgressInfo, UpdateStatus,
};

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

/// Anna version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
