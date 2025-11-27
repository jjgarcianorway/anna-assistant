//! Anna Daemon (annad) - Evidence Oracle
//!
//! The ONLY source of truth. Runs probes, provides raw JSON.
//! No interpretation, no formatting.
//!
//! v0.4.0: Auto-update scheduler for dev mode.
//! v0.5.0: Natural language configuration, hardware-aware model selection.
//! v0.9.0: Locked CLI surface, status command.
//! v0.10.0: Strict evidence discipline - LLM-A/LLM-B audit loop.
//! v0.11.0: Knowledge store, event-driven learning, user telemetry.

// Allow dead code for features planned but not yet fully wired
#![allow(dead_code)]
#![allow(unused_imports)]

mod auto_update;
mod brain;
mod orchestrator;
mod parser;
mod probe;
mod routes;
mod server;
mod state;

use anna_common::{AnnaConfigV5, KnowledgeStore};
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

    // v0.11.0: Initialize knowledge store
    let knowledge_store = KnowledgeStore::open_default()?;
    let fact_count = knowledge_store.count()?;
    info!("🧠  Knowledge store: {} facts", fact_count);

    // v0.11.0: Initialize Anna's brain
    let anna_brain = Arc::new(brain::AnnaBrain::new(knowledge_store));
    let brain_clone = Arc::clone(&anna_brain);

    // Load probes
    let probe_registry = probe::registry::ProbeRegistry::load_from_dir("probes")?;
    info!("🔧  Loaded {} probes", probe_registry.count());

    // Create state manager
    let state_manager = state::StateManager::new();

    // Create app state with brain reference
    let app_state = server::AppState::new_with_brain(probe_registry, state_manager, anna_brain);

    // Start auto-update scheduler in background
    let auto_update_scheduler = Arc::new(auto_update::AutoUpdateScheduler::new());
    let scheduler_clone = Arc::clone(&auto_update_scheduler);

    // Log v0.5.0 config
    let config = AnnaConfigV5::load();
    info!(
        "⚙️  Config: mode={}, update.enabled={}, update.channel={}, update.interval={}s",
        config.core.mode.as_str(),
        config.update.enabled,
        config.update.channel.as_str(),
        config.update.effective_interval()
    );
    info!(
        "🧠  LLM: selection_mode={}, preferred={}, fallback={}",
        config.llm.selection_mode.as_str(),
        config.llm.preferred_model,
        config.llm.fallback_model
    );

    // Start auto-update in background
    tokio::spawn(async move {
        scheduler_clone.start().await;
    });

    // v0.11.0: Start brain background tasks
    tokio::spawn(async move {
        brain_clone.start_background_tasks().await;
    });

    // Start server
    server::run(app_state).await
}
