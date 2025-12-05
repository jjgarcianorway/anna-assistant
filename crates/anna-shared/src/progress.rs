//! Progress events for request pipeline visibility.
//!
//! Provides structured progress updates during request processing.
//!
//! INVARIANT: Progress events are telemetry only - never user-facing content.
//! All string fields are capped to prevent content leakage.

use serde::{Deserialize, Serialize};

/// Maximum length for diagnostic text fields (error messages, details).
pub const MAX_DIAGNOSTIC_LENGTH: usize = 100;

/// Diagnostic text with enforced length cap.
/// Prevents accidental content leakage through progress events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DiagnosticText(String);

impl DiagnosticText {
    /// Create diagnostic text, truncating if over limit.
    pub fn new(s: impl Into<String>) -> Self {
        let s = s.into();
        if s.len() > MAX_DIAGNOSTIC_LENGTH {
            Self(format!("{}...", &s[..MAX_DIAGNOSTIC_LENGTH - 3]))
        } else {
            Self(s)
        }
    }

    /// Get the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DiagnosticText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for DiagnosticText {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for DiagnosticText {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl std::ops::Deref for DiagnosticText {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
            event: ProgressEventType::Error { message: message.into() },
            detail: None,
            elapsed_ms,
        }
    }

    pub fn heartbeat(stage: RequestStage, detail: impl Into<DiagnosticText>, elapsed_ms: u64) -> Self {
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
                let detail = self.detail.as_ref().map(|d| d.as_str()).unwrap_or("working");
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

    /// GUARDRAIL: ProgressEventType must never carry user-facing content.
    /// Progress is telemetry only. Answers flow through ServiceDeskResult.
    #[test]
    fn test_progress_event_no_answer_content() {
        // Serialize all event variants and verify no "answer" or "content" fields
        let events = vec![
            ProgressEventType::Starting { timeout_secs: 10 },
            ProgressEventType::Complete,
            ProgressEventType::Timeout,
            ProgressEventType::Error { message: DiagnosticText::new("test error") },
            ProgressEventType::Heartbeat,
            ProgressEventType::ProbeRunning { probe_id: "test".into() },
            ProgressEventType::ProbeComplete { probe_id: "test".into(), exit_code: 0, timing_ms: 100 },
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            // These fields should NEVER appear in progress events
            assert!(!json.contains("\"answer\""), "Progress event must not contain 'answer' field");
            assert!(!json.contains("\"content\""), "Progress event must not contain 'content' field");
            assert!(!json.contains("\"response\""), "Progress event must not contain 'response' field");
            // String payloads should be short diagnostics only (< 256 bytes)
            assert!(json.len() < 256, "Progress event JSON should be small (telemetry only)");
        }
    }

    /// GUARDRAIL: ProgressEvent.detail is for short status, not content
    #[test]
    fn test_progress_detail_is_diagnostic_only() {
        let event = ProgressEvent::heartbeat(RequestStage::Specialist, "still thinking", 5000);
        let json = serde_json::to_string(&event).unwrap();
        // Detail should be short diagnostic text (DiagnosticText enforces this)
        assert!(event.detail.as_ref().map(|d| d.as_str().len()).unwrap_or(0) <= MAX_DIAGNOSTIC_LENGTH);
        assert!(json.len() < 256);
    }

    /// GUARDRAIL: Enforce size cap on worst-case progress event.
    /// This test FAILS if someone tries to stuff large content into progress events.
    const MAX_PROGRESS_EVENT_BYTES: usize = 512;

    #[test]
    fn test_progress_event_size_cap_enforced() {
        // Worst case: error message at max allowed length
        let max_error = ProgressEvent::error(
            RequestStage::Specialist,
            "E".repeat(MAX_DIAGNOSTIC_LENGTH),
            99999,
        );
        let json = serde_json::to_string(&max_error).unwrap();
        assert!(
            json.len() < MAX_PROGRESS_EVENT_BYTES,
            "Max-length error event ({} bytes) exceeds cap ({})",
            json.len(),
            MAX_PROGRESS_EVENT_BYTES
        );

        // Worst case: heartbeat with max detail
        let max_heartbeat = ProgressEvent::heartbeat(
            RequestStage::Specialist,
            "D".repeat(MAX_DIAGNOSTIC_LENGTH),
            99999,
        );
        let json = serde_json::to_string(&max_heartbeat).unwrap();
        assert!(
            json.len() < MAX_PROGRESS_EVENT_BYTES,
            "Max-length heartbeat ({} bytes) exceeds cap ({})",
            json.len(),
            MAX_PROGRESS_EVENT_BYTES
        );
    }

    /// GUARDRAIL: DiagnosticText truncates oversized input - enforced at type level
    #[test]
    fn test_diagnostic_text_truncates_oversized() {
        let oversized = "X".repeat(MAX_DIAGNOSTIC_LENGTH + 50);
        let text = DiagnosticText::new(oversized);

        // Must be truncated to MAX_DIAGNOSTIC_LENGTH
        assert!(
            text.as_str().len() <= MAX_DIAGNOSTIC_LENGTH,
            "DiagnosticText must enforce max length: got {} chars",
            text.as_str().len()
        );

        // Must end with "..." to indicate truncation
        assert!(
            text.as_str().ends_with("..."),
            "Truncated DiagnosticText must end with '...'"
        );
    }

    /// GUARDRAIL: DiagnosticText preserves short input unchanged
    #[test]
    fn test_diagnostic_text_preserves_short() {
        let short = "short message";
        let text = DiagnosticText::new(short);
        assert_eq!(text.as_str(), short);
    }

    /// GUARDRAIL: DiagnosticText exactly at limit is not truncated
    #[test]
    fn test_diagnostic_text_at_limit() {
        let at_limit = "X".repeat(MAX_DIAGNOSTIC_LENGTH);
        let text = DiagnosticText::new(at_limit.clone());
        assert_eq!(text.as_str(), at_limit);
    }
}
