//! Synchronous review and completion message generation (v0.0.401).
//! Team-specific message variants for review, escalation, and completion stages.
//! Includes relationship-aware escalation (v0.0.262).

use anna_shared::progress::RequestStage;
use anna_shared::roster::{escalation_phrase, senior_response_phrase};
use anna_shared::teams::Team;

use crate::progress_tracker::ProgressTracker;

use super::generator::CommsGenerator;

impl CommsGenerator {
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

    /// Junior escalating to senior (v0.0.262: relationship-aware)
    pub fn junior_escalate(&self, progress: &mut ProgressTracker, reason: &str) {
        let junior = self.junior();
        let senior = self.senior();

        // Get relationship-aware escalation phrase
        let phrase_template = escalation_phrase(junior.person_id, senior.person_id, self.seed);
        let phrase = phrase_template.replace("{senior}", senior.display_name);

        let msg = format!("{} {}", phrase, reason);
        progress.add_internal_comms(RequestStage::Supervisor, junior.display_name, &msg);
    }

    /// Senior responding to escalation (v0.0.262: relationship-aware)
    pub fn senior_response(&self, progress: &mut ProgressTracker, helpful: bool) {
        let junior = self.junior();
        let senior = self.senior();

        // Get relationship-aware response phrase
        let phrase_template =
            senior_response_phrase(senior.person_id, junior.person_id, helpful, self.seed);
        let phrase = phrase_template.replace("{junior}", junior.display_name);

        progress.add_internal_comms(RequestStage::Supervisor, senior.display_name, &phrase);
    }

    /// Junior confirms answer is ready
    pub fn junior_done(&self, progress: &mut ProgressTracker, confidence: u8) {
        let junior = self.junior();
        let messages = if confidence >= 90 {
            vec![
                format!(
                    "All good! {}% confidence. Sending back to Anna.",
                    confidence
                ),
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

        let messages = [
            format!("Thanks {}! I'll take it from here.", junior.display_name),
            "Got it. Passing this along now.".to_string(),
            "Perfect, sending the response.".to_string(),
            format!(
                "Appreciate it, {}. Sending to the user.",
                junior.display_name
            ),
            "Response ready. Wrapping up.".to_string(),
        ];

        let msg = &messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Supervisor, "Anna", msg);
    }
}
