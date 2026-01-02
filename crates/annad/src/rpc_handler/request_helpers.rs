//! Request helper functions (v0.0.291).
//!
//! Extracted from llm_request.rs to keep files under 400 lines.

use anna_shared::rpc::{ProbeResult, RpcResponse, RuntimeContext, SpecialistDomain, TranslatorTicket};
use anna_shared::status::{HardwareInfo, LlmState};
use anna_shared::transcript::Transcript;
use tracing::{info, warn};

use crate::service_desk;
use crate::state::SharedState;

/// Configuration extracted from state for request processing.
pub struct RequestConfig {
    pub llm_config: crate::config::LlmConfig,
    pub translator_model: String,
    pub specialist_model: String,
    pub hw_cores: u32,
    pub hw_ram_gb: f64,
    pub has_gpu: bool,
    pub debug_mode: bool,
    pub models_fully_ready: bool,
}

/// Fast path configuration.
pub struct FastPathConfig {
    pub enabled: bool,
    pub snapshot_max_age_secs: u64,
}

/// Extract configuration from state for request processing.
pub async fn extract_config(state: &SharedState, id: &str) -> Result<RequestConfig, RpcResponse> {
    let state_read = state.read().await;

    if !state_read.llm.state.can_handle_requests() {
        return Err(RpcResponse::error(
            id.to_string(),
            -32002,
            format!("LLM not ready: {}", state_read.llm.state),
        ));
    }

    let models_ready = state_read.llm.state == LlmState::Ready;

    Ok(RequestConfig {
        llm_config: state_read.config.llm.clone(),
        translator_model: state_read.config.llm.translator_model.clone(),
        specialist_model: state_read.config.llm.specialist_model.clone(),
        hw_cores: state_read.hardware.cpu_cores,
        hw_ram_gb: state_read.hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
        has_gpu: state_read.hardware.gpu.is_some(),
        debug_mode: state_read.config.debug_mode(),
        models_fully_ready: models_ready,
    })
}

/// Extract fast path configuration from state.
pub async fn extract_fast_path_config(state: &SharedState) -> FastPathConfig {
    let state_read = state.read().await;
    FastPathConfig {
        enabled: state_read.config.daemon.fast_path_enabled,
        snapshot_max_age_secs: state_read.config.daemon.snapshot_max_age_secs,
    }
}

/// Record completed request in stats.
pub async fn record_request_stats(
    state: &SharedState,
    total_ms: u64,
    used_deterministic: bool,
    translator_timed_out: bool,
    specialist_timeout: bool,
) {
    let mut state = state.write().await;
    state.latency.total.add(total_ms);
    state.record_request(used_deterministic, translator_timed_out, specialist_timeout);
}

/// Build context from hardware and probe results.
pub fn build_context(hardware: &HardwareInfo, probe_results: &[ProbeResult]) -> anna_shared::rpc::RuntimeContext {
    service_desk::build_context(hardware, probe_results)
}

/// Create no evidence response.
pub fn create_no_evidence_response(
    id: &str,
    request_id: String,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    classified_domain: SpecialistDomain,
    required_evidence: &[String],
) -> RpcResponse {
    let result = service_desk::create_no_evidence_response(
        request_id,
        ticket,
        probe_results,
        transcript,
        classified_domain,
        required_evidence,
    );

    match serde_json::to_value(result) {
        Ok(v) => RpcResponse::success(id.to_string(), v),
        Err(e) => RpcResponse::error(
            id.to_string(),
            -32603,
            format!("Serialization error: {}", e),
        ),
    }
}

/// Create no data response.
pub fn create_no_data_response(
    id: &str,
    request_id: String,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    classified_domain: SpecialistDomain,
) -> RpcResponse {
    let result = service_desk::create_no_data_response(
        request_id,
        ticket,
        probe_results,
        transcript,
        classified_domain,
    );

    match serde_json::to_value(result) {
        Ok(v) => RpcResponse::success(id.to_string(), v),
        Err(e) => RpcResponse::error(
            id.to_string(),
            -32603,
            format!("Serialization error: {}", e),
        ),
    }
}

/// Create timeout response for probe stage.
pub fn create_probe_timeout_response(
    id: &str,
    request_id: String,
    ticket: TranslatorTicket,
    transcript: Transcript,
    classified_domain: SpecialistDomain,
) -> RpcResponse {
    let result = service_desk::create_timeout_response(
        request_id,
        "probes",
        Some(ticket),
        vec![],
        transcript,
        classified_domain,
    );

    match serde_json::to_value(result) {
        Ok(v) => RpcResponse::success(id.to_string(), v),
        Err(e) => RpcResponse::error(
            id.to_string(),
            -32603,
            format!("Serialization error: {}", e),
        ),
    }
}

/// Create clarification response.
pub fn create_clarification_response(
    id: &str,
    request_id: String,
    ticket: TranslatorTicket,
    question: &str,
    transcript: Transcript,
) -> RpcResponse {
    let result = service_desk::create_clarification_response(
        request_id,
        ticket,
        question,
        transcript,
    );

    match serde_json::to_value(result) {
        Ok(v) => RpcResponse::success(id.to_string(), v),
        Err(e) => RpcResponse::error(
            id.to_string(),
            -32603,
            format!("Serialization error: {}", e),
        ),
    }
}

/// Save truth ledger to disk.
pub async fn save_truth_ledger(state: &SharedState) {
    let state_read = state.read().await;
    if let Err(e) = state_read.truth_ledger.save(crate::state::TRUTH_LEDGER_PATH) {
        warn!("Failed to save truth ledger: {}", e);
    }
}

/// Log request completion.
pub fn log_request_completion(
    domain: &str,
    reliability_score: u8,
    used_deterministic: bool,
    execution_trace: Option<String>,
    total_ms: u64,
) {
    info!(
        "Request completed: domain={}, reliability={}, deterministic={}, trace={}, latency={}ms",
        domain,
        reliability_score,
        used_deterministic,
        execution_trace.unwrap_or_default(),
        total_ms
    );
}
