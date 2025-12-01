//! HTTP server for annad
//!
//! v5.0.0: Knowledge Core Phase 1 - Minimal server with health endpoint only
//! v0.65.0: Added StatsEngine for tracking answer metrics and progression.
//! v4.3.1: Added AnswerCache for persistent caching between requests.

use anna_common::KnowledgeBuilder;
use anyhow::Result;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::info;

/// v5.0.0: Application state for Knowledge Core
pub struct AppState {
    pub start_time: Instant,
    pub knowledge_builder: Arc<RwLock<KnowledgeBuilder>>,
}

impl AppState {
    /// v5.0.0: Create app state for Knowledge Core
    pub fn new_v5(knowledge_builder: Arc<RwLock<KnowledgeBuilder>>) -> Self {
        Self {
            start_time: Instant::now(),
            knowledge_builder,
        }
    }
}

/// v5.0.0: Health response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub phase: String,
    pub uptime_secs: u64,
    pub objects_tracked: usize,
}

/// v5.0.0: Health check handler
async fn health_check(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<HealthResponse> {
    let builder = state.knowledge_builder.read().await;
    let objects = builder.store().total_objects();

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        phase: "Knowledge Core".to_string(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        objects_tracked: objects,
    })
}

/// v5.0.0: Run the minimal HTTP server
pub async fn run_v5(state: AppState) -> Result<()> {
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
