//! Case Lifecycle v0.0.66 - IT Department Case Management
//!
//! v0.0.66: Service Desk Dispatcher + Department Protocol
//! - Every request creates a case folder under /var/lib/anna/cases/<case_id>/
//! - case.json: Full case state with ticket_type, outcome, evidence_summaries
//! - human.log: Fly-on-the-wall dialogue (no tool names, no evidence IDs)
//! - debug.log: Full trace with evidence IDs, tool calls, timings
//!
//! v0.0.64: Enhanced with Service Desk ticketing and Doctor lifecycle tracking
//! - Ticket ID and category/severity from Service Desk
//! - Routing plan (which doctors were selected)
//! - Evidence topics requested
//! - Doctor lifecycle stages completed
//! - Coverage checks results
//!
//! Explicit stages that map to real IT operations:
//! new -> triaged -> investigating -> plan_ready -> awaiting_confirmation
//!     -> executing -> verifying -> resolved | abandoned
//!
//! Cases are the central organizing unit for all Anna interactions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::atomic_write::atomic_write_bytes;
use crate::doctor_lifecycle::DoctorLifecycleStage;
use crate::doctor_registry::DoctorDomain;
use crate::evidence_topic::EvidenceTopic;
use crate::proactive_alerts::AlertType;
use crate::redaction::redact_transcript;
use crate::service_desk::{TicketCategory, TicketSeverity};

/// Schema version for case files v2
pub const CASE_SCHEMA_VERSION_V2: u32 = 2;

/// Base directory for case files
pub const CASE_FILES_DIR: &str = "/var/lib/anna/cases";

// ============================================================================
// v0.0.66: Ticket Type and Case Outcome
// ============================================================================

/// Type of ticket/request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TicketType {
    /// User asking a question (informational)
    #[default]
    Question,
    /// User reporting an incident (something is broken)
    Incident,
    /// User requesting a change (install, modify, etc.)
    ChangeRequest,
}

impl TicketType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TicketType::Question => "question",
            TicketType::Incident => "incident",
            TicketType::ChangeRequest => "change_request",
        }
    }

    pub fn human_label(&self) -> &'static str {
        match self {
            TicketType::Question => "Question",
            TicketType::Incident => "Incident",
            TicketType::ChangeRequest => "Change Request",
        }
    }
}

impl std::fmt::Display for TicketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.human_label())
    }
}

/// Outcome of a case
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaseOutcome {
    /// Case not yet completed
    #[default]
    Pending,
    /// Question answered successfully
    Answered,
    /// User needs to confirm before proceeding
    NeedsConfirmation,
    /// Blocked by policy (not allowed)
    BlockedByPolicy,
    /// Not enough evidence to answer/diagnose
    InsufficientEvidence,
    /// User abandoned the case
    Abandoned,
}

impl CaseOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            CaseOutcome::Pending => "pending",
            CaseOutcome::Answered => "answered",
            CaseOutcome::NeedsConfirmation => "needs_confirmation",
            CaseOutcome::BlockedByPolicy => "blocked_by_policy",
            CaseOutcome::InsufficientEvidence => "insufficient_evidence",
            CaseOutcome::Abandoned => "abandoned",
        }
    }

    pub fn is_terminal(&self) -> bool {
        !matches!(self, CaseOutcome::Pending | CaseOutcome::NeedsConfirmation)
    }
}

impl std::fmt::Display for CaseOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Case Lifecycle Stages
// ============================================================================

/// Case lifecycle status - explicit IT incident workflow stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    /// Just created, not yet triaged
    New,
    /// Intent classified, department assigned
    Triaged,
    /// Actively gathering evidence and diagnosing
    Investigating,
    /// Diagnosis complete, action plan ready
    PlanReady,
    /// Mutations proposed, waiting for user confirmation
    AwaitingConfirmation,
    /// User confirmed, executing mutations
    Executing,
    /// Mutations executed, verifying results
    Verifying,
    /// Successfully completed
    Resolved,
    /// User abandoned or cannot proceed
    Abandoned,
}

impl CaseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CaseStatus::New => "new",
            CaseStatus::Triaged => "triaged",
            CaseStatus::Investigating => "investigating",
            CaseStatus::PlanReady => "plan_ready",
            CaseStatus::AwaitingConfirmation => "awaiting_confirmation",
            CaseStatus::Executing => "executing",
            CaseStatus::Verifying => "verifying",
            CaseStatus::Resolved => "resolved",
            CaseStatus::Abandoned => "abandoned",
        }
    }

    /// Check if case is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, CaseStatus::Resolved | CaseStatus::Abandoned)
    }

    /// Check if case is active (can progress)
    pub fn is_active(&self) -> bool {
        !self.is_terminal()
    }
}

impl std::fmt::Display for CaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Department Assignment
// ============================================================================

/// IT department for case routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Department {
    /// Initial triage and general queries
    ServiceDesk,
    /// Network, WiFi, DNS, routing issues
    Networking,
    /// Disk, filesystem, storage issues
    Storage,
    /// Boot time, startup, service regressions
    Boot,
    /// Audio, PipeWire, sound issues
    Audio,
    /// GPU, display, compositor issues
    Graphics,
    /// Security, permissions, access issues
    Security,
    /// Performance, resource usage issues
    Performance,
}

impl Department {
    pub fn as_str(&self) -> &'static str {
        match self {
            Department::ServiceDesk => "service_desk",
            Department::Networking => "networking",
            Department::Storage => "storage",
            Department::Boot => "boot",
            Department::Audio => "audio",
            Department::Graphics => "graphics",
            Department::Security => "security",
            Department::Performance => "performance",
        }
    }

    /// Get actor name for transcript display
    pub fn actor_name(&self) -> &'static str {
        match self {
            Department::ServiceDesk => "anna",
            Department::Networking => "networking",
            Department::Storage => "storage",
            Department::Boot => "boot",
            Department::Audio => "audio",
            Department::Graphics => "graphics",
            Department::Security => "security",
            Department::Performance => "performance",
        }
    }

    /// Map from DoctorDomain to Department
    pub fn from_doctor_domain(domain: DoctorDomain) -> Self {
        match domain {
            DoctorDomain::Network => Department::Networking,
            DoctorDomain::Storage => Department::Storage,
            DoctorDomain::Audio => Department::Audio,
            DoctorDomain::Boot => Department::Boot,
            DoctorDomain::Graphics => Department::Graphics,
            DoctorDomain::System => Department::ServiceDesk,
        }
    }

    /// Map from AlertType to Department
    pub fn from_alert_type(alert_type: AlertType) -> Self {
        match alert_type {
            AlertType::BootRegression => Department::Boot,
            AlertType::DiskPressure => Department::Storage,
            AlertType::JournalErrorBurst => Department::ServiceDesk,
            AlertType::ServiceFailed => Department::ServiceDesk,
            AlertType::ThermalThrottling => Department::Performance,
        }
    }
}

impl std::fmt::Display for Department {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Participants
// ============================================================================

/// Case participant (actor in the IT org)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Participant {
    /// The user
    You,
    /// Service desk lead (Anna)
    Anna,
    /// Intake analyst
    Translator,
    /// QA/reliability verifier
    Junior,
    /// Daemon/operator
    Annad,
    /// Specialist doctor by department
    Specialist(Department),
}

impl Participant {
    pub fn actor_name(&self) -> String {
        match self {
            Participant::You => "you".to_string(),
            Participant::Anna => "anna".to_string(),
            Participant::Translator => "translator".to_string(),
            Participant::Junior => "junior".to_string(),
            Participant::Annad => "annad".to_string(),
            Participant::Specialist(dept) => dept.actor_name().to_string(),
        }
    }
}

impl std::fmt::Display for Participant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.actor_name())
    }
}

// ============================================================================
// Proposed Actions
// ============================================================================

/// Risk level for proposed actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionRisk {
    /// Read-only, no confirmation needed
    ReadOnly,
    /// Low risk, reversible
    Low,
    /// Medium risk, needs explicit confirmation
    Medium,
    /// High risk, needs "I assume the risk"
    High,
}

/// A proposed action in the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedAction {
    /// Action ID (A1, A2, etc.)
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Risk level
    pub risk: ActionRisk,
    /// Tool to execute
    pub tool_name: String,
    /// Tool parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Evidence supporting this action
    pub evidence_ids: Vec<String>,
    /// Whether executed
    pub executed: bool,
    /// Execution result
    pub result: Option<String>,
}

// ============================================================================
// Timeline Events
// ============================================================================

/// Timeline event type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineEventType {
    /// Case created
    CaseOpened,
    /// Status changed
    StatusChange { from: CaseStatus, to: CaseStatus },
    /// Department assigned
    DepartmentAssigned { department: Department },
    /// Participant joined
    ParticipantJoined { participant: Participant },
    /// Evidence collected
    EvidenceCollected { evidence_id: String, tool: String },
    /// Finding recorded
    FindingRecorded { finding: String },
    /// Hypothesis proposed
    HypothesisProposed { hypothesis: String, confidence: u8 },
    /// Action proposed
    ActionProposed { action_id: String },
    /// Confirmation requested
    ConfirmationRequested { phrase: String },
    /// User confirmed
    UserConfirmed,
    /// User declined
    UserDeclined,
    /// Action executed
    ActionExecuted { action_id: String, success: bool },
    /// Verification completed
    VerificationCompleted { passed: bool },
    /// Alert linked
    AlertLinked {
        alert_id: String,
        alert_type: AlertType,
    },
    /// Error occurred
    Error { message: String },
    /// Case closed
    CaseClosed { success: bool },
}

/// Timeline event in the case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    /// When this event occurred
    pub timestamp: DateTime<Utc>,
    /// Who triggered this event
    pub actor: Participant,
    /// Event type and data
    pub event: TimelineEventType,
    /// Human-readable summary
    pub summary: String,
}

impl TimelineEvent {
    pub fn new(actor: Participant, event: TimelineEventType, summary: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            actor,
            event,
            summary: summary.to_string(),
        }
    }
}

// ============================================================================
// Case File V2 Schema
// ============================================================================

/// Case file v2 - complete case with IT workflow lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseFileV2 {
    /// Schema version (2)
    pub schema_version: u32,

    // Identity
    /// Stable case ID
    pub case_id: String,
    /// When created
    pub created_at: DateTime<Utc>,
    /// When last updated
    pub updated_at: DateTime<Utc>,
    /// Case title (from translator + summary)
    pub title: String,

    // Classification
    /// Original user request
    pub request: String,
    /// Intent type
    pub intent: String,
    /// Query targets
    pub targets: Vec<String>,
    /// Risk level
    pub risk: ActionRisk,

    // Routing
    /// Assigned department
    pub assigned_department: Department,
    /// Participants who contributed
    pub participants: Vec<Participant>,

    // v0.0.64: Service Desk Ticketing
    /// Service Desk ticket ID (format: A-YYYYMMDD-XXXX)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket_id: Option<String>,
    /// Ticket category
    #[serde(default)]
    pub ticket_category: Option<TicketCategory>,
    /// Ticket severity
    #[serde(default)]
    pub ticket_severity: Option<TicketSeverity>,
    /// Primary doctor name (if routed to doctor)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_doctor: Option<String>,
    /// Secondary doctors consulted
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_doctors: Vec<String>,
    /// Evidence topics requested
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_topics: Vec<EvidenceTopic>,
    /// Doctor lifecycle stages completed
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doctor_stages_completed: Vec<DoctorLifecycleStage>,
    /// Whether doctor flow was used (vs evidence topic router)
    #[serde(default)]
    pub used_doctor_flow: bool,

    // v0.0.66: Enhanced case tracking
    /// Type of ticket: question, incident, or change_request
    #[serde(default)]
    pub ticket_type: TicketType,
    /// Case outcome
    #[serde(default)]
    pub outcome: CaseOutcome,
    /// Human-readable transcript lines (no tool names, no evidence IDs)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub human_transcript: Vec<String>,
    /// Evidence summaries (human-readable descriptions)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_summaries: Vec<String>,
    /// Path to debug.log file (full trace)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_trace_path: Option<String>,

    // Evidence
    /// Evidence IDs collected
    pub evidence_ids: Vec<String>,
    /// Evidence count
    pub evidence_count: usize,
    /// Evidence coverage percentage
    pub coverage_pct: u8,

    // Diagnosis
    /// Key findings (bullets)
    pub findings: Vec<String>,
    /// Hypotheses with confidence
    pub hypotheses: Vec<(String, u8)>,

    // Plan
    /// Proposed actions
    pub actions_proposed: Vec<ProposedAction>,
    /// Whether confirmation is required
    pub confirmation_required: bool,
    /// Confirmation phrase (if required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_phrase: Option<String>,

    // Response
    /// Final answer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_answer: Option<String>,
    /// Reliability percentage
    pub reliability_pct: u8,

    // Lifecycle
    /// Current status
    pub status: CaseStatus,
    /// Timeline (append-only)
    pub timeline: Vec<TimelineEvent>,

    // Alerts
    /// Linked alert IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_alert_ids: Vec<String>,

    // Learning
    /// Recipe ID if extracted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    /// XP gained
    pub xp_gained: u64,
}

impl CaseFileV2 {
    /// Create a new case
    pub fn new(case_id: &str, request: &str) -> Self {
        let now = Utc::now();
        let mut case = Self {
            schema_version: CASE_SCHEMA_VERSION_V2,
            case_id: case_id.to_string(),
            created_at: now,
            updated_at: now,
            title: truncate_title(request, 60),
            request: request.to_string(),
            intent: "unknown".to_string(),
            targets: Vec::new(),
            risk: ActionRisk::ReadOnly,
            assigned_department: Department::ServiceDesk,
            participants: vec![Participant::You, Participant::Anna],
            // v0.0.64: Service Desk ticketing fields
            ticket_id: None,
            ticket_category: None,
            ticket_severity: None,
            primary_doctor: None,
            secondary_doctors: Vec::new(),
            evidence_topics: Vec::new(),
            doctor_stages_completed: Vec::new(),
            used_doctor_flow: false,
            // v0.0.66: Enhanced case tracking
            ticket_type: TicketType::Question,
            outcome: CaseOutcome::Pending,
            human_transcript: Vec::new(),
            evidence_summaries: Vec::new(),
            debug_trace_path: None,
            // Evidence
            evidence_ids: Vec::new(),
            evidence_count: 0,
            coverage_pct: 0,
            findings: Vec::new(),
            hypotheses: Vec::new(),
            actions_proposed: Vec::new(),
            confirmation_required: false,
            confirmation_phrase: None,
            final_answer: None,
            reliability_pct: 0,
            status: CaseStatus::New,
            timeline: Vec::new(),
            linked_alert_ids: Vec::new(),
            recipe_id: None,
            xp_gained: 0,
        };

        // Record opening event
        case.add_event(
            Participant::Anna,
            TimelineEventType::CaseOpened,
            &format!("Case opened: {}", truncate_title(request, 40)),
        );

        case
    }

    /// Add a timeline event
    pub fn add_event(&mut self, actor: Participant, event: TimelineEventType, summary: &str) {
        self.timeline
            .push(TimelineEvent::new(actor, event, summary));
        self.updated_at = Utc::now();
    }

    /// Transition to new status
    pub fn set_status(&mut self, new_status: CaseStatus) {
        if self.status != new_status {
            let old_status = self.status;
            self.status = new_status;
            self.add_event(
                Participant::Anna,
                TimelineEventType::StatusChange {
                    from: old_status,
                    to: new_status,
                },
                &format!("Status: {} -> {}", old_status, new_status),
            );
        }
    }

    /// Assign to department
    pub fn assign_department(&mut self, department: Department, reason: &str) {
        self.assigned_department = department;
        self.add_event(
            Participant::Anna,
            TimelineEventType::DepartmentAssigned { department },
            &format!("Assigning to [{}]: {}", department.actor_name(), reason),
        );

        // Add specialist as participant
        if department != Department::ServiceDesk {
            let specialist = Participant::Specialist(department);
            if !self.participants.contains(&specialist) {
                self.participants.push(specialist.clone());
                self.add_event(
                    Participant::Anna,
                    TimelineEventType::ParticipantJoined {
                        participant: specialist,
                    },
                    &format!("[{}] joined the case", department.actor_name()),
                );
            }
        }
    }

    /// Add participant
    pub fn add_participant(&mut self, participant: Participant) {
        if !self.participants.contains(&participant) {
            self.participants.push(participant.clone());
            self.add_event(
                Participant::Anna,
                TimelineEventType::ParticipantJoined {
                    participant: participant.clone(),
                },
                &format!("[{}] joined the case", participant),
            );
        }
    }

    // =========================================================================
    // v0.0.64: Service Desk Ticketing Methods
    // =========================================================================

    /// Set ticket information from Service Desk dispatch
    pub fn set_ticket(
        &mut self,
        ticket_id: &str,
        category: TicketCategory,
        severity: TicketSeverity,
    ) {
        self.ticket_id = Some(ticket_id.to_string());
        self.ticket_category = Some(category);
        self.ticket_severity = Some(severity);
    }

    /// Set routing information (which doctors were selected)
    pub fn set_routing(
        &mut self,
        primary_doctor: Option<String>,
        secondary_doctors: Vec<String>,
        evidence_topics: Vec<EvidenceTopic>,
        use_doctor_flow: bool,
    ) {
        self.primary_doctor = primary_doctor;
        self.secondary_doctors = secondary_doctors;
        self.evidence_topics = evidence_topics;
        self.used_doctor_flow = use_doctor_flow;
    }

    /// Record completion of a doctor lifecycle stage
    pub fn record_doctor_stage(&mut self, stage: DoctorLifecycleStage) {
        if !self.doctor_stages_completed.contains(&stage) {
            self.doctor_stages_completed.push(stage);
        }
    }

    /// Add evidence
    pub fn add_evidence(&mut self, evidence_id: &str, tool: &str) {
        if !self.evidence_ids.contains(&evidence_id.to_string()) {
            self.evidence_ids.push(evidence_id.to_string());
            self.evidence_count = self.evidence_ids.len();
            self.add_event(
                Participant::Annad,
                TimelineEventType::EvidenceCollected {
                    evidence_id: evidence_id.to_string(),
                    tool: tool.to_string(),
                },
                &format!("[{}] collected via {}", evidence_id, tool),
            );
        }
    }

    /// Add finding
    pub fn add_finding(&mut self, finding: &str) {
        self.findings.push(finding.to_string());
        self.add_event(
            Participant::Specialist(self.assigned_department),
            TimelineEventType::FindingRecorded {
                finding: finding.to_string(),
            },
            finding,
        );
    }

    /// Add hypothesis
    pub fn add_hypothesis(&mut self, hypothesis: &str, confidence: u8) {
        self.hypotheses.push((hypothesis.to_string(), confidence));
        self.add_event(
            Participant::Specialist(self.assigned_department),
            TimelineEventType::HypothesisProposed {
                hypothesis: hypothesis.to_string(),
                confidence,
            },
            &format!("{} ({}% confidence)", hypothesis, confidence),
        );
    }

    /// Link an alert
    pub fn link_alert(&mut self, alert_id: &str, alert_type: AlertType) {
        if !self.linked_alert_ids.contains(&alert_id.to_string()) {
            self.linked_alert_ids.push(alert_id.to_string());
            self.add_event(
                Participant::Anna,
                TimelineEventType::AlertLinked {
                    alert_id: alert_id.to_string(),
                    alert_type,
                },
                &format!("Linked to alert {} ({:?})", alert_id, alert_type),
            );
        }
    }

    /// Propose an action
    pub fn propose_action(&mut self, action: ProposedAction) {
        let action_id = action.id.clone();
        self.actions_proposed.push(action);
        self.add_event(
            Participant::Specialist(self.assigned_department),
            TimelineEventType::ActionProposed {
                action_id: action_id.clone(),
            },
            &format!("Proposed action {}", action_id),
        );
    }

    /// Request confirmation
    pub fn request_confirmation(&mut self, phrase: &str) {
        self.confirmation_required = true;
        self.confirmation_phrase = Some(phrase.to_string());
        self.set_status(CaseStatus::AwaitingConfirmation);
        self.add_event(
            Participant::Anna,
            TimelineEventType::ConfirmationRequested {
                phrase: phrase.to_string(),
            },
            &format!("Confirmation required: \"{}\"", phrase),
        );
    }

    /// Set reliability
    pub fn set_reliability(&mut self, pct: u8) {
        self.reliability_pct = pct;
    }

    /// Set coverage
    pub fn set_coverage(&mut self, pct: u8) {
        self.coverage_pct = pct;
    }

    /// Set final answer
    pub fn set_final_answer(&mut self, answer: &str) {
        self.final_answer = Some(answer.to_string());
    }

    /// Resolve the case successfully
    pub fn resolve(&mut self, answer: &str, reliability: u8) {
        self.final_answer = Some(answer.to_string());
        self.reliability_pct = reliability;
        self.set_status(CaseStatus::Resolved);
        self.add_event(
            Participant::Anna,
            TimelineEventType::CaseClosed { success: true },
            &format!("Case resolved ({}% reliability)", reliability),
        );
    }

    /// Abandon the case
    pub fn abandon(&mut self, reason: &str) {
        self.set_status(CaseStatus::Abandoned);
        self.add_event(
            Participant::Anna,
            TimelineEventType::CaseClosed { success: false },
            &format!("Case abandoned: {}", reason),
        );
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
        atomic_write_bytes(
            &case_path.to_string_lossy(),
            redact_transcript(&case_json).as_bytes(),
        )?;

        // Save transcript.log
        let transcript = self.render_transcript();
        let transcript_path = case_dir.join("transcript.log");
        atomic_write_bytes(
            &transcript_path.to_string_lossy(),
            redact_transcript(&transcript).as_bytes(),
        )?;

        Ok(case_dir)
    }

    // =========================================================================
    // v0.0.66: Human/Debug Transcript Methods
    // =========================================================================

    /// Add a line to the human transcript
    pub fn add_human_line(&mut self, line: &str) {
        self.human_transcript.push(line.to_string());
        self.updated_at = Utc::now();
    }

    /// Add evidence summary (human-readable description)
    pub fn add_evidence_summary(&mut self, summary: &str) {
        self.evidence_summaries.push(summary.to_string());
    }

    /// Set ticket type based on request analysis
    pub fn set_ticket_type(&mut self, ticket_type: TicketType) {
        self.ticket_type = ticket_type;
    }

    /// Set case outcome
    pub fn set_outcome(&mut self, outcome: CaseOutcome) {
        self.outcome = outcome;
    }

    /// Save human.log file
    pub fn save_human_log(&self) -> io::Result<PathBuf> {
        let case_dir = self.get_dir();
        fs::create_dir_all(&case_dir)?;

        let human_log_path = case_dir.join("human.log");
        let content = self.render_human_transcript();
        atomic_write_bytes(
            &human_log_path.to_string_lossy(),
            redact_transcript(&content).as_bytes(),
        )?;

        Ok(human_log_path)
    }

    /// Save debug.log file
    pub fn save_debug_log(&self, debug_content: &str) -> io::Result<PathBuf> {
        let case_dir = self.get_dir();
        fs::create_dir_all(&case_dir)?;

        let debug_log_path = case_dir.join("debug.log");
        atomic_write_bytes(
            &debug_log_path.to_string_lossy(),
            redact_transcript(debug_content).as_bytes(),
        )?;

        Ok(debug_log_path)
    }

    /// Save all case files (case.json, human.log, debug.log)
    pub fn save_all(&mut self, debug_content: &str) -> io::Result<PathBuf> {
        let case_dir = self.get_dir();
        fs::create_dir_all(&case_dir)?;

        // Set debug trace path
        self.debug_trace_path = Some(case_dir.join("debug.log").to_string_lossy().to_string());

        // Save case.json
        self.save()?;

        // Save human.log
        self.save_human_log()?;

        // Save debug.log
        self.save_debug_log(debug_content)?;

        Ok(case_dir)
    }

    /// Render human-readable transcript (fly-on-the-wall style)
    pub fn render_human_transcript(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("=== Case {} ===", self.case_id));
        lines.push(format!(
            "Type: {} | Outcome: {}",
            self.ticket_type, self.outcome
        ));
        if let Some(ticket) = &self.ticket_id {
            lines.push(format!("Ticket: #{}", ticket));
        }
        lines.push(format!(
            "Department: {} | Reliability: {}%",
            self.assigned_department, self.reliability_pct
        ));
        lines.push(String::new());

        // Add human transcript lines
        for line in &self.human_transcript {
            lines.push(line.clone());
        }

        // Add evidence summaries if we have them
        if !self.evidence_summaries.is_empty() {
            lines.push(String::new());
            lines.push("Evidence gathered:".to_string());
            for summary in &self.evidence_summaries {
                lines.push(format!("  - {}", summary));
            }
        }

        // Add final answer if present
        if let Some(answer) = &self.final_answer {
            lines.push(String::new());
            lines.push(format!("Summary: {}", answer));
            lines.push(format!("Reliability: {}%", self.reliability_pct));
        }

        lines.join("\n")
    }

    /// Render readable transcript
    pub fn render_transcript(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("=== Case {} ===", self.case_id));
        lines.push(format!("Title: {}", self.title));
        lines.push(format!("Status: {}", self.status));
        lines.push(format!("Department: {}", self.assigned_department));
        lines.push(format!(
            "Coverage: {}% | Reliability: {}%",
            self.coverage_pct, self.reliability_pct
        ));
        lines.push(String::new());

        for event in &self.timeline {
            let ts = event.timestamp.format("%H:%M:%S");
            lines.push(format!("[{}] [{}] {}", ts, event.actor, event.summary));
        }

        lines.join("\n")
    }
}

/// Load a case file v2 from disk
pub fn load_case_v2(case_id: &str) -> io::Result<CaseFileV2> {
    let case_path = PathBuf::from(format!("{}/{}/case.json", CASE_FILES_DIR, case_id));
    let content = fs::read_to_string(&case_path)?;
    serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// List active (non-terminal) cases
pub fn list_active_cases() -> io::Result<Vec<CaseFileV2>> {
    let cases_dir = std::path::Path::new(CASE_FILES_DIR);
    if !cases_dir.exists() {
        return Ok(Vec::new());
    }

    let mut active = Vec::new();
    for entry in fs::read_dir(cases_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            if let Ok(case) = load_case_v2(&entry.file_name().to_string_lossy()) {
                if case.status.is_active() {
                    active.push(case);
                }
            }
        }
    }

    // Sort by updated_at descending
    active.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(active)
}

/// Count active cases
pub fn count_active_cases() -> usize {
    list_active_cases().map(|c| c.len()).unwrap_or(0)
}

/// Helper to truncate title
fn truncate_title(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_lifecycle_new() {
        let case = CaseFileV2::new("test-001", "why is my wifi slow");
        assert_eq!(case.status, CaseStatus::New);
        assert_eq!(case.assigned_department, Department::ServiceDesk);
        assert!(case.participants.contains(&Participant::You));
        assert!(case.participants.contains(&Participant::Anna));
        assert!(!case.timeline.is_empty());
    }

    #[test]
    fn test_case_lifecycle_transitions() {
        let mut case = CaseFileV2::new("test-002", "test request");

        case.set_status(CaseStatus::Triaged);
        assert_eq!(case.status, CaseStatus::Triaged);

        case.assign_department(Department::Networking, "network-related keywords");
        assert_eq!(case.assigned_department, Department::Networking);
        assert!(case
            .participants
            .contains(&Participant::Specialist(Department::Networking)));

        case.set_status(CaseStatus::Investigating);
        case.add_evidence("E1", "network_status");
        assert_eq!(case.evidence_count, 1);

        case.set_status(CaseStatus::PlanReady);
        case.resolve("Network is working fine", 85);
        assert_eq!(case.status, CaseStatus::Resolved);
        assert!(case.status.is_terminal());
    }

    #[test]
    fn test_department_from_alert() {
        assert_eq!(
            Department::from_alert_type(AlertType::DiskPressure),
            Department::Storage
        );
        assert_eq!(
            Department::from_alert_type(AlertType::BootRegression),
            Department::Boot
        );
        assert_eq!(
            Department::from_alert_type(AlertType::ThermalThrottling),
            Department::Performance
        );
    }

    #[test]
    fn test_case_alert_linking() {
        let mut case = CaseFileV2::new("test-003", "why is boot slow");
        case.link_alert("alert-123", AlertType::BootRegression);
        assert!(case.linked_alert_ids.contains(&"alert-123".to_string()));
    }
}
