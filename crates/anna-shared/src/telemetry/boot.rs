//! Boot time telemetry - tracks system boot time changes.
//!
//! Uses systemd-analyze time to capture boot times.
//! Stores current and previous values to compute deltas.
//!
//! v0.0.29: Initial implementation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Boot time snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BootSnapshot {
    /// Current boot time in milliseconds
    pub current_ms: Option<u64>,
    /// Previous boot time in milliseconds
    pub previous_ms: Option<u64>,
    /// Timestamp when captured (epoch seconds)
    pub captured_at_ts: u64,
}

impl BootSnapshot {
    /// Create a new snapshot with current timestamp
    pub fn new() -> Self {
        let captured_at_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            captured_at_ts,
            ..Default::default()
        }
    }

    /// Calculate delta (current - previous) in milliseconds
    /// Negative means boot got faster
    pub fn delta_ms(&self) -> Option<i64> {
        match (self.current_ms, self.previous_ms) {
            (Some(curr), Some(prev)) => Some(curr as i64 - prev as i64),
            _ => None,
        }
    }

    /// Update with new boot time, rotating current to previous
    pub fn update(&mut self, new_boot_time_ms: u64) {
        self.previous_ms = self.current_ms;
        self.current_ms = Some(new_boot_time_ms);
        self.captured_at_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }
}

/// Parse systemd-analyze time output
///
/// Example output:
/// ```text
/// Startup finished in 2.345s (firmware) + 1.234s (loader) + 3.456s (kernel) + 12.345s (userspace) = 19.380s
/// ```
///
/// We extract the total time at the end.
pub fn parse_systemd_analyze_time(output: &str) -> Option<u64> {
    let output = output.trim();

    // Find the equals sign and extract the value after it
    if let Some(eq_pos) = output.rfind('=') {
        let after_eq = output[eq_pos + 1..].trim();
        // Parse the time value (handles "19.380s", "1min 23.456s", etc.)
        return parse_time_string(after_eq);
    }

    // Alternative: look for "Startup finished in X.XXXs" format (without =)
    if output.contains("Startup finished in") {
        // Extract the time after "Startup finished in"
        if let Some(pos) = output.find("Startup finished in") {
            let after = &output[pos + 19..].trim();
            // Find the first time value (ends with 's' before any parenthesis)
            let end = after.find('(').unwrap_or(after.len());
            let time_str = after[..end].trim();
            return parse_time_string(time_str);
        }
    }

    // Final fallback: if output is just a time string
    if output.ends_with('s') && !output.contains('=') && !output.contains(' ') {
        return parse_time_string(output);
    }

    None
}

/// Parse time string like "19.380s", "1min 23.456s", "2min 5s"
fn parse_time_string(s: &str) -> Option<u64> {
    let s = s.trim();

    // Handle "Xmin Xs" or "Xmin X.XXXs" format
    if s.contains("min") {
        let parts: Vec<&str> = s.split("min").collect();
        if parts.len() >= 2 {
            let mins: f64 = parts[0].trim().parse().ok()?;
            let secs_str = parts[1].trim().trim_end_matches('s').trim();
            let secs: f64 = if secs_str.is_empty() {
                0.0
            } else {
                secs_str.parse().ok()?
            };
            let total_ms = (mins * 60.0 * 1000.0) + (secs * 1000.0);
            return Some(total_ms as u64);
        }
    }

    // Handle "X.XXXs" or "Xs" format
    if s.ends_with('s') {
        let num_str = s.trim_end_matches('s').trim();
        if let Ok(secs) = num_str.parse::<f64>() {
            return Some((secs * 1000.0) as u64);
        }
    }

    None
}

/// Get boot time snapshot file path
pub fn boot_snapshot_path() -> PathBuf {
    super::telemetry_dir().join("boot.json")
}

/// Load boot snapshot from disk
pub fn load_boot_snapshot() -> Option<BootSnapshot> {
    let path = boot_snapshot_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(snap) = serde_json::from_str(&content) {
                return Some(snap);
            }
        }
    }
    None
}

/// Save boot snapshot to disk
pub fn save_boot_snapshot(snap: &BootSnapshot) -> std::io::Result<()> {
    let path = boot_snapshot_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(snap)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(&path, content)
}

/// Capture current boot time using systemd-analyze
/// Returns boot time in milliseconds, or None if unavailable
pub fn capture_boot_time() -> Option<u64> {
    // Try to run systemd-analyze time
    let output = std::process::Command::new("systemd-analyze")
        .arg("time")
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return parse_systemd_analyze_time(&stdout);
    }

    None
}

/// Update boot snapshot with current boot time
pub fn update_boot_snapshot() -> Option<BootSnapshot> {
    let boot_time = capture_boot_time()?;

    let mut snap = load_boot_snapshot().unwrap_or_else(BootSnapshot::new);
    snap.update(boot_time);

    if save_boot_snapshot(&snap).is_ok() {
        Some(snap)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_systemd_analyze_simple() {
        let output = "Startup finished in 19.380s";
        assert_eq!(parse_systemd_analyze_time(output), Some(19380));
    }

    #[test]
    fn test_parse_systemd_analyze_full() {
        let output = "Startup finished in 2.345s (firmware) + 1.234s (loader) + 3.456s (kernel) + 12.345s (userspace) = 19.380s";
        assert_eq!(parse_systemd_analyze_time(output), Some(19380));
    }

    #[test]
    fn test_parse_systemd_analyze_with_minutes() {
        let output = "Startup finished in 1min 23.456s (firmware) + 5.678s (loader) = 1min 29.134s";
        let result = parse_systemd_analyze_time(output);
        // 1 min 29.134s = 89.134s = 89134ms
        assert_eq!(result, Some(89134));
    }

    #[test]
    fn test_parse_time_string_seconds() {
        assert_eq!(parse_time_string("19.380s"), Some(19380));
        assert_eq!(parse_time_string("5s"), Some(5000));
        assert_eq!(parse_time_string("0.5s"), Some(500));
    }

    #[test]
    fn test_parse_time_string_minutes() {
        assert_eq!(parse_time_string("1min 30s"), Some(90000));
        assert_eq!(parse_time_string("2min 15.5s"), Some(135500));
        assert_eq!(parse_time_string("1min"), Some(60000));
    }

    #[test]
    fn test_boot_snapshot_delta() {
        let mut snap = BootSnapshot::new();
        snap.current_ms = Some(20000);
        snap.previous_ms = Some(22000);

        // Current (20s) is faster than previous (22s), so delta is negative
        assert_eq!(snap.delta_ms(), Some(-2000));
    }

    #[test]
    fn test_boot_snapshot_update() {
        let mut snap = BootSnapshot::new();
        snap.update(20000);
        assert_eq!(snap.current_ms, Some(20000));
        assert_eq!(snap.previous_ms, None);

        snap.update(18000);
        assert_eq!(snap.current_ms, Some(18000));
        assert_eq!(snap.previous_ms, Some(20000));
    }

    #[test]
    fn test_boot_snapshot_serialization() {
        let mut snap = BootSnapshot::new();
        snap.current_ms = Some(19380);
        snap.previous_ms = Some(21000);

        let json = serde_json::to_string(&snap).unwrap();
        let parsed: BootSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.current_ms, Some(19380));
        assert_eq!(parsed.previous_ms, Some(21000));
    }
}
