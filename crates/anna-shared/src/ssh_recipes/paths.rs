//! SSH path helpers (v0.0.196).

use std::path::PathBuf;

/// Get the SSH directory path
pub fn ssh_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".ssh")
}

/// Get the SSH config file path
pub fn ssh_config_path() -> PathBuf {
    ssh_dir().join("config")
}
