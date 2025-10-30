use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/run/anna.sock";

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

    /// Show current configuration
    Config,

    /// Request privilege elevation for system-level operations
    Elevate {
        /// Operation to perform with elevated privileges
        operation: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    Ping,
    Doctor,
    Status,
    GetConfig,
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

    match cli.command {
        Commands::Ping => {
            let response = send_request(Request::Ping).await?;
            println!("✓ {}", response["message"].as_str().unwrap_or("OK"));
        }
        Commands::Doctor => {
            let response = send_request(Request::Doctor).await?;
            print_diagnostics(&response)?;
        }
        Commands::Status => {
            let response = send_request(Request::Status).await?;
            print_status(&response)?;
        }
        Commands::Config => {
            let response = send_request(Request::GetConfig).await?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        Commands::Elevate { operation } => {
            println!("Requesting privilege elevation for: {}", operation);
            println!("(Polkit integration pending)");
        }
    }

    Ok(())
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
    println!("{}", "=".repeat(50));

    for check in &results.checks {
        let icon = match check.status {
            Status::Pass => "✓",
            Status::Warn => "⚠",
            Status::Fail => "✗",
        };
        println!("{} {} - {}", icon, check.name, check.message);
    }

    println!("{}", "=".repeat(50));
    println!(
        "\nOverall Status: {}",
        match results.overall_status {
            Status::Pass => "✓ PASS",
            Status::Warn => "⚠ WARNING",
            Status::Fail => "✗ FAIL",
        }
    );

    Ok(())
}

fn print_status(data: &serde_json::Value) -> Result<()> {
    println!("\n📊 Anna Daemon Status\n");
    println!("Version: {}", data["version"].as_str().unwrap_or("unknown"));
    println!("Status: {}", data["uptime"].as_str().unwrap_or("unknown"));
    println!(
        "Autonomy Tier: {}",
        data["autonomy_tier"].as_u64().unwrap_or(0)
    );

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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pass,
    Warn,
    Fail,
}
