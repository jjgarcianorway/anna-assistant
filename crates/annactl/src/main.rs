//! Anna Control - CLI client for Anna Assistant
//!
//! Provides user interface to interact with the Anna daemon.
//!
//! Phase 0.3: State-aware command dispatch with no-ops
//! Citation: [archwiki:system_maintenance]

// Phase 0.3a: Commands module will be reimplemented in 0.3c
// mod commands;
pub mod errors;
mod adaptive_help; // Phase 3.8: Adaptive root help
mod chronos_commands; // Phase 1.5
mod collective_commands; // Phase 1.3
mod consensus_commands; // Phase 1.8
mod conscience_commands; // Phase 1.1
mod context_detection; // Phase 3.8: Context detection
mod empathy_commands; // Phase 1.2
mod health_commands;
mod help_commands; // Phase 3.1: Adaptive help
mod init_command; // Phase 3.9: First-run wizard
mod install_command; // Phase 0.8
mod learning_commands; // Phase 3.9: Learn and predict commands
pub mod logging;
mod mirror_commands; // Phase 1.4
mod monitor_setup; // Phase 3.1: Monitoring automation
pub mod output;
mod predictive_hints; // Phase 3.8: Post-command predictive intelligence
mod rpc_client; // Phase 0.5b
mod sentinel_cli; // Phase 1.0
mod steward_commands; // Phase 0.9
mod upgrade_command; // Phase 3.10: Auto-upgrade system

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
    /// Path to daemon socket (overrides $ANNAD_SOCKET and defaults)
    #[arg(long, global = true)]
    socket: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

/// Phase 0.3: Simplified command set - all commands check state availability
#[derive(Subcommand)]
enum Commands {
    /// Initialize Anna (first-run wizard) - creates /etc/anna and config files
    Init,

    /// Show system status and daemon health
    Status,

    /// Show available commands (adaptive and context-aware)
    Help {
        /// Show help for specific command
        command: Option<String>,

        /// Show all commands (including internal)
        #[arg(long)]
        all: bool,

        /// Output JSON only (legacy)
        #[arg(long)]
        json: bool,
    },

    /// Ping daemon (1-RTT health check)
    Ping,

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

    /// Empathy kernel (Phase 1.2)
    Empathy {
        /// Subcommand: pulse, simulate
        #[command(subcommand)]
        subcommand: EmpathySubcommand,
    },

    /// Collective mind (Phase 1.3)
    Collective {
        /// Subcommand: status, trust, explain
        #[command(subcommand)]
        subcommand: CollectiveSubcommand,
    },

    /// Mirror protocol (Phase 1.4)
    Mirror {
        /// Subcommand: reflect, audit, repair
        #[command(subcommand)]
        subcommand: MirrorSubcommand,
    },

    /// Chronos loop (Phase 1.5)
    Chronos {
        /// Subcommand: forecast, audit, align
        #[command(subcommand)]
        subcommand: ChronosSubcommand,
    },

    /// Distributed consensus (Phase 1.7 - STUB)
    Consensus {
        /// Subcommand: status, submit, reconcile
        #[command(subcommand)]
        subcommand: ConsensusSubcommand,
    },

    /// Self-update annactl binary (Phase 2.0)
    SelfUpdate {
        /// Check for updates without installing
        #[arg(long)]
        check: bool,

        /// Show available versions
        #[arg(long)]
        list: bool,
    },

    /// Show system profile and adaptive intelligence (Phase 3.0)
    Profile {
        /// Output JSON only
        #[arg(long)]
        json: bool,
    },

    /// Install monitoring stack (Grafana/Prometheus) based on system profile (Phase 3.0)
    Monitor {
        /// Subcommand: install, status
        #[command(subcommand)]
        subcommand: MonitorSubcommand,
    },

    /// Display system metrics in Prometheus format (Phase 3.3)
    Metrics {
        /// Output in Prometheus exposition format
        #[arg(long)]
        prometheus: bool,

        /// Output JSON only
        #[arg(long)]
        json: bool,
    },

    /// Analyze action history and detect patterns (Phase 3.7)
    Learn {
        /// Output JSON only
        #[arg(long)]
        json: bool,

        /// Minimum confidence level (low, medium, high, very-high)
        #[arg(long, default_value = "medium")]
        min_confidence: String,

        /// Analysis window in days
        #[arg(long, default_value = "30")]
        days: i64,
    },

    /// Show predictive intelligence and recommendations (Phase 3.7)
    Predict {
        /// Output JSON only
        #[arg(long)]
        json: bool,

        /// Show all predictions (default: only high/critical)
        #[arg(long)]
        all: bool,
    },

    /// Upgrade Anna to the latest version (Phase 3.10)
    Upgrade {
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,

        /// Check for updates without installing
        #[arg(long)]
        check: bool,
    },
}

/// Monitor subcommands (Phase 3.0)
#[derive(Subcommand)]
enum MonitorSubcommand {
    /// Install monitoring stack (adaptive based on system resources)
    Install {
        /// Force monitoring mode (full/light/minimal) - overrides auto-detection
        #[arg(long)]
        force_mode: Option<String>,

        /// Dry run (show what would be installed)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Show monitoring stack status
    Status,
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

/// Empathy subcommands (Phase 1.2)
#[derive(Subcommand)]
enum EmpathySubcommand {
    /// Show current empathy pulse
    Pulse,
    /// Simulate empathy evaluation for an action
    Simulate {
        /// Action to simulate (e.g., "SystemUpdate", "RestartService")
        action: String,
    },
}

/// Collective subcommands (Phase 1.3)
#[derive(Subcommand)]
enum CollectiveSubcommand {
    /// Show network status
    Status,
    /// Show trust details for a peer
    Trust {
        /// Peer ID to query
        peer_id: String,
    },
    /// Explain a consensus decision
    Explain {
        /// Consensus ID to explain
        consensus_id: String,
    },
}

/// Mirror subcommands (Phase 1.4 + 1.6)
#[derive(Subcommand)]
enum MirrorSubcommand {
    /// Generate manual reflection cycle
    Reflect,
    /// Summarize last peer critiques
    Audit,
    /// Trigger remediation protocol
    Repair,
    /// Audit forecast accuracy (Phase 1.6)
    AuditForecast {
        /// Window hours for audit
        #[arg(default_value = "24")]
        window: u64,
        /// Output JSON format
        #[arg(long)]
        json: bool,
    },
    /// Generate temporal self-reflection (Phase 1.6)
    ReflectTemporal {
        /// Window hours for reflection
        #[arg(default_value = "24")]
        window: u64,
        /// Output JSON format
        #[arg(long)]
        json: bool,
    },
}

/// Chronos subcommands (Phase 1.5)
#[derive(Subcommand)]
enum ChronosSubcommand {
    /// Generate temporal forecast
    Forecast {
        /// Forecast window in hours
        #[arg(default_value = "24")]
        window: u64,
    },
    /// View archived forecasts
    Audit,
    /// Align forecast parameters across network
    Align,
}

/// Consensus subcommands (Phase 1.7 - STUB)
#[derive(Subcommand)]
enum ConsensusSubcommand {
    /// Show consensus status
    Status {
        /// Round ID to query (optional, defaults to latest)
        #[arg(short, long)]
        round_id: Option<String>,
        /// Output JSON format
        #[arg(long)]
        json: bool,
    },
    /// Submit observation to consensus
    Submit {
        /// Path to observation JSON file
        observation_path: String,
    },
    /// Reconcile consensus for window
    Reconcile {
        /// Window hours for reconciliation
        #[arg(default_value = "24")]
        window: u64,
        /// Output JSON format
        #[arg(long)]
        json: bool,
    },
    /// Initialize Ed25519 keys for consensus
    InitKeys,
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
    // Phase 3.8: Intercept root help before clap parsing for adaptive display
    let args: Vec<String> = std::env::args().collect();

    // Check for root help invocation
    if args.len() == 1 || (args.len() == 2 && (args[1] == "--help" || args[1] == "-h")) {
        adaptive_help::display_adaptive_root_help(false, false);
        std::process::exit(0);
    }

    // Check for --help --all --json (all three)
    if args.len() >= 4
        && args.contains(&"--help".to_string())
        && args.contains(&"--all".to_string())
        && args.contains(&"--json".to_string())
    {
        adaptive_help::display_adaptive_root_help(true, true);
        std::process::exit(0);
    }

    // Check for --help --all
    if args.len() >= 3 && args.contains(&"--help".to_string()) && args.contains(&"--all".to_string()) {
        adaptive_help::display_adaptive_root_help(true, false);
        std::process::exit(0);
    }

    // Check for --json help
    if args.len() >= 3 && args.contains(&"--help".to_string()) && args.contains(&"--json".to_string()) {
        adaptive_help::display_adaptive_root_help(false, true);
        std::process::exit(0);
    }

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

    // Phase 1.8: Handle consensus commands early (standalone PoC, no daemon)
    // Handle self-update command (doesn't need daemon)
    if let Commands::SelfUpdate { check, list } = &cli.command {
        return execute_self_update_command(*check, *list).await;
    }

    // Handle profile command (Phase 3.0 - needs daemon)
    if let Commands::Profile { json } = &cli.command {
        return execute_profile_command(*json, cli.socket.as_deref()).await;
    }

    // Handle monitor command (Phase 3.0 - needs daemon)
    if let Commands::Monitor { subcommand } = &cli.command {
        match subcommand {
            MonitorSubcommand::Install { force_mode, dry_run } => {
                return execute_monitor_install_command(force_mode.clone(), *dry_run, cli.socket.as_deref()).await;
            }
            MonitorSubcommand::Status => {
                return execute_monitor_status_command(cli.socket.as_deref()).await;
            }
        }
    }

    // Handle metrics command (Phase 3.3 - needs daemon)
    if let Commands::Metrics { prometheus, json } = &cli.command {
        return execute_metrics_command(*prometheus, *json, cli.socket.as_deref()).await;
    }

    if let Commands::Consensus { subcommand } = &cli.command {
        match subcommand {
            ConsensusSubcommand::Status { round_id, json } => {
                return consensus_commands::execute_consensus_status_command(round_id.clone(), *json).await;
            }
            ConsensusSubcommand::Submit { observation_path } => {
                return consensus_commands::execute_consensus_submit_command(observation_path).await;
            }
            ConsensusSubcommand::Reconcile { window, json } => {
                return consensus_commands::execute_consensus_reconcile_command(*window, *json).await;
            }
            ConsensusSubcommand::InitKeys => {
                return consensus_commands::execute_consensus_init_keys_command().await;
            }
        }
    }

    // Phase 3.1: Handle help command early (doesn't need daemon)
    if matches!(cli.command, Commands::Help { .. }) {
        let socket_path = cli.socket.as_deref();
        return execute_help_command_standalone(&cli.command, socket_path, &req_id, start_time).await;
    }

    // Phase 3.9: Handle init command early (doesn't need daemon)
    if matches!(cli.command, Commands::Init) {
        return init_command::execute_init_command().await;
    }

    // Phase 3.9: Handle learning commands early (don't need daemon, use context DB directly)
    if let Commands::Learn { json, min_confidence, days } = &cli.command {
        return learning_commands::execute_learn_command(*json, min_confidence, *days).await;
    }

    if let Commands::Predict { json, all } = &cli.command {
        return learning_commands::execute_predict_command(*json, *all).await;
    }

    // Phase 3.10: Handle upgrade command early (doesn't need daemon, requires root)
    if let Commands::Upgrade { yes, check } = &cli.command {
        return upgrade_command::execute_upgrade_command(*yes, *check).await;
    }

    // Phase 0.3c: State-aware dispatch
    // Get command name first
    let command_name = match &cli.command {
        Commands::Init => "init",
        Commands::Status => "status",
        Commands::Help { .. } => "help",
        Commands::Ping => "ping",
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
        Commands::Empathy { .. } => "empathy",
        Commands::Collective { .. } => "collective",
        Commands::Mirror { .. } => "mirror",
        Commands::Chronos { .. } => "chronos",
        Commands::Consensus { .. } => "consensus",
        Commands::SelfUpdate { .. } => "self-update",
        Commands::Profile { .. } => "profile",
        Commands::Monitor { .. } => "monitor",
        Commands::Metrics { .. } => "metrics",
        Commands::Learn { .. } => "learn",
        Commands::Predict { .. } => "predict",
        Commands::Upgrade { .. } => "upgrade",
    };

    // Try to connect to daemon and get state
    let socket_path = cli.socket.as_deref();
    let (state, state_citation, capabilities) = match get_state_and_capabilities(socket_path).await {
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

    // v1.16.3: Handle ping command (simple 1-RTT check)
    if matches!(cli.command, Commands::Ping) {
        return execute_ping_command(socket_path, &req_id, start_time).await;
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
            // Phase 3.4: Check resource constraints before heavy operation
            if !*dry_run {
                if !check_resource_constraints(socket_path, "system installation").await? {
                    std::process::exit(EXIT_SUCCESS);
                }
            }
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
            // Phase 3.4: Check resource constraints before heavy operation
            if !*dry_run {
                if !check_resource_constraints(socket_path, "system update").await? {
                    std::process::exit(EXIT_SUCCESS);
                }
            }
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
        // Phase 1.2: Empathy commands
        Commands::Empathy { subcommand } => {
            match subcommand {
                EmpathySubcommand::Pulse => {
                    return empathy_commands::execute_empathy_pulse_command().await.map_err(|e| e.into());
                }
                EmpathySubcommand::Simulate { action } => {
                    return empathy_commands::execute_empathy_simulate_command(action).await.map_err(|e| e.into());
                }
            }
        }
        // Phase 1.3: Collective mind commands
        Commands::Collective { subcommand } => {
            match subcommand {
                CollectiveSubcommand::Status => {
                    return collective_commands::execute_collective_status_command().await.map_err(|e| e.into());
                }
                CollectiveSubcommand::Trust { peer_id } => {
                    return collective_commands::execute_collective_trust_command(peer_id).await.map_err(|e| e.into());
                }
                CollectiveSubcommand::Explain { consensus_id } => {
                    return collective_commands::execute_collective_explain_command(consensus_id).await.map_err(|e| e.into());
                }
            }
        }
        // Phase 1.4: Mirror protocol commands
        Commands::Mirror { subcommand } => {
            match subcommand {
                MirrorSubcommand::Reflect => {
                    return mirror_commands::execute_mirror_reflect_command().await.map_err(|e| e.into());
                }
                MirrorSubcommand::Audit => {
                    return mirror_commands::execute_mirror_audit_command().await.map_err(|e| e.into());
                }
                MirrorSubcommand::Repair => {
                    return mirror_commands::execute_mirror_repair_command().await.map_err(|e| e.into());
                }
                MirrorSubcommand::AuditForecast { window, json } => {
                    return mirror_commands::execute_mirror_audit_forecast_command(*window, *json).await.map_err(|e| e.into());
                }
                MirrorSubcommand::ReflectTemporal { window, json } => {
                    return mirror_commands::execute_mirror_reflect_temporal_command(*window, *json).await.map_err(|e| e.into());
                }
            }
        }
        // Phase 1.5: Chronos loop commands
        Commands::Chronos { subcommand } => {
            match subcommand {
                ChronosSubcommand::Forecast { window } => {
                    return chronos_commands::execute_chronos_forecast_command(*window).await.map_err(|e| e.into());
                }
                ChronosSubcommand::Audit => {
                    return chronos_commands::execute_chronos_audit_command().await.map_err(|e| e.into());
                }
                ChronosSubcommand::Align => {
                    return chronos_commands::execute_chronos_align_command().await.map_err(|e| e.into());
                }
            }
        }
        // Phase 1.8: Consensus commands (PoC)
        Commands::Consensus { subcommand } => {
            match subcommand {
                ConsensusSubcommand::Status { round_id, json } => {
                    consensus_commands::execute_consensus_status_command(round_id.clone(), *json).await?;
                    return Ok(());
                }
                ConsensusSubcommand::Submit { observation_path } => {
                    consensus_commands::execute_consensus_submit_command(observation_path).await?;
                    return Ok(());
                }
                ConsensusSubcommand::Reconcile { window, json } => {
                    consensus_commands::execute_consensus_reconcile_command(*window, *json).await?;
                    return Ok(());
                }
                ConsensusSubcommand::InitKeys => {
                    consensus_commands::execute_consensus_init_keys_command().await?;
                    return Ok(());
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

/// Execute ping command (v1.16.3)
async fn execute_ping_command(
    socket_path: Option<&str>,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let ping_start = Instant::now();

    match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(mut client) => {
            match client.ping().await {
                Ok(_) => {
                    let rtt_ms = ping_start.elapsed().as_millis();
                    println!("Pong! RTT: {}ms", rtt_ms);

                    // Log the ping command
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    let log_entry = LogEntry {
                        ts: LogEntry::now(),
                        req_id: req_id.to_string(),
                        state: "unknown".to_string(),
                        command: "ping".to_string(),
                        allowed: Some(true),
                        args: vec![],
                        exit_code: EXIT_SUCCESS,
                        citation: "[archwiki:system_maintenance]".to_string(),
                        duration_ms,
                        ok: true,
                        error: None,
                    };
                    let _ = log_entry.write();

                    std::process::exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    eprintln!("Ping failed: {}", e);

                    // Log the failed ping
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    let log_entry = LogEntry {
                        ts: LogEntry::now(),
                        req_id: req_id.to_string(),
                        state: "unknown".to_string(),
                        command: "ping".to_string(),
                        allowed: Some(true),
                        args: vec![],
                        exit_code: EXIT_DAEMON_UNAVAILABLE,
                        citation: "[archwiki:system_maintenance]".to_string(),
                        duration_ms,
                        ok: false,
                        error: Some(ErrorDetails {
                            code: "PING_FAILED".to_string(),
                            message: e.to_string(),
                        }),
                    };
                    let _ = log_entry.write();

                    std::process::exit(EXIT_DAEMON_UNAVAILABLE);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);

            // Log the connection failure
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id: req_id.to_string(),
                state: "unknown".to_string(),
                command: "ping".to_string(),
                allowed: None,
                args: vec![],
                exit_code: EXIT_DAEMON_UNAVAILABLE,
                citation: "[archwiki:system_maintenance]".to_string(),
                duration_ms,
                ok: false,
                error: Some(ErrorDetails {
                    code: "DAEMON_UNAVAILABLE".to_string(),
                    message: e.to_string(),
                }),
            };
            let _ = log_entry.write();

            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    }
}

/// Detect if Anna was installed via package manager (AUR, etc.)
fn detect_package_install() -> Option<String> {
    // Check if annactl is owned by pacman
    if let Ok(output) = std::process::Command::new("pacman")
        .args(&["-Qo", "/usr/bin/annactl"])
        .output()
    {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // Output format: "/usr/bin/annactl is owned by package-name version"
                if let Some(package) = stdout.split_whitespace().nth(4) {
                    return Some(package.to_string());
                }
            }
        }
    }

    // Check if in /usr/local (homebrew or manual install)
    if let Ok(exe_path) = std::env::current_exe() {
        if exe_path.starts_with("/usr/local") {
            return Some("homebrew/manual".to_string());
        }
    }

    None
}

/// Execute self-update command (Phase 2.0 + Phase 3.8 AUR awareness)
async fn execute_self_update_command(check: bool, list: bool) -> Result<()> {
    const REPO_OWNER: &str = "jjgarcianorway";
    const REPO_NAME: &str = "anna-assistant";
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

    // Phase 3.8: Check if installed via package manager
    if let Some(package) = detect_package_install() {
        println!("⚠️  Anna was installed via package manager: {}", package);
        println!();
        println!("Please use your package manager to update:");
        if package.contains("anna-assistant") {
            println!("  pacman -Syu              # System update (includes Anna)");
            println!("  yay -Sua                 # AUR update only");
        } else {
            println!("  Check your package manager documentation");
        }
        println!();
        println!("The self-update command is for manual/binary installations only.");
        std::process::exit(EXIT_SUCCESS);
    }

    if list {
        // List available versions from GitHub releases
        println!("Fetching available versions...");
        match reqwest::get(format!(
            "https://api.github.com/repos/{}/{}/releases",
            REPO_OWNER, REPO_NAME
        ))
        .await
        {
            Ok(response) => {
                if let Ok(releases) = response.json::<serde_json::Value>().await {
                    if let Some(releases_array) = releases.as_array() {
                        println!("\nAvailable versions:");
                        for release in releases_array.iter().take(10) {
                            if let Some(tag) = release["tag_name"].as_str() {
                                let prerelease = release["prerelease"].as_bool().unwrap_or(false);
                                let label = if prerelease { "(pre-release)" } else { "" };
                                println!("  {} {}", tag, label);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch releases: {}", e);
                std::process::exit(EXIT_GENERAL_ERROR);
            }
        }
        std::process::exit(EXIT_SUCCESS);
    }

    if check {
        // Check for updates without installing
        println!("Current version: {}", CURRENT_VERSION);
        println!("Checking for updates...");

        match reqwest::get(format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            REPO_OWNER, REPO_NAME
        ))
        .await
        {
            Ok(response) => {
                if let Ok(release) = response.json::<serde_json::Value>().await {
                    if let Some(latest_tag) = release["tag_name"].as_str() {
                        // Strip 'v' prefix if present
                        let latest_version = latest_tag.trim_start_matches('v');

                        if latest_version != CURRENT_VERSION {
                            println!("\n✨ Update available: {} → {}", CURRENT_VERSION, latest_version);
                            println!("\nTo update:");
                            println!("  1. Download: https://github.com/{}/{}/releases/tag/{}",
                                REPO_OWNER, REPO_NAME, latest_tag);
                            println!("  2. Or use package manager:");
                            println!("     - Arch: yay -S anna-assistant-bin");
                            println!("     - Homebrew: brew upgrade anna-assistant");
                        } else {
                            println!("✓ You are running the latest version ({})", CURRENT_VERSION);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to check for updates: {}", e);
                std::process::exit(EXIT_GENERAL_ERROR);
            }
        }

        std::process::exit(EXIT_SUCCESS);
    }

    // No flags - show usage
    eprintln!("Usage: annactl self-update --check | --list");
    std::process::exit(EXIT_GENERAL_ERROR);
}

/// Execute profile command (Phase 3.0)
async fn execute_profile_command(json: bool, socket_path: Option<&str>) -> Result<()> {
    use anna_common::ipc::ProfileData;

    // Connect to daemon
    let mut client = match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            eprintln!("Make sure annad is running (try: sudo systemctl start annad)");
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // Get system profile
    let profile_response = client.get_profile().await?;
    let profile = match profile_response {
        ResponseData::Profile(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetProfile"),
    };

    if json {
        // JSON output
        let json_output = serde_json::to_string_pretty(&profile)?;
        println!("{}", json_output);
    } else {
        // Human-readable output
        println!("System Profile (Phase 3.0: Adaptive Intelligence)");
        println!("==================================================\n");

        println!("Resources:");
        println!("  Memory:  {} MB total, {} MB available", profile.total_memory_mb, profile.available_memory_mb);
        println!("  CPU:     {} cores", profile.cpu_cores);
        println!("  Disk:    {} GB total, {} GB available", profile.total_disk_gb, profile.available_disk_gb);
        println!("  Uptime:  {} seconds ({:.1} hours)", profile.uptime_seconds, profile.uptime_seconds as f64 / 3600.0);
        println!();

        println!("Environment:");
        println!("  Virtualization: {}", profile.virtualization);
        println!("  Session Type:   {}", profile.session_type);
        if profile.gpu_present {
            println!("  GPU:            {} ({})",
                profile.gpu_vendor.as_deref().unwrap_or("Unknown"),
                profile.gpu_model.as_deref().unwrap_or("Unknown model")
            );
        } else {
            println!("  GPU:            Not detected");
        }
        println!();

        println!("Adaptive Intelligence:");
        println!("  Monitoring Mode: {}", profile.recommended_monitoring_mode.to_uppercase());
        println!("  Rationale:       {}", profile.monitoring_rationale);
        println!("  Constrained:     {}", if profile.is_constrained { "Yes" } else { "No" });
        println!();

        // Phase 3.0: SSH tunnel suggestions (Remote Access Policy)
        if profile.session_type.starts_with("ssh:") {
            println!("Remote Access:");
            let has_display = profile.session_type.contains("forwarding=true");
            if has_display {
                println!("  SSH X11 forwarding detected - GUI tools available");
            } else {
                println!("  SSH session detected (no X11 forwarding)");
            }

            // Show tunnel instructions for Full mode
            if profile.recommended_monitoring_mode == "full" {
                println!();
                println!("  💡 To access Grafana dashboards, create an SSH tunnel:");
                println!("     ssh -L 3000:localhost:3000 user@host");
                println!("     Then browse to: http://localhost:3000");
            } else if profile.recommended_monitoring_mode == "light" {
                println!();
                println!("  💡 Prometheus metrics available at: http://localhost:9090");
                println!("     Create SSH tunnel: ssh -L 9090:localhost:9090 user@host");
            }
            println!();
        }

        println!("Timestamp: {}", profile.timestamp);
    }

    std::process::exit(EXIT_SUCCESS);
}

/// Execute monitor install command (Phase 3.0)
async fn execute_monitor_install_command(force_mode: Option<String>, dry_run: bool, socket_path: Option<&str>) -> Result<()> {
    // Connect to daemon to get system profile
    let mut client = match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            eprintln!("Make sure annad is running (try: sudo systemctl start annad)");
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // Get system profile
    let profile_response = client.get_profile().await?;
    let profile = match profile_response {
        ResponseData::Profile(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetProfile"),
    };

    // Phase 3.2: Adaptive UI hint - warn if minimal mode
    if profile.recommended_monitoring_mode == "minimal" && force_mode.is_none() {
        println!("⚠️  Adaptive Intelligence Warning");
        println!("═══════════════════════════════════════════════════════════════");
        println!();
        println!("  Your system is running in MINIMAL mode due to limited resources.");
        println!("  Installing external monitoring tools (Prometheus/Grafana) is");
        println!("  NOT recommended as it may impact system performance.");
        println!();
        println!("  System Constraints:");
        println!("    • RAM: {} MB (recommend >2GB for light mode)", profile.total_memory_mb);
        println!("    • CPU: {} cores", profile.cpu_cores);
        println!("    • Disk: {} GB available", profile.available_disk_gb);
        println!();
        println!("  Anna's internal monitoring is active and sufficient for your system.");
        println!("  Use 'annactl health' and 'annactl status' for system insights.");
        println!();
        println!("  To override this warning: annactl monitor install --force-mode <mode>");
        println!("═══════════════════════════════════════════════════════════════");
        println!();

        // Ask for confirmation
        eprint!("Continue anyway? [y/N]: ");
        use std::io::Write;
        std::io::stderr().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Installation cancelled.");
            return Ok(());
        }
        println!();
    }

    // Determine monitoring mode
    let mode = if let Some(forced) = force_mode {
        let normalized = forced.to_lowercase();
        if !["full", "light", "minimal"].contains(&normalized.as_str()) {
            eprintln!("Error: Invalid mode '{}'. Must be: full, light, or minimal", forced);
            std::process::exit(EXIT_GENERAL_ERROR);
        }
        println!("⚠️  Using FORCED mode: {}", normalized.to_uppercase());
        println!("   (System recommendation: {})\n", profile.recommended_monitoring_mode.to_uppercase());
        normalized
    } else {
        profile.recommended_monitoring_mode.clone()
    };

    println!("Monitoring Stack Installation (Phase 3.0: Adaptive Intelligence)");
    println!("====================================================================\n");

    println!("System Profile:");
    println!("  Memory: {} MB  |  CPU: {} cores  |  Constrained: {}",
        profile.total_memory_mb, profile.cpu_cores,
        if profile.is_constrained { "Yes" } else { "No" });
    println!("  Recommended Mode: {}", profile.recommended_monitoring_mode.to_uppercase());
    println!();

    // Execute installation based on mode
    match mode.as_str() {
        "full" => monitor_setup::install_full_mode(dry_run)?,
        "light" => monitor_setup::install_light_mode(dry_run)?,
        "minimal" => monitor_setup::install_minimal_mode()?,
        _ => unreachable!("Mode validation should prevent this"),
    }

    println!();
    println!("Citation: [archwiki:prometheus][archwiki:grafana][observability:best-practices]");

    std::process::exit(EXIT_SUCCESS);
}

/// Execute monitor status command (Phase 3.0)
async fn execute_monitor_status_command(socket_path: Option<&str>) -> Result<()> {
    // Connect to daemon to get system profile
    let mut client = match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            eprintln!("Make sure annad is running (try: sudo systemctl start annad)");
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // Get system profile
    let profile_response = client.get_profile().await?;
    let profile = match profile_response {
        ResponseData::Profile(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetProfile"),
    };

    println!("Monitoring Stack Status (Phase 3.0)");
    println!("====================================\n");

    println!("System Mode: {}", profile.recommended_monitoring_mode.to_uppercase());
    println!("Rationale:   {}\n", profile.monitoring_rationale);

    // Check Prometheus status
    let prometheus_active = monitor_setup::check_service_status("prometheus")
        .unwrap_or(false);

    println!("Prometheus:");
    if prometheus_active {
        println!("  Status: ✓ Running");
        println!("  Access: http://localhost:9090");
    } else {
        println!("  Status: ✗ Not running");
        println!("  Install: annactl monitor install");
    }
    println!();

    // Check Grafana status (only for Full mode)
    if profile.recommended_monitoring_mode == "full" {
        let grafana_active = monitor_setup::check_service_status("grafana")
            .unwrap_or(false);

        println!("Grafana:");
        if grafana_active {
            println!("  Status: ✓ Running");
            println!("  Access: http://localhost:3000");
        } else {
            println!("  Status: ✗ Not running");
            println!("  Install: annactl monitor install");
        }
        println!();
    }

    println!("Internal Stats: ✓ Available (via daemon)");
    println!("  Commands: annactl status, annactl health");
    println!();

    // Phase 3.2: Adaptive UI hints - mode-specific guidance
    match profile.recommended_monitoring_mode.as_str() {
        "minimal" => {
            println!("💡 Adaptive Intelligence Hint:");
            println!("   Your system is in MINIMAL mode. External monitoring tools are");
            println!("   not recommended due to limited resources. Anna's internal stats");
            println!("   provide all essential system health information.");
            println!();
            println!("   Recommended commands:");
            println!("   • annactl status     - View system state and recommendations");
            println!("   • annactl health     - Check system health metrics");
        }
        "light" => {
            println!("💡 Adaptive Intelligence Hint:");
            println!("   Your system is in LIGHT mode. Prometheus metrics are available");
            println!("   for advanced monitoring, but Grafana dashboards are not recommended");
            println!("   due to RAM or session constraints.");
            println!();
            println!("   Prometheus metrics: http://localhost:9090");
            println!("   Internal stats: annactl status, annactl health");
        }
        "full" => {
            println!("💡 Adaptive Intelligence Hint:");
            println!("   Your system is in FULL mode. All monitoring features are available");
            println!("   including Grafana dashboards for visualization.");
            println!();
            println!("   Grafana: http://localhost:3000");
            println!("   Prometheus: http://localhost:9090");
            println!("   Internal stats: annactl status, annactl health");
        }
        _ => {}
    }

    std::process::exit(EXIT_SUCCESS);
}

/// Execute metrics command (Phase 3.3)
async fn execute_metrics_command(prometheus: bool, json: bool, socket_path: Option<&str>) -> Result<()> {
    // Connect to daemon to get system profile
    let mut client = match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            eprintln!("Make sure annad is running (try: sudo systemctl start annad)");
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // Get system profile
    let profile_response = client.get_profile().await?;
    let profile = match profile_response {
        ResponseData::Profile(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetProfile"),
    };

    if json {
        // JSON output
        let json_output = serde_json::json!({
            "total_memory_mb": profile.total_memory_mb,
            "available_memory_mb": profile.available_memory_mb,
            "cpu_cores": profile.cpu_cores,
            "total_disk_gb": profile.total_disk_gb,
            "available_disk_gb": profile.available_disk_gb,
            "uptime_seconds": profile.uptime_seconds,
            "monitoring_mode": profile.recommended_monitoring_mode,
            "is_constrained": profile.is_constrained,
            "timestamp": profile.timestamp
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else if prometheus {
        // Prometheus exposition format
        println!("# HELP anna_system_memory_total_mb Total system memory in MB");
        println!("# TYPE anna_system_memory_total_mb gauge");
        println!("anna_system_memory_total_mb {}", profile.total_memory_mb);
        println!();

        println!("# HELP anna_system_memory_available_mb Available system memory in MB");
        println!("# TYPE anna_system_memory_available_mb gauge");
        println!("anna_system_memory_available_mb {}", profile.available_memory_mb);
        println!();

        println!("# HELP anna_system_cpu_cores Number of CPU cores");
        println!("# TYPE anna_system_cpu_cores gauge");
        println!("anna_system_cpu_cores {}", profile.cpu_cores);
        println!();

        println!("# HELP anna_system_disk_total_gb Total disk space in GB");
        println!("# TYPE anna_system_disk_total_gb gauge");
        println!("anna_system_disk_total_gb {}", profile.total_disk_gb);
        println!();

        println!("# HELP anna_system_disk_available_gb Available disk space in GB");
        println!("# TYPE anna_system_disk_available_gb gauge");
        println!("anna_system_disk_available_gb {}", profile.available_disk_gb);
        println!();

        println!("# HELP anna_system_uptime_seconds System uptime in seconds");
        println!("# TYPE anna_system_uptime_seconds gauge");
        println!("anna_system_uptime_seconds {}", profile.uptime_seconds);
        println!();

        let mode_value = match profile.recommended_monitoring_mode.as_str() {
            "minimal" => 0,
            "light" => 1,
            "full" => 2,
            _ => 1, // default to light
        };
        println!("# HELP anna_profile_mode Monitoring mode (0=minimal, 1=light, 2=full)");
        println!("# TYPE anna_profile_mode gauge");
        println!("anna_profile_mode {}", mode_value);
        println!();

        let constrained_value = if profile.is_constrained { 1 } else { 0 };
        println!("# HELP anna_profile_constrained Resource-constrained status (0=no, 1=yes)");
        println!("# TYPE anna_profile_constrained gauge");
        println!("anna_profile_constrained {}", constrained_value);
    } else {
        // Human-readable output
        println!("System Metrics (Phase 3.3: Adaptive Intelligence)");
        println!("==================================================\n");

        println!("Memory:");
        println!("  Total:     {} MB", profile.total_memory_mb);
        println!("  Available: {} MB ({:.1}%)",
            profile.available_memory_mb,
            (profile.available_memory_mb as f64 / profile.total_memory_mb as f64) * 100.0
        );
        println!();

        println!("CPU:");
        println!("  Cores: {}", profile.cpu_cores);
        println!();

        println!("Disk:");
        println!("  Total:     {} GB", profile.total_disk_gb);
        println!("  Available: {} GB ({:.1}%)",
            profile.available_disk_gb,
            (profile.available_disk_gb as f64 / profile.total_disk_gb as f64) * 100.0
        );
        println!();

        println!("System:");
        println!("  Uptime: {} seconds ({:.1} hours)",
            profile.uptime_seconds,
            profile.uptime_seconds as f64 / 3600.0
        );
        println!();

        println!("Adaptive Intelligence:");
        println!("  Mode:        {}", profile.recommended_monitoring_mode.to_uppercase());
        println!("  Constrained: {}", if profile.is_constrained { "Yes" } else { "No" });
        println!("  Rationale:   {}", profile.monitoring_rationale);
        println!();

        println!("Timestamp: {}", profile.timestamp);
        println!();

        println!("💡 Tip: Use --prometheus for Prometheus format, --json for JSON");
    }

    std::process::exit(EXIT_SUCCESS);
}

/// Check system resources and warn if constrained (Phase 3.4)
async fn check_resource_constraints(socket_path: Option<&str>, operation: &str) -> Result<bool> {
    // Try to get profile from daemon
    let mut client = match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(c) => c,
        Err(_) => return Ok(true), // If daemon unavailable, proceed anyway
    };

    let profile_response = client.get_profile().await?;
    let profile = match profile_response {
        ResponseData::Profile(data) => data,
        _ => return Ok(true), // If unexpected response, proceed anyway
    };

    // Check if system is resource-constrained
    if profile.is_constrained {
        println!("⚠️  Resource Constraint Warning");
        println!("═══════════════════════════════════════════════════════════════");
        println!();
        println!("  Your system is resource-constrained:");
        println!("    • RAM: {} MB available of {} MB total ({:.1}%)",
            profile.available_memory_mb,
            profile.total_memory_mb,
            (profile.available_memory_mb as f64 / profile.total_memory_mb as f64) * 100.0
        );
        println!("    • CPU: {} cores", profile.cpu_cores);
        println!("    • Disk: {} GB available", profile.available_disk_gb);
        println!();
        println!("  Operation '{}' may:", operation);
        println!("    - Consume significant system resources");
        println!("    - Take longer than usual to complete");
        println!("    - Impact system responsiveness");
        println!();
        println!("  Consider:");
        println!("    - Closing other applications");
        println!("    - Running during off-peak hours");
        println!("    - Using --dry-run flag to preview changes");
        println!("═══════════════════════════════════════════════════════════════");
        println!();

        // Ask for confirmation
        eprint!("Proceed with operation? [y/N]: ");
        use std::io::Write;
        std::io::stderr().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            return Ok(false);
        }
        println!();
    }

    Ok(true)
}

/// Get state and capabilities from daemon (Phase 0.3d)
async fn get_state_and_capabilities(socket_path: Option<&str>) -> Result<(String, String, Vec<CommandCapabilityData>)> {
    let mut client = rpc_client::RpcClient::connect_with_path(socket_path).await?;

    // Get state
    let state_response = client.get_state().await?;
    let state_data = match state_response {
        ResponseData::StateDetection(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetState"),
    };

    // Get capabilities
    let caps_response = client.get_capabilities().await?;
    let caps_data = match caps_response {
        ResponseData::Capabilities(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetCapabilities"),
    };

    // Return state name, state citation, and full capabilities list
    // Phase 3.0: Extract commands from CapabilitiesData
    let citation = state_citation(&state_data.state);
    Ok((state_data.state, citation.to_string(), caps_data.commands))
}

/// Execute help command standalone (Phase 3.1 - doesn't require daemon)
async fn execute_help_command_standalone(
    command: &Commands,
    socket_path: Option<&str>,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let (cmd_name, show_all, _json) = match command {
        Commands::Help { command, all, json } => (command.clone(), *all, *json),
        _ => unreachable!(),
    };

    // Build display context from current system state
    let context = help_commands::build_context(socket_path).await;

    // Display adaptive help
    if let Err(e) = help_commands::display_help(cmd_name, show_all, context).await {
        eprintln!("Error displaying help: {}", e);
        std::process::exit(1);
    }

    // Log the help command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: "unknown".to_string(),
        command: "help".to_string(),
        allowed: Some(true),
        args: if show_all {
            vec!["--all".to_string()]
        } else {
            vec![]
        },
        exit_code: EXIT_SUCCESS,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: true,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(EXIT_SUCCESS);
}

/// Execute help command (Phase 0.3d - Legacy, requires daemon)
async fn execute_help_command(
    command: &Commands,
    state: &str,
    capabilities: &[CommandCapabilityData],
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let (cmd_name, show_all, json_only) = match command {
        Commands::Help { command, all, json } => (command.clone(), *all, *json),
        _ => unreachable!(),
    };

    // Phase 3.1: Use adaptive help system if not JSON mode
    if !json_only {
        // Build display context from current system state
        let socket_path = std::env::var("ANNAD_SOCKET").ok();
        let context = help_commands::build_context(socket_path.as_deref()).await;

        // Display adaptive help
        if let Err(e) = help_commands::display_help(cmd_name, show_all, context).await {
            eprintln!("Error displaying help: {}", e);
            std::process::exit(1);
        }

        // Log the help command
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let log_entry = LogEntry {
            ts: LogEntry::now(),
            req_id: req_id.to_string(),
            state: state.to_string(),
            command: "help".to_string(),
            allowed: Some(true),
            args: if show_all {
                vec!["--all".to_string()]
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

    // Legacy JSON output (for backwards compatibility)
    let mut sorted_caps = capabilities.to_vec();
    sorted_caps.sort_by(|a, b| a.name.cmp(&b.name));

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

    // Log the help command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "help".to_string(),
        allowed: Some(true),
        args: vec!["--json".to_string()],
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
        Commands::Init => {
            // Should not reach here - handled in main
            unreachable!("Init command should be handled separately");
        }
        Commands::Status => {
            // Should not reach here - handled in main
            unreachable!("Status command should be handled separately");
        }
        Commands::Help { .. } => {
            // Should not reach here - handled in main
            unreachable!("Help command should be handled separately");
        }
        Commands::Ping => {
            // Should not reach here - handled in main
            unreachable!("Ping command should be handled separately");
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
        Commands::Empathy { .. } => {
            // Should not reach here - handled in main
            unreachable!("Empathy command should be handled separately");
        }
        Commands::Collective { .. } => {
            // Should not reach here - handled in main
            unreachable!("Collective command should be handled separately");
        }
        Commands::Mirror { .. } => {
            // Should not reach here - handled in main
            unreachable!("Mirror command should be handled separately");
        }
        Commands::Chronos { .. } => {
            // Should not reach here - handled in main
            unreachable!("Chronos command should be handled separately");
        }
        Commands::Consensus { .. } => {
            // Should not reach here - handled in main
            unreachable!("Consensus command should be handled separately");
        }
        Commands::SelfUpdate { .. } => {
            // Should not reach here - handled in main
            unreachable!("SelfUpdate command should be handled separately");
        }
        Commands::Profile { .. } => {
            // Should not reach here - handled in main
            unreachable!("Profile command should be handled separately");
        }
        Commands::Monitor { .. } => {
            // Should not reach here - handled in main
            unreachable!("Monitor command should be handled separately");
        }
        Commands::Metrics { .. } => {
            // Should not reach here - handled in main
            unreachable!("Metrics command should be handled separately");
        }
        Commands::Learn { .. } => {
            // Should not reach here - handled in main
            unreachable!("Learn command should be handled separately");
        }
        Commands::Predict { .. } => {
            // Should not reach here - handled in main
            unreachable!("Predict command should be handled separately");
        }
        Commands::Upgrade { .. } => {
            // Should not reach here - handled in main
            unreachable!("Upgrade command should be handled separately");
        }
    }

    Ok(EXIT_SUCCESS)
}
