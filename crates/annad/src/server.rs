//! HTTP server for annad
//!
//! v0.65.0: Added StatsEngine for tracking answer metrics and progression.

use crate::brain::AnnaBrain;
use crate::probe::registry::ProbeRegistry;
use crate::routes;
use crate::state::StateManager;
use anna_common::StatsEngine;
use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::info;

/// Application state shared across handlers
pub struct AppState {
    pub probe_registry: Arc<RwLock<ProbeRegistry>>,
    pub state_manager: Arc<RwLock<StateManager>>,
    pub start_time: Instant,
    /// v0.11.0: Anna's brain for learning and knowledge
    pub brain: Option<Arc<AnnaBrain>>,
    /// v0.65.0: Stats engine for tracking performance and progression
    pub stats_engine: Arc<RwLock<StatsEngine>>,
}

impl AppState {
    pub fn new(probe_registry: ProbeRegistry, state_manager: StateManager) -> Self {
        // v0.65.0: Load or create stats engine
        let stats_engine = StatsEngine::load_default().unwrap_or_else(|_| StatsEngine::new());
        Self {
            probe_registry: Arc::new(RwLock::new(probe_registry)),
            state_manager: Arc::new(RwLock::new(state_manager)),
            start_time: Instant::now(),
            brain: None,
            stats_engine: Arc::new(RwLock::new(stats_engine)),
        }
    }

    /// v0.11.0: Create app state with brain reference
    /// v0.65.0: Also loads StatsEngine for progression tracking
    pub fn new_with_brain(
        probe_registry: ProbeRegistry,
        state_manager: StateManager,
        brain: Arc<AnnaBrain>,
    ) -> Self {
        // v0.65.0: Load or create stats engine
        let stats_engine = StatsEngine::load_default().unwrap_or_else(|_| StatsEngine::new());
        Self {
            probe_registry: Arc::new(RwLock::new(probe_registry)),
            state_manager: Arc::new(RwLock::new(state_manager)),
            start_time: Instant::now(),
            brain: Some(brain),
            stats_engine: Arc::new(RwLock::new(stats_engine)),
        }
    }
}

/// Run the HTTP server
pub async fn run(state: AppState) -> Result<()> {
    let state = Arc::new(state);

    let app = Router::new()
        .merge(routes::probe_routes())
        .merge(routes::state_routes())
        .merge(routes::health_routes())
        .merge(routes::update_routes()) // v0.9.0: Update state endpoint
        .merge(routes::answer_routes()) // v0.10.0: Answer engine endpoint
        .merge(routes::knowledge_routes()) // v0.11.0: Knowledge store endpoints
        .merge(routes::stats_routes()) // v0.65.0: Stats and progression endpoint
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    // Bind to localhost only for security
    let addr = "127.0.0.1:7865";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("  Listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
