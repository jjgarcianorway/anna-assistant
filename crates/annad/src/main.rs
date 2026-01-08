//! Anna daemon - simplified version.

use anna_shared::VERSION;
use anna_shared::wiki;
use anyhow::Result;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use annad::server::Server;
use annad::state::SharedState;

/// Ollama URL for embeddings
const OLLAMA_URL: &str = "http://127.0.0.1:11434";

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

    // Initialize wiki in background (don't block daemon startup)
    tokio::spawn(async {
        info!("Initializing wiki knowledge base...");
        match wiki::init_wiki(OLLAMA_URL).await {
            Ok(()) => info!("Wiki initialized successfully"),
            Err(e) => warn!("Wiki initialization failed (will use LLM-only mode): {}", e),
        }
    });

    // Create shared state
    let state = SharedState::new();

    // Create and run server
    let server = Server::new(state);
    server.run().await?;

    Ok(())
}
