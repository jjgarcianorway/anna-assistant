//! Probes - safe read-only system queries.

use anna_shared::rpc::ProbeType;
use anyhow::Result;
use std::process::Command;
use tracing::{info, warn};

/// Run a probe and return the result as a string
pub fn run_probe(probe_type: &ProbeType) -> Result<String> {
    info!("Running probe: {:?}", probe_type);

    match probe_type {
        ProbeType::TopMemory => probe_top_memory(),
        ProbeType::TopCpu => probe_top_cpu(),
        ProbeType::DiskUsage => probe_disk_usage(),
        ProbeType::NetworkInterfaces => probe_network_interfaces(),
    }
}

/// Get top processes by memory usage
fn probe_top_memory() -> Result<String> {
    let output = Command::new("ps").args(["aux", "--sort=-%mem"]).output()?;

    if !output.status.success() {
        anyhow::bail!("ps command failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Take header + top 10 processes
    let lines: Vec<&str> = stdout.lines().take(11).collect();
    Ok(lines.join("\n"))
}

/// Get top processes by CPU usage
fn probe_top_cpu() -> Result<String> {
    let output = Command::new("ps").args(["aux", "--sort=-%cpu"]).output()?;

    if !output.status.success() {
        anyhow::bail!("ps command failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Take header + top 10 processes
    let lines: Vec<&str> = stdout.lines().take(11).collect();
    Ok(lines.join("\n"))
}

/// Get disk usage
fn probe_disk_usage() -> Result<String> {
    let output = Command::new("df").args(["-h", "--total"]).output()?;

    if !output.status.success() {
        anyhow::bail!("df command failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get network interfaces
fn probe_network_interfaces() -> Result<String> {
    // Try ip command first
    let output = Command::new("ip").args(["addr", "show"]).output();

    match output {
        Ok(o) if o.status.success() => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
        _ => {
            // Fallback to ifconfig
            let output = Command::new("ifconfig").output()?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
    }
}

/// Run an arbitrary shell command (for service desk probes)
/// The caller is responsible for verifying the command is in the allowlist
pub fn run_command(cmd: &str) -> Result<String> {
    info!("Running command: {}", cmd);

    let output = Command::new("sh").args(["-c", cmd]).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Command '{}' failed: {}", cmd, stderr);
        anyhow::bail!("Command failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // Truncate very long outputs
    if stdout.len() > 8000 {
        Ok(format!("{}\n... (truncated)", &stdout[..8000]))
    } else {
        Ok(stdout)
    }
}
