//! Anna CLI - user interface to annad.
//! v0.0.83: Added --internal flag for IT department communications view.
//! v0.0.85: Added time_format module for date/tenure display.
//! v0.0.97: Added change_commands module for history/undo.
//! v0.0.109: Added ticket_display module for ticket history.
//! v0.0.110: Added staff_display module for IT department roster.

mod change_commands;
mod client;
mod commands;
mod display;
mod greeting;
mod output;
mod progress_display;
mod report_cmd;
mod staff_display;
mod stats_display;
mod theatre_render;
mod ticket_display;
mod time_format;
mod transcript_render;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{handle_history, handle_repl, handle_request, handle_reset, handle_stats, handle_status, handle_undo, handle_uninstall};
use crate::report_cmd::handle_report;
use crate::staff_display::print_staff_roster;
use crate::ticket_display::{print_ticket_history, TicketFilter};

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
    /// v0.0.109: Show Service Desk ticket history
    /// v0.0.110: Added search and filter options
    Tickets {
        /// Maximum number of tickets to show (default: 10)
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
        /// Filter by team (e.g., --team desktop)
        #[arg(short = 't', long)]
        team: Option<String>,
        /// Search in query text
        #[arg(short = 's', long)]
        search: Option<String>,
        /// Show only escalated tickets
        #[arg(short = 'e', long)]
        escalated: bool,
    },
    /// v0.0.110: Show IT department staff roster and workload
    Staff,
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
        Some(Command::Tickets { limit, team, search, escalated }) => {
            let filter = TicketFilter {
                team,
                search,
                escalated_only: escalated,
                ..Default::default()
            };
            print_ticket_history(limit, &filter);
            Ok(())
        }
        Some(Command::Staff) => {
            print_staff_roster();
            Ok(())
        }
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
