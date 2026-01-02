//! Monitor check implementations (v0.0.430).
//!
//! Functions for checking various system conditions.

use std::fs;
use std::process::Command;
use std::time::SystemTime;

/// Check disk space usage percentage
pub fn check_disk_space(path: &str) -> Result<f64, String> {
    // Read from df command
    let output = Command::new("df")
        .arg("--output=pcent")
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().skip(1) {
        if let Some(pct) = line.trim().strip_suffix('%') {
            return pct
                .parse()
                .map_err(|e: std::num::ParseFloatError| e.to_string());
        }
    }
    Err("Could not parse df output".to_string())
}

/// Check if a process is running (returns count)
pub fn check_process(name: &str) -> Result<f64, String> {
    let output = Command::new("pgrep")
        .arg("-c")
        .arg(name)
        .output()
        .map_err(|e| e.to_string())?;

    let count: f64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0.0);
    Ok(count)
}

/// Check file age in seconds since last modification
pub fn check_file_age(path: &str) -> Result<f64, String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    let modified = metadata.modified().map_err(|e| e.to_string())?;
    let age = SystemTime::now()
        .duration_since(modified)
        .map(|d| d.as_secs() as f64)
        .unwrap_or(0.0);
    Ok(age)
}

/// Run a custom command and parse output as f64
pub fn run_command(cmd: &str, args: &[String]) -> Result<f64, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|e: std::num::ParseFloatError| e.to_string())
}

/// Check system load (1-minute average)
pub fn check_system_load() -> Result<f64, String> {
    let content = fs::read_to_string("/proc/loadavg").map_err(|e| e.to_string())?;
    content
        .split_whitespace()
        .next()
        .ok_or_else(|| "Empty loadavg".to_string())?
        .parse()
        .map_err(|e: std::num::ParseFloatError| e.to_string())
}

/// Check memory usage percentage
pub fn check_memory() -> Result<f64, String> {
    let content = fs::read_to_string("/proc/meminfo").map_err(|e| e.to_string())?;
    let mut total = 0u64;
    let mut available = 0u64;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total = parse_meminfo_value(line);
        } else if line.starts_with("MemAvailable:") {
            available = parse_meminfo_value(line);
        }
    }

    if total > 0 {
        Ok(((total - available) as f64 / total as f64) * 100.0)
    } else {
        Err("Could not read memory info".to_string())
    }
}

fn parse_meminfo_value(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}
