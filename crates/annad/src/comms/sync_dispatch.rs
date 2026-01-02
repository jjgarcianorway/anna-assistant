//! Synchronous dispatch and probe message generation (v0.0.401).
//! Team-specific message variants for dispatch and probing stages.

use anna_shared::dialogue::junior_acknowledgment;
use anna_shared::progress::RequestStage;
use anna_shared::teams::Team;

use crate::progress_tracker::ProgressTracker;

use super::generator::CommsGenerator;

impl CommsGenerator {
    /// Anna dispatches the case to a team member
    pub fn dispatch(&self, progress: &mut ProgressTracker) {
        let junior = self.junior();
        let short_id = &self.case_id[..8.min(self.case_id.len())];

        // Team-specific dispatch messages
        let greetings = match self.team {
            Team::Storage => vec![
                format!(
                    "Hey {}! Disk question coming in. Case {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, storage ticket for you. {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, got a disk/storage request. {}",
                    junior.display_name, short_id
                ),
            ],
            Team::Network => vec![
                format!(
                    "Hey {}! Network query. Case {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, connectivity question. {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, network ticket incoming. {}",
                    junior.display_name, short_id
                ),
            ],
            Team::Security => vec![
                format!(
                    "Hey {}! Security matter. Case {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, access/security question. {}",
                    junior.display_name, short_id
                ),
                format!("{}, security ticket. {}", junior.display_name, short_id),
            ],
            Team::Performance => vec![
                format!(
                    "Hey {}! Performance question. Case {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, got a memory/CPU query. {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, resource usage ticket. {}",
                    junior.display_name, short_id
                ),
            ],
            Team::Services => vec![
                format!(
                    "Hey {}! Service question. Case {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, systemd ticket for you. {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, got a services request. {}",
                    junior.display_name, short_id
                ),
            ],
            Team::Hardware => vec![
                format!(
                    "Hey {}! Hardware question. Case {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, device ticket coming in. {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, got a hardware query. {}",
                    junior.display_name, short_id
                ),
            ],
            Team::Logs => vec![
                format!(
                    "Hey {}! Logs question. Case {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, journal query for you. {}",
                    junior.display_name, short_id
                ),
                format!("{}, got a logs request. {}", junior.display_name, short_id),
            ],
            _ => vec![
                format!(
                    "Hey {}! New case {} coming your way.",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, got a ticket for you. Case {}",
                    junior.display_name, short_id
                ),
                format!(
                    "{}, when you have a sec - new request. {}",
                    junior.display_name, short_id
                ),
                format!(
                    "Quick one for you, {}. Case {}",
                    junior.display_name, short_id
                ),
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
                format!(
                    "Checking disk stats... {} probe{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Running storage checks... {} command{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
            ],
            Team::Network => vec![
                format!(
                    "Testing connectivity... {} check{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Running network probes... {} query{}.",
                    probe_count,
                    if probe_count == 1 { "y" } else { "ies" }
                ),
            ],
            Team::Performance => vec![
                format!(
                    "Checking resource usage... {} probe{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Running performance checks... {} command{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Gathering memory/CPU data... {} check{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
            ],
            Team::Services => vec![
                format!(
                    "Checking service status... {} probe{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Querying systemd... {} command{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
            ],
            Team::Hardware => vec![
                format!(
                    "Scanning hardware... {} probe{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Checking device info... {} command{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
            ],
            Team::Logs => vec![
                format!(
                    "Searching logs... {} probe{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Querying journal... {} command{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
            ],
            _ => vec![
                format!(
                    "Running {} check{}...",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Gathering data... {} probe{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Let me pull some numbers. {} check{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Collecting system info... {} command{}.",
                    probe_count,
                    if probe_count == 1 { "" } else { "s" }
                ),
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
                format!(
                    "All {} probe{} succeeded.",
                    success_count,
                    if success_count == 1 { "" } else { "s" }
                ),
                format!(
                    "Got all the data. {} check{} complete.",
                    success_count,
                    if success_count == 1 { "" } else { "s" }
                ),
            ]
        } else if success_count > 0 {
            vec![
                format!(
                    "{} of {} probes returned data.",
                    success_count, self.probes_planned
                ),
                format!(
                    "Got partial data - {} probe{} worked.",
                    success_count,
                    if success_count == 1 { "" } else { "s" }
                ),
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
}
