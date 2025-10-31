use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use std::collections::HashMap;
use anna_common::{anna_narrative, anna_info};

mod doctor;
use doctor::{doctor_check, doctor_repair, doctor_rollback, doctor_setup, doctor_validate};

mod autonomy;
use autonomy::{autonomy_get, autonomy_set};

mod config_cmd;
mod persona_cmd;
mod profile;
mod profile_cmd;
mod explain_cmd;

const SOCKET_PATH: &str = "/run/anna/annad.sock";

#[derive(Parser)]
#[command(name = "annactl")]
#[command(version, about = "Anna Assistant Control CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if daemon is responsive
    Ping,

    /// System diagnostics and self-healing
    Doctor {
        #[command(subcommand)]
        action: DoctorAction,
    },

    /// Show daemon status
    Status,

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Autonomy management
    Autonomy {
        #[command(subcommand)]
        action: AutonomyAction,
    },

    /// State persistence management
    State {
        #[command(subcommand)]
        action: StateAction,
    },

    /// Telemetry management
    Telemetry {
        #[command(subcommand)]
        action: TelemetryAction,
    },

    /// Policy management
    Policy {
        #[command(subcommand)]
        action: PolicyAction,
    },

    /// Events management
    Events {
        #[command(subcommand)]
        action: EventAction,
    },

    /// Learning cache management
    Learning {
        #[command(subcommand)]
        action: LearningAction,
    },

    /// Show release highlights and what's new
    News {
        /// Show news for a specific version
        #[arg(long)]
        version: Option<String>,

        /// List all available versions
        #[arg(long)]
        list: bool,
    },

    /// Interactive guide to Anna's capabilities
    Explore,

    /// Persona management (change Anna's communication style)
    Persona {
        #[command(subcommand)]
        action: PersonaAction,
    },

    /// System profiling and health checks
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Ask Anna to do something (natural language)
    Ask {
        /// What you want Anna to do
        intent: String,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Get a configuration value (shows value and origin)
    Get {
        /// Configuration key (e.g., "ui.emojis")
        key: String,
    },

    /// Set a configuration value in user preferences
    Set {
        /// Configuration key
        key: String,

        /// New value (will be parsed as JSON, or treated as string)
        value: String,
    },

    /// Reset configuration to defaults (all or specific key)
    Reset {
        /// Specific key to reset (if omitted, resets all)
        key: Option<String>,
    },

    /// Export effective configuration to file or stdout
    Export {
        /// Output file path (if omitted, prints to stdout)
        #[arg(long)]
        path: Option<String>,
    },

    /// Import configuration from file
    Import {
        /// Input file path
        #[arg(long)]
        path: String,

        /// Replace existing config instead of merging
        #[arg(long)]
        replace: bool,
    },

    /// List all configuration values
    List,
}

#[derive(Subcommand)]
enum PersonaAction {
    /// Show current persona
    Get,

    /// Set persona
    Set {
        /// Persona name (dev, ops, gamer, minimal)
        name: String,

        /// Set as auto-detect (may change based on context)
        #[arg(long, conflicts_with = "fixed")]
        auto: bool,

        /// Set as fixed (won't change automatically)
        #[arg(long, conflicts_with = "auto")]
        fixed: bool,
    },

    /// Explain why current persona was chosen
    Why,

    /// List all available personas
    List,
}

#[derive(Subcommand)]
enum ProfileAction {
    /// Show system profile with hardware, graphics, audio, network info
    Show,

    /// Run system health checks
    Checks {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Filter by status (pass, warn, error, info)
        #[arg(long)]
        status: Option<String>,
    },
}

#[derive(Subcommand)]
enum DoctorAction {
    /// Run read-only system health check
    Check {
        /// Show verbose diagnostic information
        #[arg(long)]
        verbose: bool,
    },

    /// Run comprehensive system validation with detailed diagnostics
    Validate,

    /// Interactive system setup and optimization wizard
    Setup,

    /// Run self-healing repairs
    Repair {
        /// Show what would be fixed without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Roll back to a previous backup
    Rollback {
        /// Backup timestamp (YYYYMMDD-HHMMSS) or "list" to show available backups
        timestamp: String,

        /// Only verify integrity without restoring
        #[arg(long)]
        verify: bool,
    },
}

#[derive(Subcommand)]
enum AutonomyAction {
    /// Get current autonomy level
    Get,

    /// Set autonomy level (requires confirmation)
    Set {
        /// Autonomy level: low or high
        #[arg(value_parser = ["low", "high"])]
        level: String,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum StateAction {
    /// Save component state
    Save {
        /// Component name
        component: String,

        /// JSON data to save
        data: String,
    },

    /// Load component state
    Load {
        /// Component name
        component: String,
    },

    /// List all saved states
    List,
}

#[derive(Subcommand)]
enum TelemetryAction {
    /// Show current system telemetry snapshot
    Snapshot,

    /// Show telemetry history
    History {
        /// Number of samples to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: u32,

        /// Show samples since timestamp (ISO 8601 format)
        #[arg(long)]
        since: Option<String>,
    },

    /// Show telemetry trends for a metric
    Trends {
        /// Metric to analyze: cpu, mem, or disk
        #[arg(value_parser = ["cpu", "mem", "memory", "disk"])]
        metric: String,

        /// Time window in hours (default: 24)
        #[arg(short, long, default_value = "24")]
        hours: u32,
    },
}

#[derive(Subcommand)]
enum PolicyAction {
    /// List all loaded policies
    List,

    /// Reload policies from disk
    Reload,

    /// Evaluate policies against current state
    Eval {
        /// JSON context for evaluation (optional)
        #[arg(short, long)]
        context: Option<String>,
    },
}

#[derive(Subcommand)]
enum EventAction {
    /// Show recent events
    Show {
        /// Filter by event type
        #[arg(long)]
        event_type: Option<String>,

        /// Filter by minimum severity (info, warning, error, critical)
        #[arg(long)]
        severity: Option<String>,

        /// Maximum number of events to show
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// List all events
    List {
        /// Filter string
        #[arg(long)]
        filter: Option<String>,

        /// Maximum number of events to show
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// Clear event history
    Clear,
}

#[derive(Subcommand)]
enum LearningAction {
    /// Show learning statistics
    Stats {
        /// Specific action name (optional)
        action: Option<String>,
    },

    /// Get action recommendations
    Recommendations,

    /// Reset learning cache
    Reset {
        /// Confirm reset
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ConfigScope {
    User,
    System,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    Ping,
    Doctor,
    Status,
    ConfigGet {
        key: String,
    },
    ConfigSet {
        scope: ConfigScope,
        key: String,
        value: String,
    },
    ConfigList,
    AutonomyStatus,
    AutonomyRun {
        task: String,
    },
    StateSave {
        component: String,
        data: serde_json::Value,
    },
    StateLoad {
        component: String,
    },
    StateList,
    DoctorAutoFix,
    // Sprint 3
    PolicyEvaluate {
        context: serde_json::Value,
    },
    PolicyReload,
    PolicyList,
    EventsList {
        filter: Option<String>,
        limit: Option<usize>,
    },
    EventsShow {
        event_type: Option<String>,
        severity: Option<String>,
    },
    EventsClear,
    LearningStats {
        action: Option<String>,
    },
    LearningRecommendations,
    LearningReset,
    // Sprint 5: Telemetry
    TelemetrySnapshot,
    TelemetryHistory {
        since: Option<String>,
        limit: Option<u32>,
    },
    TelemetryTrends {
        metric: String,
        hours: u32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Response {
    Success { data: serde_json::Value },
    Error { message: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Ping => {
            let response = send_request(Request::Ping).await?;
            println!("✓ {}", response["message"].as_str().unwrap_or("OK"));
            Ok::<(), anyhow::Error>(())
        }
        Commands::Doctor { action } => match action {
            DoctorAction::Check { verbose } => {
                doctor_check(verbose).await?;
                Ok(())
            }
            DoctorAction::Validate => {
                doctor_validate().await?;
                Ok(())
            }
            DoctorAction::Setup => {
                doctor_setup().await?;
                Ok(())
            }
            DoctorAction::Repair { dry_run } => {
                doctor_repair(dry_run).await?;
                Ok(())
            }
            DoctorAction::Rollback { timestamp, verify } => {
                doctor_rollback(&timestamp, verify).await?;
                Ok(())
            }
        },
        Commands::Status => {
            let response = send_request(Request::Status).await?;
            print_status(&response)?;
            Ok(())
        }
        Commands::Config { action } => match action {
            ConfigAction::Get { key } => config_cmd::config_get(&key).await,
            ConfigAction::Set { key, value } => config_cmd::config_set(&key, &value).await,
            ConfigAction::Reset { key } => config_cmd::config_reset(key.as_deref()).await,
            ConfigAction::Export { path } => config_cmd::config_export(path.as_deref()).await,
            ConfigAction::Import { path, replace } => config_cmd::config_import(&path, replace).await,
            ConfigAction::List => config_cmd::config_list().await,
        },
        Commands::Autonomy { action } => match action {
            AutonomyAction::Get => {
                autonomy_get().await?;
                Ok(())
            }
            AutonomyAction::Set { level, yes } => {
                autonomy_set(&level, yes).await?;
                Ok(())
            }
        },
        Commands::State { action } => match action {
            StateAction::Save { component, data } => {
                let json_data: serde_json::Value = serde_json::from_str(&data)
                    .context("Invalid JSON data")?;
                let _response = send_request(Request::StateSave {
                    component: component.clone(),
                    data: json_data,
                })
                .await?;
                println!("✓ Saved state for component: {}", component);
                Ok(())
            }
            StateAction::Load { component } => {
                let response = send_request(Request::StateLoad { component: component.clone() }).await?;
                if response["found"].as_bool().unwrap_or(true) {
                    println!("{}", serde_json::to_string_pretty(&response)?);
                } else {
                    println!("No state found for component: {}", component);
                }
                Ok(())
            }
            StateAction::List => {
                let response = send_request(Request::StateList).await?;
                print_state_list(&response)?;
                Ok(())
            }
        },
        Commands::Telemetry { action } => match action {
            TelemetryAction::Snapshot => {
                let response = send_request(Request::TelemetrySnapshot).await?;
                print_telemetry_snapshot(&response)?;
                Ok(())
            }
            TelemetryAction::History { limit, since } => {
                let response = send_request(Request::TelemetryHistory {
                    since: since.clone(),
                    limit: Some(limit),
                }).await?;
                print_telemetry_history(&response)?;
                Ok(())
            }
            TelemetryAction::Trends { metric, hours } => {
                let response = send_request(Request::TelemetryTrends {
                    metric: metric.clone(),
                    hours,
                }).await?;
                print_telemetry_trends(&response)?;
                Ok(())
            }
        },
        Commands::Policy { action } => match action {
            PolicyAction::List => {
                let response = send_request(Request::PolicyList).await?;
                print_policy_list(&response)?;
                Ok(())
            }
            PolicyAction::Reload => {
                let response = send_request(Request::PolicyReload).await?;
                println!("✓ Policies reloaded: {} rules", response["loaded"].as_u64().unwrap_or(0));
                Ok(())
            }
            PolicyAction::Eval { context } => {
                let ctx = if let Some(json_str) = context {
                    serde_json::from_str(&json_str).context("Invalid JSON context")?
                } else {
                    serde_json::json!({})
                };
                let response = send_request(Request::PolicyEvaluate { context: ctx }).await?;
                print_policy_eval(&response)?;
                Ok(())
            }
        },
        Commands::Events { action } => match action {
            EventAction::Show { event_type, severity, limit } => {
                let response = send_request(Request::EventsShow {
                    event_type: event_type.clone(),
                    severity: severity.clone(),
                }).await?;
                print_events(&response, limit)?;
                Ok(())
            }
            EventAction::List { filter, limit } => {
                let response = send_request(Request::EventsList {
                    filter: filter.clone(),
                    limit: Some(limit),
                }).await?;
                print_events(&response, limit)?;
                Ok(())
            }
            EventAction::Clear => {
                let _response = send_request(Request::EventsClear).await?;
                println!("✓ Event history cleared");
                Ok(())
            }
        },
        Commands::Learning { action } => match action {
            LearningAction::Stats { action: action_name } => {
                let response = send_request(Request::LearningStats {
                    action: action_name.clone(),
                }).await?;
                print_learning_stats(&response)?;
                Ok(())
            }
            LearningAction::Recommendations => {
                let response = send_request(Request::LearningRecommendations).await?;
                print_learning_recommendations(&response)?;
                Ok(())
            }
            LearningAction::Reset { confirm } => {
                if !confirm {
                    eprintln!("Error: Use --confirm to reset learning cache");
                    std::process::exit(1);
                }
                let _response = send_request(Request::LearningReset).await?;
                println!("✓ Learning cache reset");
                Ok(())
            }
        },
        Commands::News { version, list } => {
            print_news(version.as_deref(), list)?;
            Ok(())
        },
        Commands::Explore => {
            print_explore_guide()?;
            Ok(())
        },
        Commands::Persona { action } => match action {
            PersonaAction::Get => persona_cmd::persona_get().await,
            PersonaAction::Set { name, auto, fixed } => {
                let mode = if fixed {
                    anna_common::PersonaMode::Fixed
                } else {
                    anna_common::PersonaMode::Auto
                };
                persona_cmd::persona_set(&name, mode).await
            },
            PersonaAction::Why => persona_cmd::persona_why().await,
            PersonaAction::List => persona_cmd::persona_list().await,
        },
        Commands::Profile { action } => match action {
            ProfileAction::Show => profile_cmd::profile_show().await,
            ProfileAction::Checks { json, status } =>
                profile_cmd::profile_checks(json, status.as_deref()).await,
        },
        Commands::Ask { intent } => {
            anna_narrative("I'm learning to understand natural language requests!");
            anna_info("For now, try specific commands like:");
            println!("  • annactl profile show");
            println!("  • annactl profile checks");
            println!("  • annactl config set ui.emojis on");
            println!("  • annactl persona set dev");
            Ok(())
        },
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn send_request(request: Request) -> Result<serde_json::Value> {
    let stream = match UnixStream::connect(SOCKET_PATH).await {
        Ok(s) => s,
        Err(e) => {
            // Provide helpful error message with troubleshooting steps
            eprintln!("❌ annad not running or socket unavailable");
            eprintln!();
            eprintln!("Socket path: {}", SOCKET_PATH);
            eprintln!("Error: {}", e);
            eprintln!();
            eprintln!("Troubleshooting:");
            eprintln!("  1. Check if daemon is running:");
            eprintln!("       sudo systemctl status annad");
            eprintln!();
            eprintln!("  2. View recent logs:");
            eprintln!("       sudo journalctl -u annad --since -5m | tail -n 50");
            eprintln!();
            eprintln!("  3. Check socket permissions:");
            eprintln!("       ls -lh {}", SOCKET_PATH);
            eprintln!();
            eprintln!("  4. Verify group membership:");
            eprintln!("       groups | grep anna");
            eprintln!("       (If not in 'anna' group, run: newgrp anna)");
            eprintln!();
            eprintln!("  5. Start the daemon:");
            eprintln!("       sudo systemctl start annad");
            eprintln!();
            std::process::exit(1);
        }
    };

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send request
    let json = serde_json::to_string(&request)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    // Read response
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let response: Response = serde_json::from_str(&line)?;

    match response {
        Response::Success { data } => Ok(data),
        Response::Error { message } => anyhow::bail!("Daemon error: {}", message),
    }
}

fn print_diagnostics(data: &serde_json::Value) -> Result<()> {
    let results: DiagnosticResults = serde_json::from_value(data.clone())?;

    println!("\n🔍 Anna System Diagnostics\n");
    println!("{}", "=".repeat(70));

    for check in &results.checks {
        let icon = match check.status {
            Status::Pass => "✓",
            Status::Warn => "⚠",
            Status::Fail => "✗",
        };

        println!("{} {:30} {}", icon, check.name, check.message);

        if let Some(fix_hint) = &check.fix_hint {
            println!("  → Fix: {}", fix_hint);
        }
    }

    println!("{}", "=".repeat(70));
    println!(
        "\nOverall Status: {}",
        match results.overall_status {
            Status::Pass => "✓ PASS",
            Status::Warn => "⚠ WARNING",
            Status::Fail => "✗ FAIL",
        }
    );
    println!();

    // Exit non-zero if any check failed
    if results.overall_status == Status::Fail {
        std::process::exit(1);
    }

    Ok(())
}

fn print_autofix_results(data: &serde_json::Value) -> Result<()> {
    let results: Vec<AutoFixResult> = serde_json::from_value(data.clone())?;

    println!("\n🔧 Auto-Fix Results\n");
    println!("{}", "=".repeat(70));

    for result in &results {
        let icon = if result.success {
            "✓"
        } else if result.attempted {
            "✗"
        } else {
            "○"
        };

        println!("{} {} - {}", icon, result.check_name, result.message);
    }

    println!("{}", "=".repeat(70));
    println!();

    Ok(())
}

fn print_status(data: &serde_json::Value) -> Result<()> {
    use std::fs;

    println!("\n📊 Anna Daemon Status\n");
    println!("Version:       {}", data["version"].as_str().unwrap_or("unknown"));
    println!("Status:        {}", data["uptime"].as_str().unwrap_or("unknown"));

    // Read autonomy level from config file
    let autonomy_level = fs::read_to_string("/etc/anna/autonomy.conf")
        .ok()
        .and_then(|content| {
            content.lines()
                .find(|line| line.starts_with("autonomy_level="))
                .map(|line| line.trim_start_matches("autonomy_level=").trim().to_string())
        })
        .unwrap_or_else(|| "low".to_string());

    println!("Autonomy:      {}", autonomy_level);
    println!();

    // Show recent logs from journald
    println!("📋 Recent Logs (last 15 entries):\n");
    match std::process::Command::new("journalctl")
        .args(&["-u", "annad", "-n", "15", "--no-pager", "-o", "short-precise"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let logs = String::from_utf8_lossy(&output.stdout);
            for line in logs.lines() {
                println!("  {}", line);
            }
        }
        Ok(_) => {
            println!("  (Unable to read logs - may require elevated permissions)");
        }
        Err(_) => {
            println!("  (journalctl not available)");
        }
    }
    println!();

    Ok(())
}

fn print_config_list(data: &serde_json::Value) -> Result<()> {
    let config_map: HashMap<String, String> = serde_json::from_value(data.clone())?;

    println!("\n⚙️  Configuration\n");

    let mut keys: Vec<_> = config_map.keys().collect();
    keys.sort();

    for key in keys {
        if let Some(value) = config_map.get(key) {
            println!("{:35} = {}", key, value);
        }
    }

    println!();
    Ok(())
}

fn print_autonomy_status(data: &serde_json::Value) -> Result<()> {
    println!("\n🤖 Autonomy Status\n");
    println!("Level:                  {}", data["level"].as_str().unwrap_or("unknown"));
    println!("Automatic tasks:        {}", if data["automatic_tasks_enabled"].as_bool().unwrap_or(false) { "enabled" } else { "disabled" });
    println!("Recommendations:        {}", if data["recommendations_enabled"].as_bool().unwrap_or(false) { "enabled" } else { "disabled" });

    if let Some(tasks) = data["tasks_available"].as_array() {
        println!("\nAvailable tasks:");
        for task in tasks {
            if let Some(name) = task.as_str() {
                println!("  - {}", name);
            }
        }
    }

    println!();
    Ok(())
}

fn print_task_result(data: &serde_json::Value) -> Result<()> {
    println!("\n✓ Task completed: {}", data["task"].as_str().unwrap_or("unknown"));
    println!("Success: {}", data["success"].as_bool().unwrap_or(false));
    println!("Message: {}", data["message"].as_str().unwrap_or(""));

    if let Some(actions) = data["actions_taken"].as_array() {
        if !actions.is_empty() {
            println!("\nActions taken:");
            for action in actions {
                if let Some(desc) = action.as_str() {
                    println!("  - {}", desc);
                }
            }
        }
    }

    println!();
    Ok(())
}

fn print_state_list(data: &serde_json::Value) -> Result<()> {
    let components: Vec<String> = serde_json::from_value(data.clone())?;

    println!("\n💾 Saved States\n");

    if components.is_empty() {
        println!("No saved states found.");
    } else {
        for component in components {
            println!("  - {}", component);
        }
    }

    println!();
    Ok(())
}

fn print_telemetry_snapshot(response: &serde_json::Value) -> Result<()> {
    let data = &response["data"];

    println!("\n📊 System Telemetry Snapshot\n");
    println!("  Timestamp:    {}", data["timestamp"].as_str().unwrap_or("N/A"));
    println!("  CPU Usage:    {:.1}%", data["cpu_usage"].as_f64().unwrap_or(0.0));
    println!("  Memory Usage: {:.1}%", data["mem_usage"].as_f64().unwrap_or(0.0));
    println!("  Disk Free:    {:.1}%", data["disk_free"].as_f64().unwrap_or(0.0));
    println!("  Uptime:       {} seconds", data["uptime_sec"].as_u64().unwrap_or(0));
    println!("  Network In:   {} KB", data["net_in_kb"].as_u64().unwrap_or(0));
    println!("  Network Out:  {} KB", data["net_out_kb"].as_u64().unwrap_or(0));
    println!();

    Ok(())
}

fn print_telemetry_history(response: &serde_json::Value) -> Result<()> {
    let data = &response["data"];
    let samples = data["samples"].as_array().context("Invalid samples array")?;
    let count = data["count"].as_u64().unwrap_or(0);

    println!("\n📈 Telemetry History ({} samples)\n", count);
    println!("{:<25} {:>8} {:>8} {:>8} {:>10}",
             "Timestamp", "CPU%", "MEM%", "DISK%", "Uptime(s)");
    println!("{}", "-".repeat(70));

    for sample in samples {
        let timestamp = sample["timestamp"].as_str().unwrap_or("N/A");
        let cpu = sample["cpu_usage"].as_f64().unwrap_or(0.0);
        let mem = sample["mem_usage"].as_f64().unwrap_or(0.0);
        let disk = sample["disk_free"].as_f64().unwrap_or(0.0);
        let uptime = sample["uptime_sec"].as_u64().unwrap_or(0);

        // Shorten timestamp for display (show last 19 chars: YYYY-MM-DD HH:MM:SS)
        let display_ts = if timestamp.len() > 19 {
            &timestamp[..19]
        } else {
            timestamp
        };

        println!("{:<25} {:>7.1}% {:>7.1}% {:>7.1}% {:>10}",
                 display_ts, cpu, mem, disk, uptime);
    }

    println!();
    Ok(())
}

fn print_telemetry_trends(response: &serde_json::Value) -> Result<()> {
    let data = &response["data"];

    println!("\n📉 Telemetry Trends Analysis\n");
    println!("  Metric:   {}", data["metric"].as_str().unwrap_or("N/A"));
    println!("  Period:   {} hours", data["hours"].as_u64().unwrap_or(0));
    println!("  Samples:  {}", data["samples"].as_u64().unwrap_or(0));
    println!();
    println!("  Average:  {:.1}%", data["avg"].as_f64().unwrap_or(0.0));
    println!("  Minimum:  {:.1}%", data["min"].as_f64().unwrap_or(0.0));
    println!("  Maximum:  {:.1}%", data["max"].as_f64().unwrap_or(0.0));
    println!();

    Ok(())
}

#[derive(Debug, Deserialize)]
struct DiagnosticResults {
    checks: Vec<DiagnosticCheck>,
    overall_status: Status,
}

#[derive(Debug, Deserialize)]
struct DiagnosticCheck {
    name: String,
    status: Status,
    message: String,
    fix_hint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AutoFixResult {
    check_name: String,
    attempted: bool,
    success: bool,
    message: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pass,
    Warn,
    Fail,
}

// Sprint 3 print functions
fn print_policy_list(data: &serde_json::Value) -> Result<()> {
    println!("\n📋 Policy Rules\n");

    if let Some(message) = data["message"].as_str() {
        println!("{}", message);
    } else if let Some(rules) = data["rules"].as_array() {
        if rules.is_empty() {
            println!("No policies loaded.");
        } else {
            for (idx, rule) in rules.iter().enumerate() {
                println!("{}. {}", idx + 1, rule["condition"].as_str().unwrap_or("unknown"));
                println!("   → Action: {}", rule["action"].as_str().unwrap_or("unknown"));
                println!("   Enabled: {}", rule["enabled"].as_bool().unwrap_or(false));
                println!();
            }
        }
    }

    println!();
    Ok(())
}

fn print_policy_eval(data: &serde_json::Value) -> Result<()> {
    println!("\n🔍 Policy Evaluation\n");

    if let Some(message) = data["message"].as_str() {
        println!("{}", message);
    } else {
        let matched = data["matched"].as_bool().unwrap_or(false);
        println!("Matched: {}", if matched { "yes" } else { "no" });

        if let Some(actions) = data["actions"].as_array() {
            if !actions.is_empty() {
                println!("\nActions to execute:");
                for action in actions {
                    println!("  - {:?}", action);
                }
            }
        }
    }

    println!();
    Ok(())
}

fn print_events(data: &serde_json::Value, _limit: usize) -> Result<()> {
    println!("\n📡 Events\n");

    if let Some(message) = data["message"].as_str() {
        println!("{}", message);
    } else if let Some(events) = data["events"].as_array() {
        if events.is_empty() {
            println!("No events found.");
        } else {
            for event in events {
                let id = event["id"].as_str().unwrap_or("unknown");
                let event_type = event["event_type"].as_str().unwrap_or("unknown");
                let severity = event["severity"].as_str().unwrap_or("info");
                let source = event["source"].as_str().unwrap_or("unknown");
                let message = event["message"].as_str().unwrap_or("");

                println!("[{}] {} - {} ({})", severity.to_uppercase(), event_type, message, source);
                println!("  ID: {}", id);
                println!();
            }
        }
    }

    println!();
    Ok(())
}

fn print_learning_stats(data: &serde_json::Value) -> Result<()> {
    println!("\n🧠 Learning Statistics\n");

    if let Some(global) = data.get("global") {
        println!("Global Statistics:");
        println!("  Total actions:     {}", global["total_actions"].as_u64().unwrap_or(0));
        println!("  Total outcomes:    {}", global["total_outcomes"].as_u64().unwrap_or(0));
        println!("  Success rate:      {:.1}%", global["success_rate"].as_f64().unwrap_or(0.0) * 100.0);
        println!();
    }

    if let Some(stats) = data["stats"].as_array() {
        if !stats.is_empty() {
            println!("Action Statistics:");
            for stat in stats {
                println!("\n  {}", stat["action_name"].as_str().unwrap_or("unknown"));
                println!("    Executions:      {}", stat["total_executions"].as_u64().unwrap_or(0));
                println!("    Success:         {}", stat["success_count"].as_u64().unwrap_or(0));
                println!("    Failure:         {}", stat["failure_count"].as_u64().unwrap_or(0));
                println!("    Success rate:    {:.1}%", stat["success_rate"].as_f64().unwrap_or(0.0) * 100.0);
                println!("    Avg duration:    {:.1}ms", stat["avg_duration_ms"].as_f64().unwrap_or(0.0));
            }
        }
    }

    println!();
    Ok(())
}

fn print_learning_recommendations(data: &serde_json::Value) -> Result<()> {
    println!("\n💡 Recommended Actions\n");

    if let Some(recommendations) = data["recommendations"].as_array() {
        if recommendations.is_empty() {
            println!("No recommendations available yet.");
        } else {
            println!("Actions ranked by success probability:\n");
            for (idx, rec) in recommendations.iter().enumerate() {
                if let Some(arr) = rec.as_array() {
                    if arr.len() >= 2 {
                        let name = arr[0].as_str().unwrap_or("unknown");
                        let score = arr[1].as_f64().unwrap_or(0.0);
                        println!("{}. {} (score: {:.3})", idx + 1, name, score);
                    }
                }
            }
        }
    }

    println!();
    Ok(())
}

fn print_news(version: Option<&str>, list_all: bool) -> Result<()> {
    use std::fs;
    use std::path::Path;

    let news_dir = Path::new("/usr/local/share/anna/news");

    // Fallback to local news directory if system directory doesn't exist
    let news_dir = if news_dir.exists() {
        news_dir
    } else {
        Path::new("news")
    };

    if !news_dir.exists() {
        eprintln!("Error: News directory not found");
        eprintln!("Expected: {:?} or ./news/", news_dir);
        return Ok(());
    }

    // List mode
    if list_all {
        println!("\n📰 Available Release Notes\n");

        let mut versions: Vec<String> = Vec::new();
        if let Ok(entries) = fs::read_dir(news_dir) {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".txt") {
                        let version_name = filename.trim_end_matches(".txt");
                        versions.push(version_name.to_string());
                    }
                }
            }
        }

        versions.sort();
        versions.reverse(); // Newest first

        for v in &versions {
            println!("  • {}", v);
        }

        println!("\nTo read: annactl news --version <version>");
        println!("Example: annactl news --version v0.9.4-beta\n");
        return Ok(());
    }

    // Determine which version to show
    let target_version = if let Some(v) = version {
        v.to_string()
    } else {
        // Try to get current version from /etc/anna/version
        if let Ok(installed) = fs::read_to_string("/etc/anna/version") {
            format!("v{}", installed.trim())
        } else {
            // Default to latest available
            let mut versions: Vec<String> = Vec::new();
            if let Ok(entries) = fs::read_dir(news_dir) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".txt") {
                            let version_name = filename.trim_end_matches(".txt");
                            versions.push(version_name.to_string());
                        }
                    }
                }
            }
            versions.sort();
            versions.reverse();
            versions.first().cloned().unwrap_or_else(|| "v0.9.4-beta".to_string())
        }
    };

    // Read and display the news file
    let news_file = news_dir.join(format!("{}.txt", target_version));

    if !news_file.exists() {
        eprintln!("Error: No news found for version {}", target_version);
        eprintln!("Try: annactl news --list");
        return Ok(());
    }

    let content = fs::read_to_string(&news_file)?;

    // Print with nice formatting
    println!("\n╭────────────────────────────────────────────────╮");
    for line in content.lines() {
        if line.is_empty() {
            println!("│                                                │");
        } else {
            println!("│  {:<44} │", line);
        }
    }
    println!("╰────────────────────────────────────────────────╯\n");

    Ok(())
}

fn print_explore_guide() -> Result<()> {
    println!("\n╭────────────────────────────────────────────────╮");
    println!("│                                                │");
    println!("│  🧭 Exploring Anna's Capabilities              │");
    println!("│                                                │");
    println!("╰────────────────────────────────────────────────╯\n");

    println!("┌─ Getting Started");
    println!("│");
    println!("│  annactl status");
    println!("│    → View daemon status and system health");
    println!("│");
    println!("│  annactl doctor check");
    println!("│    → Run health diagnostics");
    println!("│");
    println!("│  annactl telemetry snapshot");
    println!("│    → View current system metrics");
    println!("│");
    println!("└─ Basic commands for monitoring and verification\n");

    println!("┌─ Diagnostics & Repair");
    println!("│");
    println!("│  annactl doctor repair");
    println!("│    → Fix common issues automatically");
    println!("│");
    println!("│  annactl doctor check --verbose");
    println!("│    → Detailed health report");
    println!("│");
    println!("│  annactl telemetry trends cpu --hours 24");
    println!("│    → Analyze CPU usage over time");
    println!("│");
    println!("└─ Self-healing and system analysis\n");

    println!("┌─ Configuration");
    println!("│");
    println!("│  annactl config show");
    println!("│    → View all configuration settings");
    println!("│");
    println!("│  annactl autonomy get");
    println!("│    → Check current autonomy level");
    println!("│");
    println!("│  annactl policy list");
    println!("│    → View loaded policies");
    println!("│");
    println!("└─ System configuration and policy management\n");

    println!("┌─ Make Anna Yours (NEW in v0.9.6)");
    println!("│");
    println!("│  annactl profile show");
    println!("│    → See your system profile (hardware, graphics, audio)");
    println!("│");
    println!("│  annactl profile checks");
    println!("│    → Run health checks with remediation hints");
    println!("│");
    println!("│  annactl persona list");
    println!("│    → See available personas (dev, ops, gamer, minimal)");
    println!("│");
    println!("│  annactl persona set dev");
    println!("│    → Switch to dev persona (verbose, emojis)");
    println!("│");
    println!("│  annactl config list");
    println!("│    → See all customizable settings");
    println!("│");
    println!("│  annactl config set ui.emojis off");
    println!("│    → Turn off emojis");
    println!("│");
    println!("└─ Customize Anna's behavior and UI\n");

    println!("┌─ Telemetry & History");
    println!("│");
    println!("│  annactl telemetry history --limit 10");
    println!("│    → View historical metrics");
    println!("│");
    println!("│  annactl telemetry trends mem --hours 12");
    println!("│    → Memory usage trends");
    println!("│");
    println!("│  annactl events show --severity warning");
    println!("│    → View system events");
    println!("│");
    println!("└─ Data analysis and event tracking\n");

    println!("┌─ Learn More");
    println!("│");
    println!("│  annactl news");
    println!("│    → What's new in this version");
    println!("│");
    println!("│  annactl news --list");
    println!("│    → All available release notes");
    println!("│");
    println!("│  annactl --help");
    println!("│    → Complete command reference");
    println!("│");
    println!("└─ Documentation and help\n");

    println!("💡 Tip: Most commands support --help for detailed usage");
    println!("💡 Tip: Try 'annactl telemetry snapshot' after 60 seconds\n");

    Ok(())
}
