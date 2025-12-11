//! Hollywood UX types (v0.0.431).
//!
//! Extended transcript types for the unified renderer.

use crate::transcript_segment::{Actor, Transcript, TranscriptMode, TranscriptSegment};
use serde::{Deserialize, Serialize};

/// Extended transcript with Hollywood UX metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HollywoodTranscript {
    /// Core transcript
    #[serde(flatten)]
    pub inner: Transcript,
    /// User's original query
    pub user_query: String,
    /// Final answer text (extracted for quick access)
    pub final_answer: Option<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: Option<f32>,
    /// Staff member who handled the request
    pub handled_by: Option<String>,
    /// Department that handled the request
    pub department: Option<String>,
    /// Evidence sources used
    pub evidence_sources: Vec<String>,
    /// Total processing time (ms)
    pub processing_time_ms: u64,
    /// Outcome status
    pub outcome: TranscriptOutcome,
}

/// Transcript outcome status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptOutcome {
    /// Successfully answered
    #[default]
    Success,
    /// Partially answered
    Partial,
    /// Failed to answer
    Failed,
    /// Parse error from LLM
    ParseError,
    /// User cancelled
    Cancelled,
}

impl HollywoodTranscript {
    /// Create new transcript
    pub fn new(request_id: &str, user_query: &str) -> Self {
        Self {
            inner: Transcript::new(request_id),
            user_query: user_query.to_string(),
            final_answer: None,
            confidence: None,
            handled_by: None,
            department: None,
            evidence_sources: Vec::new(),
            processing_time_ms: 0,
            outcome: TranscriptOutcome::Success,
        }
    }

    /// Set mode
    pub fn with_mode(mut self, mode: TranscriptMode) -> Self {
        self.inner.mode = mode;
        self
    }

    /// Add segment
    pub fn add(&mut self, segment: TranscriptSegment) {
        self.inner.add(segment);
    }

    /// Add user input as first segment
    pub fn add_user_input(&mut self, query: &str) {
        self.inner.add_user_input(query);
    }

    /// Set final answer
    pub fn set_answer(&mut self, answer: &str) {
        self.final_answer = Some(answer.to_string());
        self.add(TranscriptSegment::answer(answer));
    }

    /// Set error
    pub fn set_error(&mut self, error: &str) {
        self.outcome = TranscriptOutcome::Failed;
        self.add(TranscriptSegment::error(error));
    }

    /// Set parse error
    pub fn set_parse_error(&mut self, error: &str) {
        self.outcome = TranscriptOutcome::ParseError;
        self.add(TranscriptSegment::error(error));
    }

    /// Set handler info
    pub fn set_handler(&mut self, staff: &str, department: &str) {
        self.handled_by = Some(staff.to_string());
        self.department = Some(department.to_string());
    }

    /// Set confidence
    pub fn set_confidence(&mut self, confidence: f32) {
        self.confidence = Some(confidence.clamp(0.0, 1.0));
    }

    /// Add evidence source
    pub fn add_evidence(&mut self, source: &str) {
        if !self.evidence_sources.contains(&source.to_string()) {
            self.evidence_sources.push(source.to_string());
        }
    }

    /// Finalize processing time
    pub fn finalize(&mut self) {
        self.processing_time_ms = self.inner.elapsed_secs() as u64 * 1000;
    }

    /// Get mode
    pub fn mode(&self) -> TranscriptMode {
        self.inner.mode
    }

    /// Get all segments
    pub fn segments(&self) -> &[TranscriptSegment] {
        &self.inner.segments
    }

    /// Check if successful
    pub fn is_success(&self) -> bool {
        matches!(self.outcome, TranscriptOutcome::Success)
    }
}

/// Probe result for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    /// Probe name/ID
    pub name: String,
    /// Command run (for debug)
    pub command: Option<String>,
    /// Status (ok, failed, timeout)
    pub status: ProbeStatus,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Summary output (truncated)
    pub summary: Option<String>,
    /// Raw output (for debug)
    pub raw_output: Option<String>,
}

/// Probe status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    Ok,
    Failed,
    Timeout,
    Skipped,
}

impl ProbeStatus {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::Timeout => "timeout",
            Self::Skipped => "skipped",
        }
    }
}

impl ProbeResult {
    pub fn ok(name: &str, duration_ms: u64) -> Self {
        Self {
            name: name.to_string(),
            command: None,
            status: ProbeStatus::Ok,
            duration_ms,
            summary: None,
            raw_output: None,
        }
    }

    pub fn with_summary(mut self, summary: &str) -> Self {
        self.summary = Some(summary.to_string());
        self
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.command = Some(cmd.to_string());
        self
    }

    pub fn with_raw(mut self, raw: &str) -> Self {
        self.raw_output = Some(raw.to_string());
        self
    }
}

/// Internal communication entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalComm {
    /// Relative time since request start
    pub relative_secs: f32,
    /// Staff member
    pub staff: String,
    /// Role/title
    pub role: Option<String>,
    /// Message content
    pub message: String,
}

impl InternalComm {
    pub fn new(staff: &str, role: Option<&str>, message: &str, relative_secs: f32) -> Self {
        Self {
            relative_secs,
            staff: staff.to_string(),
            role: role.map(|s| s.to_string()),
            message: message.to_string(),
        }
    }

    pub fn from_actor(actor: &Actor, message: &str, relative_secs: f32) -> Self {
        Self {
            relative_secs,
            staff: actor.name.clone(),
            role: actor.role.clone(),
            message: message.to_string(),
        }
    }

    /// Format staff name with role
    pub fn staff_display(&self) -> String {
        match &self.role {
            Some(role) => format!("{} ({})", self.staff, role),
            None => self.staff.clone(),
        }
    }
}

/// Render options for the Hollywood renderer
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Display mode
    pub mode: TranscriptMode,
    /// Terminal width
    pub width: usize,
    /// Show internal comms section
    pub show_internal_comms: bool,
    /// Show probes section
    pub show_probes: bool,
    /// Show timestamps in internal comms
    pub show_timestamps: bool,
    /// Show status footer
    pub show_footer: bool,
    /// Show evidence summary
    pub show_evidence: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            mode: TranscriptMode::Cinematic,
            width: super::DEFAULT_WIDTH,
            show_internal_comms: true,
            show_probes: true,
            show_timestamps: true,
            show_footer: true,
            show_evidence: true,
        }
    }
}

impl RenderOptions {
    pub fn cinematic() -> Self {
        Self::default()
    }

    pub fn debug() -> Self {
        Self {
            mode: TranscriptMode::Debug,
            ..Self::default()
        }
    }

    pub fn minimal() -> Self {
        Self {
            show_internal_comms: false,
            show_probes: false,
            show_timestamps: false,
            show_footer: false,
            show_evidence: true,
            ..Self::default()
        }
    }

    pub fn is_debug(&self) -> bool {
        matches!(self.mode, TranscriptMode::Debug)
    }
}
