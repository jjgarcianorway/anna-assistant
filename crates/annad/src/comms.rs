//! Internal communications helper for Service Desk Theatre (v0.0.146).
//!
//! Generates IT department chatter for fly-on-wall experience.
//! Uses roster and dialogue systems to create authentic messages.

use anna_shared::dialogue::{junior_acknowledgment, seed_from_str};
use anna_shared::progress::RequestStage;
use anna_shared::roster::{person_for, Tier};
use anna_shared::teams::Team;

use crate::progress_tracker::ProgressTracker;

/// Generate internal comms at key pipeline stages
pub struct CommsGenerator {
    team: Team,
    case_id: String,
    seed: u64,
}

impl CommsGenerator {
    /// Create a new comms generator for a request
    pub fn new(team: Team, case_id: &str) -> Self {
        Self {
            team,
            case_id: case_id.to_string(),
            seed: seed_from_str(case_id),
        }
    }

    /// Get the junior staff member for this team
    fn junior(&self) -> anna_shared::roster::PersonProfile {
        person_for(self.team, Tier::Junior)
    }

    /// Get the senior staff member for this team
    fn senior(&self) -> anna_shared::roster::PersonProfile {
        person_for(self.team, Tier::Senior)
    }

    /// Anna dispatches the case to a team member
    pub fn dispatch(&self, progress: &mut ProgressTracker) {
        let junior = self.junior();
        let short_id = &self.case_id[..8.min(self.case_id.len())];

        let greetings = [
            format!("Hey {}! New case {} coming your way.", junior.display_name, short_id),
            format!("{}, got a ticket for you. Case {}", junior.display_name, short_id),
            format!("{}, when you have a sec - new request. {}", junior.display_name, short_id),
        ];

        let msg = &greetings[(self.seed as usize) % greetings.len()];
        progress.add_internal_comms(RequestStage::Translator, "Anna", msg);
    }

    /// Junior acknowledges the case
    pub fn junior_ack(&self, progress: &mut ProgressTracker) {
        let ack = junior_acknowledgment(self.team, self.seed);
        let junior = self.junior();
        progress.add_internal_comms(RequestStage::Translator, junior.display_name, &ack);
    }

    /// Junior reports on probe progress
    pub fn junior_probing(&self, progress: &mut ProgressTracker, probe_count: usize) {
        let junior = self.junior();
        let messages = [
            format!("Running {} check{}...", probe_count, if probe_count == 1 { "" } else { "s" }),
            format!("Gathering data... {} probe{} queued.", probe_count, if probe_count == 1 { "" } else { "s" }),
            format!("Let me pull some numbers. {} check{} to run.", probe_count, if probe_count == 1 { "" } else { "s" }),
        ];

        let msg = &messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Probes, junior.display_name, msg);
    }

    /// Junior reviewing the answer
    pub fn junior_reviewing(&self, progress: &mut ProgressTracker) {
        let junior = self.junior();
        let messages = [
            "Checking the response...",
            "Verifying the data...",
            "Running quality checks...",
            "Looking good so far...",
        ];

        let msg = messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Specialist, junior.display_name, msg);
    }

    /// Junior escalating to senior
    pub fn junior_escalate(&self, progress: &mut ProgressTracker, reason: &str) {
        let junior = self.junior();
        let senior = self.senior();

        let msg = format!(
            "Hey {}, can you take a look at this? {}",
            senior.display_name, reason
        );
        progress.add_internal_comms(RequestStage::Supervisor, junior.display_name, &msg);
    }

    /// Senior responding to escalation
    pub fn senior_response(&self, progress: &mut ProgressTracker, helpful: bool) {
        let senior = self.senior();
        let messages = if helpful {
            vec![
                "Let me see... Ah, I know this one.",
                "Good catch bringing this to me.",
                "I've seen this before. Here's what we do...",
            ]
        } else {
            vec![
                "Hmm, tricky one. Let me think...",
                "That's unusual. Give me a moment.",
                "Interesting edge case here...",
            ]
        };

        let msg = messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Supervisor, senior.display_name, msg);
    }

    /// Junior confirms answer is ready
    pub fn junior_done(&self, progress: &mut ProgressTracker, confidence: u8) {
        let junior = self.junior();
        let messages = if confidence >= 90 {
            vec![
                format!("All good! {}% confidence. Sending back to Anna.", confidence),
                format!("Looks solid. {}% - ready to go.", confidence),
            ]
        } else if confidence >= 70 {
            vec![
                format!("Done. {}% confidence.", confidence),
                format!("Finished review. {}%.", confidence),
            ]
        } else {
            vec![
                format!("Best I can do is {}%. Sending it back.", confidence),
                format!("Done, but only {}% sure. Take it with a grain of salt.", confidence),
            ]
        };

        let msg = &messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Supervisor, junior.display_name, msg);
    }

    /// Anna returning with the answer
    pub fn anna_returning(&self, progress: &mut ProgressTracker) {
        let messages = [
            "Thanks team! I'll take it from here.",
            "Got it. Passing this along now.",
            "Perfect, sending the response.",
        ];

        let msg = messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Supervisor, "Anna", msg);
    }
}

/// Determine team from domain string
pub fn team_from_domain(domain: &str) -> Team {
    match domain.to_lowercase().as_str() {
        "storage" => Team::Storage,
        "network" => Team::Network,
        "security" => Team::Security,
        "packages" => Team::Desktop, // Package management is desktop team
        "desktop" => Team::Desktop,
        _ => Team::Desktop, // Default to desktop for general system queries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comms_generator_creates_messages() {
        let gen = CommsGenerator::new(Team::Desktop, "test-case-123");
        let mut progress = ProgressTracker::new();

        gen.dispatch(&mut progress);
        assert!(!progress.events().is_empty());
    }

    #[test]
    fn test_team_from_domain() {
        assert_eq!(team_from_domain("storage"), Team::Storage);
        assert_eq!(team_from_domain("NETWORK"), Team::Network);
        assert_eq!(team_from_domain("unknown"), Team::Desktop);
    }
}
