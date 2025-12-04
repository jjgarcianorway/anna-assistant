//! HTTP server for annad v7.0.0
//!
//! Server with health, stats, reset, and control endpoints.
//! All state mutations happen here in the daemon (running as root).
//! annactl is just a remote control that sends RPC requests.

use anna_common::{
    ErrorIndex, IntrusionIndex, InventoryProgress, KnowledgeBuilder, LogScanState, ServiceIndex,
    TelemetryAggregates, TelemetryState,
};
use anyhow::Result;
use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

/// Application state
#[allow(dead_code)]
pub struct AppState {
    pub start_time: Instant,
    pub knowledge_builder: Arc<RwLock<KnowledgeBuilder>>,
    /// Track scan metadata
    pub last_scan_time: RwLock<Option<Instant>>,
    pub last_scan_duration_ms: RwLock<u64>,
    pub scan_count: RwLock<u32>,
    /// Track internal errors
    pub internal_errors: RwLock<InternalErrors>,
    /// Track request counts (reserved for future metrics endpoint)
    pub cli_requests: RwLock<u64>,
    pub http_requests: RwLock<u64>,
    pub total_response_time_ms: RwLock<u64>,
}

/// Internal error tracking (Anna's own errors, not system errors)
#[derive(Default, Clone, Serialize)]
pub struct InternalErrors {
    /// Crashes since last boot
    #[serde(default)]
    pub crashes: u32,
    /// Failed subprocess calls (pacman, systemctl, etc)
    #[serde(alias = "subprocess_failures")]
    pub command_failures: u32,
    /// Parser failures (man output, --help, etc)
    #[serde(alias = "parser_failures")]
    pub parse_errors: u32,
}

impl AppState {
    pub fn new(knowledge_builder: Arc<RwLock<KnowledgeBuilder>>) -> Self {
        Self {
            start_time: Instant::now(),
            knowledge_builder,
            last_scan_time: RwLock::new(None),
            last_scan_duration_ms: RwLock::new(0),
            scan_count: RwLock::new(0),
            internal_errors: RwLock::new(InternalErrors::default()),
            cli_requests: RwLock::new(0),
            http_requests: RwLock::new(0),
            total_response_time_ms: RwLock::new(0),
        }
    }

    /// Record a scan completion
    pub async fn record_scan(&self, duration_ms: u64) {
        *self.last_scan_time.write().await = Some(Instant::now());
        *self.last_scan_duration_ms.write().await = duration_ms;
        *self.scan_count.write().await += 1;
    }

    /// Record an internal error (reserved for future use)
    #[allow(dead_code)]
    pub async fn record_error(&self, error_type: &str) {
        let mut errors = self.internal_errors.write().await;
        match error_type {
            "crash" => errors.crashes += 1,
            "subprocess" | "command" => errors.command_failures += 1,
            "parser" | "parse" => errors.parse_errors += 1,
            _ => {}
        }
    }

    /// Record a request (reserved for future metrics endpoint)
    #[allow(dead_code)]
    pub async fn record_request(&self, response_time_ms: u64) {
        *self.cli_requests.write().await += 1;
        *self.total_response_time_ms.write().await += response_time_ms;
    }
}

/// Health response (basic)
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub phase: String,
    pub uptime_secs: u64,
    pub objects_tracked: usize,
}

/// v7.0.0: Stats response for annactl status
#[derive(Serialize)]
pub struct StatsResponse {
    // Daemon info
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub pid: u32,

    // Restarts (placeholder - would need systemd integration)
    #[serde(default)]
    pub restarts_24h: u32,

    // Inventory counts
    pub commands_count: usize,
    pub packages_count: usize,
    pub services_count: usize,

    // Scan info
    pub last_scan_secs_ago: Option<u64>,

    // Internal errors (Anna's own, not system)
    pub internal_errors: InternalErrors,
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

/// v6.1.0: Stats handler - detailed daemon telemetry
/// v7.0.0: Stats handler - Anna-only telemetry
async fn stats_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<StatsResponse> {
    let builder = state.knowledge_builder.read().await;
    let (commands, packages, services) = builder.store().count_by_type();

    let last_scan_time = state.last_scan_time.read().await;
    let last_scan_secs_ago = last_scan_time.map(|t| t.elapsed().as_secs());

    Json(StatsResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        pid: std::process::id(),
        restarts_24h: 0, // TODO: query systemd for restart count
        commands_count: commands,
        packages_count: packages,
        services_count: services,
        last_scan_secs_ago,
        internal_errors: state.internal_errors.read().await.clone(),
    })
}

/// Get process memory from /proc/self/status (reserved for future diagnostics)
#[allow(dead_code)]
fn get_process_memory() -> (u64, u64) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let mut rss_kb = 0u64;
    let mut peak_kb = 0u64;

    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            if let Some(val) = line.split_whitespace().nth(1) {
                rss_kb = val.parse().unwrap_or(0);
            }
        } else if line.starts_with("VmPeak:") {
            if let Some(val) = line.split_whitespace().nth(1) {
                peak_kb = val.parse().unwrap_or(0);
            }
        }
    }

    (rss_kb, peak_kb)
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
        info!(
            "[RESET] Rescan complete: {} cmds, {} pkgs, {} svcs",
            commands, packages, services
        );
    }

    let success = errors.is_empty();
    let message = if success {
        format!(
            "Reset complete. Cleared {} items and rescanned inventory.",
            cleared
        )
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
pub async fn run(state: Arc<AppState>) -> Result<()> {
    let app = Router::new()
        .route("/v1/health", get(health_check))
        .route("/v1/stats", get(stats_handler))
        .route("/v1/reset", post(reset_handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = "127.0.0.1:7865";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("  Listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
