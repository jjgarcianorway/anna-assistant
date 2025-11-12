//! Anna Daemon - Autonomous System Administrator
//!
//! The Anna daemon (`annad`) is the core of the Anna Assistant system. It runs as a systemd service
//! and provides intelligent system monitoring and recommendations for Arch Linux.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           annad (Daemon)                │
//! │                                         │
//! │  ┌──────────┐  ┌───────────────────┐  │
//! │  │Telemetry │  │   Recommenders    │  │
//! │  │Collector │  │ • System-wide     │  │
//! │  └────┬─────┘  │ • Intelligent     │  │
//! │       │        │ • Context-aware   │  │
//! │       ▼        └─────────┬─────────┘  │
//! │  ┌──────────┐            │            │
//! │  │  Facts   │◄───────────┘            │
//! │  │  Cache   │                         │
//! │  └────┬─────┘            ▼            │
//! │       │        ┌───────────────────┐  │
//! │       └───────►│  Advice Store     │  │
//! │                └─────────┬─────────┘  │
//! │                          │            │
//! │  ┌──────────┐            │            │
//! │  │ Watcher  │            │            │
//! │  │(inotify) │            ▼            │
//! │  └────┬─────┘  ┌───────────────────┐  │
//! │       │        │   RPC Server      │  │
//! │       └───────►│ (Unix Socket)     │  │
//! │                └─────────┬─────────┘  │
//! └──────────────────────────┼───────────-┘
//!                            │
//!                            ▼
//!                    /run/anna/anna.sock
//!                            │
//!                            ▼
//!                     ┌──────────────┐
//!                     │   annactl    │
//!                     │   (Client)   │
//!                     └──────────────┘
//! ```
//!
//! # Features
//!
//! - **System Telemetry**: Collects comprehensive system facts (hardware, packages, configs)
//! - **Wiki-Strict Recommendations**: Detection rules based solely on Arch Wiki and man pages
//! - **Auto-Refresh**: Monitors filesystem changes and automatically updates recommendations
//! - **Notifications**: Alerts users of critical issues via GUI or terminal
//! - **IPC Server**: Serves requests from `annactl` via Unix socket
//! - **Audit Logging**: Records all applied actions with Wiki citations
//!
//! # Modules
//!
//! - `telemetry` - System fact collection
//! - `recommender` - Wiki-strict detection rules for system and desktop administration
//! - `rpc_server` - Unix socket IPC server
//! - `executor` - Safe command execution with audit logging
//! - `audit` - Audit trail management with Wiki citations
//! - `watcher` - Filesystem monitoring with inotify
//! - `notifier` - User notification system
//!
//! # Usage
//!
//! The daemon is typically run as a systemd service:
//!
//! ```bash
//! sudo systemctl start annad
//! sudo systemctl enable annad
//! ```
//!
//! For development/testing:
//!
//! ```bash
//! sudo ./target/debug/annad
//! ```

mod action_history;
mod audit;
mod autonomy;
mod chronos; // Phase 1.5: Chronos Loop
mod collective; // Phase 1.3: Collective Mind
mod conscience; // Phase 1.1: Conscience layer
mod consensus; // Phase 1.7: Distributed Consensus (STUB)
mod empathy; // Phase 1.2: Empathy Kernel
mod executor;
mod health; // Phase 0.5: Health subsystem
mod install; // Phase 0.8: Installation subsystem
mod mirror; // Phase 1.4: Mirror Protocol
mod mirror_audit; // Phase 1.6: Mirror Audit
mod network; // Phase 1.9: Network layer for distributed consensus
mod notifier;
mod profile; // Phase 3.0: Adaptive Intelligence & Smart Profiling
mod recommender;
mod recovery; // Phase 0.6: Recovery framework
mod repair; // Phase 0.7: Repair subsystem
mod rpc_server;
mod sentinel; // Phase 1.0: Sentinel framework
mod snapshotter;
mod state; // Phase 0.2: State machine
mod steward; // Phase 0.9: System steward
mod telemetry;
mod watcher;
mod wiki_cache;

use anyhow::Result;
use rpc_server::DaemonState;
use std::env;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

// Version is embedded at build time
const VERSION: &str = env!("ANNA_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments (rc.13.2: user mode support)
    let args: Vec<String> = env::args().collect();
    let mut user_mode = false;
    let mut foreground = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("annad {}", VERSION);
                return Ok(());
            }
            "--user" => user_mode = true,
            "--foreground" => foreground = true,
            "--help" | "-h" => {
                println!("Anna Assistant Daemon {}", VERSION);
                println!();
                println!("USAGE:");
                println!("    annad [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    --user        Run in user mode (no root, no systemd required)");
                println!("    --foreground  Stay in foreground (for supervisors)");
                println!("    --version     Print version information");
                println!("    --help        Print this help message");
                println!();
                println!("SOCKET PATHS:");
                println!("    System mode: /run/anna/anna.sock (requires systemd + anna group)");
                println!("    User mode:   $XDG_RUNTIME_DIR/anna/anna.sock or /tmp/anna-$UID/anna.sock");
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                eprintln!("Run 'annad --help' for usage information");
                return Err(anyhow::anyhow!("Invalid argument"));
            }
        }
    }

    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    if user_mode {
        info!("Anna Daemon {} starting in USER MODE", VERSION);
    } else {
        info!("Anna Daemon {} starting in SYSTEM MODE", VERSION);
    }

    // Store mode for RPC server
    let socket_mode = if user_mode { "user" } else { "system" };

    // Collect initial system facts
    let facts = telemetry::collect_facts().await?;
    info!(
        "System facts collected: {} packages installed",
        facts.installed_packages
    );

    // Generate recommendations
    let mut advice = recommender::generate_advice(&facts);

    let advice_count_before_dedup = advice.len();

    // Deduplicate advice by ID (keep first occurrence)
    let mut seen_ids = std::collections::HashSet::new();
    advice.retain(|a| seen_ids.insert(a.id.clone()));

    let duplicates_removed = advice_count_before_dedup - advice.len();
    if duplicates_removed > 0 {
        info!("Removed {} duplicate advice items", duplicates_removed);
    }

    info!(
        "Generated {} recommendations (Wiki-strict only)",
        advice.len()
    );

    // Initialize daemon state
    let state = Arc::new(DaemonState::new(VERSION.to_string(), facts, advice).await?);

    info!("Anna Daemon ready");

    // Initialize Sentinel framework (Phase 1.0)
    info!("Initializing Sentinel framework...");
    match sentinel::initialize().await {
        Ok(sentinel_daemon) => {
            info!("Sentinel framework initialized - autonomous monitoring enabled");

            // Store sentinel in daemon state (Phase 1.1: for conscience access)
            let sentinel_arc = Arc::new(sentinel_daemon);
            {
                // SAFETY: We're converting Arc<DaemonState> to a mutable reference
                // This is safe because we're the only ones with access at this point
                let state_ptr = Arc::as_ptr(&state) as *mut rpc_server::DaemonState;
                unsafe {
                    (*state_ptr).sentinel = Some(Arc::clone(&sentinel_arc));
                }
            }

            // Spawn sentinel daemon as background task
            tokio::spawn(async move {
                if let Err(e) = sentinel_arc.run().await {
                    tracing::error!("Sentinel daemon error: {}", e);
                }
            });
        }
        Err(e) => {
            tracing::warn!("Failed to initialize Sentinel framework: {}", e);
            tracing::warn!("Continuing without autonomous monitoring");
        }
    }

    // Initialize Collective Mind (Phase 1.3)
    info!("Initializing Collective Mind...");
    match collective::CollectiveMind::new().await {
        Ok(collective_mind) => {
            info!("Collective Mind initialized - distributed cooperation enabled");

            // Store collective in daemon state
            let collective_arc = Arc::new(collective_mind);
            {
                // SAFETY: We're converting Arc<DaemonState> to a mutable reference
                // This is safe because we're the only ones with access at this point
                let state_ptr = Arc::as_ptr(&state) as *mut rpc_server::DaemonState;
                unsafe {
                    (*state_ptr).collective = Some(Arc::clone(&collective_arc));
                }
            }

            // Start collective daemon as background task
            let collective_task = Arc::clone(&collective_arc);
            tokio::spawn(async move {
                if let Err(e) = collective_task.start().await {
                    tracing::error!("Collective Mind daemon error: {}", e);
                }
            });
        }
        Err(e) => {
            tracing::warn!("Failed to initialize Collective Mind: {}", e);
            tracing::warn!("Continuing without distributed cooperation");
        }
    }

    // Initialize Mirror Protocol (Phase 1.4)
    info!("Initializing Mirror Protocol...");
    match mirror::MirrorProtocol::new("anna_node_1".to_string(), "mirror_key_placeholder".to_string()).await {
        Ok(mirror_protocol) => {
            info!("Mirror Protocol initialized - metacognition enabled");

            // Store mirror in daemon state
            let mirror_arc = Arc::new(mirror_protocol);
            {
                // SAFETY: We're converting Arc<DaemonState> to a mutable reference
                // This is safe because we're the only ones with access at this point
                let state_ptr = Arc::as_ptr(&state) as *mut rpc_server::DaemonState;
                unsafe {
                    (*state_ptr).mirror = Some(Arc::clone(&mirror_arc));
                }
            }

            // Start mirror daemon as background task
            let mirror_task = Arc::clone(&mirror_arc);
            tokio::spawn(async move {
                if let Err(e) = mirror_task.start().await {
                    tracing::error!("Mirror Protocol daemon error: {}", e);
                }
            });
        }
        Err(e) => {
            tracing::warn!("Failed to initialize Mirror Protocol: {}", e);
            tracing::warn!("Continuing without recursive introspection");
        }
    }

    // Initialize Chronos Loop (Phase 1.5)
    info!("Initializing Chronos Loop...");
    match chronos::ChronosLoop::new().await {
        Ok(chronos_loop) => {
            info!("Chronos Loop initialized - temporal consciousness enabled");

            // Store chronos in daemon state
            let chronos_arc = Arc::new(chronos_loop);
            {
                // SAFETY: We're converting Arc<DaemonState> to a mutable reference
                // This is safe because we're the only ones with access at this point
                let state_ptr = Arc::as_ptr(&state) as *mut rpc_server::DaemonState;
                unsafe {
                    (*state_ptr).chronos = Some(Arc::clone(&chronos_arc));
                }
            }

            // Start chronos daemon as background task
            let chronos_task = Arc::clone(&chronos_arc);
            tokio::spawn(async move {
                if let Err(e) = chronos_task.start().await {
                    tracing::error!("Chronos Loop daemon error: {}", e);
                }
            });
        }
        Err(e) => {
            tracing::warn!("Failed to initialize Chronos Loop: {}", e);
            tracing::warn!("Continuing without temporal reasoning");
        }
    }

    // Initialize Mirror Audit (Phase 1.6)
    info!("Initializing Mirror Audit...");
    match mirror_audit::MirrorAudit::new(
        "/var/lib/anna/mirror_audit/state.json".to_string(),
        "/var/log/anna/mirror-audit.jsonl".to_string(),
    )
    .await
    {
        Ok(mirror_audit_system) => {
            info!("Mirror Audit initialized - temporal self-reflection enabled");

            // Store mirror audit in daemon state
            let mirror_audit_arc = Arc::new(tokio::sync::RwLock::new(mirror_audit_system));
            {
                // SAFETY: We're converting Arc<DaemonState> to a mutable reference
                // This is safe because we're the only ones with access at this point
                let state_ptr = Arc::as_ptr(&state) as *mut rpc_server::DaemonState;
                unsafe {
                    (*state_ptr).mirror_audit = Some(Arc::clone(&mirror_audit_arc));
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to initialize Mirror Audit: {}", e);
            tracing::warn!("Continuing without temporal audit");
        }
    }

    // Set up system watcher for automatic advice refresh
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let _system_watcher = watcher::SystemWatcher::new(event_tx)?;

    let refresh_state = Arc::clone(&state);
    let mut last_check = std::time::Instant::now();

    // Spawn refresh task
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle file system events
                Some(event) = event_rx.recv() => {
                    match event {
                        watcher::SystemEvent::PackageChange => {
                            info!("Package change detected - refreshing advice");
                            refresh_advice(&refresh_state).await;
                        }
                        watcher::SystemEvent::ConfigChange(path) => {
                            info!("Config change detected: {} - refreshing advice", path);
                            refresh_advice(&refresh_state).await;
                        }
                        watcher::SystemEvent::Reboot => {
                            info!("System reboot detected - refreshing advice");
                            refresh_advice(&refresh_state).await;
                        }
                    }
                }
                // Check for reboot every 30 seconds
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                    if watcher::check_reboot(last_check).await {
                        info!("System reboot detected - refreshing advice");
                        refresh_advice(&refresh_state).await;
                        last_check = std::time::Instant::now();
                    }
                }
            }
        }
    });

    // Spawn autonomous maintenance task
    tokio::spawn(async move {
        // Run autonomy every 6 hours
        let autonomy_interval = tokio::time::Duration::from_secs(6 * 60 * 60);

        loop {
            tokio::time::sleep(autonomy_interval).await;

            info!("Running scheduled autonomous maintenance");
            if let Err(e) = autonomy::run_autonomous_maintenance().await {
                tracing::error!("Autonomous maintenance error: {}", e);
            }
        }
    });

    // Spawn auto-update check task
    tokio::spawn(async move {
        // Check for updates every 2 hours (more frequent during active beta development)
        let update_check_interval = tokio::time::Duration::from_secs(2 * 60 * 60);

        loop {
            tokio::time::sleep(update_check_interval).await;

            info!("Running scheduled update check");
            match anna_common::updater::check_for_updates().await {
                Ok(update_info) => {
                    if update_info.is_update_available {
                        info!(
                            "Update available: {} → {}",
                            update_info.current_version, update_info.latest_version
                        );

                        // Auto-install updates (always-on, no tier required)
                        info!("Auto-installing update...");
                        match anna_common::updater::perform_update(&update_info).await {
                            Ok(_) => {
                                info!("Auto-update installed successfully! Daemon will restart.");

                                // Send notification to user
                                let _ = std::process::Command::new("notify-send")
                                    .arg("--app-name=Anna Assistant")
                                    .arg("--icon=system-software-update")
                                    .arg("--expire-time=10000")
                                    .arg("Anna Updated Automatically")
                                    .arg(&format!(
                                        "Updated from {} to {}",
                                        update_info.current_version, update_info.latest_version
                                    ))
                                    .spawn();

                                // Daemon will be restarted by systemd after binary replacement
                            }
                            Err(e) => {
                                tracing::error!("Auto-update failed: {}", e);
                            }
                        }
                    } else {
                        info!("Already on latest version: {}", update_info.current_version);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to check for updates: {}", e);
                }
            }
        }
    });

    // Start RPC server
    tokio::select! {
        result = rpc_server::start_server(state) => {
            if let Err(e) = result {
                tracing::error!("RPC server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down gracefully");
        }
    }

    Ok(())
}

/// Refresh system facts and regenerate advice
async fn refresh_advice(state: &Arc<DaemonState>) {
    match telemetry::collect_facts().await {
        Ok(facts) => {
            let advice = recommender::generate_advice(&facts);

            // Check for critical issues and notify users
            notifier::check_and_notify_critical(&advice).await;

            // Phase 0.2c: Re-detect system state and log transitions
            match crate::state::detect_state() {
                Ok(new_state) => {
                    let old_state = state.current_state.read().await.state;

                    // Log state transition if changed
                    if new_state.state != old_state {
                        info!(
                            "State transition: {} → {} - {}",
                            old_state, new_state.state, new_state.citation
                        );
                    }

                    // Update cached state
                    *state.current_state.write().await = new_state;
                }
                Err(e) => {
                    tracing::error!("State detection failed during refresh: {}", e);
                }
            }

            // Update state
            *state.facts.write().await = facts;
            *state.advice.write().await = advice.clone();

            info!("Advice refreshed: {} recommendations", advice.len());
        }
        Err(e) => {
            tracing::error!("Failed to refresh advice: {}", e);
        }
    }
}
