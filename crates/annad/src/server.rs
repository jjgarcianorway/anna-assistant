//! HTTP server for annad

use crate::probe::registry::ProbeRegistry;
use crate::routes;
use crate::state::StateManager;
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
}

impl AppState {
    pub fn new(probe_registry: ProbeRegistry, state_manager: StateManager) -> Self {
        Self {
            probe_registry: Arc::new(RwLock::new(probe_registry)),
            state_manager: Arc::new(RwLock::new(state_manager)),
            start_time: Instant::now(),
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
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    // Bind to localhost only for security
    let addr = "127.0.0.1:7865";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("  Listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
