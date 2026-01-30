//! Anna daemon - simplified version.
//! v0.0.924: Added proactive helper tool installation on startup
//! v0.0.927: Added LLM warmup on startup for faster first query
//! v0.0.942: Added memory optimization on startup
//! v0.0.999: Added systemd watchdog for automatic recovery from freezes

use anna_shared::config::{get_ollama_url, AnnaConfig};
use anna_shared::deps::{missing_diagnostic_tools, install_missing_diagnostic_tools};
use anna_shared::memory::Memory;
use anna_shared::VERSION;
use anna_shared::wiki;
use anyhow::Result;
use sd_notify::NotifyState;  // Used for watchdog pings
use tracing::{debug, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use annad::ollama;
use annad::server::Server;
use annad::state::SharedState;

#[tokio::main]
async fn main() -> Result<()> {
    // Handle --version flag (needed for update verification)
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" | "-V" => {
                println!("annad {}", VERSION);
                return Ok(());
            }
            "--help" | "-h" => {
                println!("annad {} - Anna Assistant Daemon", VERSION);
                println!();
                println!("Usage: annad");
                println!();
                println!("The daemon runs as a systemd service.");
                println!("Control it with: systemctl start|stop|restart annad");
                return Ok(());
            }
            _ => {}
        }
    }

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting annad v{}", VERSION);
    info!("=== DAEMON STARTUP BEGIN ===");

    // v0.0.893: Initialize wiki with retry loop
    // v0.0.895: Use centralized config for Ollama URL
    let ollama_url = get_ollama_url();
    tokio::spawn(async move {
        info!("Initializing wiki knowledge base...");
        let mut attempts = 0;
        let max_attempts = 3;
        let mut delay = std::time::Duration::from_secs(2);
        loop {
            attempts += 1;
            match wiki::init_wiki(&ollama_url).await {
                Ok(()) => { info!("Wiki initialized successfully"); break; }
                Err(e) if attempts >= max_attempts => {
                    warn!("Wiki init failed after {} attempts (LLM-only mode): {}", attempts, e);
                    break;
                }
                Err(e) => {
                    warn!("Wiki init attempt {}/{} failed: {}, retrying in {:?}", attempts, max_attempts, e, delay);
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }
    });

    // Warm up command cache in background (pre-cache static system info)
    std::thread::spawn(|| {
        annad::core_loop::warm_up_cache();
        // v0.0.953: Run proactive health checks after cache warm-up
        annad::core_loop::run_health_checks();
        // v0.3.95: Run self-healing on startup
        let healed = annad::self_healing::run_self_healing();
        if !healed.is_empty() {
            let successful: Vec<_> = healed.iter().filter(|r| r.success).collect();
            if !successful.is_empty() {
                info!("Startup self-healing: fixed {} issues", successful.len());
            }
        }
    });

    // v0.0.954: Periodic health checks (every 5 minutes)
    tokio::spawn(async {
        use std::time::Duration;
        // Wait 5 minutes before first periodic check (initial check runs at startup)
        tokio::time::sleep(Duration::from_secs(300)).await;
        loop {
            tokio::task::spawn_blocking(|| {
                debug!("Running periodic health check...");
                annad::core_loop::run_health_checks();
            }).await.ok();
            tokio::time::sleep(Duration::from_secs(300)).await; // Every 5 minutes
        }
    });

    // v0.0.924: Check for missing diagnostic tools
    let config = AnnaConfig::load().unwrap_or_default();
    if config.auto_install_helpers {
        tokio::spawn(async move {
            let missing = missing_diagnostic_tools();
            if !missing.is_empty() {
                info!("Found {} missing diagnostic tools, installing...", missing.len());
                for (tool, desc) in &missing {
                    debug!("Missing: {} - {}", tool, desc);
                }
                let installed = tokio::task::spawn_blocking(install_missing_diagnostic_tools)
                    .await
                    .unwrap_or_default();
                if !installed.is_empty() {
                    info!("Installed diagnostic tools: {}", installed.join(", "));
                }
            }
        });
    } else {
        // Just log if tools are missing but auto-install is disabled
        let missing = missing_diagnostic_tools();
        if !missing.is_empty() {
            debug!("{} diagnostic tools not installed (enable auto_install_helpers to install)", missing.len());
        }
    }

    // v0.0.927: Warm up LLM connection in background (loads model into memory)
    tokio::spawn(async {
        // Wait a bit for Ollama to be fully ready
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        if ollama::is_running().await {
            info!("Warming up LLM connection...");
            // Send a minimal prompt to load the model
            match ollama::chat_with_timeout(
                "llama3.2",  // Default model, will be overridden by actual config
                "Hi",
                10,
            ).await {
                Ok(_) => info!("LLM warmup complete - model loaded"),
                Err(e) => debug!("LLM warmup skipped: {}", e),
            }
        } else {
            debug!("Ollama not running, skipping LLM warmup");
        }
    });

    // v0.0.942: Optimize memory on startup (deduplicate and compact)
    std::thread::spawn(|| {
        match Memory::load() {
            Ok(mut memory) => {
                let initial_exp = memory.experiences.len();
                let initial_clusters = memory.clusters.len();

                if initial_exp > 0 {
                    let (exp_removed, clusters_removed) = memory.optimize(1000); // Max 1000 experiences
                    if exp_removed > 0 || clusters_removed > 0 {
                        info!(
                            "Memory optimized: {} experiences ({} removed), {} clusters ({} removed)",
                            memory.experiences.len(), exp_removed,
                            memory.clusters.len(), clusters_removed
                        );
                        if let Err(e) = memory.save() {
                            warn!("Failed to save optimized memory: {}", e);
                        }
                    } else {
                        debug!("Memory already optimized: {} experiences, {} clusters", initial_exp, initial_clusters);
                    }
                }
            }
            Err(e) => debug!("No memory to optimize: {}", e),
        }
    });

    // Create shared state
    let state = SharedState::new();

    // v0.1.1: Binary watcher - auto-restart when binary changes (for dev workflow)
    tokio::spawn(async {
        annad::binary_watcher::binary_watch_loop().await;
    });

    // v0.0.999: Systemd watchdog - ping every 30s to prove we're alive
    // If we freeze, systemd will kill and restart us
    tokio::spawn(async {
        use std::time::Duration;
        let watchdog_usec = std::env::var("WATCHDOG_USEC")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        if watchdog_usec > 0 {
            // Ping at half the watchdog interval
            let ping_interval = Duration::from_micros(watchdog_usec / 2);
            info!("Watchdog enabled, pinging every {:?}", ping_interval);

            loop {
                tokio::time::sleep(ping_interval).await;
                let _ = sd_notify::notify(false, &[NotifyState::Watchdog]);
            }
        } else {
            debug!("Watchdog not enabled (not running under systemd?)");
        }
    });

    // Create and run server
    // v0.3.38: Systemd notify moved to server.run() AFTER socket is ready
    info!("=== CALLING SERVER.RUN() ===");
    let server = Server::new(state);
    let result = server.run().await;
    info!("=== SERVER.RUN() RETURNED: {:?} ===", result.is_ok());
    result
}
