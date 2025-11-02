// Anna v0.12.7 - Health Command Implementation
// Real-time daemon health metrics with TUI display

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/run/anna/annad.sock";

/// Health status from daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// RPC latency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RpcLatencyMetrics {
    avg_ms: f64,
    p50_ms: u64,
    p95_ms: u64,
    p99_ms: u64,
    min_ms: u64,
    max_ms: u64,
    sample_count: usize,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryMetrics {
    current_mb: f64,
    peak_mb: f64,
    limit_mb: u64,
    vmsize_mb: f64,
    threads: usize,
}

/// Queue health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueueMetrics {
    depth: usize,
    rate_per_sec: f64,
    oldest_event_sec: u64,
    total_processed: u64,
}

/// Complete health snapshot from daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthSnapshot {
    status: HealthStatus,
    uptime_sec: u64,
    rpc_latency: Option<RpcLatencyMetrics>,
    memory: Option<MemoryMetrics>,
    queue: Option<QueueMetrics>,
    capabilities_active: usize,
    capabilities_degraded: usize,
    timestamp: u64,
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

/// Show daemon health metrics
pub async fn show_health(json: bool, watch: bool) -> Result<()> {
    if watch {
        if json {
            eprintln!("Warning: JSON output not supported in watch mode");
        }
        show_health_watch().await?;
    } else {
        let health = fetch_health_metrics().await?;

        if json {
            println!("{}", serde_json::to_string_pretty(&health)?);
        } else {
            print_health_tui(&health)?;
        }
    }

    Ok(())
}

/// Show daemon health in watch mode with live updates
async fn show_health_watch() -> Result<()> {
    use crate::watch_mode::{WatchConfig, WatchMode, print_watch_header, print_watch_footer};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    let config = WatchConfig {
        interval: Duration::from_secs(2),
        use_alternate_screen: true,
        clear_screen: true,
    };

    let mut watch = WatchMode::new(config);
    let start_time = Instant::now();
    let last_health: Rc<RefCell<Option<HealthSnapshot>>> = Rc::new(RefCell::new(None));

    watch.run(|iteration| {
        let elapsed = start_time.elapsed();
        let last_health = Rc::clone(&last_health);

        async move {
            let health = fetch_health_metrics().await?;

            // Get previous health for delta calculation
            let prev_health = last_health.borrow().clone();

            // Display watch header
            print_watch_header("Daemon Health", iteration, elapsed);

            // Display health metrics with deltas
            print_health_watch_display(&health, prev_health.as_ref())?;

            // Display footer
            print_watch_footer();

            // Store for next iteration delta
            *last_health.borrow_mut() = Some(health);

            Ok(())
        }
    }).await?;

    Ok(())
}

/// Display health metrics in watch mode with delta indicators
fn print_health_watch_display(health: &HealthSnapshot, last: Option<&HealthSnapshot>) -> Result<()> {
    use crate::watch_mode::{format_delta, format_count_delta};

    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";
    let green = "\x1b[32m";
    let yellow = "\x1b[33m";
    let red = "\x1b[31m";

    // Overall status
    let status_color = match health.status {
        HealthStatus::Healthy => green,
        HealthStatus::Warning => yellow,
        HealthStatus::Critical => red,
        HealthStatus::Unknown => dim,
    };
    println!("{}│{}  {}Overall Status:{} {}{:?}{}",
        dim, reset, bold, reset, status_color, health.status, reset);
    println!("{}│{}", dim, reset);

    // Uptime
    let hours = health.uptime_sec / 3600;
    let mins = (health.uptime_sec % 3600) / 60;
    println!("{}│{}  {}Uptime:{} {}h {}m", dim, reset, bold, reset, hours, mins);
    println!("{}│{}", dim, reset);

    // RPC Latency
    println!("{}│{}  {}RPC Latency:{}", dim, reset, bold, reset);
    if let Some(ref latency) = health.rpc_latency {
        println!("{}│{}    p50:  {:>4}ms", dim, reset, latency.p50_ms);
        println!("{}│{}    p95:  {:>4}ms", dim, reset, latency.p95_ms);
        println!("{}│{}    p99:  {:>4}ms", dim, reset, latency.p99_ms);

        if let Some(last) = last {
            if let Some(ref last_latency) = last.rpc_latency {
                let delta = format_delta(last_latency.p99_ms as f64, latency.p99_ms as f64);
                println!("{}│{}    Delta: {}", dim, reset, delta);
            }
        }
    } else {
        println!("{}│{}    No samples yet", dim, reset);
    }
    println!("{}│{}", dim, reset);

    // Memory
    if let Some(ref memory) = health.memory {
        println!("{}│{}  {}Memory:{}", dim, reset, bold, reset);
        println!("{}│{}    Current:  {:.1} MB", dim, reset, memory.current_mb);
        println!("{}│{}    Peak:     {:.1} MB", dim, reset, memory.peak_mb);
        println!("{}│{}    Limit:    {} MB", dim, reset, memory.limit_mb);

        if let Some(last) = last {
            if let Some(ref last_mem) = last.memory {
                let delta = format_delta(last_mem.current_mb, memory.current_mb);
                println!("{}│{}    Delta:    {}", dim, reset, delta);
            }
        }
        println!("{}│{}", dim, reset);
    }

    // Queue
    if let Some(ref queue) = health.queue {
        println!("{}│{}  {}Queue:{}", dim, reset, bold, reset);
        println!("{}│{}    Depth:         {}", dim, reset, queue.depth);
        println!("{}│{}    Rate:          {:.2} events/sec", dim, reset, queue.rate_per_sec);
        println!("{}│{}    Total Processed: {}", dim, reset, queue.total_processed);

        if let Some(last) = last {
            if let Some(ref last_q) = last.queue {
                let depth_delta = format_count_delta(last_q.depth as u64, queue.depth as u64);
                let proc_delta = format_count_delta(last_q.total_processed, queue.total_processed);
                println!("{}│{}    Depth Delta:    {}", dim, reset, depth_delta);
                println!("{}│{}    Processed Delta: {}", dim, reset, proc_delta);
            }
        }
        println!("{}│{}", dim, reset);
    }

    // Capabilities
    println!("{}│{}  {}Capabilities:{}", dim, reset, bold, reset);
    println!("{}│{}    Active:   {}", dim, reset, health.capabilities_active);
    println!("{}│{}    Degraded: {}", dim, reset, health.capabilities_degraded);

    Ok(())
}

/// Fetch health metrics from daemon via RPC
async fn fetch_health_metrics() -> Result<HealthSnapshot> {
    use tokio::time::{timeout, Duration};

    // Connect with timeout
    let stream = match timeout(
        Duration::from_secs(2),
        UnixStream::connect(SOCKET_PATH),
    )
    .await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            anyhow::bail!(
                "Failed to connect to annad (socket: {})\nError: {}\nIs the daemon running? Try: sudo systemctl status annad",
                SOCKET_PATH,
                e
            );
        }
        Err(_) => {
            anyhow::bail!("Timeout connecting to daemon");
        }
    };

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send request
    let request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "get_health_metrics".to_string(),
        params: None,
        id: 1,
    };

    let json = serde_json::to_string(&request)?;

    // Write with timeout
    match timeout(Duration::from_secs(2), async {
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await
    })
    .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => anyhow::bail!("Write error: {}", e),
        Err(_) => anyhow::bail!("Timeout writing to daemon"),
    }

    // Read response with timeout
    let mut line = String::new();
    match timeout(Duration::from_secs(5), reader.read_line(&mut line)).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => anyhow::bail!("Read error: {}", e),
        Err(_) => anyhow::bail!("Timeout reading from daemon"),
    }

    let response: RpcResponse = serde_json::from_str(&line)?;

    if let Some(error) = response.error {
        anyhow::bail!("RPC error {}: {}", error.code, error.message);
    }

    let result = response.result.context("No result in response")?;
    let health: HealthSnapshot = serde_json::from_value(result)?;

    Ok(health)
}

/// Print health metrics with TUI formatting
fn print_health_tui(health: &HealthSnapshot) -> Result<()> {
    println!("\n╭─ Anna Daemon Health ─────────────────────────────────────────────");
    println!("│");

    // Overall status
    let (status_icon, status_text, status_color) = match health.status {
        HealthStatus::Healthy => ("✓", "Healthy", "\x1b[32m"),        // Green
        HealthStatus::Warning => ("⚠", "Warning", "\x1b[33m"),        // Yellow
        HealthStatus::Critical => ("✗", "Critical", "\x1b[31m"),      // Red
        HealthStatus::Unknown => ("?", "Unknown", "\x1b[37m"),        // Gray
    };

    println!("│  Status:   {}{} {}\x1b[0m", status_color, status_icon, status_text);

    // Uptime
    let uptime_hours = health.uptime_sec / 3600;
    let uptime_mins = (health.uptime_sec % 3600) / 60;
    println!("│  Uptime:   {}h {}m", uptime_hours, uptime_mins);

    println!("│");

    // RPC Latency
    if let Some(ref latency) = health.rpc_latency {
        println!("│  RPC Latency:");
        println!("│    Average:     {:>6.1} ms", latency.avg_ms);
        println!("│    p50:         {:>6} ms", latency.p50_ms);
        println!("│    p95:         {:>6} ms", latency.p95_ms);
        println!("│    p99:         {:>6} ms", latency.p99_ms);
        println!("│    Range:       {:>6}-{} ms", latency.min_ms, latency.max_ms);
        println!("│    Samples:     {}", latency.sample_count);

        // Visual indicator for p99
        let p99_status = if latency.p99_ms < 100 {
            "✓ Excellent"
        } else if latency.p99_ms < 500 {
            "⚠ Acceptable"
        } else {
            "✗ Slow"
        };
        println!("│    Health:      {}", p99_status);
    } else {
        println!("│  RPC Latency:  No samples yet");
    }

    println!("│");

    // Memory Usage
    if let Some(ref memory) = health.memory {
        println!("│  Memory Usage:");
        println!("│    Current:     {:>6.1} MB", memory.current_mb);
        println!("│    Peak:        {:>6.1} MB", memory.peak_mb);
        println!("│    Limit:       {:>6} MB", memory.limit_mb);
        println!("│    VmSize:      {:>6.1} MB", memory.vmsize_mb);
        println!("│    Threads:     {}", memory.threads);

        // Progress bar
        let mem_pct = (memory.current_mb / memory.limit_mb as f64 * 100.0) as f32;
        let bar = progress_bar(mem_pct, 20);
        let bar_color = if mem_pct < 70.0 {
            "\x1b[32m" // Green
        } else if mem_pct < 85.0 {
            "\x1b[33m" // Yellow
        } else {
            "\x1b[31m" // Red
        };
        println!("│    Usage:       {}{}\x1b[0m {:>5.1}%", bar_color, bar, mem_pct);
    } else {
        println!("│  Memory Usage:  Not available");
    }

    println!("│");

    // Queue Health
    if let Some(ref queue) = health.queue {
        println!("│  Event Queue:");
        println!("│    Depth:       {}", queue.depth);
        println!("│    Processed:   {}", queue.total_processed);

        let queue_status = if queue.depth < 50 {
            "✓ Normal"
        } else if queue.depth < 100 {
            "⚠ Elevated"
        } else {
            "✗ Backlog"
        };
        println!("│    Health:      {}", queue_status);
    } else {
        println!("│  Event Queue:  Not available");
    }

    println!("│");

    // Capabilities
    let total_caps = health.capabilities_active + health.capabilities_degraded;
    println!("│  Capabilities:");
    println!("│    Active:      {} / {}", health.capabilities_active, total_caps);
    if health.capabilities_degraded > 0 {
        println!("│    Degraded:    {} ⚠", health.capabilities_degraded);
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────────────────");
    println!();

    // Recommendations
    if matches!(health.status, HealthStatus::Warning | HealthStatus::Critical) {
        println!("Recommendations:");

        if let Some(ref latency) = health.rpc_latency {
            if latency.p99_ms > 500 {
                println!("  • High RPC latency detected - check system load");
            }
        }

        if let Some(ref memory) = health.memory {
            let mem_pct = (memory.current_mb / memory.limit_mb as f64 * 100.0) as f32;
            if mem_pct > 85.0 {
                println!("  • Memory usage high - daemon may be killed by systemd");
                println!("    Consider increasing MemoryMax in annad.service");
            }
        }

        if let Some(ref queue) = health.queue {
            if queue.depth > 100 {
                println!("  • Event queue backlog detected - events may be delayed");
            }
        }

        if health.capabilities_degraded > 0 {
            println!("  • Some capabilities degraded - run: annactl capabilities");
        }

        println!();
    }

    Ok(())
}

/// Draw a Unicode progress bar
fn progress_bar(pct: f32, width: usize) -> String {
    let filled = (pct / 100.0 * width as f32) as usize;
    let filled = filled.min(width);

    "█".repeat(filled) + &"░".repeat(width - filled)
}
