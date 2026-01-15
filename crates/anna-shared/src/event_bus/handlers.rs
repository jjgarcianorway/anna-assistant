//! EventBus convenience methods for emitting specific event types.

use super::types::{Event, LlmPurpose, StepType, TicketEvent};
use super::EventBus;

impl EventBus {
    // === Step convenience methods ===

    /// Emit step started event
    pub fn step_started(&self, step_type: StepType, description: &str) -> String {
        let step_id = uuid::Uuid::new_v4().to_string();
        self.emit(Event::StepStarted {
            step_id: step_id.clone(),
            step_type,
            description: description.to_string(),
        });
        step_id
    }

    /// Emit step finished event
    pub fn step_finished(&self, step_id: &str, step_type: StepType, duration_ms: u64, success: bool) {
        self.emit(Event::StepFinished {
            step_id: step_id.to_string(),
            step_type,
            duration_ms,
            success,
        });
    }

    // === Probe convenience methods ===

    /// Emit probe started event
    pub fn probe_started(&self, command: &str) -> String {
        let probe_id = uuid::Uuid::new_v4().to_string();
        let display_command = super::redact_command(command);
        self.emit(Event::ProbeStarted {
            probe_id: probe_id.clone(),
            command: command.to_string(),
            display_command,
        });
        probe_id
    }

    /// Emit probe finished event
    pub fn probe_finished(&self, probe_id: &str, exit_code: i32, output: &str, duration_ms: u64) {
        let output_summary = super::redact_output(output);
        self.emit(Event::ProbeFinished {
            probe_id: probe_id.to_string(),
            exit_code,
            output_summary,
            duration_ms,
        });
    }

    // === LLM convenience methods ===

    /// Emit LLM started event
    pub fn llm_started(&self, purpose: LlmPurpose, model: &str) -> String {
        let request_id = uuid::Uuid::new_v4().to_string();
        self.emit(Event::LlmStarted {
            request_id: request_id.clone(),
            purpose,
            model: model.to_string(),
        });
        request_id
    }

    /// Emit LLM token event
    pub fn llm_token(&self, request_id: &str, token: &str) {
        self.emit(Event::LlmToken {
            request_id: request_id.to_string(),
            token: token.to_string(),
        });
    }

    /// Emit LLM finished event
    pub fn llm_finished(
        &self,
        request_id: &str,
        duration_ms: u64,
        tokens_used: Option<u32>,
        success: bool,
    ) {
        self.emit(Event::LlmFinished {
            request_id: request_id.to_string(),
            duration_ms,
            tokens_used,
            success,
        });
    }

    // === Status convenience methods ===

    /// Emit warning event
    pub fn warning(&self, code: &str, message: &str, source: Option<&str>) {
        self.emit(Event::Warning {
            code: code.to_string(),
            message: message.to_string(),
            source: source.map(|s| s.to_string()),
        });
    }

    /// Emit error event
    pub fn error(&self, code: &str, message: &str, source: Option<&str>, recoverable: bool) {
        self.emit(Event::Error {
            code: code.to_string(),
            message: message.to_string(),
            source: source.map(|s| s.to_string()),
            recoverable,
        });
    }

    /// Emit progress event
    pub fn progress(&self, operation: &str, current: u64, total: Option<u64>, message: Option<&str>) {
        self.emit(Event::Progress {
            operation: operation.to_string(),
            current,
            total,
            message: message.map(|s| s.to_string()),
        });
    }

    // === Answer convenience methods ===

    /// Emit answer ready event
    pub fn answer_ready(&self, answer: &str, confidence: f32, citations: Vec<String>) {
        self.emit(Event::AnswerReady {
            answer: answer.to_string(),
            confidence,
            citations,
        });
    }

    /// Emit investigation needed event
    pub fn investigation_needed(&self, reason: &str, suggested_probes: Vec<String>) {
        self.emit(Event::InvestigationNeeded {
            reason: reason.to_string(),
            suggested_probes,
        });
    }

    // === Ticket convenience methods ===

    /// Emit ticket created event.
    pub fn ticket_created(&self, ticket_id: &str, department: &str, question: &str) {
        self.emit(Event::TicketLifecycle(TicketEvent::Created {
            ticket_id: ticket_id.to_string(),
            department: department.to_string(),
            question_summary: super::truncate_for_display(question, 50),
        }));
    }

    /// Emit ticket assigned event.
    pub fn ticket_assigned(
        &self,
        ticket_id: &str,
        specialist_id: &str,
        specialist_name: &str,
        department: &str,
    ) {
        self.emit(Event::TicketLifecycle(TicketEvent::Assigned {
            ticket_id: ticket_id.to_string(),
            specialist_id: specialist_id.to_string(),
            specialist_name: specialist_name.to_string(),
            department: department.to_string(),
        }));
    }

    /// Emit ticket resolved event.
    pub fn ticket_resolved(
        &self,
        ticket_id: &str,
        specialist_id: &str,
        specialist_name: &str,
        confidence: f32,
        learned_recipe: bool,
    ) {
        self.emit(Event::TicketLifecycle(TicketEvent::Resolved {
            ticket_id: ticket_id.to_string(),
            specialist_id: specialist_id.to_string(),
            specialist_name: specialist_name.to_string(),
            confidence,
            learned_recipe,
        }));
    }
}
