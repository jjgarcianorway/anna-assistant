//! Probe handler - manages probe execution and evidence checking (v0.0.291).
//!
//! Extracted from llm_request.rs to keep files under 400 lines.

use anna_shared::rpc::{ProbeResult, RpcResponse, SpecialistDomain, TranslatorTicket};
use tracing::info;

use crate::comms::CommsGenerator;
use crate::probe_stage::{check_evidence_validity, execute_probe_stage};
use crate::progress_tracker::ProgressTracker;
use crate::router::DeterministicRoute;
use crate::state::SharedState;

use super::helpers::save_progress;
use super::request_helpers::{create_no_evidence_response, create_probe_timeout_response, RequestConfig};

/// Execute probe stage and handle timeouts.
/// Returns (probe_results, timeout_response) - if timeout_response is Some, return it immediately.
pub async fn execute_and_handle_probes(
    id: &str,
    request_id: &str,
    state: &SharedState,
    ticket: &TranslatorTicket,
    config: &RequestConfig,
    progress: &mut ProgressTracker,
    comms: &mut CommsGenerator,
    classified_domain: SpecialistDomain,
) -> (Vec<ProbeResult>, Option<RpcResponse>) {
    let probe_stage_result =
        execute_probe_stage(state, ticket, &config.llm_config, progress, comms).await;

    // Handle probe timeout
    if probe_stage_result.timed_out {
        let response = create_probe_timeout_response(
            id,
            request_id.to_string(),
            ticket.clone(),
            progress.transcript_clone(),
            classified_domain,
        );
        return (vec![], Some(response));
    }

    (probe_stage_result.results, None)
}

/// Check evidence validity and handle no evidence case.
/// Returns Some(RpcResponse) if evidence check failed, None otherwise.
pub async fn check_and_handle_evidence(
    id: &str,
    request_id: &str,
    det_route: &DeterministicRoute,
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    progress: &mut ProgressTracker,
    classified_domain: SpecialistDomain,
    state: &SharedState,
) -> Option<RpcResponse> {
    let valid_evidence_count = check_evidence_validity(probe_results);

    if det_route.capability.evidence_required && valid_evidence_count == 0 {
        info!("v0.45.7: No valid evidence collected but evidence required - returning deterministic failure");
        save_progress(state, progress).await;

        let required_evidence: Vec<String> = det_route
            .capability
            .required_evidence
            .iter()
            .map(|k| k.to_string())
            .collect();

        return Some(create_no_evidence_response(
            id,
            request_id.to_string(),
            ticket.clone(),
            probe_results.to_vec(),
            progress.transcript_clone(),
            classified_domain,
            &required_evidence,
        ));
    }

    None
}
