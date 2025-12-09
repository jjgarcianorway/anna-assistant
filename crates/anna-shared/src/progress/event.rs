//! Progress event types (v0.0.238).
//!
//! v0.0.238: Added StreamingToken for real-time word-by-word output.

use serde::{Deserialize, Serialize};

use super::types::{DiagnosticText, RequestStage};

/// Progress event during request processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    /// Current stage
    pub stage: RequestStage,
    /// Event type
    pub event: ProgressEventType,
    /// Optional detail message (capped to MAX_DIAGNOSTIC_LENGTH)
    pub detail: Option<DiagnosticText>,
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
    Error { message: DiagnosticText },
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
    /// v0.0.145: LLM generation progress (token count, not content)
    Generation { tokens: usize },
    /// v0.0.145: Internal comms message (from IT department staff)
    InternalComms {
        from: String,
        message: DiagnosticText,
    },
    /// v0.0.238: Streaming token for real-time output (actual token text)
    StreamingToken {
        /// The token text to display
        token: String,
        /// Whether this is the final token of the response
        is_final: bool,
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

    pub fn error(stage: RequestStage, message: impl Into<DiagnosticText>, elapsed_ms: u64) -> Self {
        Self {
            stage,
            event: ProgressEventType::Error {
                message: message.into(),
            },
            detail: None,
            elapsed_ms,
        }
    }

    pub fn heartbeat(
        stage: RequestStage,
        detail: impl Into<DiagnosticText>,
        elapsed_ms: u64,
    ) -> Self {
        Self {
            stage,
            event: ProgressEventType::Heartbeat,
            detail: Some(detail.into()),
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

    /// v0.0.145: LLM generation progress (token count)
    pub fn generation(stage: RequestStage, tokens: usize, elapsed_ms: u64) -> Self {
        Self {
            stage,
            event: ProgressEventType::Generation { tokens },
            detail: None,
            elapsed_ms,
        }
    }

    /// v0.0.145: Internal comms from IT staff
    pub fn internal_comms(
        stage: RequestStage,
        from: impl Into<String>,
        message: impl Into<DiagnosticText>,
        elapsed_ms: u64,
    ) -> Self {
        Self {
            stage,
            event: ProgressEventType::InternalComms {
                from: from.into(),
                message: message.into(),
            },
            detail: None,
            elapsed_ms,
        }
    }

    /// v0.0.238: Streaming token for real-time output
    pub fn streaming_token(
        stage: RequestStage,
        token: impl Into<String>,
        is_final: bool,
        elapsed_ms: u64,
    ) -> Self {
        Self {
            stage,
            event: ProgressEventType::StreamingToken {
                token: token.into(),
                is_final,
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
                format!("[anna] {} error: {}", self.stage, message.as_str())
            }
            ProgressEventType::Heartbeat => {
                let detail = self
                    .detail
                    .as_ref()
                    .map(|d| d.as_str())
                    .unwrap_or("working");
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
            ProgressEventType::Generation { tokens } => {
                format!("[anna] generating... {} tokens", tokens)
            }
            ProgressEventType::InternalComms { from, message } => {
                format!("[{}] {}", from, message.as_str())
            }
            ProgressEventType::StreamingToken { token, is_final } => {
                if *is_final {
                    format!("[anna] streaming done: \"{}\"", token)
                } else {
                    format!("[anna] token: \"{}\"", token)
                }
            }
        }
    }
}
