//! Case File Schema v1 (v0.0.57) - Canonical case persistence format
//!
//! Complete audit trail for every request, stored atomically.
//! Structure: /var/lib/anna/cases/<case_id>/
//!   - case.json       (primary: full case data)
//!   - summary.txt     (human-readable summary)
//!   - transcript.log  (readable transcript)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::atomic_write::atomic_write_bytes;
use crate::case_engine::{CaseActor, CaseEvent, CasePhase, IntentType, PhaseTiming};
use crate::doctor_registry::DoctorDomain;
use crate::redaction::redact_transcript;

/// Schema version for case files
pub const CASE_SCHEMA_VERSION: u32 = 1;

/// Base directory for case files
pub const CASE_FILES_DIR: &str = "/var/lib/anna/cases";

// ============================================================================
// Case File V1 Schema
// ============================================================================

/// Complete case file (case.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseFileV1 {
    /// Schema version
    pub schema_version: u32,
    /// Unique case ID
    pub case_id: String,
    /// User's original request
    pub request: String,
    /// When the case started
    pub started_at: DateTime<Utc>,
    /// When the case ended
    pub ended_at: Option<DateTime<Utc>>,
    /// Total duration in milliseconds
    pub duration_ms: u64,

    // Intent and routing
    /// Classified intent type
    pub intent: IntentType,
    /// Intent classification confidence (0-100)
    pub intent_confidence: u8,
    /// Query target for SYSTEM_QUERY (e.g., "cpu", "disk_free")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_target: Option<String>,
    /// Problem domain for DIAGNOSE
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_domain: Option<String>,

    // Doctor information (for DIAGNOSE)
    /// Doctor domain selected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctor_domain: Option<DoctorDomain>,
    /// Doctor ID selected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctor_id: Option<String>,
    /// Doctor selection confidence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctor_confidence: Option<u32>,

    // Evidence
    /// Evidence collected (ID -> summary)
    pub evidence: Vec<EvidenceRecordV1>,
    /// Total evidence count
    pub evidence_count: usize,

    // Response
    /// Draft answer from synthesis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_answer: Option<String>,
    /// Final answer sent to user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_answer: Option<String>,
    /// Reliability score (0-100)
    pub reliability_score: u8,

    // Outcome
    /// Whether the case succeeded
    pub success: bool,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    // Timeline
    /// Phase timings
    pub phase_timings: Vec<PhaseTimingRecord>,
    /// All events
    pub events: Vec<CaseEvent>,

    // Learning
    /// Whether a recipe was extracted
    pub recipe_extracted: bool,
    /// Recipe ID if extracted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    /// XP gained
    pub xp_gained: u64,

    // Evidence coverage (v0.0.57)
    /// Evidence coverage percentage (0-100)
    #[serde(default)]
    pub evidence_coverage_percent: u8,
    /// Missing evidence fields
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_evidence_fields: Vec<String>,
    /// Whether evidence retry was triggered
    #[serde(default)]
    pub evidence_retry_triggered: bool,
}

/// Evidence record in case file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecordV1 {
    /// Evidence ID (E1, E2, etc.)
    pub id: String,
    /// Tool that produced this evidence
    pub tool_name: String,
    /// Summary of the evidence
    pub summary: String,
    /// When collected
    pub timestamp: DateTime<Utc>,
    /// Duration to collect (ms)
    pub duration_ms: u64,
    /// Whether this evidence is sensitive
    pub restricted: bool,
}

/// Phase timing record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTimingRecord {
    pub phase: CasePhase,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
}

impl From<(CasePhase, &PhaseTiming)> for PhaseTimingRecord {
    fn from((phase, timing): (CasePhase, &PhaseTiming)) -> Self {
        Self {
            phase,
            started_at: timing.started_at,
            ended_at: timing.ended_at,
            duration_ms: timing.duration_ms,
        }
    }
}

// ============================================================================
// Case File Builder
// ============================================================================

impl CaseFileV1 {
    /// Create a new case file
    pub fn new(case_id: &str, request: &str) -> Self {
        Self {
            schema_version: CASE_SCHEMA_VERSION,
            case_id: case_id.to_string(),
            request: request.to_string(),
            started_at: Utc::now(),
            ended_at: None,
            duration_ms: 0,
            intent: IntentType::SystemQuery,
            intent_confidence: 0,
            query_target: None,
            problem_domain: None,
            doctor_domain: None,
            doctor_id: None,
            doctor_confidence: None,
            evidence: Vec::new(),
            evidence_count: 0,
            draft_answer: None,
            final_answer: None,
            reliability_score: 0,
            success: false,
            error: None,
            phase_timings: Vec::new(),
            events: Vec::new(),
            recipe_extracted: false,
            recipe_id: None,
            xp_gained: 0,
            evidence_coverage_percent: 0,
            missing_evidence_fields: Vec::new(),
            evidence_retry_triggered: false,
        }
    }

    /// Set evidence coverage (v0.0.57)
    pub fn set_evidence_coverage(&mut self, percent: u8, missing: Vec<String>, retry_triggered: bool) {
        self.evidence_coverage_percent = percent;
        self.missing_evidence_fields = missing;
        self.evidence_retry_triggered = retry_triggered;
    }

    /// Set intent classification
    pub fn set_intent(&mut self, intent: IntentType, confidence: u8) {
        self.intent = intent;
        self.intent_confidence = confidence;
    }

    /// Set query target
    pub fn set_query_target(&mut self, target: &str) {
        self.query_target = Some(target.to_string());
    }

    /// Set problem domain
    pub fn set_problem_domain(&mut self, domain: &str) {
        self.problem_domain = Some(domain.to_string());
    }

    /// Set doctor selection
    pub fn set_doctor(&mut self, domain: DoctorDomain, id: &str, confidence: u32) {
        self.doctor_domain = Some(domain);
        self.doctor_id = Some(id.to_string());
        self.doctor_confidence = Some(confidence);
    }

    /// Add evidence
    pub fn add_evidence(&mut self, id: &str, tool_name: &str, summary: &str, duration_ms: u64, restricted: bool) {
        self.evidence.push(EvidenceRecordV1 {
            id: id.to_string(),
            tool_name: tool_name.to_string(),
            summary: summary.to_string(),
            timestamp: Utc::now(),
            duration_ms,
            restricted,
        });
        self.evidence_count = self.evidence.len();
    }

    /// Set draft answer
    pub fn set_draft(&mut self, draft: &str) {
        self.draft_answer = Some(draft.to_string());
    }

    /// Set final answer and reliability
    pub fn set_response(&mut self, answer: &str, reliability: u8) {
        self.final_answer = Some(answer.to_string());
        self.reliability_score = reliability;
    }

    /// Add events from CaseState
    pub fn set_events(&mut self, events: Vec<CaseEvent>) {
        self.events = events;
    }

    /// Set phase timings
    pub fn set_phase_timings(&mut self, timings: impl IntoIterator<Item = (CasePhase, PhaseTiming)>) {
        self.phase_timings = timings
            .into_iter()
            .map(|(phase, timing)| PhaseTimingRecord::from((phase, &timing)))
            .collect();
    }

    /// Set learning outcome
    pub fn set_learning(&mut self, recipe_id: Option<&str>, xp_gained: u64) {
        self.recipe_extracted = recipe_id.is_some();
        self.recipe_id = recipe_id.map(|s| s.to_string());
        self.xp_gained = xp_gained;
    }

    /// Complete the case
    pub fn complete(&mut self, success: bool, error: Option<&str>) {
        self.ended_at = Some(Utc::now());
        self.duration_ms = (self.ended_at.unwrap() - self.started_at).num_milliseconds().max(0) as u64;
        self.success = success;
        self.error = error.map(|s| s.to_string());
    }

    /// Get the directory path for this case
    pub fn get_dir(&self) -> PathBuf {
        PathBuf::from(format!("{}/{}", CASE_FILES_DIR, self.case_id))
    }

    /// Save case file atomically
    pub fn save(&self) -> io::Result<PathBuf> {
        let case_dir = self.get_dir();
        fs::create_dir_all(&case_dir)?;

        // Save case.json
        let case_json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let case_path = case_dir.join("case.json");
        atomic_write_bytes(&case_path.to_string_lossy(), redact_transcript(&case_json).as_bytes())?;

        // Save summary.txt
        let summary = self.render_summary();
        let summary_path = case_dir.join("summary.txt");
        atomic_write_bytes(&summary_path.to_string_lossy(), redact_transcript(&summary).as_bytes())?;

        // Save transcript.log
        let transcript = self.render_transcript();
        let transcript_path = case_dir.join("transcript.log");
        atomic_write_bytes(&transcript_path.to_string_lossy(), redact_transcript(&transcript).as_bytes())?;

        Ok(case_dir)
    }

    /// Render human-readable summary
    pub fn render_summary(&self) -> String {
        let mut lines = vec![
            format!("Case: {}", self.case_id),
            format!("Request: {}", self.request),
            format!("Intent: {} ({}%)", self.intent, self.intent_confidence),
        ];

        if let Some(target) = &self.query_target {
            lines.push(format!("Query Target: {}", target));
        }
        if let Some(domain) = &self.problem_domain {
            lines.push(format!("Problem Domain: {}", domain));
        }
        if let Some(doctor_id) = &self.doctor_id {
            lines.push(format!("Doctor: {} ({:?})", doctor_id, self.doctor_domain));
        }

        lines.push(format!("Evidence: {} items", self.evidence_count));
        lines.push(format!("Evidence Coverage: {}%", self.evidence_coverage_percent));
        if !self.missing_evidence_fields.is_empty() {
            lines.push(format!("Missing Fields: {}", self.missing_evidence_fields.join(", ")));
        }
        if self.evidence_retry_triggered {
            lines.push("Evidence Retry: Yes".to_string());
        }
        lines.push(format!("Reliability: {}%", self.reliability_score));
        lines.push(format!("Success: {}", self.success));
        lines.push(format!("Duration: {}ms", self.duration_ms));

        if self.recipe_extracted {
            lines.push(format!("Recipe: {}", self.recipe_id.as_deref().unwrap_or("unknown")));
        }
        if self.xp_gained > 0 {
            lines.push(format!("XP Gained: {}", self.xp_gained));
        }

        if let Some(error) = &self.error {
            lines.push(format!("Error: {}", error));
        }

        lines.join("\n")
    }

    /// Render readable transcript
    pub fn render_transcript(&self) -> String {
        let mut lines = Vec::new();

        for event in &self.events {
            let actor = format!("[{}]", event.actor);
            let phase = format!("[{}]", event.phase);
            lines.push(format!("{} {} {}: {}", actor, phase, event.event_type_str(), event.summary));
        }

        if lines.is_empty() {
            lines.push("(no events recorded)".to_string());
        }

        lines.join("\n")
    }
}

impl CaseEvent {
    fn event_type_str(&self) -> &'static str {
        match self.event_type {
            crate::case_engine::CaseEventType::RequestReceived => "request",
            crate::case_engine::CaseEventType::IntentClassified => "classified",
            crate::case_engine::CaseEventType::DoctorSelected => "doctor",
            crate::case_engine::CaseEventType::EvidencePlanned => "planned",
            crate::case_engine::CaseEventType::ToolExecuted => "tool",
            crate::case_engine::CaseEventType::EvidenceCollected => "evidence",
            crate::case_engine::CaseEventType::AnswerDrafted => "draft",
            crate::case_engine::CaseEventType::VerificationResult => "verified",
            crate::case_engine::CaseEventType::ResponseSent => "response",
            crate::case_engine::CaseEventType::CaseRecorded => "recorded",
            crate::case_engine::CaseEventType::RecipeExtracted => "recipe",
            crate::case_engine::CaseEventType::Error => "ERROR",
            crate::case_engine::CaseEventType::PhaseTransition => "->",
        }
    }
}

// ============================================================================
// Case File Loading
// ============================================================================

/// Load a case file from disk
pub fn load_case(case_id: &str) -> io::Result<CaseFileV1> {
    let case_path = PathBuf::from(format!("{}/{}/case.json", CASE_FILES_DIR, case_id));
    let content = fs::read_to_string(&case_path)?;
    serde_json::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// List recent case IDs
pub fn list_recent_case_ids(limit: usize) -> io::Result<Vec<String>> {
    let cases_dir = Path::new(CASE_FILES_DIR);
    if !cases_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = fs::read_dir(cases_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    // Sort by name descending (case IDs include timestamp)
    entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    let case_ids: Vec<String> = entries
        .into_iter()
        .take(limit)
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    Ok(case_ids)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_file_creation() {
        let case = CaseFileV1::new("test-123", "what cpu do I have");
        assert_eq!(case.case_id, "test-123");
        assert_eq!(case.request, "what cpu do I have");
        assert_eq!(case.schema_version, CASE_SCHEMA_VERSION);
    }

    #[test]
    fn test_case_file_intent() {
        let mut case = CaseFileV1::new("test-456", "test");
        case.set_intent(IntentType::Diagnose, 85);
        assert_eq!(case.intent, IntentType::Diagnose);
        assert_eq!(case.intent_confidence, 85);
    }

    #[test]
    fn test_case_file_evidence() {
        let mut case = CaseFileV1::new("test-789", "test");
        case.add_evidence("E1", "hw_snapshot_cpu", "AMD Ryzen 7", 50, false);
        case.add_evidence("E2", "memory_info", "16 GiB RAM", 30, false);
        assert_eq!(case.evidence_count, 2);
    }

    #[test]
    fn test_case_file_summary() {
        let mut case = CaseFileV1::new("test-summary", "what cpu");
        case.set_intent(IntentType::SystemQuery, 95);
        case.set_query_target("cpu");
        case.set_response("AMD Ryzen 7", 85);
        case.complete(true, None);

        let summary = case.render_summary();
        assert!(summary.contains("test-summary"));
        assert!(summary.contains("SYSTEM_QUERY"));
        assert!(summary.contains("cpu"));
        assert!(summary.contains("85%"));
    }
}
