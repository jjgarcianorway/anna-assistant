//! Anna Daemon (annad) - Evidence Oracle
//!
//! The ONLY source of truth. Runs probes, provides raw JSON.
//! No interpretation, no formatting.
//!
//! v0.4.0: Auto-update scheduler for dev mode.

mod auto_update;
mod parser;
mod probe;
mod routes;
mod server;
mod state;

use anyhow::Result;
use std::sync::Arc;
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

    info!("🤖  Anna Daemon v{}", env!("CARGO_PKG_VERSION"));
    info!("📋  Evidence Oracle starting...");

    // Load probes
    let probe_registry = probe::registry::ProbeRegistry::load_from_dir("probes")?;
    info!("🔧  Loaded {} probes", probe_registry.count());

    // Create state manager
    let state_manager = state::StateManager::new();

    // Create app state
    let app_state = server::AppState::new(probe_registry, state_manager);

    // Start auto-update scheduler in background
    let auto_update_scheduler = Arc::new(auto_update::AutoUpdateScheduler::new());
    let scheduler_clone = Arc::clone(&auto_update_scheduler);

    // Log update config
    let update_config = anna_common::load_update_config();
    info!(
        "🔄  Update config: channel={}, auto={}, interval={}s",
        update_config.channel.as_str(),
        update_config.auto,
        update_config.effective_interval()
    );

    tokio::spawn(async move {
        scheduler_clone.start().await;
    });

    // Start server
    server::run(app_state).await
}
