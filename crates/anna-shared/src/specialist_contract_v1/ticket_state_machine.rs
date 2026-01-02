//! State machine for ticket lifecycle management.

use super::{ResolutionCriteria, StateTransition, TicketState};

/// State machine for ticket lifecycle.
#[derive(Debug, Clone)]
pub struct TicketStateMachine {
    /// Current state.
    pub state: TicketState,
    /// Transition history.
    pub history: Vec<StateTransition>,
    /// Case ID.
    pub case_id: String,
}

impl TicketStateMachine {
    /// Create new state machine for a ticket.
    pub fn new(case_id: &str) -> Self {
        Self {
            state: TicketState::Open,
            history: Vec::new(),
            case_id: case_id.to_string(),
        }
    }

    /// Transition to RESOLVED.
    pub fn resolve(&mut self, criteria: &ResolutionCriteria) -> Result<(), &'static str> {
        if !criteria.is_resolved() {
            return Err(criteria.failure_reason().unwrap_or("Unknown reason"));
        }

        let transition = StateTransition::new(
            self.state,
            TicketState::Resolved,
            &format!("Resolved with confidence {:.2}", criteria.confidence),
        );
        self.history.push(transition);
        self.state = TicketState::Resolved;
        Ok(())
    }

    /// Transition to FAILED_PROBE.
    pub fn fail_probe(&mut self, failed_probes: &[&str]) {
        let reason = if failed_probes.is_empty() {
            "Required probes failed".to_string()
        } else {
            format!("Probes failed: {}", failed_probes.join(", "))
        };

        let transition = StateTransition::new(self.state, TicketState::FailedProbe, &reason);
        self.history.push(transition);
        self.state = TicketState::FailedProbe;
    }

    /// Transition to FAILED_SPECIALIST.
    pub fn fail_specialist(&mut self, reason: &str) {
        let transition = StateTransition::new(self.state, TicketState::FailedSpecialist, reason);
        self.history.push(transition);
        self.state = TicketState::FailedSpecialist;
    }

    /// Transition to NEED_CLARIFICATION.
    pub fn need_clarification(&mut self, question: &str) {
        let transition = StateTransition::new(
            self.state,
            TicketState::NeedClarification,
            &format!("Need clarification: {}", question),
        );
        self.history.push(transition);
        self.state = TicketState::NeedClarification;
    }

    /// Transition to ESCALATED.
    pub fn escalate(&mut self, reason: &str) {
        let transition = StateTransition::new(self.state, TicketState::Escalated, reason);
        self.history.push(transition);
        self.state = TicketState::Escalated;
    }

    /// Resume from NEED_CLARIFICATION to OPEN.
    pub fn resume(&mut self) {
        if self.state == TicketState::NeedClarification {
            let transition = StateTransition::new(
                self.state,
                TicketState::Open,
                "Clarification received, resuming",
            );
            self.history.push(transition);
            self.state = TicketState::Open;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_contract_v1::ResolutionCriteria;

    #[test]
    fn test_state_machine_resolve() {
        let mut sm = TicketStateMachine::new("DSK-0101");
        assert_eq!(sm.state, TicketState::Open);

        let criteria = ResolutionCriteria::new(true, true, 0.85);
        sm.resolve(&criteria).unwrap();
        assert_eq!(sm.state, TicketState::Resolved);
        assert_eq!(sm.history.len(), 1);
    }

    #[test]
    fn test_state_machine_fail_probe() {
        let mut sm = TicketStateMachine::new("DSK-0101");
        sm.fail_probe(&["systemd_analyze", "free_h"]);
        assert_eq!(sm.state, TicketState::FailedProbe);
    }

    #[test]
    fn test_state_machine_fail_specialist() {
        let mut sm = TicketStateMachine::new("DSK-0101");
        sm.fail_specialist("Specialist timeout after 2 retries");
        assert_eq!(sm.state, TicketState::FailedSpecialist);
    }
}
