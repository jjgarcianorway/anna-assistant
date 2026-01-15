//! Shared types for Anna.
//! Contains types for daemon-client communication and shared functionality.
//!
//! ARCHITECTURAL INVARIANT: Anna is system-wide with ZERO state in user home directories.
//! All paths are defined in the `paths` module and use /etc/anna, /var/lib/anna, /run/anna.

pub mod action_plan;
pub mod claim_gate;
pub mod config;
pub mod deps;
pub mod docs;
pub mod event_bus;
pub mod experiment;
pub mod exposure;
pub mod helpers;
pub mod intent_class;
pub mod memory;
pub mod migration;
pub mod monitor;
pub mod outcome_ledger;
pub mod paths;
pub mod policy;
pub mod probe_ledger;
pub mod profile;
pub mod recipe;
pub mod rpc;
pub mod safe_ops;
pub mod session;
pub mod status;
pub mod stats;
pub mod teaching;
pub mod telemetry_consumer;
pub mod timeline;
pub mod update_ledger;
pub mod user_context;
pub mod version;
pub mod wiki;

// Re-export paths for convenience
pub use paths::{paths, Paths};

// Socket path (uses system paths, can be overridden with ANNA_SOCKET env var)
pub fn socket_path() -> String {
    std::env::var("ANNA_SOCKET")
        .unwrap_or_else(|_| paths().socket_file().to_string_lossy().to_string())
}

// Version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// GitHub repo for updates
pub const GITHUB_REPO: &str = "jjgarcianorway/anna-assistant";

// Default update check interval (60 seconds)
pub const DEFAULT_UPDATE_CHECK_INTERVAL: u64 = 60;
