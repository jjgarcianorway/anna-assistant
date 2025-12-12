//! Probe execution stage for the RPC handler pipeline.
//!
//! Extracted from rpc_handler.rs (v0.0.165) for modularization.
//! v0.0.254: Updated to use async LLM-powered dialogue.

use anna_shared::progress::RequestStage;
use anna_shared::rpc::{ProbeResult, TranslatorTicket};
use std::time::Instant;
use tokio::time::{timeout, Duration};

use crate::config::LlmConfig;

use crate::comms::CommsGenerator;
use crate::probe_runner;
use crate::progress_tracker::ProgressTracker;
use crate::state::SharedState;

/// Result of probe stage execution
pub struct ProbeStageResult {
    /// Probe results (empty if timeout)
    pub results: Vec<ProbeResult>,
    /// Whether the stage timed out
    pub timed_out: bool,
    /// Count of successful probes
    pub success_count: usize,
}

/// Execute probe stage with timeout and progress tracking
pub async fn execute_probe_stage(
    state: &SharedState,
    ticket: &TranslatorTicket,
    llm_config: &LlmConfig,
    progress: &mut ProgressTracker,
    comms: &mut CommsGenerator,
) -> ProbeStageResult {
    progress.start_stage(RequestStage::Probes, llm_config.probes_total_timeout_secs);

    // Report probe progress if we have probes to run
    if !ticket.needs_probes.is_empty() {
        comms
            .junior_probing_async(progress, ticket.needs_probes.len())
            .await;
        save_progress(state, progress).await;
    }

    let probes_start = Instant::now();

    match timeout(
        Duration::from_secs(llm_config.probes_total_timeout_secs),
        probe_runner::run_probes(state, ticket, llm_config, progress),
    )
    .await
    {
        Ok(results) => {
            progress.complete_stage(RequestStage::Probes);

            // Record probes latency
            {
                state
                    .write()
                    .await
                    .latency
                    .probes
                    .add(probes_start.elapsed().as_millis() as u64);
            }

            // Count successful probes
            let success_count = results.iter().filter(|p| p.exit_code == 0).count();
            comms
                .junior_probes_done_async(progress, success_count)
                .await;
            save_progress(state, progress).await;

            ProbeStageResult {
                results,
                timed_out: false,
                success_count,
            }
        }
        Err(_) => {
            progress.timeout_stage(RequestStage::Probes);
            save_progress(state, progress).await;

            ProbeStageResult {
                results: vec![],
                timed_out: true,
                success_count: 0,
            }
        }
    }
}

/// Check if probes collected valid evidence
pub fn check_evidence_validity(probe_results: &[ProbeResult]) -> usize {
    use anna_shared::parsers::parse_probe_result;
    probe_results
        .iter()
        .filter(|p| parse_probe_result(p).is_valid_evidence())
        .count()
}

/// Save progress events to state for polling
async fn save_progress(state: &SharedState, progress: &ProgressTracker) {
    state.write().await.progress_events = progress.events().to_vec();
}
