//! Async LLM-powered message generation functions (v0.0.254).
//! These try LLM generation first, falling back to static messages on failure.
//! v0.0.401: Includes subtle learning hints when applicable.

use anna_shared::dialogue::junior_acknowledgment;
use anna_shared::progress::RequestStage;

use crate::learning_capture::get_learning_hint;
use crate::progress_tracker::ProgressTracker;

use super::dialogue_gen;
use super::generator::CommsGenerator;

impl CommsGenerator {
    /// v0.0.254: Anna dispatches with LLM-generated or static fallback
    /// v0.0.401: Now includes subtle learning hints when applicable
    pub async fn dispatch_async(&self, progress: &mut ProgressTracker) {
        let junior = self.junior();
        let short_id = &self.case_id[..8.min(self.case_id.len())];

        // v0.0.401: Check for learning hints from previous similar cases
        if let Some(hint) = get_learning_hint(&self.query) {
            progress.add_internal_comms(RequestStage::Translator, "Anna", &hint);
        }

        // Try LLM generation if model is configured
        if let Some(ref model) = self.dialogue_model {
            if let Some(msg) =
                dialogue_gen::gen_dispatch(model, &junior, &self.case_id, &self.query).await
            {
                progress.add_internal_comms(RequestStage::Translator, "Anna", &msg);
                return;
            }
        }

        // Simple fallback
        let msg = format!(
            "Hey {}! Case {} coming your way.",
            junior.display_name, short_id
        );
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
    pub async fn junior_probing_async(
        &mut self,
        progress: &mut ProgressTracker,
        probe_count: usize,
    ) {
        self.probes_planned = probe_count;
        let junior = self.junior();

        if let Some(ref model) = self.dialogue_model {
            if let Some(msg) = dialogue_gen::gen_junior_probing(model, &junior, probe_count).await {
                progress.add_internal_comms(RequestStage::Probes, junior.display_name, &msg);
                return;
            }
        }

        // Fallback
        let msg = format!(
            "Running {} check{}...",
            probe_count,
            if probe_count == 1 { "" } else { "s" }
        );
        progress.add_internal_comms(RequestStage::Probes, junior.display_name, &msg);
    }

    /// v0.0.254: Junior probes done with LLM or static fallback
    pub async fn junior_probes_done_async(
        &self,
        progress: &mut ProgressTracker,
        success_count: usize,
    ) {
        let junior = self.junior();

        if let Some(ref model) = self.dialogue_model {
            if let Some(msg) = dialogue_gen::gen_junior_probes_done(
                model,
                &junior,
                success_count,
                self.probes_planned,
            )
            .await
            {
                progress.add_internal_comms(RequestStage::Probes, junior.display_name, &msg);
                return;
            }
        }

        // Fallback
        let msg = if success_count == self.probes_planned && self.probes_planned > 0 {
            format!(
                "All {} probe{} succeeded.",
                success_count,
                if success_count == 1 { "" } else { "s" }
            )
        } else if success_count > 0 {
            format!(
                "{} of {} probes returned data.",
                success_count, self.probes_planned
            )
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
