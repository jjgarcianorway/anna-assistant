//! Ticket clarification methods (v0.0.215).

use super::ticket_struct::Ticket;
use super::types::TicketStatus;

impl Ticket {
    /// Check if ticket is awaiting clarification
    pub fn is_awaiting_clarification(&self) -> bool {
        self.status == TicketStatus::AwaitingClarification
    }

    /// Check if ticket is verifying a clarification
    pub fn is_verifying_clarification(&self) -> bool {
        self.status == TicketStatus::VerifyingClarification
    }

    /// Check if more clarification rounds are allowed
    pub fn can_ask_clarification(&self) -> bool {
        self.clarification_rounds < self.clarification_rounds_max
    }

    /// Set pending clarification question
    pub fn set_pending_clarification(&mut self, id: &str, prompt: &str) {
        self.pending_clarification_id = Some(id.to_string());
        self.pending_clarification_prompt = Some(prompt.to_string());
        self.clarification_answer = None;
        self.status = TicketStatus::AwaitingClarification;
    }

    /// Set user's clarification answer and move to verification
    pub fn set_clarification_answer(&mut self, answer: &str) {
        self.clarification_answer = Some(answer.to_string());
        self.status = TicketStatus::VerifyingClarification;
    }

    /// Mark clarification as verified and record fact learned
    pub fn complete_clarification(&mut self, fact_key: Option<&str>) {
        self.clarification_rounds = self.clarification_rounds.saturating_add(1);
        if let Some(key) = fact_key {
            self.facts_learned.push(key.to_string());
        }
        self.pending_clarification_id = None;
        self.pending_clarification_prompt = None;
        self.clarification_answer = None;
        self.status = TicketStatus::New; // Ready for next step
    }

    /// Mark clarification as failed and prepare for retry or follow-up
    pub fn fail_clarification(&mut self) {
        // Don't increment rounds on failure - give them another chance with better choices
        self.clarification_answer = None;
        self.status = TicketStatus::AwaitingClarification;
    }

    /// Clear clarification state (when proceeding without it)
    pub fn clear_clarification(&mut self) {
        self.pending_clarification_id = None;
        self.pending_clarification_prompt = None;
        self.clarification_answer = None;
        self.status = TicketStatus::New;
    }
}
