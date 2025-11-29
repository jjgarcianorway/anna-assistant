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
//! v0.65.0: Daemon robustness - panic hooks, graceful shutdown.

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

use anna_common::{
    AnnaConfigV5, KnowledgeStore, is_first_run, mark_initialized,
    llm_provision::{needs_autoprovision, run_full_autoprovision, LlmSelection},
    permissions::{auto_fix_permissions, PermissionsHealthCheck},
};
use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // v0.65.0: Set up panic hook for robust error handling
    setup_panic_hook();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annad=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("[*]  Anna Daemon v{}", env!("CARGO_PKG_VERSION"));
    info!("[>]  Evidence Oracle starting...");

    // v2.1.0: Permissions health check and auto-fix
    let health = PermissionsHealthCheck::run();
    if !health.all_ok {
        warn!("[!]  Permissions issues detected, attempting auto-fix...");
        let fixes = auto_fix_permissions();
        for fix in &fixes {
            if fix.success {
                info!("[+]  {}: {}", fix.path.display(), fix.action);
            } else {
                warn!("[!]  Failed to fix {}: {:?}", fix.path.display(), fix.error);
            }
        }
        // Re-check
        let health2 = PermissionsHealthCheck::run();
        if health2.all_ok {
            info!("[+]  All permissions issues resolved");
        } else {
            warn!("[!]  Some permissions issues remain:");
            for issue in health2.issues() {
                warn!("      - {}", issue);
            }
            warn!("[!]  XP/telemetry may not persist. Run: sudo chmod -R 777 /var/lib/anna /var/log/anna");
        }
    } else {
        info!("[+]  Permissions OK");
    }

    // v0.11.0: Initialize knowledge store
    let knowledge_store = KnowledgeStore::open_default()?;
    let fact_count = knowledge_store.count()?;
    info!("[K]  Knowledge store: {} facts", fact_count);

    // v0.11.0: Initialize Anna's brain
    let anna_brain = Arc::new(brain::AnnaBrain::new(knowledge_store));
    let brain_clone = Arc::clone(&anna_brain);

    // Load probes - try multiple paths in order of preference
    let probe_paths = [
        "/usr/share/anna/probes", // System install location
        "/var/lib/anna/probes",   // Data directory fallback
        "probes",                 // Development/local fallback
    ];

    let mut probe_registry = None;
    for path in &probe_paths {
        if std::path::Path::new(path).exists() {
            match probe::registry::ProbeRegistry::load_from_dir(path) {
                Ok(registry) if registry.count() > 0 => {
                    info!("[+]  Loaded {} probes from {}", registry.count(), path);
                    probe_registry = Some(registry);
                    break;
                }
                Ok(_) => {
                    info!("[~]  Found {} but it's empty, trying next...", path);
                }
                Err(e) => {
                    warn!("[!]  Failed to load probes from {}: {}", path, e);
                }
            }
        }
    }

    let probe_registry = probe_registry.unwrap_or_else(|| {
        warn!("[!]  No probes found in any location! Using empty registry.");
        probe::registry::ProbeRegistry::default()
    });

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
        "[C]  Config: mode={}, update.enabled={}, update.channel={}, update.interval={}s",
        config.core.mode.as_str(),
        config.update.enabled,
        config.update.channel.as_str(),
        config.update.effective_interval()
    );

    // v0.15.18: Check for role-specific model config
    if config.llm.needs_role_model_migration() {
        let suggested_junior = config.llm.suggest_junior_model();
        warn!(
            "[!]  Config uses legacy single-model setup. For optimal performance, run the installer to get role-specific models:"
        );
        warn!("    curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash");
        warn!(
            "    Or manually add to config.toml: junior_model = \"{}\", senior_model = \"{}\"",
            suggested_junior, config.llm.preferred_model
        );
        info!(
            "[L]  LLM: selection_mode={}, preferred={} (used for both junior/senior), fallback={}",
            config.llm.selection_mode.as_str(),
            config.llm.preferred_model,
            config.llm.fallback_model
        );
    } else {
        info!(
            "[L]  LLM: selection_mode={}, junior={}, senior={}, fallback={}",
            config.llm.selection_mode.as_str(),
            config.llm.get_junior_model(),
            config.llm.get_senior_model(),
            config.llm.fallback_model
        );
    }

    // v2.0.0: LLM Autoprovision - self-provisioning models
    if is_first_run() || needs_autoprovision() {
        info!("[*]  Running LLM autoprovision...");
        let result = run_full_autoprovision(|msg| {
            info!("[P]  {}", msg);
        });

        if result.ollama_installed {
            info!("[+]  Ollama was installed during autoprovision");
        }
        if !result.models_installed.is_empty() {
            info!("[+]  Installed models: {:?}", result.models_installed);
        }
        info!(
            "[+]  Selected: Junior={} (score {:.2}), Senior={} (score {:.2})",
            result.selection.junior_model,
            result.selection.junior_score,
            result.selection.senior_model,
            result.selection.senior_score
        );
        if !result.errors.is_empty() {
            for err in &result.errors {
                warn!("[!]  Autoprovision error: {}", err);
            }
        }

        // Mark as initialized after successful autoprovision
        if let Err(e) = mark_initialized() {
            warn!("[!]  Failed to create initialization marker: {}", e);
        }
    } else {
        // Not first run - load existing selection
        let selection = LlmSelection::load();
        info!(
            "[L]  LLM Selection: Junior={} (score {:.2}), Senior={} (score {:.2})",
            selection.junior_model,
            selection.junior_score,
            selection.senior_model,
            selection.senior_score
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

/// v0.65.0: Set up a panic hook for robust error handling
/// Ensures panics are logged and don't silently kill the daemon
fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Log the panic with full details
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };

        // Write to stderr (will be captured by systemd)
        eprintln!();
        eprintln!("[!!!]  PANIC in Anna Daemon");
        eprintln!("[!!!]  Location: {}", location);
        eprintln!("[!!!]  Message: {}", message);
        eprintln!("[!!!]  v0.65.0: Daemon will exit. Check journalctl -u annad for details.");
        eprintln!();

        // Also call the default hook for backtrace
        default_hook(panic_info);
    }));
}
