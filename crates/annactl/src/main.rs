//! Anna Control - CLI client for Anna Assistant
//!
//! Provides user interface to interact with the Anna daemon.
//!
//! Phase 0.3: State-aware command dispatch with no-ops
//! Citation: [archwiki:system_maintenance]

// Phase 0.3a: Commands module will be reimplemented in 0.3c
// mod commands;
mod rpc_client;
pub mod errors;
pub mod output;
pub mod logging;

use anyhow::Result;
use clap::{Parser, Subcommand};
use errors::*;
use anna_common::ipc::{ResponseData, StateDetectionData, CommandCapabilityData};
use output::CommandOutput;
use logging::{LogEntry, ErrorDetails};
use std::time::Instant;

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

/// Get canonical citation for a state (Phase 0.3d)
fn state_citation(state: &str) -> &'static str {
    match state {
        "iso_live" => "[archwiki:installation_guide]",
        "recovery_candidate" => "[archwiki:Chroot#Using_arch-chroot]",
        "post_install_minimal" => "[archwiki:General_recommendations]",
        "configured" => "[archwiki:System_maintenance]",
        "degraded" => "[archwiki:System_maintenance#Troubleshooting]",
        "unknown" => "[archwiki:General_recommendations]",
        _ => "[archwiki:General_recommendations]",
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = Instant::now();
    let req_id = LogEntry::generate_req_id();

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
    let (state, state_citation, capabilities) = match get_state_and_capabilities().await {
        Ok(result) => result,
        Err(e) => {
            // Daemon unavailable
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id,
                state: "unknown".to_string(),
                command: command_name.to_string(),
                allowed: None,
                args: vec![],
                exit_code: EXIT_DAEMON_UNAVAILABLE,
                citation: "[archwiki:system_maintenance]".to_string(),
                duration_ms,
                ok: false,
                error: Some(ErrorDetails {
                    code: "DAEMON_UNAVAILABLE".to_string(),
                    message: format!("Failed to connect to daemon: {}", e),
                }),
            };
            let _ = log_entry.write();

            let output = CommandOutput::daemon_unavailable(command_name.to_string());
            output.print();
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // Phase 0.3d: Handle help command specially
    if matches!(cli.command, Commands::Help { .. }) {
        return execute_help_command(&cli.command, &state, &capabilities, &req_id, start_time).await;
    }

    // Check if command is allowed in current state
    let allowed = capabilities.iter().any(|cap| cap.name == command_name);

    if !allowed {
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let log_entry = LogEntry {
            ts: LogEntry::now(),
            req_id,
            state: state.clone(),
            command: command_name.to_string(),
            allowed: Some(false),
            args: vec![],
            exit_code: EXIT_COMMAND_NOT_AVAILABLE,
            citation: state_citation.clone(),
            duration_ms,
            ok: false,
            error: None,
        };
        let _ = log_entry.write();

        let output = CommandOutput::not_available(state.clone(), command_name.to_string(), state_citation);
        output.print();
        std::process::exit(EXIT_COMMAND_NOT_AVAILABLE);
    }

    // Command is allowed - execute no-op handler and log
    let exit_code = execute_noop_command(&cli.command, &state).await?;
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Find the citation for this specific command
    let command_citation = capabilities
        .iter()
        .find(|cap| cap.name == command_name)
        .map(|cap| cap.citation.clone())
        .unwrap_or_else(|| state_citation.clone());

    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id,
        state: state.clone(),
        command: command_name.to_string(),
        allowed: Some(true),
        args: vec![],
        exit_code,
        citation: command_citation,
        duration_ms,
        ok: true,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

/// Get state and capabilities from daemon (Phase 0.3d)
async fn get_state_and_capabilities() -> Result<(String, String, Vec<CommandCapabilityData>)> {
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

    // Return state name, state citation, and full capabilities list
    let citation = state_citation(&state_data.state);
    Ok((state_data.state, citation.to_string(), capabilities))
}

/// Execute help command (Phase 0.3d)
async fn execute_help_command(
    command: &Commands,
    state: &str,
    capabilities: &[CommandCapabilityData],
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let json_only = matches!(command, Commands::Help { json: true });

    // Sort capabilities alphabetically by name
    let mut sorted_caps = capabilities.to_vec();
    sorted_caps.sort_by(|a, b| a.name.cmp(&b.name));

    if json_only {
        // JSON output only
        let commands: Vec<serde_json::Value> = sorted_caps
            .iter()
            .map(|cap| {
                serde_json::json!({
                    "name": cap.name,
                    "desc": cap.description,
                    "citation": cap.citation
                })
            })
            .collect();

        let output = serde_json::json!({
            "version": VERSION,
            "ok": true,
            "state": state,
            "commands": commands
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Human-readable output
        println!("current state: {}", state);
        for cap in &sorted_caps {
            println!(
                "{:<16}  {:<50}  {}",
                cap.name, cap.description, cap.citation
            );
        }
    }

    // Log the help command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "help".to_string(),
        allowed: Some(true),
        args: if json_only { vec!["--json".to_string()] } else { vec![] },
        exit_code: EXIT_SUCCESS,
        citation: state_citation(state).to_string(),
        duration_ms,
        ok: true,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(EXIT_SUCCESS);
}

/// Execute no-op command handler (Phase 0.3c)
/// All commands just print success message and return exit code
async fn execute_noop_command(command: &Commands, state: &str) -> Result<i32> {
    match command {
        Commands::Status => {
            println!("[anna] state: {}", state);
            println!("[anna] status: OK (no-op placeholder)");
        }
        Commands::Help { .. } => {
            // Should not reach here - handled in main
            unreachable!("Help command should be handled separately");
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

    Ok(EXIT_SUCCESS)
}
