//! Constants and configuration paths for Anna
//! Centralized location for all system-wide constants

/// Socket path for annad (use socket_path() for env override support)
pub const SOCKET_PATH: &str = "/run/anna/anna.sock";

/// State directory for Anna (use state_dir() for env override support)
pub const STATE_DIR: &str = "/var/lib/anna";

/// Ledger file path
pub const LEDGER_PATH: &str = "/var/lib/anna/ledger.json";

/// Config file path
pub const CONFIG_PATH: &str = "/var/lib/anna/config.json";

/// Update check interval in seconds (default, can be overridden by config)
pub const DEFAULT_UPDATE_CHECK_INTERVAL: u64 = 60;

/// GitHub repository for version checks
pub const GITHUB_REPO: &str = "jjgarcianorway/anna-assistant";

/// Get socket path with env override support (ANNA_SOCKET)
pub fn socket_path() -> String {
    std::env::var("ANNA_SOCKET").unwrap_or_else(|_| SOCKET_PATH.to_string())
}

/// Get state directory with env override support (ANNA_STATE_DIR)
pub fn state_dir() -> String {
    std::env::var("ANNA_STATE_DIR").unwrap_or_else(|_| STATE_DIR.to_string())
}
