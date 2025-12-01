//! HTTP server for annad v5.3.0
//!
//! Minimal server with health endpoint only.

use anna_common::KnowledgeBuilder;
use anyhow::Result;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::info;

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

/// Run the HTTP server
pub async fn run(state: AppState) -> Result<()> {
    let state = Arc::new(state);

    let app = Router::new()
        .route("/v1/health", get(health_check))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = "127.0.0.1:7865";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("  Listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
