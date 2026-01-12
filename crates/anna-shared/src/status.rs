//! Daemon status types.
//! v0.0.924: Added memory health fields
//! v0.1.0: Added update timing and extended status fields

use serde::{Deserialize, Serialize};

/// Overall daemon status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub state: DaemonState,
    pub version: String,
    pub ollama_running: bool,
    pub model: Option<String>,
    pub uptime_secs: u64,
    pub gpu: Option<String>,
    pub vram_mb: Option<u64>,
    /// v0.0.924: Number of experiences in memory
    #[serde(default)]
    pub memory_experiences: usize,
    /// v0.0.924: Memory health issues (if any)
    #[serde(default)]
    pub memory_health_issues: Vec<String>,
    /// v0.1.0: Update check timing
    #[serde(default)]
    pub update_check_interval_secs: u64,
    /// v0.1.0: Last update check timestamp (RFC3339)
    #[serde(default)]
    pub last_update_check: Option<String>,
    /// v0.1.0: Next update check timestamp (RFC3339)
    #[serde(default)]
    pub next_update_check: Option<String>,
    /// v0.1.0: Latest available version from GitHub
    #[serde(default)]
    pub latest_version: Option<String>,
    /// v0.1.0: Update check state
    #[serde(default)]
    pub update_state: UpdateCheckState,
    /// v0.1.0: Number of active patterns
    #[serde(default)]
    pub pattern_count: usize,
    /// v0.1.0: Number of learned recipes
    #[serde(default)]
    pub recipe_count: usize,
}

/// Daemon state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DaemonState {
    #[default]
    Starting,
    Ready,
    Error,
}

impl std::fmt::Display for DaemonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonState::Starting => write!(f, "STARTING"),
            DaemonState::Ready => write!(f, "READY"),
            DaemonState::Error => write!(f, "ERROR"),
        }
    }
}

/// Update check state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UpdateCheckState {
    #[default]
    NeverChecked,
    Success,
    Failed,
    Checking,
}

impl std::fmt::Display for UpdateCheckState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateCheckState::NeverChecked => write!(f, "NEVER_CHECKED"),
            UpdateCheckState::Success => write!(f, "OK"),
            UpdateCheckState::Failed => write!(f, "FAILED"),
            UpdateCheckState::Checking => write!(f, "CHECKING"),
        }
    }
}
