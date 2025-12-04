//! Case Engine v0.0.55 - Deterministic state machine for request processing
//!
//! 10-phase lifecycle:
//! 1. Intake - Receive and validate request
//! 2. Triage - Classify intent (SYSTEM_QUERY, DIAGNOSE, ACTION_REQUEST, HOWTO, META)
//! 3. DoctorSelect - Select doctor for DIAGNOSE intents
//! 4. EvidencePlan - Plan which evidence to collect
//! 5. EvidenceGather - Collect evidence using tools
//! 6. SynthesisDraft - Generate answer draft
//! 7. JuniorVerify - Verify with Junior model
//! 8. Respond - Send response to user
//! 9. RecordCase - Persist case file
//! 10. LearnRecipe - Extract recipe if appropriate

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::doctor_registry::DoctorDomain;

// ============================================================================
// Phase Enum
// ============================================================================

/// The 10 phases of the Case Engine lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CasePhase {
    /// Phase 1: Receive and validate the user request
    Intake,
    /// Phase 2: Classify intent using translator
    Triage,
    /// Phase 3: Select appropriate doctor (for DIAGNOSE intent)
    DoctorSelect,
    /// Phase 4: Plan which evidence to collect
    EvidencePlan,
    /// Phase 5: Execute tool calls to gather evidence
    EvidenceGather,
    /// Phase 6: Generate answer draft from evidence
    SynthesisDraft,
    /// Phase 7: Verify answer with Junior model
    JuniorVerify,
    /// Phase 8: Send response to user
    Respond,
    /// Phase 9: Persist case file to disk
    RecordCase,
    /// Phase 10: Extract recipe if reliability >= 80%
    LearnRecipe,
}

impl CasePhase {
    /// Get the next phase in the normal flow
    pub fn next(&self) -> Option<CasePhase> {
        match self {
            CasePhase::Intake => Some(CasePhase::Triage),
            CasePhase::Triage => Some(CasePhase::DoctorSelect),
            CasePhase::DoctorSelect => Some(CasePhase::EvidencePlan),
            CasePhase::EvidencePlan => Some(CasePhase::EvidenceGather),
            CasePhase::EvidenceGather => Some(CasePhase::SynthesisDraft),
            CasePhase::SynthesisDraft => Some(CasePhase::JuniorVerify),
            CasePhase::JuniorVerify => Some(CasePhase::Respond),
            CasePhase::Respond => Some(CasePhase::RecordCase),
            CasePhase::RecordCase => Some(CasePhase::LearnRecipe),
            CasePhase::LearnRecipe => None, // Terminal phase
        }
    }

    /// Is this a terminal phase?
    pub fn is_terminal(&self) -> bool {
        matches!(self, CasePhase::LearnRecipe)
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CasePhase::Intake => "Receiving request",
            CasePhase::Triage => "Classifying intent",
            CasePhase::DoctorSelect => "Selecting doctor",
            CasePhase::EvidencePlan => "Planning evidence collection",
            CasePhase::EvidenceGather => "Gathering evidence",
            CasePhase::SynthesisDraft => "Drafting response",
            CasePhase::JuniorVerify => "Verifying response",
            CasePhase::Respond => "Responding",
            CasePhase::RecordCase => "Recording case",
            CasePhase::LearnRecipe => "Learning recipe",
        }
    }
}

impl std::fmt::Display for CasePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

// ============================================================================
// Intent Taxonomy
// ============================================================================

/// Canonical intent types - the 5 categories from v0.0.55 spec
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentType {
    /// Questions about the system (CPU, disk, kernel, etc.)
    SystemQuery,
    /// Problem diagnosis - "X not working", "fix Y"
    Diagnose,
    /// Mutation requests - "install X", "edit file Y"
    ActionRequest,
    /// How-to questions - "how do I..."
    Howto,
    /// Meta queries - "what can you do", "show status"
    Meta,
}

impl IntentType {
    /// Parse from string (case-insensitive)
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('_', "").as_str() {
            "systemquery" | "query" | "system" => Some(IntentType::SystemQuery),
            "diagnose" | "diagnostic" | "problem" | "troubleshoot" => Some(IntentType::Diagnose),
            "actionrequest" | "action" | "mutation" | "command" => Some(IntentType::ActionRequest),
            "howto" | "how" | "help" | "explain" => Some(IntentType::Howto),
            "meta" | "status" | "about" | "introspection" => Some(IntentType::Meta),
            _ => None,
        }
    }

    /// Get the typical phase flow for this intent type
    /// (Some intents skip certain phases)
    pub fn skips_doctor_select(&self) -> bool {
        // Only DIAGNOSE goes through DoctorSelect
        !matches!(self, IntentType::Diagnose)
    }

    /// Whether this intent requires evidence collection
    pub fn requires_evidence(&self) -> bool {
        matches!(
            self,
            IntentType::SystemQuery | IntentType::Diagnose | IntentType::ActionRequest
        )
    }
}

impl std::fmt::Display for IntentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentType::SystemQuery => write!(f, "SYSTEM_QUERY"),
            IntentType::Diagnose => write!(f, "DIAGNOSE"),
            IntentType::ActionRequest => write!(f, "ACTION_REQUEST"),
            IntentType::Howto => write!(f, "HOWTO"),
            IntentType::Meta => write!(f, "META"),
        }
    }
}

// ============================================================================
// Case Event (Structured events for transcript)
// ============================================================================

/// Structured event in a case (for transcript)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseEvent {
    /// Unique event ID within the case (E1, E2, etc.)
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Phase when this event occurred
    pub phase: CasePhase,
    /// Actor who generated this event
    pub actor: CaseActor,
    /// Event type
    pub event_type: CaseEventType,
    /// Short description
    pub summary: String,
    /// Detailed data (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Duration if this was a timed operation (ms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Actors in the case engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaseActor {
    User,
    Anna,
    Translator,
    Junior,
    Senior,
    Doctor,
    Engine,
}

impl std::fmt::Display for CaseActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaseActor::User => write!(f, "you"),
            CaseActor::Anna => write!(f, "anna"),
            CaseActor::Translator => write!(f, "translator"),
            CaseActor::Junior => write!(f, "junior"),
            CaseActor::Senior => write!(f, "senior"),
            CaseActor::Doctor => write!(f, "doctor"),
            CaseActor::Engine => write!(f, "engine"),
        }
    }
}

/// Types of events that can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseEventType {
    /// Request received
    RequestReceived,
    /// Intent classified
    IntentClassified,
    /// Doctor selected
    DoctorSelected,
    /// Evidence plan created
    EvidencePlanned,
    /// Tool executed
    ToolExecuted,
    /// Evidence collected
    EvidenceCollected,
    /// Answer drafted
    AnswerDrafted,
    /// Verification result
    VerificationResult,
    /// Response sent
    ResponseSent,
    /// Case recorded
    CaseRecorded,
    /// Recipe extracted
    RecipeExtracted,
    /// Error occurred
    Error,
    /// Phase transition
    PhaseTransition,
}

// ============================================================================
// Case State
// ============================================================================

/// Current state of a case being processed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseState {
    /// Unique case ID
    pub case_id: String,
    /// User's original request
    pub request: String,
    /// Current phase
    pub phase: CasePhase,
    /// Classified intent (set after Triage)
    pub intent: Option<IntentType>,
    /// Selected doctor domain (set after DoctorSelect for DIAGNOSE)
    pub doctor_domain: Option<DoctorDomain>,
    /// Selected doctor ID
    pub doctor_id: Option<String>,
    /// Evidence IDs collected
    pub evidence_ids: Vec<String>,
    /// Tool results (tool_name -> result)
    pub tool_results: HashMap<String, serde_json::Value>,
    /// Draft answer (set after SynthesisDraft)
    pub draft_answer: Option<String>,
    /// Reliability score (0-100, set after JuniorVerify)
    pub reliability_score: Option<u8>,
    /// Final answer (set after Respond)
    pub final_answer: Option<String>,
    /// Events timeline
    pub events: Vec<CaseEvent>,
    /// Phase timings
    pub phase_timings: HashMap<CasePhase, PhaseTiming>,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// End time (set when complete)
    pub ended_at: Option<DateTime<Utc>>,
    /// Whether the case completed successfully
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Timing for a phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTiming {
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
}

impl CaseState {
    /// Create a new case state
    pub fn new(case_id: &str, request: &str) -> Self {
        let now = Utc::now();
        let mut state = Self {
            case_id: case_id.to_string(),
            request: request.to_string(),
            phase: CasePhase::Intake,
            intent: None,
            doctor_domain: None,
            doctor_id: None,
            evidence_ids: Vec::new(),
            tool_results: HashMap::new(),
            draft_answer: None,
            reliability_score: None,
            final_answer: None,
            events: Vec::new(),
            phase_timings: HashMap::new(),
            started_at: now,
            ended_at: None,
            success: false,
            error: None,
        };
        // Start timing for Intake phase
        state.phase_timings.insert(CasePhase::Intake, PhaseTiming {
            started_at: now,
            ended_at: None,
            duration_ms: None,
        });
        state
    }

    /// Get next event ID
    pub fn next_event_id(&self) -> String {
        format!("E{}", self.events.len() + 1)
    }

    /// Add an event
    pub fn add_event(&mut self, actor: CaseActor, event_type: CaseEventType, summary: &str) {
        let event = CaseEvent {
            id: self.next_event_id(),
            timestamp: Utc::now(),
            phase: self.phase,
            actor,
            event_type,
            summary: summary.to_string(),
            data: None,
            duration_ms: None,
        };
        self.events.push(event);
    }

    /// Add an event with data
    pub fn add_event_with_data(
        &mut self,
        actor: CaseActor,
        event_type: CaseEventType,
        summary: &str,
        data: serde_json::Value,
    ) {
        let event = CaseEvent {
            id: self.next_event_id(),
            timestamp: Utc::now(),
            phase: self.phase,
            actor,
            event_type,
            summary: summary.to_string(),
            data: Some(data),
            duration_ms: None,
        };
        self.events.push(event);
    }

    /// Transition to next phase
    pub fn transition_to(&mut self, next_phase: CasePhase) {
        let now = Utc::now();

        // End timing for current phase
        if let Some(timing) = self.phase_timings.get_mut(&self.phase) {
            timing.ended_at = Some(now);
            timing.duration_ms = Some(
                (now - timing.started_at).num_milliseconds().max(0) as u64
            );
        }

        // Start timing for next phase
        self.phase_timings.insert(next_phase, PhaseTiming {
            started_at: now,
            ended_at: None,
            duration_ms: None,
        });

        // Record transition event
        self.add_event(
            CaseActor::Engine,
            CaseEventType::PhaseTransition,
            &format!("{} -> {}", self.phase, next_phase),
        );

        self.phase = next_phase;
    }

    /// Skip to a phase (used when skipping DoctorSelect for non-DIAGNOSE intents)
    pub fn skip_to(&mut self, target_phase: CasePhase) {
        self.transition_to(target_phase);
    }

    /// Mark case as complete
    pub fn complete(&mut self, success: bool, error: Option<String>) {
        let now = Utc::now();
        self.ended_at = Some(now);
        self.success = success;
        self.error = error;

        // End timing for current phase
        if let Some(timing) = self.phase_timings.get_mut(&self.phase) {
            timing.ended_at = Some(now);
            timing.duration_ms = Some(
                (now - timing.started_at).num_milliseconds().max(0) as u64
            );
        }
    }

    /// Get total duration in milliseconds
    pub fn total_duration_ms(&self) -> u64 {
        let end = self.ended_at.unwrap_or_else(Utc::now);
        (end - self.started_at).num_milliseconds().max(0) as u64
    }

    /// Check if the case should learn a recipe
    pub fn should_learn_recipe(&self) -> bool {
        // Gate: reliability >= 80%, success, and >= 2 evidence items
        self.success
            && self.reliability_score.map(|r| r >= 80).unwrap_or(false)
            && self.evidence_ids.len() >= 2
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_transitions() {
        assert_eq!(CasePhase::Intake.next(), Some(CasePhase::Triage));
        assert_eq!(CasePhase::Triage.next(), Some(CasePhase::DoctorSelect));
        assert_eq!(CasePhase::LearnRecipe.next(), None);
        assert!(CasePhase::LearnRecipe.is_terminal());
    }

    #[test]
    fn test_intent_parsing() {
        assert_eq!(IntentType::parse("system_query"), Some(IntentType::SystemQuery));
        assert_eq!(IntentType::parse("DIAGNOSE"), Some(IntentType::Diagnose));
        assert_eq!(IntentType::parse("action"), Some(IntentType::ActionRequest));
        assert_eq!(IntentType::parse("howto"), Some(IntentType::Howto));
        assert_eq!(IntentType::parse("meta"), Some(IntentType::Meta));
        assert_eq!(IntentType::parse("unknown"), None);
    }

    #[test]
    fn test_intent_doctor_skip() {
        assert!(IntentType::SystemQuery.skips_doctor_select());
        assert!(!IntentType::Diagnose.skips_doctor_select());
        assert!(IntentType::Howto.skips_doctor_select());
    }

    #[test]
    fn test_case_state_creation() {
        let state = CaseState::new("test-123", "what cpu do I have");
        assert_eq!(state.case_id, "test-123");
        assert_eq!(state.phase, CasePhase::Intake);
        assert!(state.intent.is_none());
        assert!(state.events.is_empty());
    }

    #[test]
    fn test_case_state_transitions() {
        let mut state = CaseState::new("test-456", "test request");
        state.transition_to(CasePhase::Triage);
        assert_eq!(state.phase, CasePhase::Triage);
        assert_eq!(state.events.len(), 1);
        assert!(state.phase_timings.contains_key(&CasePhase::Intake));
        assert!(state.phase_timings.contains_key(&CasePhase::Triage));
    }

    #[test]
    fn test_case_state_events() {
        let mut state = CaseState::new("test-789", "test request");
        state.add_event(CaseActor::User, CaseEventType::RequestReceived, "test request");
        assert_eq!(state.events.len(), 1);
        assert_eq!(state.events[0].id, "E1");
        assert_eq!(state.next_event_id(), "E2");
    }

    #[test]
    fn test_should_learn_recipe() {
        let mut state = CaseState::new("test-learn", "test request");
        state.success = true;
        state.reliability_score = Some(85);
        state.evidence_ids = vec!["E1".to_string(), "E2".to_string()];
        assert!(state.should_learn_recipe());

        state.reliability_score = Some(75);
        assert!(!state.should_learn_recipe());

        state.reliability_score = Some(85);
        state.evidence_ids = vec!["E1".to_string()];
        assert!(!state.should_learn_recipe());
    }
}
