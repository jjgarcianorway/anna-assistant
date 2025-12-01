//! Anna CLI (annactl) v5.5.0 - Telemetry Reset
//!
//! Pure system intelligence - no LLM, no Q&A.
//!
//! Commands:
//! - annactl status               Anna's health and daemon status
//! - annactl stats                Daemon activity statistics
//! - annactl knowledge            Overview of what Anna knows
//! - annactl knowledge stats      Coverage and quality statistics
//! - annactl knowledge <category> List objects in category (editors, shells, etc)
//! - annactl knowledge <name>     Full object profile
//! - annactl reset                Clear all data and restart
//! - annactl version              Show version and install info
//! - annactl cpu                  CPU info and top processes
//! - annactl ram                  Memory usage summary
//! - annactl disk                 Disk usage summary
//! - annactl help                 Show help info

mod commands;

use anna_common::AnnaConfig;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

/// v5.5.0: Known category names for `knowledge <category>` queries
const CATEGORY_NAMES: [&str; 8] = [
    "editors", "terminals", "shells", "browsers",
    "compositors", "wms", "tools", "services"
];

fn is_category_query(name: &str) -> bool {
    let lower = name.to_lowercase();
    CATEGORY_NAMES.contains(&lower.as_str())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annactl=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer().without_time().with_target(false))
        .init();

    let _config = AnnaConfig::load();

    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [] => run_help(),
        [cmd] if cmd.eq_ignore_ascii_case("status") => commands::status::run().await,
        [cmd] if cmd.eq_ignore_ascii_case("stats") => commands::stats::run().await,
        [cmd] if cmd.eq_ignore_ascii_case("knowledge") => commands::knowledge::run().await,
        [cmd, sub] if cmd.eq_ignore_ascii_case("knowledge") && sub.eq_ignore_ascii_case("stats") => {
            commands::knowledge_stats::run().await
        }
        // v5.5.0: Category queries like `knowledge editors`
        [cmd, name] if cmd.eq_ignore_ascii_case("knowledge") && is_category_query(name) => {
            commands::knowledge_category::run(name).await
        }
        [cmd, name] if cmd.eq_ignore_ascii_case("knowledge") => {
            commands::knowledge_detail::run(name).await
        }
        [cmd] if cmd.eq_ignore_ascii_case("reset") => run_reset().await,
        // v5.5.0: System info commands
        [cmd] if cmd.eq_ignore_ascii_case("cpu") => run_cpu_info().await,
        [cmd] if cmd.eq_ignore_ascii_case("ram") || cmd.eq_ignore_ascii_case("memory") => run_ram_info().await,
        [cmd] if cmd.eq_ignore_ascii_case("disk") => run_disk_info().await,
        // Handle quoted strings like "cpu info" as aliases
        [quoted] if quoted.to_lowercase().contains("cpu") => run_cpu_info().await,
        [quoted] if quoted.to_lowercase().contains("ram") || quoted.to_lowercase().contains("memory") => run_ram_info().await,
        [quoted] if quoted.to_lowercase().contains("disk") => run_disk_info().await,
        [flag] if flag == "-V" || flag == "--version" || flag.eq_ignore_ascii_case("version") => {
            commands::version::run()
        }
        [flag] if flag == "-h" || flag == "--help" || flag.eq_ignore_ascii_case("help") => {
            run_help()
        }
        _ => run_unknown_command(&args),
    }
}

// ============================================================================
// Help Command
// ============================================================================

fn run_help() -> Result<()> {
    println!();
    println!("{}", format!("ANNA v{} - Telemetry Core", VERSION).bold());
    println!("{}", THIN_SEP);
    println!();
    println!("  Anna is a system intelligence daemon that tracks every");
    println!("  executable, package, and service on your Linux system.");
    println!();
    println!("{}", "COMMANDS:".bold());
    println!("  annactl status              Daemon health and status");
    println!("  annactl stats               Daemon activity statistics");
    println!("  annactl knowledge           Knowledge overview by category");
    println!("  annactl knowledge stats     Coverage and quality statistics");
    println!("  annactl knowledge <name>    Full object profile");
    println!("  annactl reset               Clear all data and restart");
    println!("  annactl version             Show version info");
    println!("  annactl cpu                 CPU info and top processes");
    println!("  annactl ram                 Memory usage summary");
    println!("  annactl disk                Disk usage summary");
    println!("  annactl help                Show this help");
    println!();
    println!("{}", "CATEGORY QUERIES:".bold());
    println!("  annactl knowledge editors   List installed editors");
    println!("  annactl knowledge shells    List installed shells");
    println!("  annactl knowledge terminals List installed terminals");
    println!("  annactl knowledge browsers  List installed browsers");
    println!("  annactl knowledge services  List indexed services");
    println!();
    println!("{}", "WHAT ANNA TRACKS:".bold());
    println!("  - Commands on PATH (binaries, scripts)");
    println!("  - Packages with version history");
    println!("  - Systemd services (active/enabled/failed)");
    println!("  - Process activity (CPU/memory usage)");
    println!("  - Errors from system logs");
    println!("  - Intrusion detection patterns");
    println!();
    Ok(())
}

// ============================================================================
// Reset Command v5.5.0 - Proper Error Handling
// ============================================================================

async fn run_reset() -> Result<()> {
    println!();
    println!("{}", "[RESET]".yellow());
    println!();

    let mut errors_occurred = false;
    let mut failed_paths: Vec<String> = Vec::new();

    // v5.5.0: Complete wipe of ALL anna data
    let dirs_to_clear = [
        "/var/lib/anna/knowledge",
        "/var/lib/anna/telemetry",
    ];

    for dir in &dirs_to_clear {
        if std::path::Path::new(dir).exists() {
            match std::fs::remove_dir_all(dir) {
                Ok(_) => {
                    // Recreate the empty directory
                    let _ = std::fs::create_dir_all(dir);
                    println!("  {} Cleared: {}", "+".green(), dir);
                }
                Err(e) => {
                    println!("  {} Failed to clear {}: {}", "!".red(), dir, e);
                    failed_paths.push(format!("{}: {}", dir, e));
                    errors_occurred = true;
                }
            }
        }
    }

    // Clear ALL state files
    let files_to_clear = [
        "/var/lib/anna/telemetry_state.json",
        "/var/lib/anna/log_scan_state.json",
        "/var/lib/anna/error_index.json",
        "/var/lib/anna/intrusion_index.json",
        "/var/lib/anna/service_index.json",
        "/var/lib/anna/knowledge/knowledge_v5.json",
        "/var/lib/anna/knowledge/telemetry_v5.json",
        "/var/lib/anna/knowledge/inventory_progress.json",
        "/var/lib/anna/knowledge/log_scan_state.json",
    ];

    for file in &files_to_clear {
        if std::path::Path::new(file).exists() {
            match std::fs::remove_file(file) {
                Ok(_) => println!("  {} Removed: {}", "+".green(), file),
                Err(e) => {
                    println!("  {} Failed to remove {}: {}", "!".red(), file, e);
                    failed_paths.push(format!("{}: {}", file, e));
                    errors_occurred = true;
                }
            }
        }
    }

    println!();

    // v5.5.0: Report status clearly
    if errors_occurred {
        println!("  {}  Reset partially failed.", "WARN".yellow());
        println!();
        println!("  The following paths could not be cleared:");
        for path in &failed_paths {
            println!("    - {}", path.red());
        }
        println!();
        println!("  This usually means permission issues.");
        println!("  Try running: sudo annactl reset");
        println!();
        // Return non-zero exit code for scripts
        std::process::exit(1);
    } else {
        println!("  {}  Reset complete.", "OK".green());
        println!();
        println!("  Daemon will rebuild inventory on restart.");
        println!("  Use 'annactl status' to monitor progress.");
        println!();
    }

    Ok(())
}

// ============================================================================
// System Info Commands v5.5.0
// ============================================================================

async fn run_cpu_info() -> Result<()> {
    println!();
    println!("{}", "  CPU Information".bold());
    println!("{}", THIN_SEP);
    println!();

    // CPU model
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("model name") {
                if let Some(model) = line.split(':').nth(1) {
                    println!("{}", "[MODEL]".cyan());
                    println!("  {}", model.trim());
                    println!();
                    break;
                }
            }
        }
    }

    // Core count
    let core_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    println!("{}", "[CORES]".cyan());
    println!("  {} logical cores", core_count);
    println!();

    // Load averages
    if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = loadavg.split_whitespace().collect();
        if parts.len() >= 3 {
            println!("{}", "[LOAD AVERAGE]".cyan());
            println!("  1m:  {}", parts[0]);
            println!("  5m:  {}", parts[1]);
            println!("  15m: {}", parts[2]);
            println!();
        }
    }

    // Top processes by CPU (from ps)
    println!("{}", "[TOP PROCESSES BY CPU]".cyan());
    let output = std::process::Command::new("ps")
        .args(["axo", "pid,%cpu,%mem,comm", "--sort=-%cpu"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for (i, line) in stdout.lines().take(6).enumerate() {
            if i == 0 {
                println!("  {}", line.dimmed());
            } else {
                println!("  {}", line);
            }
        }
    } else {
        println!("  (unable to fetch process list)");
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

async fn run_ram_info() -> Result<()> {
    println!();
    println!("{}", "  Memory Information".bold());
    println!("{}", THIN_SEP);
    println!();

    // Parse /proc/meminfo
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        let mut total_kb: u64 = 0;
        let mut free_kb: u64 = 0;
        let mut available_kb: u64 = 0;
        let mut buffers_kb: u64 = 0;
        let mut cached_kb: u64 = 0;
        let mut swap_total_kb: u64 = 0;
        let mut swap_free_kb: u64 = 0;

        for line in meminfo.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let value = parts[1].parse::<u64>().unwrap_or(0);
                match parts[0] {
                    "MemTotal:" => total_kb = value,
                    "MemFree:" => free_kb = value,
                    "MemAvailable:" => available_kb = value,
                    "Buffers:" => buffers_kb = value,
                    "Cached:" => cached_kb = value,
                    "SwapTotal:" => swap_total_kb = value,
                    "SwapFree:" => swap_free_kb = value,
                    _ => {}
                }
            }
        }

        println!("{}", "[MEMORY]".cyan());
        println!("  Total:     {}", format_kb(total_kb));
        println!("  Available: {}", format_kb(available_kb));
        println!("  Free:      {}", format_kb(free_kb));
        println!("  Buffers:   {}", format_kb(buffers_kb));
        println!("  Cached:    {}", format_kb(cached_kb));

        if total_kb > 0 {
            let used_kb = total_kb.saturating_sub(available_kb);
            let percent = (used_kb as f64 / total_kb as f64 * 100.0) as u64;
            println!("  Used:      {}%", percent);
        }
        println!();

        if swap_total_kb > 0 {
            println!("{}", "[SWAP]".cyan());
            println!("  Total:     {}", format_kb(swap_total_kb));
            println!("  Free:      {}", format_kb(swap_free_kb));
            let swap_used = swap_total_kb.saturating_sub(swap_free_kb);
            let swap_percent = (swap_used as f64 / swap_total_kb as f64 * 100.0) as u64;
            println!("  Used:      {}%", swap_percent);
            println!();
        }
    }

    // Top processes by memory
    println!("{}", "[TOP PROCESSES BY MEMORY]".cyan());
    let output = std::process::Command::new("ps")
        .args(["axo", "pid,%mem,%cpu,comm,rss", "--sort=-rss"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for (i, line) in stdout.lines().take(6).enumerate() {
            if i == 0 {
                println!("  {}", line.dimmed());
            } else {
                println!("  {}", line);
            }
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

async fn run_disk_info() -> Result<()> {
    println!();
    println!("{}", "  Disk Information".bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[FILESYSTEMS]".cyan());

    let output = std::process::Command::new("df")
        .args(["-h", "--output=source,size,used,avail,pcent,target"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for (i, line) in stdout.lines().enumerate() {
            // Skip tmpfs and other virtual filesystems
            if line.contains("tmpfs") || line.contains("devtmpfs") ||
               line.contains("efivarfs") || line.contains("/run") ||
               line.contains("/dev/shm") {
                continue;
            }
            if i == 0 {
                println!("  {}", line.dimmed());
            } else {
                println!("  {}", line);
            }
        }
    } else {
        println!("  (unable to fetch disk info)");
    }

    println!();

    // Block devices
    println!("{}", "[BLOCK DEVICES]".cyan());
    let output = std::process::Command::new("lsblk")
        .args(["-o", "NAME,SIZE,TYPE,MOUNTPOINT", "-d"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for (i, line) in stdout.lines().take(10).enumerate() {
            if i == 0 {
                println!("  {}", line.dimmed());
            } else {
                println!("  {}", line);
            }
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

fn format_kb(kb: u64) -> String {
    if kb >= 1048576 {
        format!("{:.1} GB", kb as f64 / 1048576.0)
    } else if kb >= 1024 {
        format!("{:.1} MB", kb as f64 / 1024.0)
    } else {
        format!("{} KB", kb)
    }
}

// ============================================================================
// Unknown Command
// ============================================================================

fn run_unknown_command(args: &[String]) -> Result<()> {
    println!();
    println!("{}", "[UNKNOWN COMMAND]".yellow());
    println!();
    println!("  '{}' is not a recognized command.", args.join(" "));
    println!();
    println!("  Run 'annactl help' for available commands.");
    println!();
    Ok(())
}
