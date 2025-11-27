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
//! v0.16.1: Dynamic model registry, on-demand LLM loading.

// Allow dead code for features planned but not yet fully wired
#![allow(dead_code)]
#![allow(unused_imports)]

mod auto_update;
mod brain;
mod model_registry_fetcher;
mod orchestrator;
mod parser;
mod probe;
mod routes;
mod server;
mod state;

use anna_common::{AnnaConfigV5, KnowledgeStore};
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};
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

    // Log config
    let config = AnnaConfigV5::load();
    info!(
        "⚙️  Config: mode={}, update.enabled={}, update.channel={}, update.interval={}s",
        config.core.mode.as_str(),
        config.update.enabled,
        config.update.channel.as_str(),
        config.update.effective_interval()
    );

    // v0.15.18: Check for role-specific model config
    if config.llm.needs_role_model_migration() {
        let suggested_junior = config.llm.suggest_junior_model();
        warn!(
            "⚠️  Config uses legacy single-model setup. For optimal performance, run the installer to get role-specific models:"
        );
        warn!("    curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash");
        warn!(
            "    Or manually add to config.toml: junior_model = \"{}\", senior_model = \"{}\"",
            suggested_junior,
            config.llm.preferred_model
        );
        info!(
            "🧠  LLM: selection_mode={}, preferred={} (used for both junior/senior), fallback={}",
            config.llm.selection_mode.as_str(),
            config.llm.preferred_model,
            config.llm.fallback_model
        );
    } else {
        info!(
            "🧠  LLM: selection_mode={}, junior={}, senior={}, fallback={}",
            config.llm.selection_mode.as_str(),
            config.llm.get_junior_model(),
            config.llm.get_senior_model(),
            config.llm.fallback_model
        );
    }

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
