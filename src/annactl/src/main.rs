use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use std::collections::HashMap;

mod doctor;
use doctor::{doctor_check, doctor_repair, doctor_rollback};

mod autonomy;
use autonomy::{autonomy_get, autonomy_set};

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
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., "autonomy.level")
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Scope: user or system
        #[arg(value_enum)]
        scope: ConfigScope,

        /// Configuration key
        key: String,

        /// New value
        value: String,
    },

    /// List all configuration values
    List,
}

#[derive(Subcommand)]
enum DoctorAction {
    /// Run read-only system health check
    Check {
        /// Show verbose diagnostic information
        #[arg(long)]
        verbose: bool,
    },

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
    /// List recent telemetry events
    List {
        /// Maximum number of events to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Show statistics summary
    Stats,
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
            DoctorAction::Repair { dry_run } => {
                doctor_repair(dry_run).await?;
                Ok(())
            }
            DoctorAction::Rollback { timestamp } => {
                doctor_rollback(&timestamp).await?;
                Ok(())
            }
        },
        Commands::Status => {
            let response = send_request(Request::Status).await?;
            print_status(&response)?;
            Ok(())
        }
        Commands::Config { action } => match action {
            ConfigAction::Get { key } => {
                let response = send_request(Request::ConfigGet { key: key.clone() }).await?;
                println!("{} = {}", key, response["value"].as_str().unwrap_or(""));
                Ok(())
            }
            ConfigAction::Set { scope, key, value } => {
                let response = send_request(Request::ConfigSet {
                    scope,
                    key: key.clone(),
                    value: value.clone(),
                })
                .await?;
                println!(
                    "✓ Set {} = {} (scope: {})",
                    key,
                    value,
                    response["scope"].as_str().unwrap_or("")
                );
                Ok(())
            }
            ConfigAction::List => {
                let response = send_request(Request::ConfigList).await?;
                print_config_list(&response)?;
                Ok(())
            }
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
            TelemetryAction::List { limit } => {
                print_telemetry_list(limit)?;
                Ok(())
            }
            TelemetryAction::Stats => {
                print_telemetry_stats()?;
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

fn print_telemetry_list(limit: usize) -> Result<()> {
    use std::fs;
    use std::path::PathBuf;

    let mut telemetry_paths: Vec<PathBuf> = vec![
        PathBuf::from("/var/lib/anna/events"),
    ];

    // Add user telemetry path if HOME is set
    if let Some(home) = dirs::home_dir() {
        telemetry_paths.push(home.join(".local/share/anna/events"));
    }

    let mut all_events = Vec::new();

    for path in telemetry_paths {
        if path.exists() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let file_path = entry.path();
                    if file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                        if let Ok(contents) = fs::read_to_string(&file_path) {
                            for line in contents.lines().rev().take(limit) {
                                all_events.push(line.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\n📊 Recent Telemetry Events\n");

    if all_events.is_empty() {
        println!("No telemetry events found.");
    } else {
        for event in all_events.iter().take(limit) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(event) {
                println!("{}", serde_json::to_string_pretty(&json)?);
                println!("{}", "-".repeat(70));
            }
        }
    }

    println!();
    Ok(())
}

fn print_telemetry_stats() -> Result<()> {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    let mut telemetry_paths: Vec<PathBuf> = vec![
        PathBuf::from("/var/lib/anna/events"),
    ];

    // Add user telemetry path if HOME is set
    if let Some(home) = dirs::home_dir() {
        telemetry_paths.push(home.join(".local/share/anna/events"));
    }

    let mut event_counts: HashMap<String, usize> = HashMap::new();
    let mut total_events = 0;

    for path in telemetry_paths {
        if path.exists() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let file_path = entry.path();
                    if file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                        if let Ok(contents) = fs::read_to_string(&file_path) {
                            for line in contents.lines() {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                                    if let Some(event_type) = json["type"].as_str() {
                                        *event_counts.entry(event_type.to_string()).or_insert(0) += 1;
                                        total_events += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\n📈 Telemetry Statistics\n");
    println!("Total events: {}", total_events);
    println!("\nBy type:");

    let mut sorted: Vec<_> = event_counts.iter().collect();
    sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

    for (event_type, count) in sorted {
        println!("  {:30} {}", event_type, count);
    }

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
