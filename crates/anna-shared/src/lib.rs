//! Shared types and utilities for Anna components.

pub mod error;
pub mod ledger;
pub mod progress;
pub mod rpc;
pub mod status;
pub mod transcript;
pub mod ui;

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
