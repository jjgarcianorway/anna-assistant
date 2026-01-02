//! TicketLog implementation methods.

use super::types::{ProbeLog, SolverOutput, TicketLog, TicketResult};
use super::utils::{chrono_timestamp, ticket_log_dir};
use crate::rpc::{ProbeResult, SpecialistDomain};
use crate::ticket_state::{ErrorKind, TicketState};
use std::collections::HashMap;
use std::fs;
use tracing::debug;

impl TicketLog {
    /// Create a new ticket log entry
    pub fn new(
        id: impl Into<String>,
        domain: SpecialistDomain,
        intent: impl Into<String>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            timestamp: chrono_timestamp(),
            domain: domain.to_string().to_lowercase(),
            intent: intent.into(),
            query: query.into(),
            params: HashMap::new(),
            probes: vec![],
            docs_used: vec![],
            solver_output: SolverOutput {
                solver_type: "unknown".to_string(),
                analysis: None,
                model: None,
                tokens_used: None,
            },
            commands_run: vec![],
            answer: String::new(),
            result: TicketResult::Success,
            handled_by: "unknown".to_string(),
            duration_ms: 0,
            reliability_score: 0,
            state: Some(TicketState::Created),
            error_kind: None,
            escalated: false,
            escalation_path: None,
            llm_calls: 0,
            retry_count: 0,
        }
    }

    /// v0.0.407: Set explicit state
    pub fn with_state(mut self, state: TicketState) -> Self {
        self.state = Some(state);
        self
    }

    /// v0.0.407: Set error info
    pub fn with_error(mut self, kind: ErrorKind) -> Self {
        self.error_kind = Some(kind);
        self.state = Some(TicketState::Failed);
        self.result = TicketResult::Failed;
        self
    }

    /// v0.0.407: Set escalation info
    pub fn with_escalation(mut self, path: &str) -> Self {
        self.escalated = true;
        self.escalation_path = Some(path.to_string());
        self
    }

    /// v0.0.407: Set LLM call count
    pub fn with_llm_calls(mut self, count: u8) -> Self {
        self.llm_calls = count;
        self
    }

    /// Set parameters
    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = params;
        self
    }

    /// Add probes from probe results
    pub fn with_probes(mut self, probes: &[ProbeResult]) -> Self {
        self.probes = probes.iter().map(ProbeLog::from).collect();
        self
    }

    /// Set solver output
    pub fn with_solver(mut self, solver_type: impl Into<String>) -> Self {
        self.solver_output.solver_type = solver_type.into();
        self
    }

    /// Set LLM details
    pub fn with_llm_details(
        mut self,
        model: &str,
        analysis: Option<String>,
        tokens: Option<u32>,
    ) -> Self {
        self.solver_output.model = Some(model.to_string());
        self.solver_output.analysis = analysis;
        self.solver_output.tokens_used = tokens;
        self
    }

    /// Set answer
    pub fn with_answer(mut self, answer: impl Into<String>, result: TicketResult) -> Self {
        self.answer = answer.into();
        self.result = result;
        self
    }

    /// Set metrics
    pub fn with_metrics(mut self, duration_ms: u64, reliability: u8) -> Self {
        self.duration_ms = duration_ms;
        self.reliability_score = reliability;
        self
    }

    /// Set handler (who processed this)
    pub fn with_handler(mut self, handler: impl Into<String>) -> Self {
        self.handled_by = handler.into();
        self
    }

    /// v0.0.407: Check if this ticket represents a real success
    pub fn is_real_success(&self) -> bool {
        // Use explicit state if it's terminal (Success/Failed)
        if let Some(state) = &self.state {
            if matches!(state, TicketState::Success | TicketState::Failed) {
                return *state == TicketState::Success;
            }
        }
        // Legacy or in-progress: use result field
        self.result == TicketResult::Success && self.reliability_score >= 50
    }

    /// v0.0.407: Check if this ticket reached answered state
    pub fn reached_answered(&self) -> bool {
        if let Some(state) = &self.state {
            return matches!(
                state,
                TicketState::Answered | TicketState::CommandsRun | TicketState::Success
            );
        }
        // Legacy: has non-empty answer
        !self.answer.is_empty()
    }

    /// v0.0.407: Check if this is an LLM failure
    /// v0.0.409: Also includes ValidationFailed (LLM output invalid content)
    pub fn is_llm_failure(&self) -> bool {
        matches!(
            &self.error_kind,
            Some(ErrorKind::LlmTimeout)
                | Some(ErrorKind::LlmParseError)
                | Some(ErrorKind::ValidationFailed)
        )
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let dir = ticket_log_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", self.id));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&path, json)?;
        debug!("Saved ticket log: {}", path.display());
        Ok(())
    }

    /// Load from disk
    pub fn load(id: &str) -> Result<Self, std::io::Error> {
        let path = ticket_log_dir().join(format!("{}.json", id));
        let json = fs::read_to_string(&path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_log_creation() {
        let log = TicketLog::new(
            "SVC-0001",
            SpecialistDomain::Services,
            "diagnose",
            "why is sshd failing",
        )
        .with_solver("llm:junior")
        .with_handler("llm:junior")
        .with_metrics(500, 85);

        assert_eq!(log.id, "SVC-0001");
        assert_eq!(log.domain, "services");
        assert_eq!(log.handled_by, "llm:junior");
        assert_eq!(log.reliability_score, 85);
    }
}
