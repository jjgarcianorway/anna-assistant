//! Shared types for Anna.
//! Contains types for daemon-client communication and shared functionality.

pub mod config;
pub mod deps;
pub mod helpers;
pub mod memory;
pub mod monitor;
pub mod profile;
pub mod recipe;
pub mod rpc;
pub mod safe_ops;
pub mod session;
pub mod skill_promotion;
pub mod status;
pub mod stats;
pub mod update_ledger;
pub mod user_context;
pub mod version;
pub mod wiki;

// Socket path (can be overridden with ANNA_SOCKET env var)
pub fn socket_path() -> String {
    std::env::var("ANNA_SOCKET").unwrap_or_else(|_| "/run/anna.sock".to_string())
}

// Version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// GitHub repo for updates
pub const GITHUB_REPO: &str = "jjgarcianorway/anna-assistant";

// Default update check interval (60 seconds)
pub const DEFAULT_UPDATE_CHECK_INTERVAL: u64 = 60;
