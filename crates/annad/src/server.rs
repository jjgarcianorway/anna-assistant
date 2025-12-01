//! HTTP server for annad v5.6.0
//!
//! Server with health, reset, and control endpoints.
//! All state mutations happen here in the daemon (running as root).
//! annactl is just a remote control that sends RPC requests.

use anna_common::{
    KnowledgeBuilder, TelemetryAggregates,
    ErrorIndex, IntrusionIndex, LogScanState, ServiceIndex,
    TelemetryState, InventoryProgress,
};
use anyhow::Result;
use axum::{routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

/// Application state
pub struct AppState {
    pub start_time: Instant,
    pub knowledge_builder: Arc<RwLock<KnowledgeBuilder>>,
}

impl AppState {
    pub fn new(knowledge_builder: Arc<RwLock<KnowledgeBuilder>>) -> Self {
        Self {
            start_time: Instant::now(),
            knowledge_builder,
        }
    }
}

/// Health response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub phase: String,
    pub uptime_secs: u64,
    pub objects_tracked: usize,
}

/// Reset response
#[derive(Serialize)]
pub struct ResetResponse {
    pub success: bool,
    pub message: String,
    pub cleared_items: usize,
    pub errors: Vec<String>,
}

/// Reset request (optional parameters)
#[derive(Deserialize, Default)]
pub struct ResetRequest {
    /// If true, also clear error and intrusion logs
    #[serde(default)]
    pub clear_logs: bool,
}

/// Health check handler
async fn health_check(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<HealthResponse> {
    let builder = state.knowledge_builder.read().await;
    let objects = builder.store().total_objects();

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        phase: "Telemetry Core".to_string(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        objects_tracked: objects,
    })
}

/// v5.6.0: Reset handler - clears all state and triggers rescan
/// This runs as root in annad, so it has permission to delete everything.
async fn reset_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(request): Json<ResetRequest>,
) -> Json<ResetResponse> {
    info!("[RESET] Received reset request");

    let mut cleared = 0;
    let mut errors = Vec::new();

    // Clear knowledge store
    {
        let mut builder = state.knowledge_builder.write().await;
        builder.store_mut().clear();
        if let Err(e) = builder.save() {
            errors.push(format!("Failed to save cleared store: {}", e));
        } else {
            cleared += 1;
        }
    }

    // Clear telemetry aggregates
    let telemetry = TelemetryAggregates::new();
    if let Err(e) = telemetry.save() {
        errors.push(format!("Failed to clear telemetry: {}", e));
    } else {
        cleared += 1;
    }

    // Clear telemetry state
    let telemetry_state = TelemetryState::default();
    if let Err(e) = telemetry_state.save() {
        errors.push(format!("Failed to clear telemetry state: {}", e));
    } else {
        cleared += 1;
    }

    // Clear inventory progress
    let progress = InventoryProgress::new();
    if let Err(e) = progress.save() {
        errors.push(format!("Failed to clear inventory progress: {}", e));
    } else {
        cleared += 1;
    }

    // Clear log scan state
    let log_scan = LogScanState::new();
    if let Err(e) = log_scan.save() {
        errors.push(format!("Failed to clear log scan state: {}", e));
    } else {
        cleared += 1;
    }

    // Clear service index
    let service_index = ServiceIndex::new();
    if let Err(e) = service_index.save() {
        errors.push(format!("Failed to clear service index: {}", e));
    } else {
        cleared += 1;
    }

    // Optionally clear error and intrusion logs
    if request.clear_logs {
        let error_index = ErrorIndex::new();
        if let Err(e) = error_index.save() {
            errors.push(format!("Failed to clear error index: {}", e));
        } else {
            cleared += 1;
        }

        let intrusion_index = IntrusionIndex::new();
        if let Err(e) = intrusion_index.save() {
            errors.push(format!("Failed to clear intrusion index: {}", e));
        } else {
            cleared += 1;
        }
    }

    // Trigger immediate full inventory rescan
    {
        let mut builder = state.knowledge_builder.write().await;
        info!("[RESET] Starting full inventory rescan...");
        builder.collect_full_inventory();
        if let Err(e) = builder.save() {
            warn!("[RESET] Failed to save after rescan: {}", e);
            errors.push(format!("Failed to save after rescan: {}", e));
        }

        let (commands, packages, services) = builder.store().count_by_type();
        info!("[RESET] Rescan complete: {} cmds, {} pkgs, {} svcs",
              commands, packages, services);
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Reset complete. Cleared {} items and rescanned inventory.", cleared)
    } else {
        format!("Reset completed with {} errors.", errors.len())
    };

    info!("[RESET] {}", message);

    Json(ResetResponse {
        success,
        message,
        cleared_items: cleared,
        errors,
    })
}

/// Run the HTTP server
pub async fn run(state: AppState) -> Result<()> {
    let state = Arc::new(state);

    let app = Router::new()
        .route("/v1/health", get(health_check))
        .route("/v1/reset", post(reset_handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = "127.0.0.1:7865";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("  Listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
