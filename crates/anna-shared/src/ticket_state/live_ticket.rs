//! Live ticket tracking with explicit state machine

use serde::{Deserialize, Serialize};

use super::{
    error::ErrorKind, handler::HandlerType, outcome::TicketOutcome, state::TicketState,
    transition::StateTransition,
};

/// Live ticket tracking with explicit state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTicket {
    /// Unique ticket ID
    pub id: String,
    /// Creation timestamp (Unix millis)
    pub created_at: u64,
    /// Last update timestamp (Unix millis)
    pub updated_at: u64,
    /// Original user question
    pub user_question: String,
    /// Domain classification
    pub domain: String,
    /// Intent classification
    pub intent: String,
    /// Extracted parameters
    pub params: std::collections::HashMap<String, String>,
    /// Current state
    pub state: TicketState,
    /// v0.0.411: Outcome - semantic meaning of resolution
    #[serde(default)]
    pub outcome: Option<TicketOutcome>,
    /// Handler type
    pub handler: Option<HandlerType>,
    /// Staff member who handled this (e.g., "Sofia (Jr)", "Tomas (Sr)")
    #[serde(default)]
    pub handled_by: Option<String>,
    /// Error kind if failed
    pub error_kind: Option<ErrorKind>,
    /// Error details (for logging)
    pub error_detail: Option<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Evidence summary (short string)
    pub evidence_summary: Option<String>,
    /// Evidence IDs used
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    /// Whether ticket was escalated
    pub escalated: bool,
    /// Escalation path (e.g., "recipe→llm", "junior→senior")
    pub escalation_path: Option<String>,
    /// Staff member escalated to
    #[serde(default)]
    pub escalated_to: Option<String>,
    /// Final answer (if produced)
    pub answer: Option<String>,
    /// State transition history
    pub transitions: Vec<StateTransition>,
    /// LLM call count
    pub llm_calls: u8,
    /// Retry count
    pub retry_count: u8,
    /// v0.0.408: Knowledge item IDs attached to this ticket
    #[serde(default)]
    pub knowledge_ids: Vec<String>,
    /// v0.0.408: Knowledge search keywords used
    #[serde(default)]
    pub knowledge_keywords: Vec<String>,
    /// v0.0.411: Duration in milliseconds
    #[serde(default)]
    pub duration_ms: u64,
}

impl LiveTicket {
    /// Create a new ticket
    pub fn new(id: impl Into<String>, question: impl Into<String>) -> Self {
        let now = current_millis();
        Self {
            id: id.into(),
            created_at: now,
            updated_at: now,
            user_question: question.into(),
            domain: String::new(),
            intent: String::new(),
            params: std::collections::HashMap::new(),
            state: TicketState::Created,
            outcome: None,
            handler: None,
            handled_by: None,
            error_kind: None,
            error_detail: None,
            confidence: 0.0,
            evidence_summary: None,
            evidence_ids: vec![],
            escalated: false,
            escalation_path: None,
            escalated_to: None,
            answer: None,
            transitions: vec![],
            llm_calls: 0,
            retry_count: 0,
            knowledge_ids: vec![],
            knowledge_keywords: vec![],
            duration_ms: 0,
        }
    }

    /// v0.0.408: Attach knowledge items to this ticket
    pub fn attach_knowledge(&mut self, item_ids: Vec<String>, keywords: Vec<String>) {
        self.knowledge_ids = item_ids;
        self.knowledge_keywords = keywords;
        self.updated_at = current_millis();
    }

    /// v0.0.408: Check if any knowledge was attached
    pub fn has_knowledge(&self) -> bool {
        !self.knowledge_ids.is_empty()
    }

    /// Transition to a new state
    pub fn transition(&mut self, new_state: TicketState, reason: Option<String>) {
        let transition = StateTransition {
            from: self.state,
            to: new_state,
            at: current_millis(),
            reason,
        };
        self.transitions.push(transition);
        self.state = new_state;
        self.updated_at = current_millis();
    }

    /// Mark as planned (after translator)
    pub fn mark_planned(&mut self, domain: &str, intent: &str) {
        self.domain = domain.to_string();
        self.intent = intent.to_string();
        self.transition(
            TicketState::Planned,
            Some("Translator completed".to_string()),
        );
    }

    /// Mark probes as run
    pub fn mark_probes_run(&mut self, evidence_summary: Option<String>) {
        self.evidence_summary = evidence_summary;
        self.transition(TicketState::ProbesRun, Some("Probes executed".to_string()));
    }

    /// Mark docs as attached
    pub fn mark_docs_attached(&mut self) {
        self.transition(
            TicketState::DocsAttached,
            Some("Documentation fetched".to_string()),
        );
    }

    /// Mark LLM request sent
    pub fn mark_llm_requested(&mut self, handler: HandlerType) {
        self.handler = Some(handler);
        self.llm_calls += 1;
        self.transition(
            TicketState::LlmRequested,
            Some("LLM request sent".to_string()),
        );
    }

    /// Mark LLM failure
    pub fn mark_llm_failed(&mut self, kind: ErrorKind, detail: Option<String>) {
        self.error_kind = Some(kind.clone());
        self.error_detail = detail.clone();
        self.transition(
            TicketState::LlmFailed,
            Some(format!("LLM failed: {}", kind)),
        );
    }

    /// Mark as answered (confidence 0.0-1.0)
    pub fn mark_answered(&mut self, answer: &str, confidence: f32) {
        self.answer = Some(answer.to_string());
        self.confidence = confidence;
        self.transition(TicketState::Answered, Some("Answer produced".to_string()));
    }

    /// Mark commands run
    pub fn mark_commands_run(&mut self) {
        self.transition(
            TicketState::CommandsRun,
            Some("Commands executed".to_string()),
        );
    }

    /// v0.0.411: Mark as successful with explicit outcome
    pub fn mark_success_with_outcome(&mut self, outcome: TicketOutcome, handler: &str) {
        self.outcome = Some(outcome);
        self.handled_by = Some(handler.to_string());
        self.duration_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(TicketState::Success, Some(format!("Outcome: {}", outcome)));
    }

    /// Mark as successful (defaults to Success outcome)
    pub fn mark_success(&mut self) {
        self.outcome = Some(TicketOutcome::Success);
        self.duration_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(
            TicketState::Success,
            Some("Completed successfully".to_string()),
        );
    }

    /// v0.0.411: Mark as failed with explicit outcome
    pub fn mark_failed_with_outcome(
        &mut self,
        kind: ErrorKind,
        detail: Option<String>,
        handler: Option<&str>,
    ) {
        self.error_kind = Some(kind.clone());
        self.error_detail = detail;
        self.outcome = Some(kind.to_outcome());
        self.handled_by = handler.map(|s| s.to_string());
        self.duration_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(TicketState::Failed, Some(format!("Failed: {}", kind)));
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, kind: ErrorKind, detail: Option<String>) {
        self.mark_failed_with_outcome(kind, detail, None);
    }

    /// v0.0.411: Mark as partial answer
    pub fn mark_partial(&mut self, answer: &str, handler: &str) {
        self.answer = Some(answer.to_string());
        self.outcome = Some(TicketOutcome::Partial);
        self.handled_by = Some(handler.to_string());
        self.duration_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(TicketState::Success, Some("Partial answer".to_string()));
    }

    /// v0.0.411: Mark as cannot answer safely
    pub fn mark_cannot_answer(&mut self, reason: &str, handler: &str) {
        self.error_detail = Some(reason.to_string());
        self.outcome = Some(TicketOutcome::CannotAnswerSafely);
        self.handled_by = Some(handler.to_string());
        self.error_kind = Some(ErrorKind::MissingEvidence);
        self.duration_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(
            TicketState::Success,
            Some("Cannot answer safely".to_string()),
        );
    }

    /// Mark as escalated
    pub fn mark_escalated(&mut self, path: &str) {
        self.escalated = true;
        self.escalation_path = Some(path.to_string());
    }

    /// v0.0.411: Mark escalation to specific staff
    pub fn mark_escalated_to(&mut self, staff: &str) {
        self.escalated = true;
        self.escalated_to = Some(staff.to_string());
    }

    /// v0.0.411: Set handler (staff who handled this ticket)
    pub fn set_handler(&mut self, handler: &str) {
        self.handled_by = Some(handler.to_string());
    }

    /// v0.0.411: Add evidence IDs
    pub fn add_evidence(&mut self, evidence_ids: Vec<String>) {
        self.evidence_ids = evidence_ids;
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Check if ticket is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self.state, TicketState::Success | TicketState::Failed)
    }

    /// Check if ticket was successful (outcome is success)
    pub fn is_success(&self) -> bool {
        // Only consider it a success if the outcome is explicitly Success
        // (Partial and CannotAnswerSafely are "resolved" but not "success")
        self.outcome.map(|o| o.is_success()).unwrap_or(false)
    }

    /// v0.0.411: Check if outcome is an error
    pub fn is_error(&self) -> bool {
        self.outcome.map(|o| o.is_error()).unwrap_or(false)
    }

    /// Check if ticket reached answered state (for stats)
    pub fn reached_answered(&self) -> bool {
        matches!(
            self.state,
            TicketState::Answered | TicketState::CommandsRun | TicketState::Success
        )
    }

    /// Get duration in milliseconds
    pub fn get_duration_ms(&self) -> u64 {
        if self.duration_ms > 0 {
            self.duration_ms
        } else {
            self.updated_at.saturating_sub(self.created_at)
        }
    }

    /// v0.0.411: Get outcome or derive from state
    pub fn get_outcome(&self) -> TicketOutcome {
        if let Some(outcome) = self.outcome {
            outcome
        } else if self.state == TicketState::Failed {
            self.error_kind
                .as_ref()
                .map(|k| k.to_outcome())
                .unwrap_or(TicketOutcome::ErrorInternal)
        } else if self.state == TicketState::Success {
            if self.confidence >= 0.8 {
                TicketOutcome::Success
            } else if self.confidence >= 0.5 {
                TicketOutcome::Partial
            } else {
                TicketOutcome::CannotAnswerSafely
            }
        } else {
            TicketOutcome::ErrorInternal
        }
    }

    /// Get handler string for logging
    pub fn handler_string(&self) -> String {
        self.handler
            .as_ref()
            .map(|h| h.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// Get current time in milliseconds since Unix epoch
fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
