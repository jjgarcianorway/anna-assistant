//! Status Command v7.37.0 - Auto-Update & Instrumentation Engine
//!
//! v7.37.0: Functional auto-update and auto-install with explicit clean statements
//! - Auto-update scheduler shows real timestamps (never shows "never" after first run)
//! - Auto-install shows installed tools with dates, scopes, versions
//! - Explicit clean statements in all logs sections
//! - Internal paths created on daemon start
//!
//! v7.31.0: Concrete telemetry readiness, correct percent formatting, truthful updates
//!
//! Sections:
//! - [VERSION]             Single unified Anna version
//! - [DAEMON]              State, uptime, PID, restarts
//! - [HEALTH]              Overall health status
//! - [BOOT SNAPSHOT]       Boot times, uptime, incident summary (v7.23.0)
//! - [DATA]                Software and hardware data readiness (v7.29.0, renamed from KDB)
//! - [INVENTORY]           What Anna has indexed + drift indicator (v7.23.0)
//! - [TELEMETRY]           Real telemetry with top CPU/memory and trends
//! - [TELEMETRY SUMMARY]   Services with notable trends (v7.20.0)
//! - [LOG SUMMARY]         Components with new patterns since baseline (v7.20.0)
//! - [RESOURCE HOTSPOTS]   Top resource consumers with health notes
//! - [HOTSPOTS]            Compact cross-reference of sw/hw hotspots (v7.24.0)
//! - [ATTACHMENTS]         Connected peripherals: USB, Bluetooth, Thunderbolt (v7.25.0)
//! - [INSTRUMENTATION]     Tools installed by Anna and available tools (v7.26.0)
//! - [RECENT CHANGES]      Last 5 system changes from journal (v7.18.0)
//! - [UPDATES]             Auto-update schedule and last result
//! - [PATHS]               Config, data, logs, docs paths (v7.12.0: local docs detection)
//! - [INTERNAL ERRORS]     Anna's own pipeline errors
//! - [ALERTS]              Hardware alerts from health checks
//! - [TOPOLOGY HINTS]      High-impact services and driver stacks (v7.19.0)
//! - [ANNA TOOLCHAIN]      Diagnostic tool readiness (v7.22.0)
//! - [ANNA NEEDS]          Missing tools and docs
//!
//! v7.27.0: Simplified [INSTRUMENTATION], status Anna-only (no host-wide journal spam)
//! v7.24.0: [HOTSPOTS] compact cross-reference section
//! v7.23.0: [BOOT SNAPSHOT] with incidents, [INVENTORY] with drift indicator
//! v7.22.0: [ANNA TOOLCHAIN] for diagnostic tool tracking
//! v7.20.0: [TELEMETRY SUMMARY] + [LOG SUMMARY] with deterministic trend labels
//! v7.19.0: [TOPOLOGY HINTS] shows services with many deps and multi-stack drivers
//! v7.18.0: [LAST BOOT] shows boot health, [RECENT CHANGES] shows system changes
//! v7.12.0: [PATHS] now shows local docs status
//! NO journalctl system errors. NO host-wide log counts.

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

use anna_common::config::{AnnaConfig, UpdateState, SYSTEM_CONFIG_DIR, DATA_DIR};
use anna_common::format_duration_secs;
use anna_common::{TelemetryDb, DataStatus, WINDOW_24H, format_bytes_human, get_logical_cores};
use anna_common::grounded::health::{collect_hardware_alerts, HealthStatus};
use anna_common::grounded::network::get_network_summary;
use anna_common::{AnnaNeeds, NeedStatus};
use anna_common::{OpsLogReader, INTERNAL_DIR, OPS_LOG_FILE};
// v7.18.0: Change journal and boot timeline
use anna_common::{get_recent_changes, get_current_boot_summary};
// v7.19.0: Service topology hints
use anna_common::grounded::service_topology::{get_high_impact_services, get_gpu_driver_stacks};
// v7.20.0: Telemetry trends and log baselines
use anna_common::{get_process_trends, TrendDirection, get_components_with_new_patterns};
// v7.21.0: Topology for DATA section
use anna_common::topology_map::build_hardware_topology;
// v7.22.0: Toolchain hygiene
use anna_common::toolchain::{check_toolchain, ToolCategory};
// v7.23.0: Boot snapshot and inventory drift
use anna_common::boot_snapshot::BootSnapshot;
use anna_common::inventory_drift::DriftSummary;
// v7.24.0: Hotspots
use anna_common::hotspots::{get_software_hotspots, get_hardware_hotspots, format_status_hotspots_section};
// v7.25.0: Peripherals
use anna_common::grounded::peripherals::{
    get_usb_summary, get_bluetooth_summary, get_thunderbolt_summary, BluetoothState,
};
// v7.27.0: Instrumentation (simplified per spec)
use anna_common::InstrumentationManifest;
// v7.37.0: Chrono for duration calculation
use chrono;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

// Sync is considered stale after 5 minutes
const STALE_THRESHOLD_SECS: u64 = 300;

/// Run the status command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Status".bold());
    println!("{}", THIN_SEP);
    println!();

    // [VERSION]
    print_version_section();

    // [DAEMON]
    let daemon_stats = get_daemon_stats().await;
    print_daemon_section(&daemon_stats);

    // [HEALTH]
    print_health_section(&daemon_stats);

    // [BOOT SNAPSHOT] - v7.23.0: Boot times, uptime, incidents
    print_boot_snapshot_section();

    // [DATA] - v7.29.0: Data readiness (renamed from KDB)
    print_kdb_section(&daemon_stats);

    // [INVENTORY]
    print_inventory_section(&daemon_stats);

    // [TELEMETRY] - v7.9.0: Unified section with trends
    print_telemetry_section_v79();

    // [TELEMETRY SUMMARY] - v7.20.0: Notable trend changes
    print_telemetry_summary_section();

    // [LOG SUMMARY] - v7.20.0: Components with new patterns since baseline
    print_log_summary_section();

    // [RESOURCE HOTSPOTS] - v7.11.0: Top consumers with health notes
    print_resource_hotspots_section();

    // [HOTSPOTS] - v7.24.0: Compact cross-reference
    print_hotspots_xref_section();

    // [ATTACHMENTS] - v7.25.0: Connected peripherals
    print_attachments_section();

    // [INSTRUMENTATION] - v7.26.0: Auto-installed tools
    print_instrumentation_section();

    // [RECENT CHANGES] - v7.18.0: Change journal
    print_recent_changes_section();

    // [UPDATES]
    print_updates_section();

    // [PATHS]
    print_paths_section();

    // [INTERNAL ERRORS]
    print_internal_errors_section(&daemon_stats);

    // [ALERTS]
    print_alerts_section();

    // [TOPOLOGY HINTS] - v7.19.0
    print_topology_hints_section();

    // [ANNA NEEDS] - v7.6.0
    print_anna_needs_section();

    // [ANNA TOOLCHAIN] - v7.22.0
    print_toolchain_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_version_section() {
    println!("{}", "[VERSION]".cyan());
    println!("  Anna:       v{}", VERSION);
    println!();
}

fn print_daemon_section(stats: &Option<DaemonStats>) {
    println!("{}", "[DAEMON]".cyan());

    match stats {
        Some(s) => {
            println!("  Status:     {}", "running".green());
            println!("  Uptime:     {}", format_duration_secs(s.uptime_secs));
            println!("  PID:        {}", s.pid);
            println!("  Restarts:   {} (last 24h)", s.restarts_24h);
        }
        None => {
            println!("  Status:     {}", "stopped".red());
            println!("  Uptime:     -");
            println!("  PID:        -");
            println!("  Restarts:   -");
        }
    }

    println!();
}

/// Health signal derived from internal metrics (v7.3.0)
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Unknown is for future use
enum HealthSignal {
    /// Everything nominal
    Ok,
    /// Warning conditions (degraded but working)
    Warning(String),
    /// Critical conditions (action needed)
    Critical(String),
    /// Cannot determine (daemon not running)
    Unknown,
}

fn print_health_section(stats: &Option<DaemonStats>) {
    println!("{}", "[HEALTH]".cyan());

    match stats {
        Some(s) => {
            // Compute health signals
            let mut warnings = Vec::new();
            let mut criticals = Vec::new();

            // 1. Daemon restarts (many restarts = unstable)
            if s.restarts_24h >= 5 {
                criticals.push(format!("{} restarts in 24h", s.restarts_24h));
            } else if s.restarts_24h >= 2 {
                warnings.push(format!("{} restarts in 24h", s.restarts_24h));
            }

            // 2. Crashes (any crash is concerning)
            if s.crashes > 0 {
                criticals.push(format!("{} crashes", s.crashes));
            }

            // 3. Command failures (high rate = integration issue)
            if s.command_failures >= 10 {
                criticals.push(format!("{} command failures", s.command_failures));
            } else if s.command_failures >= 3 {
                warnings.push(format!("{} command failures", s.command_failures));
            }

            // 4. Parse errors (high rate = data quality issue)
            if s.parse_errors >= 20 {
                warnings.push(format!("{} parse errors", s.parse_errors));
            }

            // 5. Sync staleness
            if let Some(secs) = s.last_scan_secs_ago {
                if secs > 600 { // More than 10 minutes
                    warnings.push(format!("sync stale ({}m ago)", secs / 60));
                }
            } else {
                warnings.push("no sync completed".to_string());
            }

            // 6. Telemetry DB health
            match TelemetryDb::open_readonly() {
                Some(db) => {
                    match db.get_data_status() {
                        DataStatus::NoData => warnings.push("no telemetry data".to_string()),
                        DataStatus::NotEnoughData { .. } => warnings.push("limited telemetry".to_string()),
                        _ => {} // OK
                    }
                }
                None => warnings.push("telemetry DB unavailable".to_string()),
            }

            // Determine overall status
            let overall = if !criticals.is_empty() {
                HealthSignal::Critical(criticals.join(", "))
            } else if !warnings.is_empty() {
                HealthSignal::Warning(warnings.join(", "))
            } else {
                HealthSignal::Ok
            };

            // Display overall
            match &overall {
                HealthSignal::Ok => {
                    println!("  Overall:    {} all systems nominal", "✓".green());
                }
                HealthSignal::Warning(reason) => {
                    println!("  Overall:    {} {}", "⚠".yellow(), reason.yellow());
                }
                HealthSignal::Critical(reason) => {
                    println!("  Overall:    {} {}", "✗".red(), reason.red());
                }
                HealthSignal::Unknown => {
                    println!("  Overall:    {}", "-".dimmed());
                }
            }

            // Individual health indicators
            let daemon_health = if s.restarts_24h == 0 && s.crashes == 0 {
                "stable".green().to_string()
            } else if s.crashes > 0 {
                "unstable".red().to_string()
            } else {
                "recovering".yellow().to_string()
            };
            println!("  Daemon:     {}", daemon_health);

            let config = AnnaConfig::load();
            let telemetry_health = if !config.telemetry.enabled {
                "disabled".dimmed().to_string()
            } else {
                match TelemetryDb::open_readonly() {
                    Some(db) => {
                        // v7.29.0: Use window status for consistency with TELEMETRY section
                        let window = db.get_window_status("");
                        if window.w24h_ready {
                            "collecting".green().to_string()
                        } else if window.w1h_ready {
                            "warming up".yellow().to_string()
                        } else {
                            match db.get_data_status() {
                                DataStatus::NotEnoughData { .. } => "starting".yellow().to_string(),
                                DataStatus::NoData | DataStatus::Disabled => "no data".dimmed().to_string(),
                                _ => "starting".yellow().to_string(),
                            }
                        }
                    },
                    None => "unavailable".red().to_string(),
                }
            };
            println!("  Telemetry:  {}", telemetry_health);

            let sync_health = match s.last_scan_secs_ago {
                Some(secs) if secs < STALE_THRESHOLD_SECS => "current".green().to_string(),
                Some(secs) if secs < 600 => "recent".green().to_string(),
                Some(_) => "stale".yellow().to_string(),
                None => "pending".yellow().to_string(),
            };
            println!("  Sync:       {}", sync_health);
        }
        None => {
            println!("  Overall:    {} daemon not running", "✗".red());
            println!("  Daemon:     {}", "stopped".red());
            println!("  Telemetry:  -");
            println!("  Sync:       -");
        }
    }

    println!();
}

/// v7.23.0: [BOOT SNAPSHOT] section - boot times, uptime, incident summary
fn print_boot_snapshot_section() {
    println!("{}", "[BOOT SNAPSHOT]".cyan());

    let snapshot = BootSnapshot::current();

    println!("  Current boot:");
    println!("    started:        {}", snapshot.boot_started.format("%Y-%m-%d %H:%M:%S %Z"));
    println!("    uptime:         {}", snapshot.uptime);

    if let Some(ref anna_start) = snapshot.anna_started {
        println!("    Anna start:     {}", anna_start.format("%Y-%m-%d %H:%M:%S %Z"));
    }
    if let Some(ref anna_uptime) = snapshot.anna_uptime {
        println!("    Anna uptime:    {}", anna_uptime);
    }
    println!();

    if snapshot.incidents.is_empty() {
        println!("  Incidents (current boot):");
        println!("    {}", "none recorded at warning or above".green());
    } else {
        // v7.29.0: Filter incidents to Anna-relevant only (from Anna services)
        let anna_incidents: Vec<_> = snapshot.incidents.iter()
            .filter(|i| i.message.contains("anna") || i.message.contains("annad") ||
                       i.message.contains("annactl") || i.pattern_id.starts_with("anna"))
            .take(10)
            .collect();

        if anna_incidents.is_empty() {
            println!("  Incidents (Anna-specific this boot):");
            println!("    {}", "none recorded".green());
        } else {
            println!("  Incidents (Anna-specific this boot):");
            for incident in &anna_incidents {
                let count_str = if incident.count == 1 {
                    "(1x)".to_string()
                } else {
                    format!("({}x)", incident.count)
                };
                // v7.29.0: No truncation - show full message wrapped
                println!(
                    "    {} {} {}",
                    incident.pattern_id,
                    incident.message,
                    count_str.dimmed()
                );
            }
        }
    }

    println!();
}

/// v7.18.0: [LAST BOOT] section - boot timeline with kernel, duration, failed units (kept for compatibility)
#[allow(dead_code)]
fn print_last_boot_section() {
    println!("{}", "[LAST BOOT]".cyan());

    match get_current_boot_summary() {
        Some(boot) => {
            println!("  Started:    {}", boot.format_time());
            println!("  Kernel:     {}", boot.kernel);

            // Duration to graphical/multi-user target
            if let Some(duration) = boot.duration_to_graphical {
                println!("  Duration:   {:.0}s to {}", duration, "graphical.target".dimmed());
            } else {
                println!("  Duration:   {}", "unknown".dimmed());
            }

            println!();
            println!("  Health:");

            // Failed units
            let failed_str = if boot.failed_units == 0 {
                "0".green().to_string()
            } else {
                boot.failed_units.to_string().red().to_string()
            };
            println!("    Failed units:   {}", failed_str);

            // Services with warnings
            let warn_str = if boot.services_with_warnings == 0 {
                "0".green().to_string()
            } else {
                format!("{} services with warnings", boot.services_with_warnings)
                    .yellow()
                    .to_string()
            };
            println!("    Warnings:       {}", warn_str);

            // Slow units
            if !boot.slow_units.is_empty() {
                let slow_list: Vec<String> = boot.slow_units
                    .iter()
                    .take(3)
                    .map(|u| format!("{} ({:.1}s)", u.name, u.duration_secs))
                    .collect();
                println!("    Slow units:     {}", slow_list.join(", ").yellow());
            }
        }
        None => {
            println!("  {}", "Boot information not available".dimmed());
        }
    }

    println!();
}

/// v7.29.0: [DATA] section - data readiness (renamed from [KDB])
fn print_kdb_section(stats: &Option<DaemonStats>) {
    println!("{}", "[DATA]".cyan());

    match stats {
        Some(s) => {
            // Software data readiness
            println!("  Software:   {} packages, {} commands, {} services",
                s.packages_count,
                s.commands_count,
                s.services_count);

            // Hardware data readiness from topology
            let hw_topology = build_hardware_topology();

            let gpu_count = hw_topology.gpus.len();
            let storage_count = hw_topology.storage.len();
            let net_count = hw_topology.network.len();

            let total_hw = gpu_count + storage_count + net_count;

            if total_hw > 0 {
                let parts: Vec<String> = vec![
                    if gpu_count > 0 { Some(format!("{} GPU{}", gpu_count, if gpu_count > 1 { "s" } else { "" })) } else { None },
                    if storage_count > 0 { Some(format!("{} storage", storage_count)) } else { None },
                    if net_count > 0 { Some(format!("{} network", net_count)) } else { None },
                ].into_iter().flatten().collect();

                println!("  Hardware:   {} devices ({})", total_hw, parts.join(", "));
            } else {
                println!("  Hardware:   {}", "not detected".dimmed());
            }
        }
        None => {
            println!("  Software:   {} (daemon not running)", "-".dimmed());
            println!("  Hardware:   {} (daemon not running)", "-".dimmed());
        }
    }

    println!();
}

/// v7.18.0: [RECENT CHANGES] section - last 5 system changes
fn print_recent_changes_section() {
    println!("{}", "[RECENT CHANGES]".cyan());
    println!("  {}", "(source: pacman.log, change journal)".dimmed());

    let changes = get_recent_changes(5);

    if changes.is_empty() {
        println!("  No recent changes recorded.");
    } else {
        println!("  Last {} events:", changes.len());
        for event in &changes {
            println!("    {}", event.format_short());
        }
    }

    println!();
}

/// v7.11.0: [RESOURCE HOTSPOTS] section with health notes
/// Shows top CPU and RAM consumers with links to recent logs
fn print_resource_hotspots_section() {
    use anna_common::config::AnnaConfig;

    println!("{}", "[RESOURCE HOTSPOTS]".cyan());

    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        println!("  {}", "Telemetry disabled in config.".dimmed());
        println!();
        return;
    }

    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => {
            println!("  {}", "(telemetry DB not available)".dimmed());
            println!();
            return;
        }
    };

    let data_status = db.get_data_status();

    // Need enough data
    if !matches!(data_status, DataStatus::PartialWindow { .. } | DataStatus::Ok { .. }) {
        println!("  Telemetry warming up. Hotspots available after sufficient data collection.");
        println!();
        return;
    }

    // Get CPU and RAM hotspots
    let cpu_hotspot = db.get_cpu_hotspot(WINDOW_24H).ok().flatten();
    let ram_hotspot = db.get_ram_hotspot(WINDOW_24H).ok().flatten();

    if cpu_hotspot.is_none() && ram_hotspot.is_none() {
        println!("  No significant resource consumers detected in last 24h.");
        println!();
        return;
    }

    println!("  {}", "(top resource consumers in last 24h)".dimmed());
    println!();

    // CPU hotspot with health note
    if let Some(cpu) = &cpu_hotspot {
        let context = cpu.context.as_ref().map(|c| format!(", {}", c)).unwrap_or_default();
        println!("  CPU:        {} ({}{})", cpu.name.cyan(), cpu.display, context);

        // Check for related errors in journalctl
        let note = get_health_note_for_process(&cpu.name);
        if let Some(note_text) = note {
            println!("              {}", format!("Note: {}", note_text).yellow());
        }
    }

    // RAM hotspot with health note
    if let Some(ram) = &ram_hotspot {
        println!("  RAM:        {} ({} RSS peak)", ram.name.cyan(), ram.display);

        // Check for related errors in journalctl
        if cpu_hotspot.as_ref().map(|c| &c.name) != Some(&ram.name) {
            let note = get_health_note_for_process(&ram.name);
            if let Some(note_text) = note {
                println!("              {}", format!("Note: {}", note_text).yellow());
            }
        }
    }

    println!();
}

/// Get health note for a process by checking recent journalctl errors
fn get_health_note_for_process(name: &str) -> Option<String> {
    use std::process::Command;

    // Try to find the service unit for this process
    let unit_name = format!("{}.service", name);

    // Check for recent errors in journalctl for this unit
    let output = Command::new("journalctl")
        .args([
            "-u", &unit_name,
            "-p", "err..alert",  // Only errors and above
            "-b",                // Current boot only
            "-n", "1",           // Just check if any exist
            "--no-pager",
            "-q",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let line_count = stdout.lines().count();
            if line_count > 0 {
                return Some(format!("has errors this boot - see `annactl sw {}`", name));
            }
        }
    }

    None
}

fn print_inventory_section(stats: &Option<DaemonStats>) {
    println!("{}", "[INVENTORY]".cyan());

    match stats {
        Some(s) => {
            println!("  Packages:   {}  {}", s.packages_count, "(from pacman -Q)".dimmed());
            println!("  Commands:   {}  {}", s.commands_count, "(from $PATH)".dimmed());
            println!("  Services:   {}  {}", s.services_count, "(from systemctl)".dimmed());

            // v7.13.0: Network summary
            let net_summary = get_network_summary();
            println!("  Network:    {}  {}", net_summary.format_compact(), "(from /sys/class/net)".dimmed());

            // v7.23.0: Sync status with drift indicator
            let drift = DriftSummary::compute();
            let drift_str = drift.format_status_line();
            let sync_color = if drift.has_changes {
                drift_str.yellow().to_string()
            } else {
                drift_str.green().to_string()
            };
            println!("  Sync:       {}", sync_color);
        }
        None => {
            println!("  Packages:   -");
            println!("  Commands:   -");
            println!("  Services:   -");
            println!("  Network:    -");
            println!("  Sync:       {}", "(daemon not running)".dimmed());
        }
    }

    println!();
}


/// v7.31.0: Unified [TELEMETRY] section with concrete readiness
/// Shows: Readiness status, Windows available, Top CPU/memory identities with trends
fn print_telemetry_section_v79() {
    use anna_common::config::AnnaConfig;
    use anna_common::{TelemetryReadiness, get_logical_cpu_count};

    println!("{}", "[TELEMETRY]".cyan());

    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        println!("  {}", "Telemetry disabled in config (/etc/anna/config.toml).".dimmed());
        println!();
        return;
    }

    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => {
            println!("  {}", "(telemetry DB not available)".dimmed());
            println!();
            return;
        }
    };

    // v7.31.0: Get concrete readiness status
    let readiness = match db.get_telemetry_readiness() {
        Ok(r) => r,
        Err(_) => {
            println!("  {}", "(error reading telemetry status)".dimmed());
            println!();
            return;
        }
    };

    // Show readiness status
    println!("  Status: {}", readiness.format());

    // If not ready for any window, show what we're collecting
    if matches!(readiness, TelemetryReadiness::NoData | TelemetryReadiness::Collecting { .. }) {
        println!();
        return;
    }

    // Show available windows
    let availability = anna_common::WindowAvailability::from_readiness(&readiness);
    println!("  Windows: {}", availability.format_available());
    println!();

    // Top CPU identities (max 3)
    let logical_cores = get_logical_cpu_count();
    if let Ok(top_cpu) = db.top_cpu_with_trend(3) {
        if !top_cpu.is_empty() {
            let max_percent = logical_cores * 100;
            println!("  Top CPU identities (0-{}% for {} logical cores):",
                max_percent, logical_cores);
            for entry in &top_cpu {
                // v7.31.0: Only show trend if 7d data is available
                let trend_str = if availability.w7d {
                    match entry.cpu_trend {
                        Some(trend) => format!("  {}", trend.format_vs_7d().dimmed()),
                        None => String::new(),
                    }
                } else {
                    String::new()
                };
                println!("    {:<16} avg {:.1}%, peak {:.1}%{}",
                    entry.name.cyan(),
                    entry.avg_cpu_percent,
                    entry.peak_cpu_percent,
                    trend_str);
            }
            println!();
        }
    }

    // Top memory identities (max 3)
    if let Ok(top_mem) = db.top_memory_with_trend(3) {
        if !top_mem.is_empty() {
            println!("  Top memory identities:");
            for entry in &top_mem {
                // v7.31.0: Only show trend if 7d data is available
                let trend_str = if availability.w7d {
                    match entry.memory_trend {
                        Some(trend) => format!("  {}", trend.format_vs_7d().dimmed()),
                        None => String::new(),
                    }
                } else {
                    String::new()
                };
                println!("    {:<16} avg {}, peak {}{}",
                    entry.name.cyan(),
                    format_bytes_human(entry.avg_mem_bytes),
                    format_bytes_human(entry.peak_mem_bytes),
                    trend_str);
            }
            println!();
        }
    }
}

/// v7.20.0: [TELEMETRY SUMMARY] section - show services with notable trends
fn print_telemetry_summary_section() {
    use anna_common::config::AnnaConfig;

    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        return;  // Skip if telemetry disabled (already shown in [TELEMETRY])
    }

    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => return,
    };

    // Get identities with notable trends
    let mut notable: Vec<(String, TrendDirection, &'static str)> = Vec::new();

    // Get top CPU and memory identities and check their trends
    if let Ok(top_cpu) = db.top_cpu_with_trend(10) {
        for entry in &top_cpu {
            let trends = get_process_trends(&entry.name);

            // Check CPU trend - only include increasing trends
            if trends.cpu_trend_24h_vs_7d.is_increasing()
                && !matches!(trends.cpu_trend_24h_vs_7d, TrendDirection::Stable | TrendDirection::InsufficientData)
            {
                notable.push((entry.name.clone(), trends.cpu_trend_24h_vs_7d, "CPU"));
            }
        }
    }

    if let Ok(top_mem) = db.top_memory_with_trend(10) {
        for entry in &top_mem {
            let trends = get_process_trends(&entry.name);

            // Check memory trend - only include increasing trends
            if trends.mem_trend_24h_vs_7d.is_increasing()
                && !matches!(trends.mem_trend_24h_vs_7d, TrendDirection::Stable | TrendDirection::InsufficientData)
            {
                // Avoid duplicates from CPU list
                if !notable.iter().any(|(n, _, r)| n == &entry.name && *r == "memory") {
                    notable.push((entry.name.clone(), trends.mem_trend_24h_vs_7d, "memory"));
                }
            }
        }
    }

    // Only show section if there are notable trends
    if notable.is_empty() {
        return;
    }

    println!("{}", "[TELEMETRY SUMMARY]".cyan());
    println!("  {}", "(services with increasing resource usage 24h vs 7d)".dimmed());
    println!();

    // Sort by trend severity (MuchHigher first)
    notable.sort_by(|a, b| {
        let a_severity = match a.1 {
            TrendDirection::MuchHigher => 3,
            TrendDirection::Higher => 2,
            TrendDirection::SlightlyHigher => 1,
            _ => 0,
        };
        let b_severity = match b.1 {
            TrendDirection::MuchHigher => 3,
            TrendDirection::Higher => 2,
            TrendDirection::SlightlyHigher => 1,
            _ => 0,
        };
        b_severity.cmp(&a_severity)
    });

    for (name, trend, resource) in notable.iter().take(5) {
        let label = trend.label();
        let colored_label = match label {
            "much higher" => label.red().to_string(),
            "higher" => label.yellow().to_string(),
            _ => label.to_string(),
        };
        println!("  {:<16} {} {}", name.cyan(), resource, colored_label);
    }

    if notable.len() > 5 {
        println!("  (and {} more with notable trends)", notable.len() - 5);
    }

    println!();
}

/// v7.20.0: [LOG SUMMARY] section - components with new patterns since baseline
fn print_log_summary_section() {
    let components_with_new = get_components_with_new_patterns();

    // Only show if there are components with new patterns
    if components_with_new.is_empty() {
        return;
    }

    println!("{}", "[LOG SUMMARY]".cyan());
    println!("  {}", "(components with new patterns since baseline)".dimmed());
    println!();

    for (component, new_count) in components_with_new.iter().take(5) {
        let count_colored = if *new_count > 5 {
            new_count.to_string().red().to_string()
        } else {
            new_count.to_string().yellow().to_string()
        };
        println!("  {:<20} {} new patterns", component.cyan(), count_colored);
    }

    if components_with_new.len() > 5 {
        println!("  (and {} more components)", components_with_new.len() - 5);
    }

    println!();
}

fn print_updates_section() {
    println!("{}", "[UPDATES]".cyan());

    let state = UpdateState::load();

    // v7.34.0: Check if daemon is running for truthful status
    let daemon_running = is_daemon_running();

    // Mode from state (v7.34.0: state is authoritative)
    println!("  Mode:       {}", state.format_mode());

    // Target: what is being checked
    println!("  Target:     Anna releases (GitHub)");

    // Interval
    println!("  Interval:   {}", state.format_interval());

    // Last check - v7.37.0: show "not yet" instead of "never" when daemon running
    use anna_common::config::UpdateMode;
    let last_check_str = if state.last_check_at == 0 {
        if daemon_running && state.mode == UpdateMode::Auto {
            // Daemon running but never checked - first check pending
            "not yet (first check pending)".to_string()
        } else if !daemon_running {
            "never (daemon not running)".to_string()
        } else {
            "never".to_string()
        }
    } else {
        state.format_last_check()
    };
    println!("  Last check: {}", last_check_str);

    // Result
    println!("  Result:     {}", state.format_last_result());

    // Next check - v7.37.0: show proper status based on daemon state
    let next_str = match state.mode {
        UpdateMode::Auto => {
            if !daemon_running {
                "not running (daemon down)".to_string()
            } else if state.next_check_at == 0 {
                "pending initialization".to_string()
            } else {
                state.format_next_check()
            }
        }
        UpdateMode::Manual => "n/a (manual mode)".to_string(),
    };
    println!("  Next check: {}", next_str);

    // Show error if last check failed
    if let Some(ref error) = state.last_error {
        println!("  Error:      {}", error.dimmed());
    }

    // Show available version if update is available
    if let Some(ref ver) = state.last_checked_version_available {
        if !state.last_checked_version_installed.is_empty() {
            println!("  Available:  {} -> {}",
                state.last_checked_version_installed,
                ver.green());
        }
    }

    println!();
}

/// Check if annad daemon is running
fn is_daemon_running() -> bool {
    std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", "annad"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn print_paths_section() {
    println!("{}", "[PATHS]".cyan());

    // Config path
    let config_path = format!("{}/config.toml", SYSTEM_CONFIG_DIR);
    let config_exists = Path::new(&config_path).exists();
    if config_exists {
        println!("  Config:     {}", config_path);
    } else {
        println!("  Config:     {} {}", config_path, "(missing)".yellow());
    }

    // Data path
    let data_exists = Path::new(DATA_DIR).exists();
    if data_exists {
        println!("  Data:       {}", DATA_DIR);
    } else {
        println!("  Data:       {} {}", DATA_DIR, "(missing)".yellow());
    }

    // Internal dir - v7.37.0: show status clearly
    let internal_exists = Path::new(INTERNAL_DIR).exists();
    if internal_exists {
        println!("  Internal:   {} {}", INTERNAL_DIR, "(ready)".green());
    } else {
        // v7.37.0: If daemon is running, this is a problem
        if is_daemon_running() {
            println!("  Internal:   {} {}", INTERNAL_DIR, "(missing - daemon error)".red());
        } else {
            println!("  Internal:   {} {}", INTERNAL_DIR, "(will create on daemon start)".dimmed());
        }
    }

    // v7.37.0: ops.log status with explicit clean statement
    let ops_status = get_ops_log_status();
    println!("  Ops log:    {}", ops_status);

    // Logs
    println!("  Logs:       {}", "journalctl -u annad".dimmed());

    // v7.12.0: Local docs detection
    let docs_status = get_local_docs_status();
    println!("  Docs:       {}", docs_status);

    println!();
}

/// v7.37.0: Get ops.log status for [PATHS] section with explicit clean statement
fn get_ops_log_status() -> String {
    let reader = OpsLogReader::new();

    if !reader.exists() {
        // v7.37.0: Explicit clean statement
        return format!("{} {}", OPS_LOG_FILE, "(no ops recorded yet, clean)".dimmed());
    }

    let summary = reader.get_summary();
    if summary.total_entries == 0 {
        return format!("{} {}", OPS_LOG_FILE, "(empty, clean)".dimmed());
    }

    format!("{} ({})", OPS_LOG_FILE, summary.format_compact())
}

/// v7.12.0: Detect local documentation sources for config discovery
fn get_local_docs_status() -> String {
    let mut sources = Vec::new();

    // Check for arch-wiki-lite (preferred)
    if Path::new("/usr/share/doc/arch-wiki/text").exists() {
        sources.push("arch-wiki-lite");
    }
    // Check for arch-wiki-docs
    else if Path::new("/usr/share/doc/arch-wiki/html").exists() {
        sources.push("arch-wiki-docs");
    }

    // Check for man pages
    if Path::new("/usr/share/man").exists() {
        sources.push("man pages");
    }

    // Check for /usr/share/doc
    if Path::new("/usr/share/doc").exists() {
        sources.push("/usr/share/doc");
    }

    if sources.is_empty() {
        "(no local docs detected)".dimmed().to_string()
    } else {
        sources.join(", ")
    }
}

fn print_internal_errors_section(stats: &Option<DaemonStats>) {
    println!("{}", "[INTERNAL ERRORS]".cyan());

    match stats {
        Some(s) => {
            println!("  Crashes:          {}", s.crashes);
            println!("  Command failures: {}", s.command_failures);
            println!("  Parse errors:     {}", s.parse_errors);
        }
        None => {
            println!("  {}", "(daemon not running)".dimmed());
        }
    }

    println!();
}

fn print_alerts_section() {
    println!("{}", "[ALERTS]".cyan());

    let alerts = collect_hardware_alerts();

    let critical_count = alerts.iter().filter(|a| a.severity == HealthStatus::Critical).count();
    let warning_count = alerts.iter().filter(|a| a.severity == HealthStatus::Warning).count();

    println!("  Critical:   {}", if critical_count > 0 {
        critical_count.to_string().red().to_string()
    } else {
        "0".green().to_string()
    });
    println!("  Warnings:   {}", if warning_count > 0 {
        warning_count.to_string().yellow().to_string()
    } else {
        "0".green().to_string()
    });

    if !alerts.is_empty() {
        println!();
        for alert in &alerts {
            let severity_str = match alert.severity {
                HealthStatus::Critical => "CRITICAL".red().to_string(),
                HealthStatus::Warning => "WARNING".yellow().to_string(),
                _ => "INFO".dimmed().to_string(),
            };

            println!("  - {} {}:{}    {}",
                severity_str,
                alert.scope.as_str(),
                alert.identifier,
                alert.reason
            );

            if let Some(ref cmd) = alert.see_command {
                println!("    See:      {}", cmd.dimmed());
            }
        }
    }

    println!();
}

/// Print [TOPOLOGY HINTS] section - v7.19.0
/// Shows services with many reverse dependencies and hardware with multiple driver stacks
fn print_topology_hints_section() {
    let high_impact = get_high_impact_services();
    let gpu_stacks = get_gpu_driver_stacks();

    // Only show if there's something to report
    if high_impact.is_empty() && gpu_stacks.is_empty() {
        return;
    }

    println!("{}", "[TOPOLOGY HINTS]".cyan());
    println!("  {}", "(source: systemctl, lsmod)".dimmed());

    // High-impact services (those with many reverse deps)
    if !high_impact.is_empty() {
        println!();
        println!("  High-impact services:");
        for hint in high_impact.iter().take(3) {
            println!("    {} ({} {} it)",
                     hint.unit.cyan(),
                     hint.reverse_dep_count,
                     hint.dep_type);
        }
    }

    // GPU driver stacks
    if !gpu_stacks.is_empty() {
        println!();
        println!("  Driver stacks:");
        for stack in &gpu_stacks {
            if stack.additional_modules.is_empty() {
                println!("    {} {} (primary: {})",
                         stack.component,
                         "[single driver]".green(),
                         stack.primary_driver);
            } else {
                println!("    {} {} ({} + {})",
                         stack.component,
                         "[multi-module]".yellow(),
                         stack.primary_driver,
                         stack.additional_modules.join(", "));
            }
        }
    }

    println!();
}

/// Print [ANNA NEEDS] section - v7.6.0
fn print_anna_needs_section() {
    println!("{}", "[ANNA NEEDS]".cyan());

    let needs = AnnaNeeds::check_all();
    let summary = needs.summary();

    // Show counts
    if summary.total_unsatisfied() == 0 {
        println!("  All tools installed. Anna is fully functional.");
        println!();
        return;
    }

    // Open (missing)
    let missing: Vec<_> = needs.by_status(NeedStatus::Missing);
    if !missing.is_empty() {
        println!("  Open:");
        for need in &missing {
            let pkg_hint = need.package.as_ref()
                .map(|p| format!(" (install: {})", p))
                .unwrap_or_default();
            println!("    - {}:{:<12} {} {}",
                need.need_type.as_str(),
                need.id.trim_start_matches(&format!("{}:", need.need_type.as_str())),
                need.reason,
                format!("(scope: {}){}", need.scope.as_str(), pkg_hint).dimmed()
            );
        }
    }

    // Blocked
    let blocked: Vec<_> = needs.by_status(NeedStatus::Blocked);
    if !blocked.is_empty() {
        println!();
        println!("  Blocked:");
        for need in &blocked {
            println!("    - {}:{:<12} auto-install disabled in /etc/anna/config.toml",
                need.need_type.as_str(),
                need.id.trim_start_matches(&format!("{}:", need.need_type.as_str()))
            );
        }
    }

    // Failed
    let failed: Vec<_> = needs.by_status(NeedStatus::Failed);
    if !failed.is_empty() {
        println!();
        println!("  Failed:");
        for need in &failed {
            let error = need.error.as_deref().unwrap_or("unknown error");
            println!("    - {}:{:<12} {} (see Anna logs)",
                need.need_type.as_str(),
                need.id.trim_start_matches(&format!("{}:", need.need_type.as_str())),
                error
            );
        }
    }

    println!();
}

// ============================================================================
// [ATTACHMENTS] Section - v7.25.0
// ============================================================================

/// v7.25.0: Connected peripherals section
fn print_attachments_section() {
    let usb = get_usb_summary();
    let bt = get_bluetooth_summary();
    let tb = get_thunderbolt_summary();

    // Only show if we have relevant data
    let has_usb = usb.device_count > 0;
    let has_bt = !bt.adapters.is_empty();
    let has_tb = tb.controller_count > 0 || tb.device_count > 0;

    if !has_usb && !has_bt && !has_tb {
        return; // Nothing to show
    }

    println!("{}", "[ATTACHMENTS]".cyan());
    println!("  {}", "(connected peripherals)".dimmed());

    // USB summary
    if has_usb {
        // Count by class
        let hid = usb.devices.iter().filter(|d| d.device_class == "HID").count();
        let storage = usb.devices.iter().filter(|d| d.device_class == "Mass Storage").count();
        let hubs = usb.devices.iter().filter(|d| d.is_hub).count();
        let other = usb.device_count as usize;

        let mut usb_parts = Vec::new();
        if hid > 0 { usb_parts.push(format!("{} HID", hid)); }
        if storage > 0 { usb_parts.push(format!("{} storage", storage)); }
        if other > hid + storage { usb_parts.push(format!("{} other", other - hid - storage)); }

        let usb_info = if usb_parts.is_empty() {
            format!("{} device(s)", usb.device_count)
        } else {
            usb_parts.join(", ")
        };

        println!("  USB:          {} controller(s), {} ({} hub(s))",
            usb.root_hubs, usb_info, hubs);
    }

    // Bluetooth summary
    if has_bt {
        for adapter in &bt.adapters {
            let state = match adapter.state {
                BluetoothState::Up => "UP".green().to_string(),
                BluetoothState::Blocked => "BLOCKED".yellow().to_string(),
                _ => "down".dimmed().to_string(),
            };
            println!("  Bluetooth:    {} ({}) [{}]", adapter.name, adapter.manufacturer, state);
        }
    }

    // Thunderbolt summary
    if has_tb {
        let tb_gen = tb.controllers.iter()
            .filter_map(|c| c.generation)
            .max()
            .map(|g| format!("TB{}", g))
            .unwrap_or_else(|| "Thunderbolt".to_string());

        let tb_info = if tb.device_count > 0 {
            format!("{} attached", tb.device_count)
        } else {
            "no devices".to_string()
        };

        println!("  Thunderbolt:  {} controller(s) ({}, {})",
            tb.controller_count, tb_gen, tb_info);
    }

    println!();
}

/// v7.22.0: Anna toolchain hygiene section
fn print_toolchain_section() {
    println!("{}", "[ANNA TOOLCHAIN]".cyan());

    let status = check_toolchain();
    let _summary = status.summary();

    // Documentation
    let doc_status = if status.is_category_ready(ToolCategory::Documentation) {
        "ready".green().to_string()
    } else {
        "missing".red().to_string()
    };
    println!("  Local wiki:     {}", doc_status);

    // Storage tools
    let storage_missing: Vec<_> = status.tools.iter()
        .filter(|t| t.tool.category == ToolCategory::Storage && !t.available)
        .map(|t| t.tool.name.as_str())
        .collect();
    if storage_missing.is_empty() {
        println!("  Storage tools:  {}", "ready".green());
    } else {
        println!("  Storage tools:  missing {}", storage_missing.join(", ").red());
    }

    // Network tools
    let network_missing: Vec<_> = status.tools.iter()
        .filter(|t| t.tool.category == ToolCategory::Network && !t.available)
        .map(|t| t.tool.name.as_str())
        .collect();
    if network_missing.is_empty() {
        println!("  Network tools:  {}", "ready".green());
    } else {
        println!("  Network tools:  missing {}", network_missing.join(", ").red());
    }

    println!();
}

// ============================================================================
// Daemon API Client
// ============================================================================

struct DaemonStats {
    uptime_secs: u64,
    pid: u32,
    restarts_24h: u32,
    packages_count: usize,
    commands_count: usize,
    services_count: usize,
    last_scan_secs_ago: Option<u64>,
    crashes: u32,
    command_failures: u32,
    parse_errors: u32,
}

#[derive(serde::Deserialize)]
struct StatsResponse {
    uptime_secs: u64,
    pid: u32,
    #[serde(default)]
    restarts_24h: u32,
    commands_count: usize,
    packages_count: usize,
    services_count: usize,
    last_scan_secs_ago: Option<u64>,
    internal_errors: InternalErrors,
}

#[derive(serde::Deserialize)]
struct InternalErrors {
    #[serde(default)]
    crashes: u32,
    #[serde(default, alias = "subprocess_failures")]
    command_failures: u32,
    #[serde(default, alias = "parser_failures")]
    parse_errors: u32,
}

async fn get_daemon_stats() -> Option<DaemonStats> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;

    let response = client
        .get("http://127.0.0.1:7865/v1/stats")
        .send()
        .await
        .ok()?;

    let stats: StatsResponse = response.json().await.ok()?;

    Some(DaemonStats {
        uptime_secs: stats.uptime_secs,
        pid: stats.pid,
        restarts_24h: stats.restarts_24h,
        packages_count: stats.packages_count,
        commands_count: stats.commands_count,
        services_count: stats.services_count,
        last_scan_secs_ago: stats.last_scan_secs_ago,
        crashes: stats.internal_errors.crashes,
        command_failures: stats.internal_errors.command_failures,
        parse_errors: stats.internal_errors.parse_errors,
    })
}

// ============================================================================
// [INSTRUMENTATION] Section - v7.27.0
// ============================================================================

/// v7.37.0: [INSTRUMENTATION] section - tools installed by Anna for metrics
/// Per spec: If none installed, print explicit clean statement
fn print_instrumentation_section() {
    let manifest = InstrumentationManifest::load();
    let config = AnnaConfig::load();

    println!("{}", "[INSTRUMENTATION]".cyan());

    // Show auto-install status
    let auto_install_str = if config.instrumentation.auto_install_enabled {
        "enabled".green().to_string()
    } else {
        "disabled".dimmed().to_string()
    };
    println!("  Auto-install:   {}", auto_install_str);

    // Show AUR gate status
    let aur_str = if config.instrumentation.allow_aur {
        "allowed".yellow().to_string()
    } else {
        "blocked".dimmed().to_string()
    };
    println!("  AUR packages:   {}", aur_str);

    // Show rate limit
    let today_count = manifest.recent_attempts
        .iter()
        .filter(|a| a.success && a.attempted_at > chrono::Utc::now() - chrono::Duration::hours(24))
        .count();
    println!("  Rate limit:     {}/{} installs today",
        today_count, config.instrumentation.max_installs_per_day);

    // v7.37.0: Explicit clean statement if none installed
    if manifest.installed_count() == 0 {
        println!("  Installed:      {} {}", "none".dimmed(), "(clean)".green());
        println!();
        return;
    }

    // Show installed tools with details
    println!("  Installed:      {} tool(s)", manifest.installed_count());
    for tool in manifest.installed_tools() {
        let since = tool.installed_at.format("%Y-%m-%d").to_string();
        let aur_note = if tool.source == "aur" {
            " [AUR]".yellow().to_string()
        } else {
            String::new()
        };
        println!("    {} v{}{}", tool.package.cyan(), tool.version, aur_note);
        println!("      installed: {}  reason: {}", since.dimmed(), tool.reason.dimmed());
        if !tool.metrics_enabled.is_empty() {
            println!("      unlocks: {}", tool.metrics_enabled.join(", ").dimmed());
        }
    }

    println!();
}

// ============================================================================
// [HOTSPOTS] Section - v7.24.0
// ============================================================================

/// v7.24.0: [HOTSPOTS] section with compact cross-reference
fn print_hotspots_xref_section() {
    use anna_common::config::AnnaConfig;

    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        return;
    }

    let sw_hotspots = get_software_hotspots();
    let hw_hotspots = get_hardware_hotspots();

    // Only print if we have data
    if !sw_hotspots.has_data && !hw_hotspots.has_data {
        return;
    }

    let lines = format_status_hotspots_section(&sw_hotspots, &hw_hotspots);
    for line in lines {
        if line.starts_with("[HOTSPOTS]") {
            println!("{}", line.cyan());
        } else {
            println!("{}", line);
        }
    }
    println!();
}
