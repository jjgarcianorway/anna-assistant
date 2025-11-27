//! API routes for annad
//!
//! v0.9.0: Added /v1/update/state endpoint for status command
//! v0.10.0: Added /v1/answer endpoint for evidence-based answers
//! v0.11.0: Added /v1/knowledge routes for fact queries

use crate::orchestrator::AnswerEngine;
use crate::probe::executor::ProbeExecutor;
use crate::server::AppState;
use anna_common::{
    load_update_state, AnnaConfigV5, Fact, FactQuery, FinalAnswer, GetStateRequest, HealthResponse,
    InvalidateRequest, ListProbesResponse, ProbeInfo, ProbeResult, RunProbeRequest,
    SetStateRequest, StateResponse, UpdateStateResponse,
};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
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

// ============================================================================
// Update Routes (v0.9.0)
// ============================================================================

pub fn update_routes() -> Router<AppStateArc> {
    Router::new().route("/v1/update/state", get(update_state))
}

/// v0.9.0: Get current update state for status command
async fn update_state(State(_state): State<AppStateArc>) -> Json<UpdateStateResponse> {
    // Load update state from persistent storage
    let update_state = load_update_state();
    let config = AnnaConfigV5::load();

    // Build response from current state
    let last_check = update_state.last_check.map(|ts| {
        chrono::DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "unknown".to_string())
    });

    // v0.9.0: next_retry not yet implemented, derive from failed update timestamp
    let next_retry = update_state.last_failed_update.map(|ts| {
        // Default retry after 10 minutes from last failure
        let retry_ts = ts + 600;
        chrono::DateTime::from_timestamp(retry_ts, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "unknown".to_string())
    });

    // Determine status description
    let status = if !config.is_auto_update_enabled() {
        "disabled".to_string()
    } else if update_state.last_check.is_none() {
        "never_checked".to_string()
    } else {
        match &update_state.last_result {
            Some(r) => format!("{:?}", r).to_lowercase(),
            None => "idle".to_string(),
        }
    };

    Json(UpdateStateResponse {
        latest_version: update_state.latest_available_version.clone(),
        status,
        last_check,
        download_in_progress: false, // v0.9.0: Not yet tracking in-progress downloads
        download_progress_bytes: None,
        download_total_bytes: None,
        ready_to_apply: false,
        daemon_busy: false, // v0.9.0: Not yet tracking busy state
        next_retry,
        last_failure: update_state.last_failure_reason.clone(),
    })
}

// ============================================================================
// Answer Routes (v0.10.0)
// ============================================================================

/// Request to get an evidence-based answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerRequest {
    pub question: String,
}

pub fn answer_routes() -> Router<AppStateArc> {
    Router::new().route("/v1/answer", post(answer_question))
}

/// v0.10.0: Process a question through the LLM-A/LLM-B audit loop
async fn answer_question(
    State(state): State<AppStateArc>,
    Json(req): Json<AnswerRequest>,
) -> Result<Json<FinalAnswer>, (StatusCode, String)> {
    info!("Processing question: {}", req.question);

    // v0.11.0: Record query for telemetry
    if let Some(brain) = &state.brain {
        brain.record_query(&req.question).await;
    }

    // Get models from config - supports role-specific models for junior/senior
    // Priority: explicit config > hardware auto-detection > defaults
    let config = AnnaConfigV5::load();
    let (junior_model, senior_model) = if !config.llm.needs_role_model_migration() {
        // Explicit junior/senior models configured - always use them
        let junior = config.llm.get_junior_model().to_string();
        let senior = config.llm.get_senior_model().to_string();
        info!(
            "🎯  Using configured models - Junior: {}, Senior: {}",
            junior, senior
        );
        (Some(junior), Some(senior))
    } else if config.llm.selection_mode.as_str() == "manual" {
        // Manual mode without role-specific: use preferred_model for both
        let model = config.llm.preferred_model.clone();
        info!("🎯  Using manual mode - Model: {} (for both roles)", model);
        (Some(model.clone()), Some(model))
    } else {
        // Auto selection - use hardware profile (no explicit models configured)
        let profile = anna_common::HardwareProfile::detect();
        let recommendation = profile.select_model();
        info!(
            "🎯  Auto-selected model: {} (reason: {}, GPU: {:?}, VRAM: {:?}GB, driver_functional: {})",
            recommendation.model,
            recommendation.reason,
            profile.gpu_vendor,
            profile.vram_gb,
            profile.gpu_driver_functional
        );
        (
            Some(recommendation.model.clone()),
            Some(recommendation.model),
        )
    };

    let engine = AnswerEngine::with_role_models(junior_model, senior_model);
    info!(
        "🤖  Engine ready - Junior: {}, Senior: {}",
        engine.junior_model(),
        engine.senior_model()
    );

    // Check if LLM is available
    if !engine.is_available().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "LLM backend (Ollama) is not available".to_string(),
        ));
    }

    // Process the question
    match engine.process_question(&req.question).await {
        Ok(answer) => {
            info!(
                "Answer ready: confidence={} ({:?})",
                answer.scores.overall, answer.confidence_level
            );
            Ok(Json(answer))
        }
        Err(e) => {
            error!("Failed to process question: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

// ============================================================================
// Knowledge Routes (v0.11.0)
// ============================================================================

/// Request to query facts from knowledge store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeQueryRequest {
    /// Entity prefix filter (optional)
    pub entity_prefix: Option<String>,
    /// Entity exact match (optional)
    pub entity: Option<String>,
    /// Attribute filter (optional)
    pub attribute: Option<String>,
    /// Minimum confidence (default: 0.0)
    pub min_confidence: Option<f64>,
    /// Only active facts (default: true)
    pub active_only: Option<bool>,
    /// Maximum results (default: 100)
    pub limit: Option<usize>,
}

/// Response containing queried facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeQueryResponse {
    pub facts: Vec<Fact>,
    pub total_count: usize,
}

/// Response with knowledge store stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStatsResponse {
    pub fact_count: usize,
    pub entity_types: Vec<String>,
    pub last_updated: Option<String>,
}

pub fn knowledge_routes() -> Router<AppStateArc> {
    Router::new()
        .route("/v1/knowledge/query", post(knowledge_query))
        .route("/v1/knowledge/stats", get(knowledge_stats))
}

/// v0.11.0: Query facts from knowledge store
async fn knowledge_query(
    State(state): State<AppStateArc>,
    Json(req): Json<KnowledgeQueryRequest>,
) -> Result<Json<KnowledgeQueryResponse>, (StatusCode, String)> {
    let brain = state.brain.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Knowledge store not initialized".to_string(),
        )
    })?;

    let store = brain.store.read().await;

    // Build query from request
    let mut query = FactQuery::new();

    if let Some(prefix) = &req.entity_prefix {
        query = query.entity_prefix(prefix);
    }

    if let Some(entity) = &req.entity {
        query = query.entity(entity);
    }

    if let Some(attr) = &req.attribute {
        query = query.attribute(attr);
    }

    if let Some(conf) = req.min_confidence {
        query = query.min_confidence(conf);
    }

    if req.active_only.unwrap_or(true) {
        query = query.active_only();
    }

    query = query.limit(req.limit.unwrap_or(100));

    // Execute query
    let facts = store.query(&query).map_err(|e| {
        error!("Knowledge query failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(KnowledgeQueryResponse {
        total_count: facts.len(),
        facts,
    }))
}

/// v0.11.0: Get knowledge store statistics
async fn knowledge_stats(
    State(state): State<AppStateArc>,
) -> Result<Json<KnowledgeStatsResponse>, (StatusCode, String)> {
    let brain = state.brain.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Knowledge store not initialized".to_string(),
        )
    })?;

    let store = brain.store.read().await;

    let fact_count = store
        .count()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Get unique entity types
    let all_facts = store
        .query(&FactQuery::new().limit(1000))
        .unwrap_or_default();
    let mut entity_types: Vec<String> = all_facts
        .iter()
        .map(|f| f.entity.split(':').next().unwrap_or("unknown").to_string())
        .collect();
    entity_types.sort();
    entity_types.dedup();

    // Get most recent update
    let last_updated = all_facts
        .iter()
        .map(|f| f.last_seen)
        .max()
        .map(|dt| dt.to_rfc3339());

    Ok(Json(KnowledgeStatsResponse {
        fact_count,
        entity_types,
        last_updated,
    }))
}
