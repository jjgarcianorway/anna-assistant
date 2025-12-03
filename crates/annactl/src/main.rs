//! Anna CLI (annactl) v0.0.4 - Real Junior Verifier
//!
//! Public CLI surface (strict):
//! - annactl                  REPL mode (interactive)
//! - annactl <request...>     one-shot natural language request
//! - annactl status           self-status
//! - annactl --version        version (also: -V)
//!
//! All other commands route through natural language processing.
//! Internal capabilities (sw, hw, snapshots) are accessed via requests.
//!
//! v0.0.4: Junior becomes real via Ollama, Translator stays deterministic.
//! Multi-party dialogue transcript:
//! [you] -> [anna] -> [translator] -> [anna] -> [annad] -> [anna] -> [junior] -> [anna] -> [you]

mod commands;
mod pipeline;

use anna_common::{AnnaConfig, OllamaClient, select_junior_model};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use std::io::{self, Write};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const THIN_SEP: &str = "------------------------------------------------------------";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annactl=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer().without_time().with_target(false))
        .init();

    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        // annactl (no args) - REPL mode
        [] => run_repl().await,

        // annactl --version or -V
        [cmd] if cmd == "--version" || cmd == "-V" => run_version(),

        // annactl --help or -h (show help, not REPL)
        [cmd] if cmd == "--help" || cmd == "-h" => run_help(),

        // annactl status - self-status
        [cmd] if cmd.eq_ignore_ascii_case("status") => commands::status::run().await,

        // Everything else is a natural language request
        _ => run_request(&args.join(" ")).await,
    }
}

/// Check and display Junior LLM status
async fn check_junior_status() -> Option<String> {
    let config = AnnaConfig::load();

    if !config.junior.enabled {
        println!("  {} Junior LLM disabled in config", "[!]".yellow());
        return None;
    }

    let client = OllamaClient::with_url(&config.junior.ollama_url);

    if !client.is_available().await {
        println!("  {} Ollama not available at {}", "[!]".yellow(), config.junior.ollama_url);
        println!("      Install: curl -fsSL https://ollama.ai/install.sh | sh");
        println!("      Junior will use fallback (deterministic) scoring.");
        return None;
    }

    // Get model
    let model = if config.junior.model.is_empty() {
        match client.list_models().await {
            Ok(models) => {
                let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
                match select_junior_model(&model_names) {
                    Some(m) => m,
                    None => {
                        println!("  {} Ollama running but no models installed", "[!]".yellow());
                        println!("      Install: ollama pull qwen2.5:1.5b");
                        println!("      Junior will use fallback (deterministic) scoring.");
                        return None;
                    }
                }
            }
            Err(_) => {
                println!("  {} Failed to list Ollama models", "[!]".yellow());
                return None;
            }
        }
    } else {
        match client.has_model(&config.junior.model).await {
            Ok(true) => config.junior.model.clone(),
            _ => {
                println!("  {} Model '{}' not found", "[!]".yellow(), config.junior.model);
                println!("      Install: ollama pull {}", config.junior.model);
                println!("      Junior will use fallback (deterministic) scoring.");
                return None;
            }
        }
    };

    println!("  {} Junior LLM: {} via Ollama", "[*]".green(), model);
    Some(model)
}

/// REPL mode - interactive natural language chat
async fn run_repl() -> Result<()> {
    println!();
    println!("{}", "  Anna Assistant v0.0.4".bold());
    println!("{}", THIN_SEP);
    println!("  Natural language interface to your system.");

    // Check Junior status
    check_junior_status().await;

    println!("  Type your question or request. Type 'exit' to quit.");
    println!("{}", THIN_SEP);
    println!();

    loop {
        print!("  {} ", "you>".bright_white().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("exit")
            || input.eq_ignore_ascii_case("quit")
            || input.eq_ignore_ascii_case("bye")
            || input == "q"
        {
            println!();
            println!("  Goodbye!");
            println!();
            break;
        }

        if input.eq_ignore_ascii_case("help") {
            print_repl_help();
            continue;
        }

        if input.eq_ignore_ascii_case("status") {
            commands::status::run().await?;
            continue;
        }

        // Process as natural language request
        process_request(input).await;
    }

    Ok(())
}

/// One-shot natural language request
async fn run_request(request: &str) -> Result<()> {
    println!();
    process_request(request).await;
    println!();
    Ok(())
}

/// Process a natural language request through the pipeline
async fn process_request(request: &str) {
    // v0.0.3: Full multi-party dialogue pipeline
    // [you] -> [anna] -> [translator] -> [anna] -> [annad] -> [anna] -> [junior] -> [anna] -> [you]
    pipeline::process(request).await;
}

/// Print REPL help
fn print_repl_help() {
    println!();
    println!("  {}", "Commands:".bold());
    println!("    exit, quit, bye, q  - leave REPL");
    println!("    status              - show Anna's status");
    println!("    help                - show this help");
    println!();
    println!("  {}", "Examples:".bold());
    println!("    what CPU do I have?");
    println!("    is nginx running?");
    println!("    show me disk usage");
    println!();
}

/// Print version
fn run_version() -> Result<()> {
    println!("annactl v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

/// Print help (--help flag)
fn run_help() -> Result<()> {
    println!();
    println!("{}", "  Anna Assistant".bold());
    println!("{}", THIN_SEP);
    println!("  annactl                  interactive mode (REPL)");
    println!("  annactl <request>        one-shot natural language request");
    println!("  annactl status           show Anna's status");
    println!("  annactl --version        show version");
    println!("{}", THIN_SEP);
    println!();
    println!("  {}", "Examples:".bold());
    println!("    annactl \"what CPU do I have?\"");
    println!("    annactl \"is docker running?\"");
    println!("    annactl \"show me recent errors\"");
    println!();
    Ok(())
}
