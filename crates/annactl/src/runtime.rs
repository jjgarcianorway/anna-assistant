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

    // Beta.233: Handle help and version flags with correct exit codes
    if args.len() >= 2 {
        match args[1].as_str() {
            "-h" | "--help" => {
                // Parse will print help and exit with 0
                use clap::Parser;
                if let Err(e) = crate::cli::Cli::try_parse() {
                    // For help, clap returns an error but we want exit 0
                    if e.kind() == clap::error::ErrorKind::DisplayHelp {
                        println!("{}", e);
                        std::process::exit(0);
                    }
                }
            }
            "-V" | "--version" => {
                println!("annactl {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => {}
        }
    }

    // Command 1: No arguments → TUI disabled in 6.0.0
    if args.len() == 1 {
        eprintln!("❌ Interactive TUI is disabled in version 6.0.0 (prototype reset)");
        eprintln!();
        eprintln!("Available commands:");
        eprintln!("  annactl status          - System health check");
        eprintln!("  annactl \"<question>\"    - One-shot natural language query");
        eprintln!("  annactl --help          - Show detailed help");
        eprintln!();
        eprintln!("The TUI will return in a future release as a stable feature.");
        std::process::exit(1);
    }

    // Check for natural language query (not a subcommand or flag)
    if args.len() >= 2 {
        let first_arg = &args[1];

        // Known subcommands that should not be treated as natural language queries
        // Beta.233: Added "version" subcommand
        // Beta.236: Removed "brain" from public interface - kept as hidden internal command
        // 6.4.x: Removed plan/selftest as public commands - two-command UX only
        // 6.20.0: Removed "config" - config changes go through natural language
        let known_subcommands = ["status", "version", "brain"];

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

        // Beta.233: Command 4: Version
        Some(crate::cli::Commands::Version) => {
            println!("annactl {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        // 6.20.0: Config command removed - use natural language: annactl "disable emojis"
        // No command → TUI disabled in 6.0.0
        None => {
            eprintln!("❌ Interactive TUI is disabled in version 6.0.0 (prototype reset)");
            eprintln!("Use 'annactl status' or 'annactl \"<question>\"' instead.");
            std::process::exit(1);
        }
    }
}
