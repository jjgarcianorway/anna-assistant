use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use std::collections::HashMap;

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

    /// Run system diagnostics
    Doctor {
        /// Run auto-fix for failed checks (requires autonomy != off)
        #[arg(long)]
        autofix: bool,
    },

    /// Show daemon status
    Status,

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Autonomy management (Sprint 2)
    Autonomy {
        #[command(subcommand)]
        action: AutonomyAction,
    },

    /// State persistence management (Sprint 2)
    State {
        #[command(subcommand)]
        action: StateAction,
    },

    /// Telemetry management (Sprint 2)
    Telemetry {
        #[command(subcommand)]
        action: TelemetryAction,
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
enum AutonomyAction {
    /// Show autonomy status
    Status,

    /// Run an autonomous task
    Run {
        /// Task name (doctor, telemetry_cleanup, config_sync)
        task: String,
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
        Commands::Doctor { autofix } => {
            if autofix {
                let response = send_request(Request::DoctorAutoFix).await?;
                print_autofix_results(&response)?;
            } else {
                let response = send_request(Request::Doctor).await?;
                print_diagnostics(&response)?;
            }
            Ok(())
        }
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
            AutonomyAction::Status => {
                let response = send_request(Request::AutonomyStatus).await?;
                print_autonomy_status(&response)?;
                Ok(())
            }
            AutonomyAction::Run { task } => {
                let response = send_request(Request::AutonomyRun { task: task.clone() }).await?;
                print_task_result(&response)?;
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
    let stream = UnixStream::connect(SOCKET_PATH)
        .await
        .context("Failed to connect to daemon. Is annad running?")?;

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
    println!("\n📊 Anna Daemon Status\n");
    println!("Version:       {}", data["version"].as_str().unwrap_or("unknown"));
    println!("Status:        {}", data["uptime"].as_str().unwrap_or("unknown"));
    println!(
        "Autonomy:      {}",
        data["autonomy_level"].as_str().unwrap_or("unknown")
    );
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
