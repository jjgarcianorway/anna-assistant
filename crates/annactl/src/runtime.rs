//! Runtime - Application dispatch and execution
//!
//! Beta.146: Extracted from main.rs to keep it minimal
//!
//! Responsibilities:
//! - Determine which mode to run (TUI, CLI query, status, etc.)
//! - Dispatch to appropriate handlers
//! - Coordinate between modules

use anyhow::Result;
use std::time::Instant;

use crate::errors::*;
use crate::logging::LogEntry;

/// Main application entry point after CLI parsing
///
/// This is called from main() with parsed arguments.
/// Determines which mode to run and dispatches appropriately.
pub async fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // If no arguments, start TUI
    if args.len() == 1 {
        return crate::tui_v2::run().await;
    }

    // Check for natural language query (not a subcommand or flag)
    if args.len() >= 2 {
        let first_arg = &args[1];

        // Known subcommands that should not be treated as natural language queries
        let known_subcommands = ["status", "version", "ping", "tui", "historian"];

        let is_flag = first_arg.starts_with("--") || first_arg.starts_with("-");
        let is_known_command = known_subcommands.contains(&first_arg.as_str());

        // If not a flag and not a known command, treat as natural language
        if !is_flag && !is_known_command {
            let query = args[1..].join(" ");
            return crate::llm_query_handler::handle_one_shot_query(&query).await;
        }
    }

    // Parse CLI for structured commands
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse()?;

    // Dispatch based on command
    let start_time = Instant::now();
    let req_id = LogEntry::generate_req_id();

    dispatch_command(cli, start_time, req_id).await
}

/// Dispatch to appropriate command handler
async fn dispatch_command(
    cli: crate::cli::Cli,
    _start_time: Instant,
    _req_id: String,
) -> Result<()> {
    use crate::cli::Commands;

    match cli.command {
        Some(Commands::Status { json: _ }) => {
            // TODO: Re-implement status command
            println!("Status command temporarily disabled during refactoring");
            Ok(())
        }

        Some(Commands::Tui) => {
            crate::tui_v2::run().await
        }

        Some(Commands::Ping) => {
            println!("pong");
            Ok(())
        }

        Some(Commands::Version) => {
            println!("annactl {}", env!("ANNA_VERSION"));
            Ok(())
        }

        Some(Commands::Historian { action: _ }) => {
            // TODO: Re-implement historian command
            println!("Historian command temporarily disabled during refactoring");
            Ok(())
        }

        None => {
            crate::tui_v2::run().await
        }
    }
}
