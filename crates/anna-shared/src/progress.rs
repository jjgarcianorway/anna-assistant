//! Progress events for request pipeline visibility.
//!
//! Provides structured progress updates during request processing.

use serde::{Deserialize, Serialize};

/// Stage of request processing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStage {
    Translator,
    Probes,
    Specialist,
    Supervisor,
}

impl std::fmt::Display for RequestStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Translator => write!(f, "translator"),
            Self::Probes => write!(f, "probes"),
            Self::Specialist => write!(f, "specialist"),
            Self::Supervisor => write!(f, "supervisor"),
        }
    }
}

/// Progress event during request processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    /// Current stage
    pub stage: RequestStage,
    /// Event type
    pub event: ProgressEventType,
    /// Optional detail message
    pub detail: Option<String>,
    /// Elapsed time since request started (ms)
    pub elapsed_ms: u64,
}

/// Type of progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressEventType {
    /// Stage starting
    Starting { timeout_secs: u64 },
    /// Stage completed successfully
    Complete,
    /// Stage timed out
    Timeout,
    /// Stage failed with error
    Error { message: String },
    /// Heartbeat (still working)
    Heartbeat,
    /// Probe-specific: running a probe
    ProbeRunning { probe_id: String },
    /// Probe-specific: probe completed
    ProbeComplete {
        probe_id: String,
        exit_code: i32,
        timing_ms: u64,
    },
}

impl ProgressEvent {
    pub fn starting(stage: RequestStage, timeout_secs: u64, elapsed_ms: u64) -> Self {
        Self {
            stage,
            event: ProgressEventType::Starting { timeout_secs },
            detail: None,
            elapsed_ms,
        }
    }

    pub fn complete(stage: RequestStage, elapsed_ms: u64) -> Self {
        Self {
            stage,
            event: ProgressEventType::Complete,
            detail: None,
            elapsed_ms,
        }
    }

    pub fn timeout(stage: RequestStage, elapsed_ms: u64) -> Self {
        Self {
            stage,
            event: ProgressEventType::Timeout,
            detail: None,
            elapsed_ms,
        }
    }

    pub fn error(stage: RequestStage, message: String, elapsed_ms: u64) -> Self {
        Self {
            stage,
            event: ProgressEventType::Error { message },
            detail: None,
            elapsed_ms,
        }
    }

    pub fn heartbeat(stage: RequestStage, detail: String, elapsed_ms: u64) -> Self {
        Self {
            stage,
            event: ProgressEventType::Heartbeat,
            detail: Some(detail),
            elapsed_ms,
        }
    }

    pub fn probe_running(probe_id: &str, elapsed_ms: u64) -> Self {
        Self {
            stage: RequestStage::Probes,
            event: ProgressEventType::ProbeRunning {
                probe_id: probe_id.to_string(),
            },
            detail: None,
            elapsed_ms,
        }
    }

    pub fn probe_complete(probe_id: &str, exit_code: i32, timing_ms: u64, elapsed_ms: u64) -> Self {
        Self {
            stage: RequestStage::Probes,
            event: ProgressEventType::ProbeComplete {
                probe_id: probe_id.to_string(),
                exit_code,
                timing_ms,
            },
            detail: None,
            elapsed_ms,
        }
    }

    /// Format for debug display
    pub fn format_debug(&self) -> String {
        match &self.event {
            ProgressEventType::Starting { timeout_secs } => {
                format!(
                    "[anna->{}] starting (timeout {}s)",
                    self.stage, timeout_secs
                )
            }
            ProgressEventType::Complete => {
                format!("[anna] {} complete", self.stage)
            }
            ProgressEventType::Timeout => {
                format!("[anna] {} TIMEOUT after {}ms", self.stage, self.elapsed_ms)
            }
            ProgressEventType::Error { message } => {
                format!("[anna] {} error: {}", self.stage, message)
            }
            ProgressEventType::Heartbeat => {
                let detail = self.detail.as_deref().unwrap_or("working");
                format!(
                    "[anna] still working: {} ({:.1}s)",
                    detail,
                    self.elapsed_ms as f64 / 1000.0
                )
            }
            ProgressEventType::ProbeRunning { probe_id } => {
                format!("[anna->probe] running {} (timeout 4s)", probe_id)
            }
            ProgressEventType::ProbeComplete {
                probe_id,
                exit_code,
                timing_ms,
            } => {
                format!(
                    "[anna] probe {} complete exit={} time={}ms",
                    probe_id, exit_code, timing_ms
                )
            }
        }
    }
}

/// Timeout configuration for each stage
#[derive(Debug, Clone, Copy)]
pub struct TimeoutConfig {
    pub translator_secs: u64,
    pub probe_each_secs: u64,
    pub probes_total_secs: u64,
    pub specialist_secs: u64,
    pub supervisor_secs: u64,
    pub heartbeat_interval_secs: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            translator_secs: 8,
            probe_each_secs: 4,
            probes_total_secs: 10,
            specialist_secs: 12,
            supervisor_secs: 8,
            heartbeat_interval_secs: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_event_format() {
        let event = ProgressEvent::starting(RequestStage::Translator, 8, 0);
        assert!(event.format_debug().contains("translator"));
        assert!(event.format_debug().contains("8s"));
    }

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.translator_secs, 8);
        assert_eq!(config.specialist_secs, 12);
    }
}
