//! Anna CLI - user interface to annad.
//! v0.0.144: Simplified CLI - only essential commands, everything else via natural language.
//! v0.0.304: Added user-friendly error presentation module.
//! v0.0.323: Added learning command to show probe learning stats.

mod change_commands;
mod client;
mod commands;
mod display;
mod errors;
mod greeting;
mod live_request;
mod output;
mod progress_display;
mod spinner;
mod stats_display;
mod stats_display_v2;
mod status_display_v2;
mod theatre_render;
mod transcript_render;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{handle_learning, handle_repl, handle_request, handle_reset, handle_stats, handle_status, handle_uninstall};

/// Anna - Local AI Assistant
#[derive(Parser)]
#[command(name = "annactl")]
#[command(version = anna_shared::VERSION)]
#[command(about = "Local AI assistant for Linux systems - ask questions in plain English")]
#[command(disable_help_subcommand = true)]
#[command(
    after_help = "EXAMPLES:
    annactl \"what's using all my memory?\"
    annactl \"show failed services\"
    annactl \"what's my IP address?\"
    annactl \"is my disk getting full?\"
    annactl status              # Show daemon status and health
    annactl stats               # Show staff performance report
    annactl                     # Interactive mode (REPL)

NATURAL LANGUAGE:
    Anna understands plain English. Just ask what you want to know about your system.
    Examples: disk space, memory usage, CPU load, network config, failed services, logs.

INTERACTIVE MODE:
    Run 'annactl' without arguments to enter interactive mode.
    Type 'show internal comms' to see how Anna's team works on your request.
    Type 'annactl help' to see what Anna can do for you."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Natural language request (e.g., \"what's my disk usage?\")
    #[arg(trailing_var_arg = true)]
    request: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Show Anna's status, daemon health, and LLM models
    Status,
    /// Show Service Desk staff performance report
    Stats,
    /// Show what Anna has learned (probe effectiveness)
    Learning,
    /// Reset learned data and event history
    Reset,
    /// Remove Anna completely from the system
    Uninstall,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Status) => handle_status().await,
        Some(Command::Stats) => handle_stats().await,
        Some(Command::Learning) => handle_learning(),
        Some(Command::Reset) => handle_reset().await,
        Some(Command::Uninstall) => handle_uninstall().await,
        None => {
            if cli.request.is_empty() {
                // No args - enter REPL mode
                handle_repl().await
            } else {
                // Join args as a request
                let request = cli.request.join(" ");
                handle_request(&request).await
            }
        }
    }
}
