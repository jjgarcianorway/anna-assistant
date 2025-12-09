//! Daemon health types (v0.0.211).

use serde::{Deserialize, Serialize};

/// Daemon health status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DaemonInfo {
    /// Whether daemon is running
    pub running: bool,
    /// Process ID if running
    pub pid: Option<u32>,
    /// Uptime in seconds
    pub uptime_s: Option<u64>,
    /// Last error message if any
    pub last_error: Option<String>,
}

impl DaemonInfo {
    pub fn running(pid: u32, uptime_s: u64) -> Self {
        Self {
            running: true,
            pid: Some(pid),
            uptime_s: Some(uptime_s),
            last_error: None,
        }
    }

    pub fn not_running() -> Self {
        Self {
            running: false,
            pid: None,
            uptime_s: None,
            last_error: None,
        }
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.last_error = Some(error.into());
        self
    }
}
