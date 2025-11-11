//! Anna Control - CLI client for Anna Assistant
//!
//! Provides user interface to interact with the Anna daemon.
//!
//! Phase 0.3: State-aware command dispatch with no-ops
//! Citation: [archwiki:system_maintenance]

// Phase 0.3a: Commands module will be reimplemented in 0.3c
// mod commands;
pub mod errors;
mod conscience_commands; // Phase 1.1
mod health_commands;
mod install_command; // Phase 0.8
pub mod logging;
pub mod output;
mod rpc_client; // Phase 0.5b
mod sentinel_cli; // Phase 1.0
mod steward_commands; // Phase 0.9

use anna_common::ipc::{CommandCapabilityData, ResponseData};
use anyhow::Result;
use clap::{Parser, Subcommand};
use errors::*;
use logging::{ErrorDetails, LogEntry};
use output::CommandOutput;
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
    Install {
        /// Dry run (simulate without executing)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

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

    /// Check system health (all states) - Phase 0.5
    Health {
        /// Output JSON only
        #[arg(long)]
        json: bool,
    },

    /// Run system diagnostics (configured, degraded states) - Phase 0.5
    Doctor {
        /// Output JSON only
        #[arg(long)]
        json: bool,
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

    /// Repair failed probes (Phase 0.7)
    Repair {
        /// Probe to repair (or "all" for all failed probes)
        #[arg(default_value = "all")]
        probe: String,

        /// Dry run (simulate without executing)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// System audit - integrity and security check (Phase 0.9)
    Audit,

    /// Sentinel management (Phase 1.0)
    Sentinel {
        /// Subcommand: status, metrics
        #[command(subcommand)]
        subcommand: SentinelSubcommand,
    },

    /// Configuration management (Phase 1.0)
    Config {
        /// Subcommand: get, set
        #[command(subcommand)]
        subcommand: ConfigSubcommand,
    },

    /// Conscience governance (Phase 1.1)
    Conscience {
        /// Subcommand: review, explain, approve, reject, introspect
        #[command(subcommand)]
        subcommand: ConscienceSubcommand,
    },
}

/// Sentinel subcommands
#[derive(Subcommand)]
enum SentinelSubcommand {
    /// Show sentinel status
    Status,
    /// Show sentinel metrics
    Metrics,
}

/// Config subcommands
#[derive(Subcommand)]
enum ConfigSubcommand {
    /// Get configuration value
    Get,
    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
}

/// Conscience subcommands (Phase 1.1)
#[derive(Subcommand)]
enum ConscienceSubcommand {
    /// Show pending actions requiring review
    Review,
    /// Explain a conscience decision
    Explain {
        /// Decision ID to explain
        decision_id: String,
    },
    /// Approve a flagged action
    Approve {
        /// Decision ID to approve
        decision_id: String,
    },
    /// Reject a flagged action
    Reject {
        /// Decision ID to reject
        decision_id: String,
    },
    /// Run manual introspection
    Introspect,
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

    // Phase 0.5c3: Custom error handling for unknown flags (exit 64)
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // Check if it's an unknown argument/flag error
            use clap::error::ErrorKind;
            let exit_code = match err.kind() {
                ErrorKind::UnknownArgument => EXIT_COMMAND_NOT_AVAILABLE,
                ErrorKind::InvalidSubcommand => EXIT_COMMAND_NOT_AVAILABLE,
                _ => 2, // Default clap error code
            };

            // Print the error message
            eprintln!("{}", err);

            // Log the error
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id,
                state: "unknown".to_string(),
                command: "parse_error".to_string(),
                allowed: Some(false),
                args: std::env::args().skip(1).collect(),
                exit_code,
                citation: "[archwiki:General_recommendations]".to_string(),
                duration_ms,
                ok: false,
                error: Some(ErrorDetails {
                    code: "INVALID_ARGUMENT".to_string(),
                    message: err.to_string(),
                }),
            };
            let _ = log_entry.write();

            std::process::exit(exit_code);
        }
    };

    // Phase 0.3c: State-aware dispatch
    // Get command name first
    let command_name = match &cli.command {
        Commands::Status => "status",
        Commands::Help { .. } => "help",
        Commands::Update { .. } => "update",
        Commands::Install { .. } => "install",
        Commands::Rescue { .. } => "rescue",
        Commands::Backup { .. } => "backup",
        Commands::Health { .. } => "health",
        Commands::Doctor { .. } => "doctor",
        Commands::Rollback { .. } => "rollback",
        Commands::Triage => "triage",
        Commands::CollectLogs { .. } => "collect-logs",
        Commands::Repair { .. } => "repair",
        Commands::Audit => "audit",
        Commands::Sentinel { .. } => "sentinel",
        Commands::Config { .. } => "config",
        Commands::Conscience { .. } => "conscience",
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
        return execute_help_command(&cli.command, &state, &capabilities, &req_id, start_time)
            .await;
    }

    // Phase 0.5b: Handle health commands specially (they bypass state checks)
    match &cli.command {
        Commands::Health { json } => {
            return health_commands::execute_health_command(*json, &state, &req_id, start_time)
                .await;
        }
        Commands::Doctor { json } => {
            return health_commands::execute_doctor_command(*json, &state, &req_id, start_time)
                .await;
        }
        Commands::Repair { probe, dry_run } => {
            return health_commands::execute_repair_command(
                probe,
                *dry_run,
                &state,
                &req_id,
                start_time,
            )
            .await;
        }
        Commands::Install { dry_run } => {
            return install_command::execute_install_command(
                *dry_run,
                &req_id,
                &state,
                start_time,
            )
            .await;
        }
        Commands::Rescue { subcommand } => {
            if subcommand.as_deref() == Some("list") {
                return health_commands::execute_rescue_list_command(&req_id, start_time).await;
            }
        }
        // Phase 0.9: Steward commands
        Commands::Status => {
            return steward_commands::execute_status_command(&req_id, &state, start_time).await;
        }
        Commands::Update { dry_run } => {
            return steward_commands::execute_update_command(*dry_run, &req_id, &state, start_time)
                .await;
        }
        Commands::Audit => {
            return steward_commands::execute_audit_command(&req_id, &state, start_time).await;
        }
        // Phase 1.0: Sentinel commands
        Commands::Sentinel { subcommand } => {
            match subcommand {
                SentinelSubcommand::Status => {
                    return sentinel_cli::execute_sentinel_status_command(&req_id, &state, start_time).await;
                }
                SentinelSubcommand::Metrics => {
                    return sentinel_cli::execute_sentinel_metrics_command(&req_id, &state, start_time).await;
                }
            }
        }
        Commands::Config { subcommand } => {
            match subcommand {
                ConfigSubcommand::Get => {
                    return sentinel_cli::execute_config_get_command(&req_id, &state, start_time).await;
                }
                ConfigSubcommand::Set { key, value } => {
                    return sentinel_cli::execute_config_set_command(key, value, &req_id, &state, start_time).await;
                }
            }
        }
        // Phase 1.1: Conscience commands
        Commands::Conscience { subcommand } => {
            match subcommand {
                ConscienceSubcommand::Review => {
                    return conscience_commands::execute_conscience_review_command(&req_id, &state, start_time).await;
                }
                ConscienceSubcommand::Explain { decision_id } => {
                    return conscience_commands::execute_conscience_explain_command(decision_id, &req_id, &state, start_time).await;
                }
                ConscienceSubcommand::Approve { decision_id } => {
                    return conscience_commands::execute_conscience_approve_command(decision_id, &req_id, &state, start_time).await;
                }
                ConscienceSubcommand::Reject { decision_id } => {
                    return conscience_commands::execute_conscience_reject_command(decision_id, &req_id, &state, start_time).await;
                }
                ConscienceSubcommand::Introspect => {
                    return conscience_commands::execute_conscience_introspect_command(&req_id, &state, start_time).await;
                }
            }
        }
        _ => {}
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

        let output =
            CommandOutput::not_available(state.clone(), command_name.to_string(), state_citation);
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
        args: if json_only {
            vec!["--json".to_string()]
        } else {
            vec![]
        },
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
            // Should not reach here - handled in main
            unreachable!("Status command should be handled separately");
        }
        Commands::Help { .. } => {
            // Should not reach here - handled in main
            unreachable!("Help command should be handled separately");
        }
        Commands::Update { .. } => {
            // Should not reach here - handled in main
            unreachable!("Update command should be handled separately");
        }
        Commands::Install { .. } => {
            // Should not reach here - handled separately
            unreachable!("Install command should be handled separately");
        }
        Commands::Audit => {
            // Should not reach here - handled in main
            unreachable!("Audit command should be handled separately");
        }
        Commands::Rescue { subcommand } => {
            println!("[anna] rescue command allowed in state: {}", state);
            println!("[anna] subcommand: {:?} (no-op)", subcommand);
        }
        Commands::Backup { dest } => {
            println!("[anna] backup command allowed in state: {}", state);
            println!(
                "[anna] dest: {:?} (no-op - no actual backup performed)",
                dest
            );
        }
        Commands::Health { .. } => {
            // Should not reach here - handled in main
            unreachable!("Health command should be handled separately");
        }
        Commands::Doctor { .. } => {
            // Should not reach here - handled in main
            unreachable!("Doctor command should be handled separately");
        }
        Commands::Repair { .. } => {
            // Should not reach here - handled in main
            unreachable!("Repair command should be handled separately");
        }
        Commands::Rollback { target } => {
            println!("[anna] rollback command allowed in state: {}", state);
            println!(
                "[anna] target: {:?} (no-op - no actual rollback performed)",
                target
            );
        }
        Commands::Triage => {
            println!("[anna] triage command allowed in state: {}", state);
            println!("[anna] (no-op - no actual triage performed)");
        }
        Commands::CollectLogs { output } => {
            println!("[anna] collect-logs command allowed in state: {}", state);
            println!("[anna] output: {:?} (no-op - no logs collected)", output);
        }
        Commands::Sentinel { .. } => {
            // Should not reach here - handled in main
            unreachable!("Sentinel command should be handled separately");
        }
        Commands::Config { .. } => {
            // Should not reach here - handled in main
            unreachable!("Config command should be handled separately");
        }
        Commands::Conscience { .. } => {
            // Should not reach here - handled in main
            unreachable!("Conscience command should be handled separately");
        }
    }

    Ok(EXIT_SUCCESS)
}
