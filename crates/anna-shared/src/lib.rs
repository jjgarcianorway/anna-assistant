//! Shared types for Anna - minimal version.
//! Only contains essential types for daemon-client communication.

pub mod rpc;
pub mod status;
pub mod version;
pub mod update_ledger;

// Socket path
pub fn socket_path() -> String {
    "/run/anna.sock".to_string()
}

// Version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// GitHub repo for updates
pub const GITHUB_REPO: &str = "jjgarcianorway/anna-assistant";

// Default update check interval (60 seconds)
pub const DEFAULT_UPDATE_CHECK_INTERVAL: u64 = 60;
