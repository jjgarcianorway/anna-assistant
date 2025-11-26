//! Anna Daemon (annad) - Evidence Oracle
//!
//! The ONLY source of truth. Runs probes, provides raw JSON.
//! No interpretation, no formatting.

mod parser;
mod probe;
mod routes;
mod server;
mod state;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annad=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("  Anna Daemon v{}", env!("CARGO_PKG_VERSION"));
    info!("  Evidence Oracle starting...");

    // Load probes
    let probe_registry = probe::registry::ProbeRegistry::load_from_dir("probes")?;
    info!("  Loaded {} probes", probe_registry.count());

    // Create state manager
    let state_manager = state::StateManager::new();

    // Create app state
    let app_state = server::AppState::new(probe_registry, state_manager);

    // Start server
    server::run(app_state).await
}
