// Anna v0.12.0 Control CLI - Consolidated Interface with JSON Support
// Commands: version, status, sensors, net, disk, top, events, export,
//           doctor, radar, classify

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

mod advisor_cmd;
mod doctor_cmd;
mod error_display;
mod health_cmd;
mod hw_cmd;
mod reload_cmd;
mod snapshot_cmd;
mod storage_cmd;
mod watch_mode;

const SOCKET_PATH: &str = "/run/anna/annad.sock";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "annactl")]
#[command(version, about = "Anna v0.12.0 - Event-Driven Intelligence CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version information
    Version,

    /// Show daemon status and health
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show verbose details
        #[arg(short, long)]
        verbose: bool,
        /// Watch mode (update every 2s)
        #[arg(short, long)]
        watch: bool,
    },

    /// Collect telemetry snapshots
    Collect {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Number of snapshots to retrieve (default: 1)
        #[arg(short, long, default_value = "1")]
        limit: u32,
    },

    /// Classify system persona
    Classify {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show radar scores
    Radar {
        #[command(subcommand)]
        action: RadarAction,
    },

    /// Show hardware profile (CPU, GPU, storage, network)
    Hw {
        #[command(subcommand)]
        action: HwAction,
    },

    /// Run Arch Linux advisor (system optimization recommendations)
    Advisor {
        #[command(subcommand)]
        action: AdvisorAction,
    },

    /// Show storage profile (Btrfs intelligence)
    Storage {
        #[command(subcommand)]
        action: StorageAction,
    },

    /// Show CPU, memory, temperatures, and battery
    Sensors {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show detailed sensor information
        #[arg(short, long)]
        detail: bool,
    },

    /// Show network interfaces and connectivity
    Net {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show detailed network information
        #[arg(short, long)]
        detail: bool,
    },

    /// Show disk usage and SMART status
    Disk {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show detailed disk information
        #[arg(short, long)]
        detail: bool,
    },

    /// Show top processes by CPU and memory
    Top {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Number of processes to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show recent system events
    Events {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Time window (5m, 1h, 1d)
        #[arg(long)]
        since: Option<String>,
        /// Number of events to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Export telemetry data
    Export {
        /// Output path (default: stdout)
        #[arg(short, long)]
        path: Option<String>,
        /// Time window (5m, 1h, 1d)
        #[arg(long)]
        since: Option<String>,
        /// Output as JSON (always JSON for export)
        #[arg(long)]
        json: bool,
    },

    /// Run system health checks and repairs
    Doctor {
        #[command(subcommand)]
        check: DoctorCheck,
    },

    /// Show daemon health metrics
    Health {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Watch mode (update every 1s)
        #[arg(short, long)]
        watch: bool,
    },

    /// Reload daemon configuration (sends SIGHUP)
    Reload {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Validate configuration file
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum DoctorCheck {
    /// Run preflight checks before installation
    Pre {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Run postflight checks after installation
    Post {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Repair installation issues
    Repair {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Run comprehensive health checks
    Check {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show detailed diagnostics
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Subcommand)]
enum RadarAction {
    /// Show radar scores
    Show {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum HwAction {
    /// Show hardware profile
    Show {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show detailed device information
        #[arg(short, long)]
        wide: bool,
    },
}

#[derive(Subcommand)]
enum AdvisorAction {
    /// Run Arch Linux advisor
    Arch {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Explain specific advice by ID
        #[arg(long)]
        explain: Option<String>,
    },
}

#[derive(Subcommand)]
enum StorageAction {
    /// Show Btrfs storage profile
    Btrfs {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show detailed subvolume information
        #[arg(short, long)]
        wide: bool,
        /// Explain specific topic (snapshots, compression, scrub, balance)
        #[arg(long)]
        explain: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Validate configuration file syntax
    Validate {
        /// Path to config file (default: /etc/anna/config.toml)
        #[arg(short, long)]
        path: Option<String>,
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
            println!(
                "Build: {} {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            );
            Ok(())
        }
        Commands::Status { json, verbose, watch } => {
            if watch {
                if json {
                    eprintln!("Warning: JSON output not supported in watch mode");
                }
                show_status_watch(verbose).await?;
            } else {
                let params = serde_json::json!({ "verbose": verbose });
                let response = rpc_call_with_retry("status", Some(params)).await?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&response)?);
                } else {
                    print_status(&response, verbose)?;
                }
            }
            Ok(())
        }
        Commands::Sensors { json, detail } => {
            let params = serde_json::json!({ "detail": detail });
            let response = rpc_call_with_retry("sensors", Some(params)).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_sensors(&response, detail)?;
            }
            Ok(())
        }
        Commands::Net { json, detail } => {
            let params = serde_json::json!({ "detail": detail });
            let response = rpc_call_with_retry("net", Some(params)).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_net(&response, detail)?;
            }
            Ok(())
        }
        Commands::Disk { json, detail } => {
            let params = serde_json::json!({ "detail": detail });
            let response = rpc_call_with_retry("disk", Some(params)).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_disk(&response, detail)?;
            }
            Ok(())
        }
        Commands::Top { json, limit } => {
            let params = serde_json::json!({ "limit": limit });
            let response = rpc_call_with_retry("top", Some(params)).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_top(&response, limit)?;
            }
            Ok(())
        }
        Commands::Events { json, since, limit } => {
            let params = serde_json::json!({ "since": since, "limit": limit });
            let response = rpc_call_with_retry("events", Some(params)).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_events(&response)?;
            }
            Ok(())
        }
        Commands::Export {
            path,
            since,
            json: _,
        } => {
            // Export is always JSON
            let params = serde_json::json!({ "since": since });
            let response = rpc_call_with_retry("export", Some(params)).await?;
            let output = serde_json::to_string_pretty(&response)?;
            if let Some(path_str) = path {
                std::fs::write(&path_str, output)?;
                println!("✓ Exported to {}", path_str);
            } else {
                println!("{}", output);
            }
            Ok(())
        }
        Commands::Hw { action } => {
            match action {
                HwAction::Show { json, wide } => {
                    hw_cmd::show_hardware(json, wide).await?;
                }
            }
            Ok(())
        }
        Commands::Advisor { action } => {
            match action {
                AdvisorAction::Arch { json, explain } => {
                    advisor_cmd::run_advisor(json, explain).await?;
                }
            }
            Ok(())
        }
        Commands::Storage { action } => {
            match action {
                StorageAction::Btrfs { json, wide, explain } => {
                    storage_cmd::show_btrfs(json, wide, explain.clone()).await?;
                }
            }
            Ok(())
        }
        Commands::Doctor { check } => {
            match check {
                DoctorCheck::Pre { json } => {
                    doctor_cmd::doctor_pre(json)?;
                }
                DoctorCheck::Post { json } => {
                    doctor_cmd::doctor_post(json)?;
                }
                DoctorCheck::Repair { json, yes } => {
                    doctor_cmd::doctor_repair(json, yes)?;
                }
                DoctorCheck::Check { json, verbose } => {
                    doctor_cmd::doctor_check(json, verbose).await?;
                }
            }
            Ok(())
        }
        Commands::Collect { json, limit } => {
            let params = serde_json::json!({ "limit": limit });
            let response = rpc_call_with_retry("collect", Some(params)).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_collect(&response)?;
            }
            Ok(())
        }
        Commands::Classify { json } => {
            let response = rpc_call_with_retry("classify", None).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_classify(&response)?;
            }
            Ok(())
        }
        Commands::Radar { action } => {
            match action {
                RadarAction::Show { json } => {
                    let response = rpc_call_with_retry("radar_show", None).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&response)?);
                    } else {
                        print_radar_show(&response)?;
                    }
                }
            }
            Ok(())
        }
        Commands::Health { json, watch } => {
            health_cmd::show_health(json, watch).await?;
            Ok(())
        }
        Commands::Reload { verbose } => {
            reload_cmd::reload_config(verbose).await?;
            Ok(())
        }
        Commands::Config { action } => {
            match action {
                ConfigAction::Validate { path, verbose } => {
                    reload_cmd::validate_config(path, verbose)?;
                }
            }
            Ok(())
        }
    }
}

/// RPC call with retry logic and exponential backoff
async fn rpc_call_with_retry(method: &str, params: Option<JsonValue>) -> Result<JsonValue> {
    use error_display::{display_error, display_retry_attempt, display_retry_exhausted, display_retry_success, RetryInfo, RpcError, RpcErrorCode};
    use std::time::Duration;

    // Retry policy
    const MAX_ATTEMPTS: u32 = 3;
    const INITIAL_DELAY_MS: u64 = 100;
    const MAX_DELAY_MS: u64 = 5000;
    const BACKOFF_MULTIPLIER: f64 = 2.0;
    const JITTER_FACTOR: f64 = 0.1;

    let start_time = Instant::now();
    let mut last_error: Option<anyhow::Error> = None;

    for attempt in 0..MAX_ATTEMPTS {
        match rpc_call(method, params.clone()).await {
            Ok(result) => {
                // Success!
                if attempt > 0 {
                    display_retry_success(attempt + 1, start_time.elapsed());
                }
                return Ok(result);
            }
            Err(e) => {
                // Classify error
                let rpc_error = classify_error(&e);
                let is_retryable = is_error_retryable(&e);

                last_error = Some(e);

                // Show error on first attempt or if not retryable
                if attempt == 0 || !is_retryable {
                    let retry_info = if is_retryable && attempt < MAX_ATTEMPTS - 1 {
                        Some(RetryInfo {
                            attempt,
                            max_attempts: MAX_ATTEMPTS,
                            elapsed: start_time.elapsed(),
                            next_delay: Some(calculate_delay(attempt, INITIAL_DELAY_MS, MAX_DELAY_MS, BACKOFF_MULTIPLIER, JITTER_FACTOR)),
                        })
                    } else {
                        None
                    };
                    display_error(&rpc_error, retry_info.as_ref());
                }

                // If not retryable or last attempt, give up
                if !is_retryable || attempt >= MAX_ATTEMPTS - 1 {
                    if attempt > 0 {
                        display_retry_exhausted(attempt + 1, start_time.elapsed());
                    }
                    return Err(last_error.unwrap());
                }

                // Calculate delay and wait
                let delay = calculate_delay(attempt, INITIAL_DELAY_MS, MAX_DELAY_MS, BACKOFF_MULTIPLIER, JITTER_FACTOR);
                display_retry_attempt(attempt, MAX_ATTEMPTS, delay);
                tokio::time::sleep(delay).await;
                print!("\r"); // Clear the retry message
            }
        }
    }

    // Should never reach here, but just in case
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
}

/// Calculate exponential backoff delay with jitter
fn calculate_delay(attempt: u32, initial_ms: u64, max_ms: u64, multiplier: f64, jitter: f64) -> std::time::Duration {
    use rand::Rng;

    let base_delay = initial_ms as f64 * multiplier.powi(attempt as i32);
    let capped_delay = base_delay.min(max_ms as f64);

    let mut rng = rand::thread_rng();
    let jitter_range = capped_delay * jitter;
    let jitter_value = rng.gen_range(-jitter_range..=jitter_range);
    let final_delay = (capped_delay + jitter_value).max(0.0);

    std::time::Duration::from_millis(final_delay as u64)
}

/// Classify anyhow::Error into structured RpcError
fn classify_error(e: &anyhow::Error) -> error_display::RpcError {
    use error_display::{RpcError, RpcErrorCode};

    let error_str = format!("{}", e);
    let error_lower = error_str.to_lowercase();

    let code = if error_lower.contains("connection refused") || error_lower.contains("no such file") {
        RpcErrorCode::ConnectionRefused
    } else if error_lower.contains("timeout") {
        RpcErrorCode::ConnectionTimeout
    } else if error_lower.contains("permission denied") {
        RpcErrorCode::PermissionDenied
    } else if error_lower.contains("connection reset") {
        RpcErrorCode::ConnectionReset
    } else if error_lower.contains("broken pipe") {
        RpcErrorCode::ConnectionClosed
    } else if error_lower.contains("invalid json") || error_lower.contains("malformed") {
        RpcErrorCode::MalformedJson
    } else if error_lower.contains("database") {
        RpcErrorCode::DatabaseError
    } else if error_lower.contains("storage") {
        RpcErrorCode::StorageError
    } else if error_lower.contains("config") {
        RpcErrorCode::ConfigParseError
    } else {
        RpcErrorCode::InternalError
    };

    RpcError::new(code).with_context(error_str)
}

/// Check if error is retryable
fn is_error_retryable(e: &anyhow::Error) -> bool {
    let error_str = format!("{}", e).to_lowercase();

    // Connection issues - retryable
    if error_str.contains("connection refused")
        || error_str.contains("connection reset")
        || error_str.contains("timeout")
        || error_str.contains("broken pipe") {
        return true;
    }

    // Resource issues - retryable
    if error_str.contains("resource busy")
        || error_str.contains("try again") {
        return true;
    }

    // Client errors - not retryable
    if error_str.contains("permission denied")
        || error_str.contains("no such file")
        || error_str.contains("invalid")
        || error_str.contains("malformed") {
        return false;
    }

    // Default: don't retry unless explicitly identified as retryable
    false
}

async fn rpc_call(method: &str, params: Option<JsonValue>) -> Result<JsonValue> {
    use tokio::time::{timeout, Duration};

    // Configurable timeouts
    const CONNECT_TIMEOUT_SECS: u64 = 2;
    const WRITE_TIMEOUT_SECS: u64 = 2;
    const READ_TIMEOUT_SECS: u64 = 5;

    // Connect with timeout
    let stream = match timeout(
        Duration::from_secs(CONNECT_TIMEOUT_SECS),
        UnixStream::connect(SOCKET_PATH),
    )
    .await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            anyhow::bail!(
                "Failed to connect to annad (socket: {})\n\
                 Error: {}\n\
                 Is the daemon running? Try: sudo systemctl status annad",
                SOCKET_PATH,
                e
            );
        }
        Err(_) => {
            eprintln!(
                "WARN: timeout (connect) - daemon not responding after {}s",
                CONNECT_TIMEOUT_SECS
            );
            std::process::exit(7);
        }
    };

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

    // Write with timeout
    match timeout(Duration::from_secs(WRITE_TIMEOUT_SECS), async {
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await
    })
    .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => anyhow::bail!("Write error: {}", e),
        Err(_) => {
            eprintln!(
                "WARN: timeout (write) - daemon not responding after {}s",
                WRITE_TIMEOUT_SECS
            );
            std::process::exit(7);
        }
    }

    // Read response with timeout
    let mut line = String::new();
    match timeout(
        Duration::from_secs(READ_TIMEOUT_SECS),
        reader.read_line(&mut line),
    )
    .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => anyhow::bail!("Read error: {}", e),
        Err(_) => {
            eprintln!(
                "WARN: timeout (read) - daemon not responding after {}s",
                READ_TIMEOUT_SECS
            );
            std::process::exit(7);
        }
    }

    let response: RpcResponse = serde_json::from_str(&line)?;

    if let Some(error) = response.error {
        anyhow::bail!("RPC error {}: {}", error.code, error.message);
    }

    response.result.context("No result in response")
}

/// Show status in watch mode with live updates
async fn show_status_watch(verbose: bool) -> Result<()> {
    use watch_mode::{WatchConfig, WatchMode, print_watch_header, print_watch_footer};
    use std::time::{Duration, Instant};
    use std::cell::RefCell;
    use std::rc::Rc;

    let config = WatchConfig {
        interval: Duration::from_secs(2),
        use_alternate_screen: true,
        clear_screen: true,
    };

    let mut watch = WatchMode::new(config);
    let start_time = Instant::now();
    let last_data: Rc<RefCell<Option<JsonValue>>> = Rc::new(RefCell::new(None));

    watch.run(|iteration| {
        let elapsed = start_time.elapsed();
        let last_data = Rc::clone(&last_data);

        async move {
            let params = serde_json::json!({ "verbose": verbose });
            let data = rpc_call_with_retry("status", Some(params)).await?;

            // Get previous data for delta calculation
            let prev_data = last_data.borrow().clone();

            // Display watch header
            print_watch_header("Daemon Status", iteration, elapsed);

            // Display status with deltas
            print_status_watch_display(&data, prev_data.as_ref(), verbose)?;

            // Display footer
            print_watch_footer();

            // Store for next iteration
            *last_data.borrow_mut() = Some(data);

            Ok(())
        }
    }).await?;

    Ok(())
}

/// Display status in watch mode with delta indicators
fn print_status_watch_display(data: &JsonValue, last: Option<&JsonValue>, verbose: bool) -> Result<()> {
    use watch_mode::format_count_delta;

    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";

    println!("{}│{}", dim, reset);
    println!("{}│{}  {}Daemon:{} {}", dim, reset, bold, reset,
        data["daemon_state"].as_str().unwrap_or("unknown"));
    println!("{}│{}  {}DB Path:{} {}", dim, reset, bold, reset,
        data["db_path"].as_str().unwrap_or("unknown"));

    // Sample count with delta
    let sample_count = data["sample_count"].as_u64().unwrap_or(0);
    print!("{}│{}  {}Sample Count:{} {}", dim, reset, bold, reset, sample_count);
    if let Some(last) = last {
        let last_count = last["sample_count"].as_u64().unwrap_or(0);
        if sample_count != last_count {
            let delta = format_count_delta(last_count, sample_count);
            print!("  ({})", delta);
        }
    }
    println!();

    // Loop load
    println!("{}│{}  {}Loop Load:{} {:.1}%", dim, reset, bold, reset,
        data["loop_load_pct"].as_f64().unwrap_or(0.0));

    if let Some(pid) = data["annad_pid"].as_u64() {
        println!("{}│{}  {}Process ID:{} {}", dim, reset, bold, reset, pid);
    }

    if verbose {
        if let Some(uptime) = data["uptime_secs"].as_u64() {
            let hours = uptime / 3600;
            let mins = (uptime % 3600) / 60;
            println!("{}│{}  {}Uptime:{} {}h {}m", dim, reset, bold, reset, hours, mins);
        }
    }

    Ok(())
}

fn print_status(data: &JsonValue, verbose: bool) -> Result<()> {
    println!("\n╭─ Anna Status ────────────────────────────────");
    println!("│");
    println!(
        "│  Daemon:       {}",
        data["daemon_state"].as_str().unwrap_or("unknown")
    );
    println!(
        "│  DB Path:      {}",
        data["db_path"].as_str().unwrap_or("unknown")
    );
    println!(
        "│  Last Sample:  {} seconds ago",
        data["last_sample_age_s"].as_u64().unwrap_or(0)
    );
    println!(
        "│  Sample Count: {}",
        data["sample_count"].as_u64().unwrap_or(0)
    );
    println!(
        "│  Loop Load:    {:.1}%",
        data["loop_load_pct"].as_f64().unwrap_or(0.0)
    );
    println!("│");

    if let Some(pid) = data["annad_pid"].as_u64() {
        println!("│  Process ID:   {}", pid);
    }

    if verbose {
        if let Some(socket) = data["socket_path"].as_str() {
            println!("│  Socket:       {}", socket);
        }
        if let Some(uptime) = data["uptime_secs"].as_u64() {
            let hours = uptime / 3600;
            let mins = (uptime % 3600) / 60;
            println!("│  Uptime:       {}h {}m", hours, mins);
        }
    }

    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

fn print_sensors(data: &JsonValue, _detail: bool) -> Result<()> {
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
                let temp_str = temp.map(|t| format!(" {}°C", t as i32)).unwrap_or_default();

                println!(
                    "│    Core {}: {} {:>5.1}%{}",
                    core["core"].as_u64().unwrap_or(0),
                    bar,
                    util,
                    temp_str
                );
            }
        }

        if let Some(load) = cpu["load_avg"].as_array() {
            println!(
                "│    Load: {:.2}, {:.2}, {:.2}",
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
        println!(
            "│  Memory: {} {:>5.1}%  ({:.1}/{:.1} GB)",
            bar, pct, used, total
        );

        if let Some(swap_total) = mem["swap_total_mb"].as_u64() {
            if swap_total > 0 {
                let swap_used = mem["swap_used_mb"].as_u64().unwrap_or(0) as f64 / 1024.0;
                let swap_pct = (swap_used / (swap_total as f64 / 1024.0) * 100.0) as f32;
                let swap_bar = progress_bar(swap_pct, 20);
                println!(
                    "│  Swap:   {} {:>5.1}%  ({:.1} GB)",
                    swap_bar, swap_pct, swap_used
                );
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

fn print_net(data: &JsonValue, _detail: bool) -> Result<()> {
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

fn print_disk(data: &JsonValue, _detail: bool) -> Result<()> {
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

fn print_top(data: &JsonValue, limit: usize) -> Result<()> {
    println!("\n╭─ Top Processes ──────────────────────────────");
    println!("│");

    if let Some(by_cpu) = data["by_cpu"].as_array() {
        println!("│  By CPU:");
        for (i, proc) in by_cpu.iter().take(limit).enumerate() {
            let name = proc["name"].as_str().unwrap_or("?");
            let cpu = proc["cpu_pct"].as_f64().unwrap_or(0.0);
            let pid = proc["pid"].as_u64().unwrap_or(0);

            println!("│    {}. {:>6.1}%  {} (PID {})", i + 1, cpu, name, pid);
        }
    }

    println!("│");

    if let Some(by_mem) = data["by_mem"].as_array() {
        println!("│  By Memory:");
        for (i, proc) in by_mem.iter().take(limit).enumerate() {
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

fn print_collect(data: &JsonValue) -> Result<()> {
    println!("\n╭─ Telemetry Snapshots ────────────────────────");
    println!("│");

    let count = data["count"].as_u64().unwrap_or(0);
    println!("│  Collected {} snapshot(s)", count);
    println!("│");

    if let Some(snapshots) = data["snapshots"].as_array() {
        for (i, snap) in snapshots.iter().enumerate() {
            println!("│  Snapshot {}", i + 1);
            if let Some(ts) = snap["ts"].as_u64() {
                use std::time::{Duration, SystemTime, UNIX_EPOCH};
                let snap_time = UNIX_EPOCH + Duration::from_secs(ts);
                let age = SystemTime::now().duration_since(snap_time).ok();
                if let Some(age) = age {
                    println!("│    Age: {} seconds ago", age.as_secs());
                }
            }

            if let Some(sensors) = snap.get("sensors") {
                if let Some(cpu) = sensors.get("cpu") {
                    if let Some(load_avg) = cpu["load_avg"].as_array() {
                        println!(
                            "│    CPU Load: {:.2}, {:.2}, {:.2}",
                            load_avg[0].as_f64().unwrap_or(0.0),
                            load_avg[1].as_f64().unwrap_or(0.0),
                            load_avg[2].as_f64().unwrap_or(0.0)
                        );
                    }
                }

                if let Some(mem) = sensors.get("mem") {
                    let used = mem["used_mb"].as_u64().unwrap_or(0);
                    let total = mem["total_mb"].as_u64().unwrap_or(1);
                    let pct = (used as f64 / total as f64) * 100.0;
                    println!("│    Memory: {:.1}% used ({} MB / {} MB)", pct, used, total);
                }
            }

            if let Some(disk) = snap.get("disk") {
                if let Some(disks) = disk["disks"].as_array() {
                    println!("│    Disks: {} mounted", disks.len());
                }
            }

            println!("│");
        }
    }

    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

fn print_classify(data: &JsonValue) -> Result<()> {
    println!("\n╭─ System Classification ──────────────────────");
    println!("│");

    if let Some(persona) = data["persona"].as_str() {
        println!("│  Persona:     {}", persona);
    }
    if let Some(confidence) = data["confidence"].as_f64() {
        println!("│  Confidence:  {:.1}%", confidence * 100.0);
    }

    println!("│");

    // Evidence
    if let Some(evidence) = data["evidence"].as_array() {
        println!("│  Evidence:");
        for item in evidence {
            if let Some(s) = item.as_str() {
                println!("│    • {}", s);
            }
        }
    }

    println!("│");

    // System Health Radar
    if let Some(health) = data["radars"].get("system_health") {
        println!("│  System Health Radar:");
        print_radar_categories(health)?;
    }

    // Network Posture Radar
    if let Some(network) = data["radars"].get("network_posture") {
        println!("│  Network Posture Radar:");
        print_radar_categories(network)?;
    }

    println!("│");
    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

fn print_radar_show(data: &JsonValue) -> Result<()> {
    println!("\n╭─ Radar Scores ───────────────────────────────");
    println!("│");

    // Health Radar
    if let Some(health) = data.get("health") {
        println!("│  Health Radar:");
        print_radar_categories(health)?;
    }

    // Network Radar
    if let Some(network) = data.get("network") {
        println!("│  Network Radar:");
        print_radar_categories(network)?;
    }

    // Overall
    if let Some(overall) = data.get("overall") {
        println!("│  Overall Scores:");
        if let Some(health_score) = overall["health_score"].as_f64() {
            println!("│    Health:  {:.1}/10.0", health_score);
        }
        if let Some(network_score) = overall["network_score"].as_f64() {
            println!("│    Network: {:.1}/10.0", network_score);
        }
        if let Some(combined) = overall["combined"].as_f64() {
            println!("│    Combined: {:.1}/10.0", combined);
        }
    }

    println!("│");
    println!("╰──────────────────────────────────────────────\n");
    Ok(())
}

fn print_radar_categories(radar: &JsonValue) -> Result<()> {
    if let Some(categories) = radar["categories"].as_object() {
        for (name, cat) in categories {
            let score = cat["score"].as_f64().unwrap_or(0.0);
            let max = cat["max"].as_f64().unwrap_or(10.0);

            if cat["score"].is_null() {
                println!("│    {:<20} N/A", name);
            } else {
                let _pct = (score / max * 100.0) as f32;
                let bar_len = (score / max * 15.0) as usize;
                let bar = "▓".repeat(bar_len) + &"░".repeat(15 - bar_len);
                println!("│    {:<20} [{}] {:>4.1}/{:.0}", name, bar, score, max);
            }
        }
    }
    println!("│");
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
                println!(
                    "│     └─ {} alerts, action: {} ({}ms)",
                    alerts, action, duration
                );
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
