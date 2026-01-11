//! Anna daemon - simplified version.
//! v0.0.924: Added proactive helper tool installation on startup

use anna_shared::config::{get_ollama_url, AnnaConfig};
use anna_shared::deps::{missing_diagnostic_tools, install_missing_diagnostic_tools};
use anna_shared::VERSION;
use anna_shared::wiki;
use anyhow::Result;
use tracing::{debug, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

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

    // Create shared state
    let state = SharedState::new();

    // Create and run server
    let server = Server::new(state);
    server.run().await?;

    Ok(())
}
