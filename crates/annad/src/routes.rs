//! API routes for annad

use crate::probe::executor::ProbeExecutor;
use crate::server::AppState;
use anna_common::{
    GetStateRequest, HealthResponse, InvalidateRequest, ListProbesResponse, ProbeInfo,
    ProbeResult, RunProbeRequest, SetStateRequest, StateResponse,
};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tracing::{error, info};

type AppStateArc = Arc<AppState>;

// ============================================================================
// Probe Routes
// ============================================================================

pub fn probe_routes() -> Router<AppStateArc> {
    Router::new()
        .route("/v1/probe/run", post(run_probe))
        .route("/v1/probe/list", get(list_probes))
}

async fn run_probe(
    State(state): State<AppStateArc>,
    Json(req): Json<RunProbeRequest>,
) -> Result<Json<ProbeResult>, (StatusCode, String)> {
    info!("  Running probe: {}", req.probe_id);

    let registry = state.probe_registry.read().await;
    let mut state_mgr = state.state_manager.write().await;

    // Check cache first (unless force refresh)
    if !req.force_refresh {
        if let Some(cached) = state_mgr.get_probe_cache(&req.probe_id) {
            info!("  Cache hit for {}", req.probe_id);
            return Ok(Json(cached));
        }
    }

    // Get probe definition
    let probe_def = registry.get(&req.probe_id).ok_or_else(|| {
        error!("  Probe not found: {}", req.probe_id);
        (
            StatusCode::NOT_FOUND,
            format!("Probe '{}' not found", req.probe_id),
        )
    })?;

    // Execute probe
    let result = ProbeExecutor::execute(probe_def).await.map_err(|e| {
        error!("  Probe execution failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    // Cache result
    state_mgr.set_probe_cache(&req.probe_id, &result, probe_def.cache_policy);

    Ok(Json(result))
}

async fn list_probes(State(state): State<AppStateArc>) -> Json<ListProbesResponse> {
    let registry = state.probe_registry.read().await;

    let probes: Vec<ProbeInfo> = registry
        .list()
        .iter()
        .map(|def| ProbeInfo {
            id: def.id.clone(),
            parser: def.parser.clone(),
            cache_policy: format!("{:?}", def.cache_policy),
        })
        .collect();

    Json(ListProbesResponse { probes })
}

// ============================================================================
// State Routes
// ============================================================================

pub fn state_routes() -> Router<AppStateArc> {
    Router::new()
        .route("/v1/state/get", post(get_state))
        .route("/v1/state/set", post(set_state))
        .route("/v1/state/invalidate", post(invalidate_state))
}

async fn get_state(
    State(state): State<AppStateArc>,
    Json(req): Json<GetStateRequest>,
) -> Json<StateResponse> {
    let state_mgr = state.state_manager.read().await;
    let value = state_mgr.get(&req.key);

    Json(StateResponse {
        key: req.key,
        found: value.is_some(),
        value,
    })
}

async fn set_state(
    State(state): State<AppStateArc>,
    Json(req): Json<SetStateRequest>,
) -> Json<StateResponse> {
    let mut state_mgr = state.state_manager.write().await;
    state_mgr.set(&req.key, req.value.clone(), req.ttl_seconds);

    Json(StateResponse {
        key: req.key,
        found: true,
        value: Some(req.value),
    })
}

async fn invalidate_state(
    State(state): State<AppStateArc>,
    Json(req): Json<InvalidateRequest>,
) -> StatusCode {
    let mut state_mgr = state.state_manager.write().await;

    if let Some(key) = req.key {
        state_mgr.invalidate(&key);
    } else if let Some(pattern) = req.pattern {
        state_mgr.invalidate_pattern(&pattern);
    } else {
        state_mgr.clear();
    }

    StatusCode::OK
}

// ============================================================================
// Health Routes
// ============================================================================

pub fn health_routes() -> Router<AppStateArc> {
    Router::new().route("/v1/health", get(health_check))
}

async fn health_check(State(state): State<AppStateArc>) -> Json<HealthResponse> {
    let registry = state.probe_registry.read().await;

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        probes_available: registry.count(),
    })
}
