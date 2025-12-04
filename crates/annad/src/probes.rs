//! Probes - safe read-only system queries.

use anna_shared::rpc::{ProbeResult, ProbeType};
use anyhow::Result;
use std::process::Command;
use std::time::Instant;
use tracing::{info, warn};

/// Maximum lines to include in stdout/stderr
const MAX_OUTPUT_LINES: usize = 50;

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

/// Truncate output to first N lines
fn truncate_lines(text: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = text.lines().take(max_lines).collect();
    let truncated = lines.len() < text.lines().count();
    let mut result = lines.join("\n");
    if truncated {
        result.push_str("\n... (truncated)");
    }
    result
}

/// Run an arbitrary shell command and return structured result
/// The caller is responsible for verifying the command is in the allowlist
pub fn run_command_structured(cmd: &str) -> ProbeResult {
    info!("Running command: {}", cmd);

    let start = Instant::now();
    let output = Command::new("sh").args(["-c", cmd]).output();
    let timing_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);

            if exit_code != 0 {
                warn!(
                    "Command '{}' failed with code {}: {}",
                    cmd, exit_code, stderr
                );
            }

            ProbeResult {
                command: cmd.to_string(),
                exit_code,
                stdout: truncate_lines(&stdout, MAX_OUTPUT_LINES),
                stderr: truncate_lines(&stderr, MAX_OUTPUT_LINES),
                timing_ms,
            }
        }
        Err(e) => {
            warn!("Command '{}' failed to execute: {}", cmd, e);
            ProbeResult {
                command: cmd.to_string(),
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Failed to execute: {}", e),
                timing_ms,
            }
        }
    }
}

/// Run an arbitrary shell command (for service desk probes)
/// The caller is responsible for verifying the command is in the allowlist
#[allow(dead_code)]
pub fn run_command(cmd: &str) -> Result<String> {
    let result = run_command_structured(cmd);
    if result.exit_code != 0 {
        anyhow::bail!("Command failed: {}", result.stderr);
    }
    Ok(result.stdout)
}
