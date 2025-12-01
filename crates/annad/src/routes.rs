//! API routes for annad
//!
//! v0.9.0: Added /v1/update/state endpoint for status command
//! v0.10.0: Added /v1/answer endpoint for evidence-based answers
//! v0.11.0: Added /v1/knowledge routes for fact queries
//! v0.65.0: Added /v1/stats endpoint and stats recording for answer routes
//! v0.90.0: Unified Architecture - replaced AnswerEngine with UnifiedEngine

use crate::orchestrator::streaming::{create_channel_emitter, response::debug_stream_response, ChannelEmitter};
use crate::orchestrator::UnifiedEngine;  // v0.90.0 unified architecture
use crate::probe::executor::ProbeExecutor;
use crate::server::AppState;
use anna_common::{
    load_update_state, AnnaConfigV5, DebugEvent, DebugEventData, DebugEventEmitter, DebugEventType,
    Fact, FactQuery, FinalAnswer, GetStateRequest, HealthResponse, InvalidateRequest,
    ListProbesResponse, PerformanceSnapshot, ProbeInfo, ProbeResult, RunProbeRequest,
    SetStateRequest, StateResponse, UpdateStateResponse,
    // v1.1.0: Telemetry imports
    telemetry_record_success, telemetry_record_failure, telemetry_record_refusal,
    TelemetryOrigin,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

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

    // v0.16.5: Collect probe names for detailed status display
    let mut probe_names: Vec<String> = registry.list().iter().map(|p| p.id.clone()).collect();
    probe_names.sort();

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        probes_available: registry.count(),
        probe_names,
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
    Router::new()
        .route("/v1/answer", post(answer_question))
        .route("/v1/answer/stream", post(answer_question_stream))
}

/// v0.10.0: Process a question through the LLM-A/LLM-B audit loop
/// v0.65.0: Records stats after each answer for progression tracking
async fn answer_question(
    State(state): State<AppStateArc>,
    Json(req): Json<AnswerRequest>,
) -> Result<Json<FinalAnswer>, (StatusCode, String)> {
    let start = Instant::now();
    info!("[Q]  Processing: {}", req.question);

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
            "[M]  Using configured models - Junior: {}, Senior: {}",
            junior, senior
        );
        (Some(junior), Some(senior))
    } else if config.llm.selection_mode.as_str() == "manual" {
        // Manual mode without role-specific: use preferred_model for both
        let model = config.llm.preferred_model.clone();
        info!("[M]  Using manual mode - Model: {} (for both roles)", model);
        (Some(model.clone()), Some(model))
    } else {
        // Auto selection - use hardware profile (no explicit models configured)
        let profile = anna_common::HardwareProfile::detect();
        let recommendation = profile.select_model();
        info!(
            "[M]  Auto-selected model: {} (reason: {}, GPU: {:?}, VRAM: {:?}GB, driver_functional: {})",
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

    // v0.90.0: UnifiedEngine with unified architecture
    // v4.3.1: Pass shared cache from AppState for persistence
    let mut engine = UnifiedEngine::new(junior_model, senior_model, state.answer_cache.clone());
    info!(
        "[E]  v0.90.0 UnifiedEngine ready - Junior: {}, Senior: {}",
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

    // Process the question via v0.90.0 unified flow
    let question_clone = req.question.clone();
    match engine.process_question(&req.question).await {
        Ok(answer) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            let reliability = answer.scores.overall;
            let iterations = answer.debug_trace.as_ref().map(|d| d.iterations.len()).unwrap_or(1) as u32;
            // Note: DebugTrace only has iterations, junior_model, senior_model, duration_secs
            let skills_used = 1u32; // Default for now
            let was_decomposed = false; // Not tracked in current DebugTrace
            let answer_success = reliability >= 0.60;

            // v0.65.0: Record stats and persist
            {
                let mut stats = state.stats_engine.write().await;
                let xp_gain = stats.record_answer(
                    &question_clone,
                    reliability,
                    latency_ms,
                    iterations,
                    skills_used,
                    was_decomposed,
                    answer_success,
                );
                if let Err(e) = stats.save_default() {
                    error!("[S]  Failed to save stats: {}", e);
                }
                if xp_gain.total > 0 {
                    info!(
                        "[S]  +{}XP  Level {} ({})  Reliability: {:.0}%",
                        xp_gain.total,
                        stats.level(),
                        stats.title(),
                        reliability * 100.0
                    );
                }
            }

            // v1.1.0: Record telemetry for successful answer
            let origin = if iterations <= 1 {
                TelemetryOrigin::Junior
            } else {
                TelemetryOrigin::Senior
            };

            // Check if refusal (low reliability)
            if reliability < 0.60 {
                telemetry_record_refusal(&question_clone, reliability, latency_ms);
            } else {
                telemetry_record_success(
                    &question_clone,
                    origin,
                    reliability,
                    latency_ms,
                    0,  // junior_ms (not tracked separately here)
                    0,  // senior_ms (not tracked separately here)
                    answer.debug_trace.as_ref()
                        .map(|d| d.iterations.len() as u32)
                        .unwrap_or(1),
                );
            }

            info!(
                "[A]  Done in {}ms  Confidence: {:.0}% ({:?})",
                latency_ms, reliability * 100.0, answer.confidence_level
            );
            Ok(Json(answer))
        }
        Err(e) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            // v1.1.0: Record telemetry for failure
            telemetry_record_failure(&question_clone, &e.to_string(), latency_ms);

            error!("[E]  Failed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// v0.43.0: Request for streaming debug answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamAnswerRequest {
    pub question: String,
    #[serde(default)]
    pub debug: bool,
}

/// v0.43.0: Process a question with streaming debug events
async fn answer_question_stream(
    State(state): State<AppStateArc>,
    Json(req): Json<StreamAnswerRequest>,
) -> Response {
    let start = Instant::now();
    info!("[STREAM]  Processing question: {}", req.question);

    // Create debug event channel
    let (tx, rx) = mpsc::unbounded_channel::<DebugEvent>();
    let emitter = create_channel_emitter(tx, req.debug);

    // Get models from config
    let config = AnnaConfigV5::load();
    let (junior_model, senior_model) = if !config.llm.needs_role_model_migration() {
        let junior = config.llm.get_junior_model().to_string();
        let senior = config.llm.get_senior_model().to_string();
        (junior, senior)
    } else if config.llm.selection_mode.as_str() == "manual" {
        let model = config.llm.preferred_model.clone();
        (model.clone(), model)
    } else {
        let profile = anna_common::HardwareProfile::detect();
        let recommendation = profile.select_model();
        (recommendation.model.clone(), recommendation.model)
    };

    // Record query for telemetry
    if let Some(brain) = &state.brain {
        brain.record_query(&req.question).await;
    }

    let question = req.question.clone();
    let question_for_stats = req.question.clone();
    let junior = junior_model.clone();
    let senior = senior_model.clone();
    let state_for_stats = state.clone();
    // v4.3.1: Clone answer cache for persistence between requests
    let answer_cache = state.answer_cache.clone();

    // Spawn orchestration task
    tokio::spawn(async move {
        // Emit stream started
        emitter.emit(
            DebugEvent::new(DebugEventType::StreamStarted, 0, "Debug stream started").with_data(
                DebugEventData::StreamMeta {
                    question: question.clone(),
                    junior_model: junior.clone(),
                    senior_model: senior.clone(),
                },
            ),
        );

        // v0.90.0: UnifiedEngine with unified architecture
        // v4.3.1: Pass shared cache for persistence between requests
        let mut engine = UnifiedEngine::new(Some(junior.clone()), Some(senior.clone()), answer_cache);

        // Check if LLM is available
        if !engine.is_available().await {
            emitter.emit(DebugEvent::new(
                DebugEventType::Error,
                0,
                "LLM backend (Ollama) is not available",
            ));
            emitter.emit(DebugEvent::new(
                DebugEventType::StreamEnded,
                0,
                "Stream ended with error",
            ));
            return;
        }

        // Process the question with debug events via v0.90.0 unified flow
        match engine.process_question_with_emitter(&question, Some(emitter.as_ref())).await {
            Ok(answer) => {
                let duration = start.elapsed().as_secs_f64();
                let latency_ms = start.elapsed().as_millis() as u64;
                let reliability = answer.scores.overall;
                let iterations = answer.debug_trace.as_ref().map(|d| d.iterations.len()).unwrap_or(1) as u32;
                let answer_success = reliability >= 0.60;

                // v0.72.0: Record stats for streaming answers too
                {
                    let mut stats = state_for_stats.stats_engine.write().await;
                    let xp_gain = stats.record_answer(
                        &question_for_stats,
                        reliability,
                        latency_ms,
                        iterations,
                        1, // skills_used
                        false, // was_decomposed
                        answer_success,
                    );
                    if let Err(e) = stats.save_default() {
                        error!("[STREAM]  Failed to save stats: {}", e);
                    }
                    if xp_gain.total > 0 {
                        info!(
                            "[STREAM]  +{}XP  Level {} ({})  Reliability: {:.0}%",
                            xp_gain.total,
                            stats.level(),
                            stats.title(),
                            reliability * 100.0
                        );
                    }
                }

                // v1.1.0: Record telemetry for streaming answer
                let origin = if iterations <= 1 {
                    TelemetryOrigin::Junior
                } else {
                    TelemetryOrigin::Senior
                };
                if reliability < 0.60 {
                    telemetry_record_refusal(&question, reliability, latency_ms);
                } else {
                    telemetry_record_success(
                        &question,
                        origin,
                        reliability,
                        latency_ms,
                        0, 0, iterations,
                    );
                }

                // v1.5.0: Emit the final answer text so streaming clients can display it
                // Apply empty answer guardrail
                let answer_origin = if iterations <= 1 { "junior" } else { "senior" };
                let final_answer_text = if answer.answer.trim().is_empty() {
                    warn!("[STREAM]  Empty answer detected, applying guardrail");
                    format!(
                        "I processed your question but couldn't generate a meaningful response.\n\n\
                         Question: {}\n\n\
                         Please try rephrasing your question or ask something more specific.",
                        question
                    )
                } else {
                    answer.answer.clone()
                };
                emitter.emit(
                    DebugEvent::new(DebugEventType::AnswerReady, 0, "Final answer").with_data(
                        DebugEventData::FinalAnswer {
                            answer: final_answer_text,
                            origin: answer_origin.to_string(),
                        },
                    ),
                );
                emitter.emit(
                    DebugEvent::new(DebugEventType::AnswerReady, 0, "Answer ready").with_data(
                        DebugEventData::AnswerSummary {
                            confidence: format!("{:?}", answer.confidence_level),
                            score: answer.scores.overall,
                            iterations_used: answer.debug_trace.as_ref().map(|d| d.iterations.len()).unwrap_or(0),
                        },
                    ),
                );
                emitter.emit(
                    DebugEvent::new(DebugEventType::StreamEnded, 0, "Stream ended").with_data(
                        DebugEventData::KeyValue {
                            pairs: vec![("duration_secs".to_string(), format!("{:.2}", duration))],
                        },
                    ),
                );
            }
            Err(e) => {
                let fail_latency_ms = start.elapsed().as_millis() as u64;
                // v1.1.0: Record telemetry for streaming failure
                telemetry_record_failure(&question, &e.to_string(), fail_latency_ms);
                error!("[STREAM]  Failed to process question: {}", e);
                emitter.emit(
                    DebugEvent::new(DebugEventType::Error, 0, e.to_string()).with_data(
                        DebugEventData::KeyValue {
                            pairs: vec![("error".to_string(), e.to_string())],
                        },
                    ),
                );
                emitter.emit(DebugEvent::new(
                    DebugEventType::StreamEnded,
                    0,
                    "Stream ended with error",
                ));
            }
        }
    });

    // Return streaming response immediately
    debug_stream_response(rx)
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

// ============================================================================
// Stats Routes (v0.65.0)
// ============================================================================

pub fn stats_routes() -> Router<AppStateArc> {
    Router::new()
        .route("/v1/stats", get(get_stats))
        .route("/v1/stats/telemetry", get(get_telemetry_stats))
}

/// v0.65.0: Get performance stats and progression
async fn get_stats(State(state): State<AppStateArc>) -> Json<PerformanceSnapshot> {
    let stats = state.stats_engine.read().await;
    Json(stats.snapshot())
}

/// v1.2.0: Get telemetry-based performance stats
async fn get_telemetry_stats(
    State(_state): State<AppStateArc>,
) -> Json<anna_common::telemetry::TelemetrySummaryComplete> {
    use anna_common::telemetry::{TelemetryReader, DEFAULT_WINDOW_SIZE};

    let reader = TelemetryReader::default_path();
    Json(reader.complete_summary(DEFAULT_WINDOW_SIZE))
}

// ============================================================================
// Benchmark Routes (v1.4.0) - Snow Leopard Benchmark
// ============================================================================

/// Request to run Snow Leopard benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRequest {
    /// Benchmark mode: "full" or "quick"
    #[serde(default = "default_benchmark_mode")]
    pub mode: String,
}

fn default_benchmark_mode() -> String {
    "full".to_string()
}

/// Response from benchmark execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResponse {
    pub success: bool,
    pub mode: String,
    pub timestamp: String,
    pub phases: usize,
    pub total_questions: usize,
    pub success_rate: f64,
    pub avg_latency_ms: u64,
    pub brain_usage_pct: f64,
    pub llm_usage_pct: f64,
    pub total_xp_anna: i64,
    pub total_xp_junior: i64,
    pub total_xp_senior: i64,
    pub report_path: Option<String>,
    pub ascii_summary: String,
    pub warnings: Vec<String>,
}

pub fn benchmark_routes() -> Router<AppStateArc> {
    Router::new()
        .route("/v1/bench/snow_leopard", post(run_snow_leopard_benchmark))
        .route("/v1/bench/last", get(get_last_benchmark))
        // v1.5.0: History and delta endpoints
        .route("/v1/bench/history", get(get_benchmark_history))
        .route("/v1/bench/delta", get(get_benchmark_delta))
}

/// v1.4.0: Run Snow Leopard benchmark
async fn run_snow_leopard_benchmark(
    State(_state): State<AppStateArc>,
    Json(req): Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkResponse>, (StatusCode, String)> {
    use anna_common::bench_snow_leopard::{
        BenchmarkMode, SnowLeopardConfig, run_benchmark, BenchmarkHistoryEntry,
    };

    info!("[BENCH]  Starting Snow Leopard benchmark (mode: {})", req.mode);

    // Parse mode
    let mode = BenchmarkMode::from_str(&req.mode);

    // Build config
    let config = match mode {
        BenchmarkMode::Full => SnowLeopardConfig::runtime_mode(),
        BenchmarkMode::Quick => SnowLeopardConfig::quick_mode(),
    };

    // Run benchmark
    let result = run_benchmark(&config).await;

    // v1.5.0: Save to history (includes last_benchmark.json)
    let history_entry = BenchmarkHistoryEntry::from_result(&result);
    if let Err(e) = history_entry.save() {
        error!("[BENCH]  Failed to save benchmark history: {}", e);
    }

    info!(
        "[BENCH]  Completed: {} questions, {:.0}% success, {}ms avg latency",
        result.total_questions,
        result.overall_success_rate(),
        result.overall_avg_latency()
    );

    // v1.5.0: Auto-tuning - only for Full benchmarks
    if mode == BenchmarkMode::Full {
        use anna_common::auto_tune::{AutoTuneConfig, AutoTuneState, auto_tune_from_benchmark};
        use anna_common::telemetry::TelemetryReader;

        // Load telemetry and current state
        let telemetry = TelemetryReader::default_path().complete_summary(50);
        let auto_tune_config = AutoTuneConfig::default();
        let mut auto_tune_state = AutoTuneState::load();

        // Apply auto-tuning
        let decision = auto_tune_from_benchmark(&telemetry, &result, &mut auto_tune_state, &auto_tune_config);

        if decision.changed {
            info!("[AUTO-TUNE]  {}", decision.explanation);
            if let Err(e) = auto_tune_state.save() {
                error!("[AUTO-TUNE]  Failed to save state: {}", e);
            }
        } else {
            info!("[AUTO-TUNE]  No tuning needed: {}", decision.explanation);
        }
    }

    // Calculate methods before moving owned fields
    let success_rate = result.overall_success_rate();
    let avg_latency_ms = result.overall_avg_latency();
    let brain_usage_pct = result.brain_usage_pct();
    let llm_usage_pct = result.llm_usage_pct();

    Ok(Json(BenchmarkResponse {
        success: result.ux_consistency_passed && result.warnings.is_empty(),
        mode: format!("{:?}", result.mode).to_lowercase(),
        timestamp: result.timestamp,
        phases: result.phases.len(),
        total_questions: result.total_questions,
        success_rate,
        avg_latency_ms,
        brain_usage_pct,
        llm_usage_pct,
        total_xp_anna: result.total_xp.anna,
        total_xp_junior: result.total_xp.junior,
        total_xp_senior: result.total_xp.senior,
        report_path: result.report_path,
        ascii_summary: result.ascii_summary,
        warnings: result.warnings,
    }))
}

/// v1.4.0: Get last benchmark summary (for status display)
async fn get_last_benchmark(
    State(_state): State<AppStateArc>,
) -> Result<Json<anna_common::bench_snow_leopard::LastBenchmarkSummary>, StatusCode> {
    use anna_common::bench_snow_leopard::LastBenchmarkSummary;

    match LastBenchmarkSummary::load() {
        Some(summary) => Ok(Json(summary)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ============================================================================
// v1.5.0: History and Delta Endpoints
// ============================================================================

/// Query params for history endpoint
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    /// Number of entries to return (default 10, max 50)
    pub limit: Option<usize>,
}

/// Query params for delta endpoint
#[derive(Debug, Deserialize)]
pub struct DeltaQuery {
    /// ID of older benchmark (optional, defaults to second-to-last)
    pub from: Option<String>,
    /// ID of newer benchmark (optional, defaults to last)
    pub to: Option<String>,
}

/// v1.5.0: Get benchmark history (last N runs)
async fn get_benchmark_history(
    State(_state): State<AppStateArc>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<anna_common::bench_snow_leopard::BenchmarkHistoryListItem>>, StatusCode> {
    use anna_common::bench_snow_leopard::BenchmarkHistoryEntry;

    let limit = query.limit.unwrap_or(10).min(50);
    let entries = BenchmarkHistoryEntry::list_recent(limit);

    if entries.is_empty() {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(Json(entries))
    }
}

/// v1.5.0: Get delta between two benchmarks
async fn get_benchmark_delta(
    State(_state): State<AppStateArc>,
    Query(query): Query<DeltaQuery>,
) -> Result<Json<anna_common::bench_snow_leopard::SnowLeopardDelta>, (StatusCode, String)> {
    use anna_common::bench_snow_leopard::{
        BenchmarkHistoryEntry, compare_benchmarks, compare_last_two_benchmarks,
    };

    // If both from and to specified, compare those specific benchmarks
    if let (Some(from_id), Some(to_id)) = (&query.from, &query.to) {
        let older = BenchmarkHistoryEntry::load(from_id)
            .ok_or((StatusCode::NOT_FOUND, format!("Benchmark '{}' not found", from_id)))?;
        let newer = BenchmarkHistoryEntry::load(to_id)
            .ok_or((StatusCode::NOT_FOUND, format!("Benchmark '{}' not found", to_id)))?;

        let delta = compare_benchmarks(&older.full_result, &newer.full_result);
        return Ok(Json(delta));
    }

    // Default: compare last two benchmarks
    match compare_last_two_benchmarks() {
        Some(delta) => Ok(Json(delta)),
        None => Err((
            StatusCode::NOT_FOUND,
            "Need at least 2 benchmark runs to compute delta".to_string(),
        )),
    }
}

// ============================================================================
// v4.3.0: Reset Routes (Factory Reset via Daemon)
// ============================================================================

pub fn reset_routes() -> Router<AppStateArc> {
    Router::new()
        .route("/v1/reset/factory", post(factory_reset))
}

/// Response from factory reset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetResponse {
    pub success: bool,
    pub components_reset: Vec<String>,
    pub components_failed: Vec<String>,
    pub errors: Vec<String>,
}

/// v4.3.0: Execute factory reset (daemon has root permissions)
/// v4.5.5: Also clears in-memory answer cache
async fn factory_reset(
    State(state): State<AppStateArc>,
) -> Result<Json<ResetResponse>, (StatusCode, String)> {
    use anna_common::execute_factory_reset;

    info!("[RESET]  Factory reset requested via daemon API");

    // v4.5.5: Clear in-memory answer cache
    state.answer_cache.write().await.clear();
    info!("[RESET]  Cleared in-memory answer cache");

    let result = execute_factory_reset();

    if result.reliability >= 0.9 {
        info!("[RESET]  Factory reset completed successfully");
        Ok(Json(ResetResponse {
            success: true,
            components_reset: vec![
                "XP store".to_string(),
                "XP events".to_string(),
                "Stats".to_string(),
                "Knowledge".to_string(),
                "LLM state".to_string(),
                "Benchmarks".to_string(),
                "Telemetry".to_string(),
                "Answer cache".to_string(), // v4.5.5
            ],
            components_failed: vec![],
            errors: vec![],
        }))
    } else {
        // Parse errors from result text
        let errors: Vec<String> = result.text
            .lines()
            .filter(|l| l.contains("Failed"))
            .map(|l| l.to_string())
            .collect();

        warn!("[RESET]  Factory reset completed with issues: {:?}", errors);
        Ok(Json(ResetResponse {
            success: false,
            components_reset: vec!["XP store".to_string(), "Stats".to_string(), "Knowledge".to_string()],
            components_failed: errors.clone(),
            errors,
        }))
    }
}
