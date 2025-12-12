//! Anna CLI - user interface to annad.
//! v0.0.144: Simplified CLI - only essential commands, everything else via natural language.
//! v0.0.304: Added user-friendly error presentation module.
//! v0.0.323: Added learning command to show probe learning stats.
//! v0.0.328: Added query test option to learning command.
//! v0.0.406: Added suggest-recipes command for recipe candidate analysis.
//! v0.0.413: Hollywood IT department view with cinematic/debug modes.

mod change_commands;
mod client;
mod commands;
mod display;
mod errors;
mod greeting;
mod live_renderer;
mod live_request;
mod output;
mod progress_display;
mod spinner;
mod stats_display;
mod stats_display_v2;
mod status_display_v2;
mod theatre_render;
mod transcript_render;

use anna_shared::transcript_segment::TranscriptMode;
use anna_shared::ui_config::UiConfig;
use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{handle_debug, handle_learning_with_query, handle_suggest_recipes, handle_repl, handle_request, handle_reset, handle_stats, handle_status, handle_uninstall, DebugCommand};

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

    /// Use cinematic mode (Hollywood IT department view)
    #[arg(long, global = true)]
    cinematic: bool,

    /// Use debug mode (raw JSON, full errors)
    #[arg(long, global = true)]
    debug: bool,

    /// Hide internal comms (IT department chatter)
    #[arg(long, global = true)]
    no_internal_comms: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Show Anna's status, daemon health, and LLM models
    Status,
    /// Show Service Desk staff performance report
    Stats,
    /// Show what Anna has learned (probe effectiveness)
    Learning {
        /// Test recommendations for a query (e.g., "how's my disk?")
        #[arg(long, short = 't')]
        test: Option<String>,
    },
    /// Analyze ticket logs and suggest recipes (v0.0.406)
    SuggestRecipes {
        /// Number of recent tickets to analyze (default: 100)
        #[arg(long, short = 'n')]
        limit: Option<usize>,
    },
    /// Debug diagnostics (v0.0.444)
    Debug {
        #[command(subcommand)]
        cmd: DebugCommand,
    },
    /// Reset learned data and event history
    Reset,
    /// Remove Anna completely from the system
    Uninstall,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Build UI config from CLI flags
    let mode = if cli.debug {
        Some(TranscriptMode::Debug)
    } else if cli.cinematic {
        Some(TranscriptMode::Cinematic)
    } else {
        None
    };

    let ui_config = UiConfig::load().with_cli_overrides(mode, cli.no_internal_comms);

    // Store config in thread-local for handlers to access
    set_ui_config(ui_config);

    match cli.command {
        Some(Command::Status) => handle_status().await,
        Some(Command::Stats) => handle_stats().await,
        Some(Command::Learning { test }) => handle_learning_with_query(test.as_deref()),
        Some(Command::SuggestRecipes { limit }) => handle_suggest_recipes(limit),
        Some(Command::Debug { cmd }) => handle_debug(cmd).await,
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

// Thread-local UI config
thread_local! {
    static UI_CONFIG: std::cell::RefCell<UiConfig> = std::cell::RefCell::new(UiConfig::default());
}

/// Set UI config
pub fn set_ui_config(config: UiConfig) {
    UI_CONFIG.with(|c| *c.borrow_mut() = config);
}

/// Get UI config
pub fn get_ui_config() -> UiConfig {
    UI_CONFIG.with(|c| c.borrow().clone())
}
