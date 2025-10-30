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
    Doctor,

    /// Show daemon status
    Status,

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
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
            Ok(())
        }
        Commands::Doctor => {
            let response = send_request(Request::Doctor).await?;
            print_diagnostics(&response)
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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pass,
    Warn,
    Fail,
}
