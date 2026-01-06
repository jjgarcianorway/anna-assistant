//! NarrativeBuilder for constructing narrative from pipeline events (v0.0.831).
//!
//! v0.0.226: Initial version.
//! v0.0.451: Enhanced for fly-on-the-wall view per VISION.md.
//! v0.0.831: Added push_segment() for custom segment injection.

use crate::dialogue::{
    anna_after_review, anna_dispatch_greeting, junior_approval, junior_escalation_request,
    seed_from_str, senior_response,
};
use crate::roster::{person_for, Tier};
use crate::teams::Team;

use super::types::NarrativeSegment;

/// Builder for constructing narrative from pipeline events
#[derive(Debug, Default)]
pub struct NarrativeBuilder {
    segments: Vec<NarrativeSegment>,
    current_team: Option<Team>,
    show_internal: bool,
    seed: u64, // For dialogue variety
}

impl NarrativeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a seed for consistent dialogue variety
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Enable showing internal IT communications
    pub fn with_internal_comms(mut self) -> Self {
        self.show_internal = true;
        self
    }

    /// Add Anna greeting based on domain
    pub fn add_greeting(&mut self, domain: &str) {
        let greeting = match domain.to_lowercase().as_str() {
            "storage" | "disk" => "Let me check that storage information for you.",
            "memory" | "ram" => "I'll look into the memory right away.",
            "network" | "wifi" => "Let me examine your network configuration.",
            "performance" | "cpu" | "slow" => "I'll analyze the system performance.",
            "service" | "services" => "Let me check those service statuses.",
            "security" => "I'll review the security information carefully.",
            "hardware" | "audio" => "Let me gather the hardware details.",
            "desktop" | "editor" => "I'll check that for you.",
            _ => "Let me look into that for you.",
        };
        self.segments.push(NarrativeSegment::anna(greeting));
    }

    /// Add probe activity narration
    pub fn add_checking(&mut self, description: &str) {
        self.segments.push(NarrativeSegment::narrator(format!(
            "Checking {}...",
            description
        )));
    }

    /// Add Anna dispatching to a team (v0.0.87: varied dialogue)
    /// v0.0.451: Use anna_to for fly-on-the-wall format
    pub fn add_dispatch(&mut self, team: Team, case_id: &str) {
        self.current_team = Some(team);
        // Update seed based on case_id for this conversation
        self.seed = seed_from_str(case_id);

        if self.show_internal {
            let greeting = anna_dispatch_greeting(team, case_id);
            // v0.0.451: Get recipient name for fly-on-the-wall view
            let recipient = person_for(team, Tier::Junior);
            self.segments
                .push(NarrativeSegment::anna_to(&recipient.display_name, greeting));
        }
    }

    /// Add junior review narration (v0.0.87: varied dialogue)
    pub fn add_junior_review(&mut self, team: Team, approved: bool, score: u8) {
        if self.show_internal {
            let response = if approved {
                junior_approval(score, self.seed)
            } else {
                junior_escalation_request(team, score, self.seed)
            };
            self.segments
                .push(NarrativeSegment::team_member(team, Tier::Junior, response));
        }
    }

    /// Add senior escalation narration (v0.0.87: varied dialogue)
    pub fn add_escalation(&mut self, team: Team, reason: &str) {
        if self.show_internal {
            let request = junior_escalation_request(team, 0, self.seed);
            // Include reason if substantial
            let full_msg = if reason.len() > 5 {
                format!("{} {}", request, reason)
            } else {
                request
            };
            self.segments
                .push(NarrativeSegment::team_member(team, Tier::Junior, full_msg));
        }
    }

    /// Add senior response (v0.0.87: varied dialogue)
    pub fn add_senior_response(&mut self, team: Team, _guidance: &str) {
        if self.show_internal {
            let response = senior_response(true, self.seed);
            self.segments
                .push(NarrativeSegment::team_member(team, Tier::Senior, response));
        }
    }

    /// Add Anna's apology for wait time (v0.0.87: varied dialogue)
    pub fn add_wait_apology(&mut self) {
        let apology = anna_after_review(true, self.seed);
        self.segments.push(NarrativeSegment::anna(apology));
    }

    /// Add Anna presenting the answer
    pub fn add_answer_intro(&mut self, confidence: u8) {
        let intro = match confidence {
            90..=100 => "Here's what I found:",
            80..=89 => "Based on my checks:",
            70..=79 => "From what I can tell:",
            _ => "I found some information, though I'm not fully certain:",
        };
        self.segments.push(NarrativeSegment::anna(intro));
    }

    /// Add clarification request narration
    pub fn add_clarification(&mut self, question: &str) {
        self.segments
            .push(NarrativeSegment::anna(question.to_string()));
    }

    /// Build the final narrative
    pub fn build(self) -> Vec<NarrativeSegment> {
        self.segments
    }

    /// Get segment count
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// v0.0.831: Push a custom segment directly
    /// Used by narrative builder to inject Message-based internal comms
    pub fn push_segment(&mut self, segment: NarrativeSegment) {
        self.segments.push(segment);
    }
}
