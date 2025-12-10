//! Anna CLI - user interface to annad.
//! v0.0.144: Simplified CLI - only essential commands, everything else via natural language.

mod change_commands;
mod client;
mod commands;
mod display;
mod greeting;
mod live_request;
mod output;
mod progress_display;
mod spinner;
mod stats_display;
mod stats_display_v2;
mod status_display_v2;
mod status_statistics;
mod status_telemetry;
mod status_learning;
mod theatre_render;
mod time_format;
mod transcript_render;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{handle_repl, handle_request, handle_stats, handle_status, handle_uninstall};

/// Anna - Local AI Assistant
#[derive(Parser)]
#[command(name = "annactl")]
#[command(version = anna_shared::VERSION)]
#[command(about = "Local AI assistant for Linux systems")]
#[command(disable_help_subcommand = true)]
#[command(
    after_help = "EXAMPLES:\n    annactl \"what processes are using the most memory?\"\n    annactl help\n    annactl status\n    annactl stats\n    annactl  # Enter REPL mode"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Natural language request to send to Anna
    #[arg(trailing_var_arg = true)]
    request: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Show Anna's status, health, and configuration
    Status,
    /// Show RPG-style statistics and achievements
    Stats,
    /// Uninstall Anna completely
    Uninstall,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Status) => handle_status().await,
        Some(Command::Stats) => handle_stats().await,
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
