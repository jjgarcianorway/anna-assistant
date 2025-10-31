// Anna v0.11.0 Control CLI - Event-Driven Intelligence Interface
// Commands: version, status, sensors, net, disk, top, radar, export,
//           events, watch, capabilities, module, alerts, fix, doctor

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

mod alerts_cmd;
mod capabilities_cmd;
mod doctor_cmd;
mod module_cmd;

const SOCKET_PATH: &str = "/run/anna/annad.sock";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "annactl")]
#[command(version, about = "Anna v0.11.0 - Event-Driven Intelligence CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version information
    Version,

    /// Show daemon status and health
    Status,

    /// Show CPU, memory, temperatures, and battery
    Sensors,

    /// Show network interfaces and connectivity
    Net,

    /// Show disk usage and SMART status
    Disk,

    /// Show top processes by CPU and memory
    Top,

    /// Show persona radar scores
    Radar,

    /// Export telemetry data as JSON
    Export {
        /// Output to file instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show recent system events (v0.11.0)
    Events {
        /// Number of events to show (default: 50)
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// Watch for live system events (v0.11.0)
    Watch,

    /// Show module capabilities and their status
    Capabilities,

    /// Enable or disable a telemetry module
    Module {
        #[command(subcommand)]
        action: ModuleAction,
    },

    /// Show current system integrity alerts
    Alerts,

    /// Fix a specific integrity issue
    Fix {
        /// Issue ID to fix
        issue_id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Run system health checks
    Doctor {
        #[command(subcommand)]
        check: DoctorCheck,
    },
}

#[derive(Subcommand)]
enum ModuleAction {
    /// Enable a module
    Enable {
        /// Module name to enable
        name: String,
    },
    /// Disable a module
    Disable {
        /// Module name to disable
        name: String,
        /// Reason for disabling
        #[arg(short, long)]
        reason: Option<String>,
    },
}

#[derive(Subcommand)]
enum DoctorCheck {
    /// Run preflight checks before installation
    Pre {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Run postflight checks after installation
    Post {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<JsonValue>,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcResponse {
    jsonrpc: String,
    result: Option<JsonValue>,
    error: Option<RpcError>,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            println!("Anna v{} - Event-Driven Intelligence", VERSION);
            println!("Build: {} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Status => {
            let response = rpc_call("status", None).await?;
            print_status(&response)?;
            Ok(())
        }
        Commands::Sensors => {
            let response = rpc_call("sensors", None).await?;
            print_sensors(&response)?;
            Ok(())
        }
        Commands::Net => {
            let response = rpc_call("net", None).await?;
            print_net(&response)?;
            Ok(())
        }
        Commands::Disk => {
            let response = rpc_call("disk", None).await?;
            print_disk(&response)?;
            Ok(())
        }
        Commands::Top => {
            let response = rpc_call("top", None).await?;
            print_top(&response)?;
            Ok(())
        }
        Commands::Radar => {
            let response = rpc_call("radar", None).await?;
            print_radar(&response)?;
            Ok(())
        }
        Commands::Export { output } => {
            let response = rpc_call("export", None).await?;
            if let Some(path) = output {
                std::fs::write(&path, serde_json::to_string_pretty(&response)?)?;
                println!("✓ Exported to {}", path);
            } else {
                println!("{}", serde_json::to_string_pretty(&response)?);
            }
            Ok(())
        }
        Commands::Events { limit } => {
            let params = serde_json::json!({ "limit": limit });
            let response = rpc_call("events", Some(params)).await?;
            print_events(&response)?;
            Ok(())
        }
        Commands::Watch => {
            print_watch_header();
            loop {
                let response = rpc_call("watch", None).await?;
                print_watch_update(&response)?;
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
        Commands::Capabilities => {
            capabilities_cmd::show_capabilities()?;
            Ok(())
        }
        Commands::Module { action } => {
            match action {
                ModuleAction::Enable { name } => {
                    module_cmd::enable_module(&name)?;
                }
                ModuleAction::Disable { name, reason } => {
                    module_cmd::disable_module(&name, reason)?;
                }
            }
            Ok(())
        }
        Commands::Alerts => {
            alerts_cmd::show_alerts()?;
            Ok(())
        }
        Commands::Fix { issue_id, yes } => {
            alerts_cmd::fix_issue(&issue_id, yes)?;
            Ok(())
        }
        Commands::Doctor { check } => {
            match check {
                DoctorCheck::Pre { verbose } => {
                    doctor_cmd::doctor_pre(verbose)?;
                }
                DoctorCheck::Post { verbose } => {
                    doctor_cmd::doctor_post(verbose)?;
                }
            }
            Ok(())
        }
    }
}

async fn rpc_call(method: &str, params: Option<JsonValue>) -> Result<JsonValue> {
    let stream = UnixStream::connect(SOCKET_PATH).await.context(format!(
        "Failed to connect to annad (socket: {})\n\
         Is the daemon running? Try: sudo systemctl status annad",
        SOCKET_PATH
    ))?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send request
    let request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: 1,
    };

    let json = serde_json::to_string(&request)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    // Read response
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let response: RpcResponse = serde_json::from_str(&line)?;

    if let Some(error) = response.error {
        anyhow::bail!("RPC error {}: {}", error.code, error.message);
    }

    response.result.context("No result in response")
}

fn print_status(data: &JsonValue) -> Result<()> {
    println!("\n╭─ Anna Status ────────────────────────────────");
    println!("│");
    println!("│  Daemon:       {}", data["daemon_state"].as_str().unwrap_or("unknown"));
    println!("│  DB Path:      {}", data["db_path"].as_str().unwrap_or("unknown"));
    println!("│  Last Sample:  {} seconds ago", data["last_sample_age_s"].as_u64().unwrap_or(0));
    println!("│  Sample Count: {}", data["sample_count"].as_u64().unwrap_or(0));
    println!("│  Loop Load:    {:.1}%", data["loop_load_pct"].as_f64().unwrap_or(0.0));
    println!("│");

    if let Some(pid) = data["annad_pid"].as_u64() {
        println!("│  Process ID:   {}", pid);
    }

    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

fn print_sensors(data: &JsonValue) -> Result<()> {
    println!("\n╭─ System Sensors ─────────────────────────────");
    println!("│");

    // CPU
    if let Some(cpu) = data.get("cpu") {
        println!("│  CPU");
        if let Some(cores) = cpu["cores"].as_array() {
            for core in cores {
                let util = core["util_pct"].as_f64().unwrap_or(0.0);
                let temp = core["temp_c"].as_f64();

                let bar = progress_bar(util as f32, 20);
                let temp_str = temp.map(|t| format!(" {}°C", t as i32))
                    .unwrap_or_default();

                println!("│    Core {}: {} {:>5.1}%{}",
                    core["core"].as_u64().unwrap_or(0),
                    bar,
                    util,
                    temp_str
                );
            }
        }

        if let Some(load) = cpu["load_avg"].as_array() {
            println!("│    Load: {:.2}, {:.2}, {:.2}",
                load[0].as_f64().unwrap_or(0.0),
                load[1].as_f64().unwrap_or(0.0),
                load[2].as_f64().unwrap_or(0.0)
            );
        }
    }

    println!("│");

    // Memory
    if let Some(mem) = data.get("mem") {
        let total = mem["total_mb"].as_u64().unwrap_or(1) as f64 / 1024.0;
        let used = mem["used_mb"].as_u64().unwrap_or(0) as f64 / 1024.0;
        let pct = (used / total * 100.0) as f32;

        let bar = progress_bar(pct, 20);
        println!("│  Memory: {} {:>5.1}%  ({:.1}/{:.1} GB)", bar, pct, used, total);

        if let Some(swap_total) = mem["swap_total_mb"].as_u64() {
            if swap_total > 0 {
                let swap_used = mem["swap_used_mb"].as_u64().unwrap_or(0) as f64 / 1024.0;
                let swap_pct = (swap_used / (swap_total as f64 / 1024.0) * 100.0) as f32;
                let swap_bar = progress_bar(swap_pct, 20);
                println!("│  Swap:   {} {:>5.1}%  ({:.1} GB)", swap_bar, swap_pct, swap_used);
            }
        }
    }

    println!("│");

    // Battery (if present)
    if let Some(power) = data.get("power") {
        let pct = power["percent"].as_u64().unwrap_or(0);
        let status = power["status"].as_str().unwrap_or("Unknown");
        let icon = match status {
            "Charging" => "🔌",
            "Discharging" => "🔋",
            "Full" => "✓",
            _ => "⚠",
        };

        println!("│  Battery: {} {}%  ({})", icon, pct, status);

        if let Some(watts) = power["power_now_w"].as_f64() {
            println!("│           {:.1}W", watts);
        }
    }

    println!("│");
    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

fn print_net(data: &JsonValue) -> Result<()> {
    println!("\n╭─ Network Interfaces ─────────────────────────");
    println!("│");

    if let Some(ifaces) = data["interfaces"].as_array() {
        for iface in ifaces {
            let name = iface["iface"].as_str().unwrap_or("?");
            let state = iface["link_state"].as_str().unwrap_or("unknown");
            let rx_kbps = iface["rx_kbps"].as_f64().unwrap_or(0.0);
            let tx_kbps = iface["tx_kbps"].as_f64().unwrap_or(0.0);

            let state_icon = match state {
                "up" => "●",
                "down" => "○",
                _ => "?",
            };

            println!("│  {} {:<12}  {}", state_icon, name, state);
            println!("│     ↓ {:>8.1} KB/s  ↑ {:>8.1} KB/s", rx_kbps, tx_kbps);

            if let Some(ipv4) = iface["ipv4_redacted"].as_str() {
                println!("│     IPv4: {}", ipv4);
            }
            println!("│");
        }
    }

    // Show default route
    if let Some(route) = data.get("default_route") {
        println!("│  Default Route: {}", route.as_str().unwrap_or("none"));
    }

    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

fn print_disk(data: &JsonValue) -> Result<()> {
    println!("\n╭─ Disk Usage ─────────────────────────────────");
    println!("│");

    if let Some(disks) = data["disks"].as_array() {
        for disk in disks {
            let mount = disk["mount"].as_str().unwrap_or("?");
            let device = disk["device"].as_str().unwrap_or("?");
            let pct = disk["pct"].as_f64().unwrap_or(0.0) as f32;
            let used = disk["used_gb"].as_f64().unwrap_or(0.0);
            let total = disk["total_gb"].as_f64().unwrap_or(0.0);

            let bar = progress_bar(pct, 20);

            println!("│  {:<20}", mount);
            println!("│    {} {:>5.1}%  ({:.1}/{:.1} GB)", bar, pct, used, total);
            println!("│    Device: {}", device);
            println!("│");
        }
    }

    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

fn print_top(data: &JsonValue) -> Result<()> {
    println!("\n╭─ Top Processes ──────────────────────────────");
    println!("│");

    if let Some(by_cpu) = data["by_cpu"].as_array() {
        println!("│  By CPU:");
        for (i, proc) in by_cpu.iter().take(5).enumerate() {
            let name = proc["name"].as_str().unwrap_or("?");
            let cpu = proc["cpu_pct"].as_f64().unwrap_or(0.0);
            let pid = proc["pid"].as_u64().unwrap_or(0);

            println!("│    {}. {:>6.1}%  {} (PID {})", i + 1, cpu, name, pid);
        }
    }

    println!("│");

    if let Some(by_mem) = data["by_mem"].as_array() {
        println!("│  By Memory:");
        for (i, proc) in by_mem.iter().take(5).enumerate() {
            let name = proc["name"].as_str().unwrap_or("?");
            let mem = proc["mem_mb"].as_f64().unwrap_or(0.0);
            let pid = proc["pid"].as_u64().unwrap_or(0);

            println!("│    {}. {:>7.1} MB  {} (PID {})", i + 1, mem, name, pid);
        }
    }

    println!("│");
    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

fn print_radar(data: &JsonValue) -> Result<()> {
    println!("\n╭─ Persona Radar ──────────────────────────────");
    println!("│");

    if let Some(personas) = data["personas"].as_array() {
        for persona in personas {
            let name = persona["name"].as_str().unwrap_or("?");
            let score = persona["score"].as_f64().unwrap_or(0.0) as f32;

            let bar_len = (score / 10.0 * 20.0) as usize;
            let bar = "▓".repeat(bar_len) + &"░".repeat(20 - bar_len);

            println!("│  {:<20} [{}] {:>4.1}", name, bar, score);

            if let Some(evidence) = persona["evidence"].as_array() {
                if !evidence.is_empty() {
                    let top = evidence[0].as_str().unwrap_or("");
                    if !top.is_empty() {
                        println!("│    └─ {}", top);
                    }
                }
            }
        }
    }

    println!("│");
    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

/// Draw a Unicode progress bar
fn progress_bar(pct: f32, width: usize) -> String {
    let filled = (pct / 100.0 * width as f32) as usize;
    let filled = filled.min(width);

    "█".repeat(filled) + &"░".repeat(width - filled)
}

fn print_events(data: &JsonValue) -> Result<()> {
    println!("\n╭─ System Events ──────────────────────────────");
    println!("│");

    let count = data["count"].as_u64().unwrap_or(0);
    let pending = data["pending"].as_u64().unwrap_or(0);

    println!("│  Showing: {} events    Pending: {}", count, pending);
    println!("│");

    if let Some(events) = data["events"].as_array() {
        for event in events {
            let ev = &event["event"];
            let domain = ev["domain"].as_str().unwrap_or("?");
            let cause = ev["cause"].as_str().unwrap_or("?");
            let ts = ev["timestamp"].as_i64().unwrap_or(0);

            let doctor = &event["doctor_result"];
            let alerts = doctor["alerts_found"].as_u64().unwrap_or(0);
            let action = doctor["action_taken"].as_str().unwrap_or("?");
            let duration = event["duration_ms"].as_u64().unwrap_or(0);

            // Format timestamp as relative time
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let age_s = now - ts;
            let time_str = if age_s < 60 {
                format!("{}s ago", age_s)
            } else if age_s < 3600 {
                format!("{}m ago", age_s / 60)
            } else {
                format!("{}h ago", age_s / 3600)
            };

            // Domain icon
            let icon = match domain {
                "packages" => "📦",
                "config" => "⚙",
                "devices" => "🔌",
                "network" => "🌐",
                "storage" => "💾",
                "kernel" => "🐧",
                _ => "•",
            };

            println!("│  {} {:<10}  {:<12}  {}", icon, domain, time_str, cause);

            if alerts > 0 {
                println!("│     └─ {} alerts, action: {} ({}ms)", alerts, action, duration);
            } else {
                println!("│     └─ no alerts, action: {} ({}ms)", action, duration);
            }

            if let Some(repair) = event.get("repair_result") {
                let success = repair["success"].as_bool().unwrap_or(false);
                let msg = repair["message"].as_str().unwrap_or("");
                let icon = if success { "✓" } else { "✗" };
                println!("│        {} Repair: {}", icon, msg);
            }

            println!("│");
        }
    }

    if count == 0 {
        println!("│  No events recorded yet.");
        println!("│  System event listeners are active.");
        println!("│");
    }

    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

fn print_watch_header() {
    println!("\n╭─ Watching System Events ─────────────────────");
    println!("│  Press Ctrl+C to stop");
    println!("╰──────────────────────────────────────────────");
    println!();
}

fn print_watch_update(data: &JsonValue) -> Result<()> {
    use std::io::Write;

    let pending = data["pending_count"].as_u64().unwrap_or(0);

    if let Some(recent) = data["recent_events"].as_array() {
        if !recent.is_empty() {
            for event in recent {
                let ev = &event["event"];
                let domain = ev["domain"].as_str().unwrap_or("?");
                let cause = ev["cause"].as_str().unwrap_or("?");

                let doctor = &event["doctor_result"];
                let alerts = doctor["alerts_found"].as_u64().unwrap_or(0);
                let action = doctor["action_taken"].as_str().unwrap_or("?");

                // Timestamp
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let ts = chrono::DateTime::from_timestamp(now as i64, 0)
                    .unwrap()
                    .format("%H:%M:%S");

                let icon = match domain {
                    "packages" => "📦",
                    "config" => "⚙",
                    "devices" => "🔌",
                    "network" => "🌐",
                    "storage" => "💾",
                    "kernel" => "🐧",
                    _ => "•",
                };

                print!("[{}] {} {:<10}  ", ts, icon, domain);
                print!("{:<30}  ", cause);

                if alerts > 0 {
                    println!("{} alerts, {}", alerts, action);
                } else {
                    println!("ok, {}", action);
                }
                std::io::stdout().flush()?;
            }
        }
    }

    if pending > 0 {
        println!("  (pending: {})", pending);
        std::io::stdout().flush()?;
    }

    Ok(())
}
