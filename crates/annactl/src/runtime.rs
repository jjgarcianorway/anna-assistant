//! Runtime - Application dispatch and execution
//!
//! Beta.200: Three-command architecture
//!
//! Responsibilities:
//! - Determine which mode to run (TUI, status, or one-shot query)
//! - Dispatch to appropriate handlers
//! - Coordinate between modules
//!
//! Commands:
//! 1. `annactl` (no args) → TUI
//! 2. `annactl status` → system health check
//! 3. `annactl "<question>"` → one-shot query

use anyhow::Result;
use std::time::Instant;

use crate::logging::LogEntry;

/// Main application entry point after CLI parsing
///
/// This is called from main() and determines which mode to run.
pub async fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Beta.214: Handle version flag with clean output (no "Error:" prefix)
    if args.len() == 2 && (args[1] == "-V" || args[1] == "--version") {
        println!("annactl {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    // Command 1: No arguments → Start TUI
    if args.len() == 1 {
        return crate::tui_v2::run().await;
    }

    // Check for natural language query (not a subcommand or flag)
    if args.len() >= 2 {
        let first_arg = &args[1];

        // Known subcommands that should not be treated as natural language queries
        // Beta.200: Only "status" is a real subcommand
        let known_subcommands = ["status"];

        let is_flag = first_arg.starts_with("--") || first_arg.starts_with("-");
        let is_known_command = known_subcommands.contains(&first_arg.as_str());

        // Command 3: Natural language query
        // If not a flag and not "status", treat as natural language
        if !is_flag && !is_known_command {
            let query = args[1..].join(" ");
            return crate::llm_query_handler::handle_one_shot_query(&query).await;
        }
    }

    // Parse CLI for structured commands (only "status" at this point)
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse()?;

    // Command 2: Status
    match cli.command {
        Some(crate::cli::Commands::Status { json }) => {
            let start_time = Instant::now();
            let req_id = LogEntry::generate_req_id();

            crate::status_command::execute_anna_status_command(json, &req_id, start_time)
                .await
        }

        // Beta.217c: Command 3: Brain Analysis
        Some(crate::cli::Commands::Brain { json, verbose }) => {
            let start_time = Instant::now();
            let req_id = LogEntry::generate_req_id();

            crate::brain_command::execute_brain_command(json, verbose, &req_id, start_time)
                .await
        }

        // No command → TUI (should be caught earlier, but handle it anyway)
        None => crate::tui_v2::run().await,
    }
}
