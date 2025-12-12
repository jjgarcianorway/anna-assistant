//! Transcript Segment - Core data model for Hollywood IT view (v0.0.413).
//!
//! Defines the minimal, explicit structure that annad produces for each request,
//! which annactl then renders. All LLM output must be wrapped in TranscriptSegments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Display mode for transcript rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptMode {
    /// Hollywood IT department view - characterful, minimal noise
    #[default]
    Cinematic,
    /// Developer view - includes raw JSON, full errors
    Debug,
}

impl std::fmt::Display for TranscriptMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptMode::Cinematic => write!(f, "cinematic"),
            TranscriptMode::Debug => write!(f, "debug"),
        }
    }
}

impl std::str::FromStr for TranscriptMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cinematic" => Ok(TranscriptMode::Cinematic),
            "debug" => Ok(TranscriptMode::Debug),
            _ => Err(format!("Unknown mode: {}", s)),
        }
    }
}

/// Kind of transcript segment - determines rendering style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentKind {
    /// User's input query
    UserInput,
    /// System status info (translator done, routing complete, etc.)
    SystemInfo,
    /// Ticket created/assigned header
    TicketHeader,
    /// Internal IT department chatter
    InternalComms,
    /// Probe execution (started, completed, output)
    ProbeRun,
    /// Message from a specialist
    SpecialistMessage,
    /// Final answer to user
    Answer,
    /// Error message
    Error,
    /// Helpful tip or status update
    Tip,
    /// Stats snippet (success rate, tickets handled)
    StatsSnippet,
    /// Debug-only raw JSON dump
    DebugJson,
    /// Progress indicator (for streaming)
    Progress,
}

impl SegmentKind {
    /// Whether this segment should be shown in cinematic mode
    pub fn show_in_cinematic(&self) -> bool {
        !matches!(self, SegmentKind::DebugJson)
    }

    /// Whether this is an internal comms segment
    pub fn is_internal(&self) -> bool {
        matches!(
            self,
            SegmentKind::InternalComms | SegmentKind::ProbeRun | SegmentKind::SystemInfo
        )
    }
}

/// Actor in the IT department
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    /// Short name: "Anna", "Sofia", "Michael", etc.
    pub name: String,
    /// Role/title: "Desktop Administrator", "Network Engineer", etc.
    pub role: Option<String>,
}

impl Actor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            role: None,
        }
    }

    pub fn with_role(name: &str, role: &str) -> Self {
        Self {
            name: name.to_string(),
            role: Some(role.to_string()),
        }
    }

    /// User actor
    pub fn user() -> Self {
        Self::new("you")
    }

    /// Anna (front desk / coordinator)
    pub fn anna() -> Self {
        Self::new("Anna")
    }

    /// System actor
    pub fn system() -> Self {
        Self::new("System")
    }

    /// Format for display
    pub fn display(&self) -> String {
        match &self.role {
            Some(role) => format!("{} ({})", self.name, role),
            None => self.name.clone(),
        }
    }
}

/// Well-known staff members
pub mod staff {
    use super::Actor;

    pub fn sofia() -> Actor {
        Actor::with_role("Sofia", "Desktop Administrator")
    }
    pub fn michael() -> Actor {
        Actor::with_role("Michael", "Network Engineer")
    }
    pub fn lars() -> Actor {
        Actor::with_role("Lars", "Storage Engineer")
    }
    pub fn tomas() -> Actor {
        Actor::with_role("Tomas", "System Analyst")
    }
    pub fn hugo() -> Actor {
        Actor::with_role("Hugo", "Services Administrator")
    }
    pub fn elena() -> Actor {
        Actor::with_role("Elena", "Security Specialist")
    }
    pub fn marcus() -> Actor {
        Actor::with_role("Marcus", "Package Manager")
    }
}

/// A single segment of the transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    /// Kind of segment
    pub kind: SegmentKind,
    /// When this segment was created (Unix timestamp ms)
    pub timestamp_ms: u64,
    /// Relative time since request start (for display)
    pub relative_secs: f32,
    /// Who is speaking/acting
    pub actor: Actor,
    /// The content to display (already formatted)
    pub content: String,
    /// Optional metadata (ticket_id, confidence, domain, etc.)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, String>,
}

impl TranscriptSegment {
    /// Create a new segment
    pub fn new(kind: SegmentKind, actor: Actor, content: &str) -> Self {
        Self {
            kind,
            timestamp_ms: now_ms(),
            relative_secs: 0.0,
            actor,
            content: content.to_string(),
            meta: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_meta(mut self, key: &str, value: &str) -> Self {
        self.meta.insert(key.to_string(), value.to_string());
        self
    }

    /// Set relative time
    pub fn at_relative(mut self, secs: f32) -> Self {
        self.relative_secs = secs;
        self
    }

    // Convenience constructors

    /// User input segment
    pub fn user_input(query: &str) -> Self {
        Self::new(SegmentKind::UserInput, Actor::user(), query)
    }

    /// System info segment
    pub fn system_info(content: &str) -> Self {
        Self::new(SegmentKind::SystemInfo, Actor::system(), content)
    }

    /// Ticket header segment
    pub fn ticket_header(ticket_id: &str, domain: &str, summary: &str) -> Self {
        Self::new(
            SegmentKind::TicketHeader,
            Actor::anna(),
            &format!("Opening ticket {} - {}", ticket_id, summary),
        )
        .with_meta("ticket_id", ticket_id)
        .with_meta("domain", domain)
    }

    /// Internal comms from a staff member
    pub fn internal_comms(actor: Actor, message: &str) -> Self {
        Self::new(SegmentKind::InternalComms, actor, message)
    }

    /// Probe run segment
    pub fn probe_run(probe_id: &str, status: &str) -> Self {
        Self::new(SegmentKind::ProbeRun, Actor::system(), status).with_meta("probe_id", probe_id)
    }

    /// Specialist message
    pub fn specialist_message(actor: Actor, message: &str) -> Self {
        Self::new(SegmentKind::SpecialistMessage, actor, message)
    }

    /// Final answer
    pub fn answer(content: &str) -> Self {
        Self::new(SegmentKind::Answer, Actor::anna(), content)
    }

    /// Error segment
    pub fn error(message: &str) -> Self {
        Self::new(SegmentKind::Error, Actor::system(), message)
    }

    /// Tip segment
    pub fn tip(message: &str) -> Self {
        Self::new(SegmentKind::Tip, Actor::system(), message)
    }

    /// Stats snippet
    pub fn stats(content: &str) -> Self {
        Self::new(SegmentKind::StatsSnippet, Actor::system(), content)
    }

    /// Debug JSON dump (only shown in debug mode)
    pub fn debug_json(label: &str, json: &str) -> Self {
        Self::new(SegmentKind::DebugJson, Actor::system(), json).with_meta("label", label)
    }

    /// Progress indicator
    pub fn progress(message: &str) -> Self {
        Self::new(SegmentKind::Progress, Actor::system(), message)
    }
}

/// Full transcript for a request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transcript {
    /// Unique request ID
    pub request_id: String,
    /// Ticket ID (if created)
    pub ticket_id: Option<String>,
    /// Display mode
    pub mode: TranscriptMode,
    /// Ordered segments
    pub segments: Vec<TranscriptSegment>,
    /// Request start time (Unix timestamp ms)
    pub started_at_ms: u64,
}

impl Transcript {
    /// Create new transcript for a request
    pub fn new(request_id: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            ticket_id: None,
            mode: TranscriptMode::Cinematic,
            segments: Vec::new(),
            started_at_ms: now_ms(),
        }
    }

    /// Set mode
    pub fn with_mode(mut self, mode: TranscriptMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set ticket ID
    pub fn with_ticket(mut self, ticket_id: &str) -> Self {
        self.ticket_id = Some(ticket_id.to_string());
        self
    }

    /// Add a segment with auto relative time
    pub fn add(&mut self, mut segment: TranscriptSegment) {
        segment.relative_secs = self.elapsed_secs();
        self.segments.push(segment);
    }

    /// Add user input as first segment
    pub fn add_user_input(&mut self, query: &str) {
        self.add(TranscriptSegment::user_input(query));
    }

    /// Elapsed time since start
    pub fn elapsed_secs(&self) -> f32 {
        let now = now_ms();
        (now.saturating_sub(self.started_at_ms)) as f32 / 1000.0
    }

    /// Get all segments of a specific kind
    pub fn segments_of_kind(&self, kind: SegmentKind) -> Vec<&TranscriptSegment> {
        self.segments.iter().filter(|s| s.kind == kind).collect()
    }

    /// Get the final answer segment if present
    pub fn answer(&self) -> Option<&TranscriptSegment> {
        self.segments
            .iter()
            .rev()
            .find(|s| s.kind == SegmentKind::Answer)
    }

    /// Get all internal comms
    pub fn internal_comms(&self) -> Vec<&TranscriptSegment> {
        self.segments
            .iter()
            .filter(|s| s.kind == SegmentKind::InternalComms)
            .collect()
    }

    /// Get all probes run
    pub fn probes(&self) -> Vec<&TranscriptSegment> {
        self.segments_of_kind(SegmentKind::ProbeRun)
    }

    /// Check if there were errors
    pub fn has_errors(&self) -> bool {
        self.segments.iter().any(|s| s.kind == SegmentKind::Error)
    }
}

/// Get current time in milliseconds
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_building() {
        let mut t = Transcript::new("req-001");
        t.add_user_input("why is nginx failing?");
        t.add(TranscriptSegment::ticket_header(
            "SRV-001",
            "services",
            "nginx investigation",
        ));
        t.add(TranscriptSegment::internal_comms(
            staff::hugo(),
            "On it. Checking service status.",
        ));
        t.add(TranscriptSegment::answer(
            "nginx is not running due to config error",
        ));

        assert_eq!(t.segments.len(), 4);
        assert!(t.answer().is_some());
        assert_eq!(t.internal_comms().len(), 1);
    }

    #[test]
    fn test_actor_display() {
        assert_eq!(Actor::anna().display(), "Anna");
        assert_eq!(staff::sofia().display(), "Sofia (Desktop Administrator)");
        assert_eq!(staff::michael().display(), "Michael (Network Engineer)");
    }
}
