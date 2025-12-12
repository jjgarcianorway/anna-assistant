//! Configuration summary types (v0.0.211).
//! v0.0.449: Enhanced with all user-visible settings per VISION.md.

use serde::{Deserialize, Serialize};

/// Configuration summary - all user-visible settings
/// v0.0.449: Enhanced per VISION.md requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigInfo {
    /// Debug mode enabled
    pub debug_mode: bool,
    /// Clean REPL mode (non-debug)
    pub repl_clean_mode: bool,
    /// Autonomy level (0-100) - risk level for skipping confirmations
    pub autonomy_level: u8,
    /// Auto-update enabled
    #[serde(default = "default_true")]
    pub auto_update: bool,
    /// Learning mode - explains why commands are run
    #[serde(default)]
    pub learning_mode: bool,
    /// Fast path enabled - use recipes before LLM
    #[serde(default = "default_true")]
    pub fast_path_enabled: bool,
    /// Internal comms - show IT team dialog
    #[serde(default)]
    pub internal_comms: bool,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub request_timeout_secs: u64,
    /// Update check interval in seconds
    #[serde(default = "default_update_interval")]
    pub update_check_interval_secs: u64,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    40
}

fn default_update_interval() -> u64 {
    600
}
