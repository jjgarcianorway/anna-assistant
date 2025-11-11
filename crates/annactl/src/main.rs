//! Anna Control - CLI client for Anna Assistant
//!
//! Provides user interface to interact with the Anna daemon.
//!
//! Phase 0.3: State-aware command dispatch with no-ops
//! Citation: [archwiki:system_maintenance]

// Phase 0.3a: Commands module will be reimplemented in 0.3c
// mod commands;
mod rpc_client;
mod errors;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use errors::*;
use anna_common::ipc::{ResponseData, StateDetectionData, CommandCapabilityData};
use output::CommandOutput;

// Version is embedded at build time
const VERSION: &str = env!("ANNA_VERSION");

#[derive(Parser)]
#[command(name = "annactl")]
#[command(about = "Anna Assistant - Autonomous system administrator", long_about = None)]
#[command(version = VERSION)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Phase 0.3: Simplified command set - all commands check state availability
#[derive(Subcommand)]
enum Commands {
    /// Show system status and daemon health
    Status,

    /// Show available commands for current state
    Help {
        /// Output JSON only
        #[arg(long)]
        json: bool,
    },

    /// Update system packages (configured state only)
    Update {
        /// Dry run (show what would be updated)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Interactive Arch Linux installation (iso_live state only)
    Install,

    /// Rescue and recovery tools (iso_live, recovery_candidate states)
    Rescue {
        /// Subcommand: detect, chroot, repair
        subcommand: Option<String>,
    },

    /// Create system backup (configured state)
    Backup {
        /// Destination path
        #[arg(short, long)]
        dest: Option<String>,
    },

    /// Check system health (all states)
    Health,

    /// Run system diagnostics (configured, degraded states)
    Doctor {
        /// Automatically fix issues
        #[arg(short, long)]
        fix: bool,
    },

    /// Rollback actions (configured, degraded states)
    Rollback {
        /// Action ID or 'last'
        target: Option<String>,
    },

    /// Analyze system issues (degraded state only)
    Triage,

    /// Collect diagnostic logs (degraded state only)
    CollectLogs {
        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },
}

// Phase 0.3: Remove all legacy subcommand enums
// Commands are now flat and state-checked at runtime

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Phase 0.3c: State-aware dispatch
    // Get command name first
    let command_name = match &cli.command {
        Commands::Status => "status",
        Commands::Help { .. } => "help",
        Commands::Update { .. } => "update",
        Commands::Install => "install",
        Commands::Rescue { .. } => "rescue",
        Commands::Backup { .. } => "backup",
        Commands::Health => "health",
        Commands::Doctor { .. } => "doctor",
        Commands::Rollback { .. } => "rollback",
        Commands::Triage => "triage",
        Commands::CollectLogs { .. } => "collect-logs",
    };

    // Try to connect to daemon and get state
    let (state, citation, allowed) = match get_state_and_check(command_name).await {
        Ok(result) => result,
        Err(_) => {
            // Daemon unavailable
            let output = CommandOutput::daemon_unavailable(command_name.to_string());
            output.print();
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // Check if command is allowed in current state
    if !allowed {
        let output = CommandOutput::not_available(state.clone(), command_name.to_string(), citation);
        output.print();
        std::process::exit(EXIT_COMMAND_NOT_AVAILABLE);
    }

    // Command is allowed - execute no-op handler
    execute_noop_command(&cli.command, &state).await
}

/// Get state from daemon and check if command is allowed
async fn get_state_and_check(command_name: &str) -> Result<(String, String, bool)> {
    let mut client = rpc_client::RpcClient::connect().await?;

    // Get state
    let state_response = client.get_state().await?;
    let state_data = match state_response {
        ResponseData::StateDetection(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetState"),
    };

    // Get capabilities
    let caps_response = client.get_capabilities().await?;
    let capabilities = match caps_response {
        ResponseData::Capabilities(caps) => caps,
        _ => anyhow::bail!("Unexpected response type for GetCapabilities"),
    };

    // Check if command is in capabilities list
    let allowed = capabilities.iter().any(|cap| cap.name == command_name);

    Ok((state_data.state, state_data.citation, allowed))
}

/// Execute no-op command handler (Phase 0.3c)
/// All commands just print success message and exit 0
async fn execute_noop_command(command: &Commands, state: &str) -> Result<()> {
    match command {
        Commands::Status => {
            println!("[anna] state: {}", state);
            println!("[anna] status: OK (no-op placeholder)");
        }
        Commands::Help { json } => {
            println!("[anna] help command (json={}) - will be implemented in 0.3d", json);
        }
        Commands::Update { dry_run } => {
            println!("[anna] update command allowed in state: {}", state);
            println!("[anna] dry_run={} (no-op - no actual update performed)", dry_run);
        }
        Commands::Install => {
            println!("[anna] install command allowed in state: {}", state);
            println!("[anna] (no-op - no actual installation performed)");
        }
        Commands::Rescue { subcommand } => {
            println!("[anna] rescue command allowed in state: {}", state);
            println!("[anna] subcommand: {:?} (no-op)", subcommand);
        }
        Commands::Backup { dest } => {
            println!("[anna] backup command allowed in state: {}", state);
            println!("[anna] dest: {:?} (no-op - no actual backup performed)", dest);
        }
        Commands::Health => {
            println!("[anna] health command allowed in state: {}", state);
            println!("[anna] (no-op - no actual health check performed)");
        }
        Commands::Doctor { fix } => {
            println!("[anna] doctor command allowed in state: {}", state);
            println!("[anna] fix={} (no-op - no actual diagnostics performed)", fix);
        }
        Commands::Rollback { target } => {
            println!("[anna] rollback command allowed in state: {}", state);
            println!("[anna] target: {:?} (no-op - no actual rollback performed)", target);
        }
        Commands::Triage => {
            println!("[anna] triage command allowed in state: {}", state);
            println!("[anna] (no-op - no actual triage performed)");
        }
        Commands::CollectLogs { output } => {
            println!("[anna] collect-logs command allowed in state: {}", state);
            println!("[anna] output: {:?} (no-op - no logs collected)", output);
        }
    }

    std::process::exit(EXIT_SUCCESS);
}
