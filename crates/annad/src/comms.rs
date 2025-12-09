//! Internal communications helper for Service Desk Theatre (v0.0.146).
//!
//! Generates IT department chatter for fly-on-wall experience.
//! Uses roster and dialogue systems to create authentic messages.
//!
//! v0.0.152: Added more variety, team-specific flavor, and probe result commentary.

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
    /// Track how many probes were planned (for commentary)
    probes_planned: usize,
}

impl CommsGenerator {
    /// Create a new comms generator for a request
    pub fn new(team: Team, case_id: &str) -> Self {
        Self {
            team,
            case_id: case_id.to_string(),
            seed: seed_from_str(case_id),
            probes_planned: 0,
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

        // Team-specific dispatch messages
        let greetings = match self.team {
            Team::Storage => vec![
                format!("Hey {}! Disk question coming in. Case {}", junior.display_name, short_id),
                format!("{}, storage ticket for you. {}", junior.display_name, short_id),
                format!("{}, got a disk/storage request. {}", junior.display_name, short_id),
            ],
            Team::Network => vec![
                format!("Hey {}! Network query. Case {}", junior.display_name, short_id),
                format!("{}, connectivity question. {}", junior.display_name, short_id),
                format!("{}, network ticket incoming. {}", junior.display_name, short_id),
            ],
            Team::Security => vec![
                format!("Hey {}! Security matter. Case {}", junior.display_name, short_id),
                format!("{}, access/security question. {}", junior.display_name, short_id),
                format!("{}, security ticket. {}", junior.display_name, short_id),
            ],
            Team::Performance => vec![
                format!("Hey {}! Performance question. Case {}", junior.display_name, short_id),
                format!("{}, got a memory/CPU query. {}", junior.display_name, short_id),
                format!("{}, resource usage ticket. {}", junior.display_name, short_id),
            ],
            Team::Services => vec![
                format!("Hey {}! Service question. Case {}", junior.display_name, short_id),
                format!("{}, systemd ticket for you. {}", junior.display_name, short_id),
                format!("{}, got a services request. {}", junior.display_name, short_id),
            ],
            Team::Hardware => vec![
                format!("Hey {}! Hardware question. Case {}", junior.display_name, short_id),
                format!("{}, device ticket coming in. {}", junior.display_name, short_id),
                format!("{}, got a hardware query. {}", junior.display_name, short_id),
            ],
            Team::Logs => vec![
                format!("Hey {}! Logs question. Case {}", junior.display_name, short_id),
                format!("{}, journal query for you. {}", junior.display_name, short_id),
                format!("{}, got a logs request. {}", junior.display_name, short_id),
            ],
            _ => vec![
                format!("Hey {}! New case {} coming your way.", junior.display_name, short_id),
                format!("{}, got a ticket for you. Case {}", junior.display_name, short_id),
                format!("{}, when you have a sec - new request. {}", junior.display_name, short_id),
                format!("Quick one for you, {}. Case {}", junior.display_name, short_id),
                format!("{}, incoming ticket. {}", junior.display_name, short_id),
            ],
        };

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
    pub fn junior_probing(&mut self, progress: &mut ProgressTracker, probe_count: usize) {
        self.probes_planned = probe_count;
        let junior = self.junior();

        // Team-specific probing messages
        let messages = match self.team {
            Team::Storage => vec![
                format!("Checking disk stats... {} probe{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
                format!("Running storage checks... {} command{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
            ],
            Team::Network => vec![
                format!("Testing connectivity... {} check{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
                format!("Running network probes... {} query{}.", probe_count, if probe_count == 1 { "y" } else { "ies" }),
            ],
            Team::Performance => vec![
                format!("Checking resource usage... {} probe{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
                format!("Running performance checks... {} command{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
                format!("Gathering memory/CPU data... {} check{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
            ],
            Team::Services => vec![
                format!("Checking service status... {} probe{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
                format!("Querying systemd... {} command{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
            ],
            Team::Hardware => vec![
                format!("Scanning hardware... {} probe{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
                format!("Checking device info... {} command{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
            ],
            Team::Logs => vec![
                format!("Searching logs... {} probe{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
                format!("Querying journal... {} command{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
            ],
            _ => vec![
                format!("Running {} check{}...", probe_count, if probe_count == 1 { "" } else { "s" }),
                format!("Gathering data... {} probe{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
                format!("Let me pull some numbers. {} check{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
                format!("Collecting system info... {} command{}.", probe_count, if probe_count == 1 { "" } else { "s" }),
            ],
        };

        let msg = &messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Probes, junior.display_name, msg);
    }

    /// Junior reports probe completion (v0.0.152)
    pub fn junior_probes_done(&self, progress: &mut ProgressTracker, success_count: usize) {
        let junior = self.junior();

        // Comment on how probes went
        let messages = if success_count == self.probes_planned && self.probes_planned > 0 {
            vec![
                format!("All {} probe{} succeeded.", success_count, if success_count == 1 { "" } else { "s" }),
                format!("Got all the data. {} check{} complete.", success_count, if success_count == 1 { "" } else { "s" }),
            ]
        } else if success_count > 0 {
            vec![
                format!("{} of {} probes returned data.", success_count, self.probes_planned),
                format!("Got partial data - {} probe{} worked.", success_count, if success_count == 1 { "" } else { "s" }),
            ]
        } else {
            vec![
                "Probes didn't return much. Working with what we have.".to_string(),
                "Limited data available. Doing my best.".to_string(),
            ]
        };

        let msg = &messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Probes, junior.display_name, msg);
    }

    /// Junior reviewing the answer
    pub fn junior_reviewing(&self, progress: &mut ProgressTracker) {
        let junior = self.junior();

        // Team-specific review messages
        let messages = match self.team {
            Team::Storage => vec![
                "Checking the disk numbers...",
                "Verifying storage calculations...",
                "Cross-referencing mount points...",
            ],
            Team::Network => vec![
                "Checking connectivity results...",
                "Verifying network data...",
                "Looking at the interface info...",
            ],
            Team::Performance => vec![
                "Checking memory/CPU numbers...",
                "Verifying resource calculations...",
                "Looking at the load data...",
            ],
            Team::Services => vec![
                "Checking service states...",
                "Verifying unit status...",
                "Looking at systemd output...",
            ],
            Team::Hardware => vec![
                "Checking device info...",
                "Verifying hardware data...",
                "Looking at the specs...",
            ],
            Team::Logs => vec![
                "Checking log entries...",
                "Verifying journal output...",
                "Looking for patterns...",
            ],
            _ => vec![
                "Checking the response...",
                "Verifying the data...",
                "Running quality checks...",
                "Looking good so far...",
                "Reviewing the output...",
            ],
        };

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
                format!("High confidence: {}%. Good to ship.", confidence),
                format!("Verified. {}% sure. Handing back.", confidence),
            ]
        } else if confidence >= 70 {
            vec![
                format!("Done. {}% confidence.", confidence),
                format!("Finished review. {}%.", confidence),
                format!("Reasonable confidence at {}%.", confidence),
            ]
        } else if confidence >= 50 {
            vec![
                format!("Best I can do is {}%. Sending it back.", confidence),
                format!("Done, but only {}% sure.", confidence),
                format!("Moderate confidence: {}%.", confidence),
            ]
        } else {
            vec![
                format!("Low confidence at {}%. User should verify.", confidence),
                format!("Only {}% sure. Take it with a grain of salt.", confidence),
            ]
        };

        let msg = &messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Supervisor, junior.display_name, msg);
    }

    /// Anna returning with the answer
    pub fn anna_returning(&self, progress: &mut ProgressTracker) {
        let junior = self.junior();

        let messages = vec![
            format!("Thanks {}! I'll take it from here.", junior.display_name),
            "Got it. Passing this along now.".to_string(),
            "Perfect, sending the response.".to_string(),
            format!("Appreciate it, {}. Sending to the user.", junior.display_name),
            "Response ready. Wrapping up.".to_string(),
        ];

        let msg = &messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Supervisor, "Anna", msg);
    }
}

/// Determine team from domain string
/// v0.0.154: Added Services, Hardware, Logs team routing
pub fn team_from_domain(domain: &str) -> Team {
    match domain.to_lowercase().as_str() {
        "storage" => Team::Storage,
        "network" => Team::Network,
        "security" => Team::Security,
        "performance" => Team::Performance,
        "system" => Team::Performance, // System queries often about performance
        "services" => Team::Services,
        "hardware" => Team::Hardware,
        "logs" => Team::Logs,
        "packages" => Team::Desktop, // Package management is desktop team
        "desktop" => Team::Desktop,
        _ => Team::Desktop, // Default to desktop for general queries
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
        assert_eq!(team_from_domain("performance"), Team::Performance);
        assert_eq!(team_from_domain("system"), Team::Performance);
        assert_eq!(team_from_domain("services"), Team::Services);
        assert_eq!(team_from_domain("hardware"), Team::Hardware);
        assert_eq!(team_from_domain("logs"), Team::Logs);
        assert_eq!(team_from_domain("unknown"), Team::Desktop);
    }
}
