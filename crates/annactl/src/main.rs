//! Anna CLI - user interface to annad.

mod client;
mod commands;
mod display;
mod output;
mod progress_display;
mod report_cmd;
mod stats_display;
mod theatre_render;
mod transcript_render;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{handle_repl, handle_request, handle_reset, handle_stats, handle_status, handle_uninstall};
use crate::report_cmd::handle_report;

/// Anna - Local AI Assistant
#[derive(Parser)]
#[command(name = "annactl")]
#[command(version = anna_shared::VERSION)]
#[command(about = "Local AI assistant for Linux systems")]
#[command(disable_help_subcommand = true)] // Prevent "help" from triggering clap help
#[command(
    after_help = "EXAMPLES:\n    annactl \"what processes are using the most memory?\"\n    annactl status\n    annactl  # Enter REPL mode\n    annactl help  # Ask Anna for help"
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
    /// Show Anna status
    Status {
        /// Show extended debug information (latency stats)
        #[arg(long)]
        debug: bool,
    },
    /// Show per-team statistics (v0.0.27)
    Stats,
    /// Generate a system health report
    Report {
        /// Output format: text (default) or md
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Uninstall Anna
    Uninstall,
    /// Reset learned data (keeps base installation)
    Reset,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Status { debug }) => handle_status(debug).await,
        Some(Command::Stats) => handle_stats().await,
        Some(Command::Report { format }) => handle_report(&format).await,
        Some(Command::Uninstall) => handle_uninstall().await,
        Some(Command::Reset) => handle_reset().await,
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
