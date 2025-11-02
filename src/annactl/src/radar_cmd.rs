//! Radar command for annactl v0.12.9 "Orion"
//!
//! Display hardware, software, and user radars

use anyhow::{Context, Result};
use anna_common::{header, section, TermCaps};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/run/anna/annad.sock";

/// Combined radar result from daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarSnapshot {
    pub hardware: HardwareRadar,
    pub software: SoftwareRadar,
    pub user: UserRadar,
}

/// Hardware radar (matches annad::radar_hardware::HardwareRadar)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRadar {
    pub overall: u8,
    pub cpu_throughput: u8,
    pub cpu_thermal: u8,
    pub memory: u8,
    pub disk_health: u8,
    pub disk_free: u8,
    pub fs_features: u8,
    pub gpu: u8,
    pub network: u8,
    pub boot: u8,
}

/// Software radar (matches annad::radar_software::SoftwareRadar)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareRadar {
    pub overall: u8,
    pub os_freshness: u8,
    pub kernel: u8,
    pub packages: u8,
    pub services: u8,
    pub security: u8,
    pub containers: u8,
    pub fs_integrity: u8,
    pub backups: u8,
    pub log_noise: u8,
}

/// User radar (matches annad::radar_user::UserRadar)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRadar {
    pub overall: u8,
    pub regularity: u8,
    pub workspace: u8,
    pub updates: u8,
    pub backups: u8,
    pub risk: u8,
    pub connectivity: u8,
    pub power: u8,
    pub warnings: u8,
}

/// Run radar command
pub async fn run_radar(json: bool, filter: Option<&str>) -> Result<()> {
    let snapshot = fetch_radar_snapshot().await?;

    if json {
        // JSON output
        let json_str = serde_json::to_string_pretty(&snapshot)?;
        println!("{}", json_str);
        return Ok(());
    }

    // TUI output
    print_radar_tui(&snapshot, filter);

    Ok(())
}

/// Fetch radar snapshot from daemon
async fn fetch_radar_snapshot() -> Result<RadarSnapshot> {
    let mut stream = UnixStream::connect(SOCKET_PATH)
        .await
        .context("Failed to connect to annad - is it running?")?;

    // Send RPC request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "radar_snapshot",
        "params": {},
        "id": 1
    });

    let request_str = serde_json::to_string(&request)?;
    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    // Read response
    let (reader, _writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let response_line = lines
        .next_line()
        .await?
        .context("No response from daemon")?;

    // Parse JSON-RPC response
    let response: JsonValue = serde_json::from_str(&response_line)?;

    if let Some(error) = response.get("error") {
        anyhow::bail!("RPC error: {}", error);
    }

    let result = response
        .get("result")
        .context("No result in RPC response")?;

    let snapshot: RadarSnapshot = serde_json::from_value(result.clone())?;

    Ok(snapshot)
}

/// Print radar snapshot with TUI
fn print_radar_tui(snapshot: &RadarSnapshot, filter: Option<&str>) {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "System Radars"));
    println!();

    // Filter display
    let show_hardware = filter.is_none() || filter == Some("hardware");
    let show_software = filter.is_none() || filter == Some("software");
    let show_user = filter.is_none() || filter == Some("user");

    if show_hardware {
        print_hardware_radar(&caps, &snapshot.hardware);
        println!();
    }

    if show_software {
        print_software_radar(&caps, &snapshot.software);
        println!();
    }

    if show_user {
        print_user_radar(&caps, &snapshot.user);
        println!();
    }

    // Summary
    if filter.is_none() {
        print_summary(&caps, snapshot);
    }
}

/// Print hardware radar
fn print_hardware_radar(caps: &TermCaps, radar: &HardwareRadar) {
    println!("{}", section(caps, "Hardware Radar"));
    println!();

    let overall_bar = score_bar(radar.overall);
    println!("  Overall: {}/10  {}", radar.overall, overall_bar);
    println!();

    print_category("CPU Throughput", radar.cpu_throughput);
    print_category("CPU Thermal", radar.cpu_thermal);
    print_category("Memory", radar.memory);
    print_category("Disk Health", radar.disk_health);
    print_category("Disk Free", radar.disk_free);
    print_category("FS Features", radar.fs_features);
    print_category("GPU", radar.gpu);
    print_category("Network", radar.network);
    print_category("Boot", radar.boot);
}

/// Print software radar
fn print_software_radar(caps: &TermCaps, radar: &SoftwareRadar) {
    println!("{}", section(caps, "Software Radar"));
    println!();

    let overall_bar = score_bar(radar.overall);
    println!("  Overall: {}/10  {}", radar.overall, overall_bar);
    println!();

    print_category("OS Freshness", radar.os_freshness);
    print_category("Kernel", radar.kernel);
    print_category("Packages", radar.packages);
    print_category("Services", radar.services);
    print_category("Security", radar.security);
    print_category("Containers", radar.containers);
    print_category("FS Integrity", radar.fs_integrity);
    print_category("Backups", radar.backups);
    print_category("Log Noise", radar.log_noise);
}

/// Print user radar
fn print_user_radar(caps: &TermCaps, radar: &UserRadar) {
    println!("{}", section(caps, "User Radar"));
    println!();

    let overall_bar = score_bar(radar.overall);
    println!("  Overall: {}/10  {}", radar.overall, overall_bar);
    println!();

    print_category("Regularity", radar.regularity);
    print_category("Workspace", radar.workspace);
    print_category("Updates", radar.updates);
    print_category("Backups", radar.backups);
    print_category("Risk", radar.risk);
    print_category("Connectivity", radar.connectivity);
    print_category("Power", radar.power);
    print_category("Warnings", radar.warnings);
}

/// Print a single category score
fn print_category(name: &str, score: u8) {
    let bar = score_bar(score);
    let color = score_color(score);

    println!("  {:<16} {}{}/10{}  {}",
             name,
             color,
             score,
             "\x1b[0m",
             bar);
}

/// Generate a visual bar for a score (0-10)
fn score_bar(score: u8) -> String {
    let filled = score as usize;
    let empty = 10 - filled;

    let color = score_color(score);
    let reset = "\x1b[0m";

    format!("{}{}{}{}",
            color,
            "█".repeat(filled),
            reset,
            "░".repeat(empty))
}

/// Get color for score
fn score_color(score: u8) -> &'static str {
    match score {
        9..=10 => "\x1b[32m",  // Green (excellent)
        7..=8 => "\x1b[36m",   // Cyan (good)
        5..=6 => "\x1b[33m",   // Yellow (moderate)
        3..=4 => "\x1b[35m",   // Magenta (poor)
        _ => "\x1b[31m",       // Red (critical)
    }
}

/// Print summary
fn print_summary(caps: &TermCaps, snapshot: &RadarSnapshot) {
    println!("{}", section(caps, "Summary"));
    println!();

    let avg = (snapshot.hardware.overall as u16
             + snapshot.software.overall as u16
             + snapshot.user.overall as u16) / 3;

    println!("  System Health: {}/10", avg);
    println!();

    // Recommendations
    if snapshot.hardware.overall < 7 {
        println!("  ⚠️  Hardware needs attention (score: {})", snapshot.hardware.overall);
    }
    if snapshot.software.overall < 7 {
        println!("  ⚠️  Software needs attention (score: {})", snapshot.software.overall);
    }
    if snapshot.user.overall < 7 {
        println!("  ⚠️  User habits need improvement (score: {})", snapshot.user.overall);
    }

    if avg >= 8 {
        println!("  ✅ System is in excellent shape!");
    } else if avg >= 6 {
        println!("  ℹ️  System is healthy with room for improvement");
    } else {
        println!("  ❌ System needs attention in multiple areas");
    }
}
