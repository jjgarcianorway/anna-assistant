//! Anna CLI - user interface to annad.
//! v0.0.83: Added --internal flag for IT department communications view.
//! v0.0.85: Added time_format module for date/tenure display.
//! v0.0.97: Added change_commands module for history/undo.
//! v0.0.111: Removed tickets/staff CLI commands - use natural language instead.
//! v0.0.113: Added reply/ticket commands for async ticket workflow.
//! v0.0.116: Removed inbox command - use natural language instead.
//! v0.0.142: Added animated spinner module for real-time feedback.

mod change_commands;
mod client;
mod commands;
mod display;
mod greeting;
mod output;
mod progress_display;
mod report_cmd;
mod spinner; // v0.0.142
mod stats_display;
mod theatre_render;
mod ticket_commands; // v0.0.113
mod time_format;
mod transcript_render;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{
    handle_history, handle_repl, handle_request, handle_reset, handle_stats, handle_status,
    handle_undo, handle_uninstall,
};
use crate::report_cmd::handle_report;
use crate::ticket_commands::{handle_email, handle_health, handle_reply, handle_ticket};

/// Anna - Local AI Assistant
#[derive(Parser)]
#[command(name = "annactl")]
#[command(version = anna_shared::VERSION)]
#[command(about = "Local AI assistant for Linux systems")]
#[command(disable_help_subcommand = true)] // Prevent "help" from triggering clap help
#[command(
    after_help = "EXAMPLES:\n    annactl \"what processes are using the most memory?\"\n    annactl status\n    annactl  # Enter REPL mode\n    annactl --internal  # Enter REPL with IT department view\n    annactl help  # Ask Anna for help"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Show internal IT department communications (fly-on-the-wall view)
    #[arg(short = 'i', long = "internal", global = true)]
    show_internal: bool,

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
    /// v0.0.97: Show config change history
    History,
    /// v0.0.97: Undo a config change by ID
    Undo {
        /// Change ID to undo (from history)
        id: String,
    },
    /// v0.0.113: Reply to an open ticket
    Reply {
        /// Case number (e.g., CN-0001-06122025)
        case: String,
        /// Your reply message
        message: String,
    },
    /// v0.0.113: Show a ticket's conversation
    Ticket {
        /// Case number (e.g., CN-0001-06122025)
        case: String,
    },
    /// v0.0.113: Configure your email for notifications
    Email {
        /// Your email address (or "off" to disable)
        address: String,
    },
    /// v0.0.114: Check Anna's health and dependencies
    Health,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let show_internal = cli.show_internal;

    match cli.command {
        Some(Command::Status { debug }) => handle_status(debug).await,
        Some(Command::Stats) => handle_stats().await,
        Some(Command::Report { format }) => handle_report(&format).await,
        Some(Command::Uninstall) => handle_uninstall().await,
        Some(Command::Reset) => handle_reset().await,
        Some(Command::History) => handle_history().await,
        Some(Command::Undo { id }) => handle_undo(&id).await,
        Some(Command::Reply { case, message }) => handle_reply(&case, &message).await,
        Some(Command::Ticket { case }) => handle_ticket(&case).await,
        Some(Command::Email { address }) => handle_email(&address).await,
        Some(Command::Health) => handle_health().await,
        None => {
            if cli.request.is_empty() {
                // No args - enter REPL mode
                handle_repl(show_internal).await
            } else {
                // Join args as a request
                let request = cli.request.join(" ");
                handle_request(&request, show_internal).await
            }
        }
    }
}
