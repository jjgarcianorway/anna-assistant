//! Probe-specific debug handlers (v0.0.446).

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use anna_shared::state_dir;

/// Path to debug log file.
fn debug_log_path() -> PathBuf {
    PathBuf::from(state_dir()).join("debug_log.json")
}

/// Show last probe outputs.
pub async fn show_probes_last() -> Result<()> {
    let path = debug_log_path();

    if !path.exists() {
        println!("No debug log found.");
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;

    // Find last entry with probes
    for line in content.lines().rev() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(probes) = json.get("probes_run") {
                if let Some(probes_arr) = probes.as_array() {
                    if !probes_arr.is_empty() {
                        println!("[probes_run]");
                        for probe in probes_arr {
                            print_probe(probe);
                        }
                        return Ok(());
                    }
                }
            }
        }
    }

    println!("No probe outputs found in debug log.");

    Ok(())
}

/// Print a single probe.
fn print_probe(probe: &serde_json::Value) {
    let id = probe
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let cmd = probe.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let status = probe
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let exit_code = probe
        .get("exit_code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    let duration = probe
        .get("duration_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    println!("  --- {} ({}) ---", id, cmd);
    println!("  status: {} (exit {})", status, exit_code);
    println!("  duration: {}ms", duration);

    if let Some(stdout) = probe.get("stdout").and_then(|v| v.as_str()) {
        if !stdout.is_empty() {
            println!("  stdout:");
            for line in stdout.lines().take(10) {
                println!("    {}", line);
            }
            if stdout.lines().count() > 10 {
                println!("    ...[truncated]");
            }
        }
    }

    if let Some(stderr) = probe.get("stderr").and_then(|v| v.as_str()) {
        if !stderr.is_empty() {
            println!("  stderr:");
            for line in stderr.lines().take(5) {
                println!("    {}", line);
            }
        }
    }
}
