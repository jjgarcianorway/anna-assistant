//! Message generation functions (v0.0.262).
//!
//! Team-specific message variants for dispatch, probing, reviewing, etc.
//! v0.0.254: Added async LLM-powered dialogue with static fallback.
//! v0.0.262: Added relationship-aware escalation dialogue.

use anna_shared::dialogue::junior_acknowledgment;
use anna_shared::progress::RequestStage;
use anna_shared::roster::{escalation_phrase, senior_response_phrase};
use anna_shared::teams::Team;

use crate::progress_tracker::ProgressTracker;

use super::dialogue_gen;
use super::generator::CommsGenerator;

impl CommsGenerator {
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

        let messages = [
            format!("Thanks {}! I'll take it from here.", junior.display_name),
            "Got it. Passing this along now.".to_string(),
            "Perfect, sending the response.".to_string(),
            format!("Appreciate it, {}. Sending to the user.", junior.display_name),
            "Response ready. Wrapping up.".to_string(),
        ];

        let msg = &messages[(self.seed as usize) % messages.len()];
        progress.add_internal_comms(RequestStage::Supervisor, "Anna", msg);
    }

    // === v0.0.254: Async LLM-powered dialogue methods ===
    // These try LLM generation first, falling back to static messages on failure.

    /// v0.0.254: Anna dispatches with LLM-generated or static fallback
    pub async fn dispatch_async(&self, progress: &mut ProgressTracker) {
        let junior = self.junior();
        let short_id = &self.case_id[..8.min(self.case_id.len())];

        // Try LLM generation if model is configured
        if let Some(ref model) = self.dialogue_model {
            if let Some(msg) = dialogue_gen::gen_dispatch(model, &junior, &self.case_id, &self.query).await {
                progress.add_internal_comms(RequestStage::Translator, "Anna", &msg);
                return;
            }
        }

        // Simple fallback
        let msg = format!("Hey {}! Case {} coming your way.", junior.display_name, short_id);
        progress.add_internal_comms(RequestStage::Translator, "Anna", &msg);
    }

    /// v0.0.254: Junior acknowledges with LLM or static fallback
    pub async fn junior_ack_async(&self, progress: &mut ProgressTracker) {
        let junior = self.junior();

        if let Some(ref model) = self.dialogue_model {
            if let Some(msg) = dialogue_gen::gen_junior_ack(model, &junior, &self.query).await {
                progress.add_internal_comms(RequestStage::Translator, junior.display_name, &msg);
                return;
            }
        }

        let ack = junior_acknowledgment(self.team, self.seed);
        progress.add_internal_comms(RequestStage::Translator, junior.display_name, &ack);
    }

    /// v0.0.254: Junior probing with LLM or static fallback
    pub async fn junior_probing_async(&mut self, progress: &mut ProgressTracker, probe_count: usize) {
        self.probes_planned = probe_count;
        let junior = self.junior();

        if let Some(ref model) = self.dialogue_model {
            if let Some(msg) = dialogue_gen::gen_junior_probing(model, &junior, probe_count).await {
                progress.add_internal_comms(RequestStage::Probes, junior.display_name, &msg);
                return;
            }
        }

        // Fallback
        let msg = format!("Running {} check{}...", probe_count, if probe_count == 1 { "" } else { "s" });
        progress.add_internal_comms(RequestStage::Probes, junior.display_name, &msg);
    }

    /// v0.0.254: Junior probes done with LLM or static fallback
    pub async fn junior_probes_done_async(&self, progress: &mut ProgressTracker, success_count: usize) {
        let junior = self.junior();

        if let Some(ref model) = self.dialogue_model {
            if let Some(msg) = dialogue_gen::gen_junior_probes_done(model, &junior, success_count, self.probes_planned).await {
                progress.add_internal_comms(RequestStage::Probes, junior.display_name, &msg);
                return;
            }
        }

        // Fallback
        let msg = if success_count == self.probes_planned && self.probes_planned > 0 {
            format!("All {} probe{} succeeded.", success_count, if success_count == 1 { "" } else { "s" })
        } else if success_count > 0 {
            format!("{} of {} probes returned data.", success_count, self.probes_planned)
        } else {
            "Limited data available. Doing my best.".to_string()
        };
        progress.add_internal_comms(RequestStage::Probes, junior.display_name, &msg);
    }

    /// v0.0.254: Junior reviewing with LLM or static fallback
    pub async fn junior_reviewing_async(&self, progress: &mut ProgressTracker) {
        let junior = self.junior();

        if let Some(ref model) = self.dialogue_model {
            if let Some(msg) = dialogue_gen::gen_junior_reviewing(model, &junior).await {
                progress.add_internal_comms(RequestStage::Specialist, junior.display_name, &msg);
                return;
            }
        }

        let msg = "Checking the response...";
        progress.add_internal_comms(RequestStage::Specialist, junior.display_name, msg);
    }

    /// v0.0.254: Junior done with LLM or static fallback
    pub async fn junior_done_async(&self, progress: &mut ProgressTracker, confidence: u8) {
        let junior = self.junior();

        if let Some(ref model) = self.dialogue_model {
            if let Some(msg) = dialogue_gen::gen_junior_done(model, &junior, confidence).await {
                progress.add_internal_comms(RequestStage::Supervisor, junior.display_name, &msg);
                return;
            }
        }

        // Fallback
        let msg = format!("Done. {}% confidence.", confidence);
        progress.add_internal_comms(RequestStage::Supervisor, junior.display_name, &msg);
    }

    /// v0.0.254: Anna returning with LLM or static fallback
    pub async fn anna_returning_async(&self, progress: &mut ProgressTracker) {
        let junior = self.junior();

        if let Some(ref model) = self.dialogue_model {
            if let Some(msg) = dialogue_gen::gen_anna_returning(model, &junior).await {
                progress.add_internal_comms(RequestStage::Supervisor, "Anna", &msg);
                return;
            }
        }

        let msg = format!("Thanks {}! I'll take it from here.", junior.display_name);
        progress.add_internal_comms(RequestStage::Supervisor, "Anna", &msg);
    }
}
