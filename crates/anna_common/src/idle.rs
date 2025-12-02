//! Anna Idle Detection v7.37.0
//!
//! Detects system idle state for safe background operations:
//! - CPU load threshold (1-minute load average)
//! - Pacman lock detection (no concurrent package operations)
//! - Network activity for heavy probes (optional)
//!
//! Used by:
//! - Auto-update scheduler (avoids checks during heavy load)
//! - Auto-install engine (defers installs when pacman is locked)
//! - Background scans (respects system load)

use std::path::Path;

/// CPU load threshold for "idle" (1-minute load average)
/// A system with 4 cores is "busy" if load > 2.0
pub const DEFAULT_LOAD_THRESHOLD: f64 = 2.0;

/// Pacman database lock file
pub const PACMAN_LOCK: &str = "/var/lib/pacman/db.lck";

/// Idle state result
#[derive(Debug, Clone, PartialEq)]
pub struct IdleState {
    /// Whether the system is considered idle
    pub is_idle: bool,
    /// Current 1-minute load average
    pub load_1m: f64,
    /// Whether pacman is locked
    pub pacman_locked: bool,
    /// Reasons why not idle (if any)
    pub busy_reasons: Vec<String>,
}

impl IdleState {
    /// Check if safe for package operations (not locked)
    pub fn safe_for_pacman(&self) -> bool {
        !self.pacman_locked
    }

    /// Check if safe for CPU-intensive operations
    pub fn safe_for_cpu_work(&self) -> bool {
        self.load_1m < DEFAULT_LOAD_THRESHOLD
    }
}

/// Check current system idle state
pub fn check_idle_state() -> IdleState {
    let mut busy_reasons = Vec::new();

    // Check CPU load
    let load_1m = get_load_average();
    if load_1m >= DEFAULT_LOAD_THRESHOLD {
        busy_reasons.push(format!("load {:.1} >= {:.1}", load_1m, DEFAULT_LOAD_THRESHOLD));
    }

    // Check pacman lock
    let pacman_locked = is_pacman_locked();
    if pacman_locked {
        busy_reasons.push("pacman locked".to_string());
    }

    let is_idle = busy_reasons.is_empty();

    IdleState {
        is_idle,
        load_1m,
        pacman_locked,
        busy_reasons,
    }
}

/// Get 1-minute load average from /proc/loadavg
fn get_load_average() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|content| {
            content.split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
        })
        .unwrap_or(0.0)
}

/// Check if pacman database is locked
pub fn is_pacman_locked() -> bool {
    Path::new(PACMAN_LOCK).exists()
}

/// Wait for pacman to be unlocked (with timeout)
/// Returns true if pacman became available, false if timed out
pub fn wait_for_pacman_unlock(timeout_secs: u64) -> bool {
    use std::time::{Duration, Instant};

    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if !is_pacman_locked() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    false
}

/// Get number of logical CPU cores for load normalization
pub fn get_cpu_count() -> usize {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|content| {
            content.lines()
                .filter(|line| line.starts_with("processor"))
                .count()
        })
        .unwrap_or(1)
}

/// Calculate normalized load (load / cpu_count)
pub fn normalized_load() -> f64 {
    let load = get_load_average();
    let cpus = get_cpu_count();
    if cpus > 0 {
        load / cpus as f64
    } else {
        load
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_idle_state() {
        let state = check_idle_state();
        // Load average should be a non-negative number
        assert!(state.load_1m >= 0.0);
        // pacman_locked is a boolean - just verify the field exists
        let _ = state.pacman_locked;
    }

    #[test]
    fn test_get_load_average() {
        let load = get_load_average();
        assert!(load >= 0.0, "Load average should be non-negative");
    }

    #[test]
    fn test_get_cpu_count() {
        let count = get_cpu_count();
        assert!(count >= 1, "Should have at least 1 CPU");
    }

    #[test]
    fn test_normalized_load() {
        let norm = normalized_load();
        assert!(norm >= 0.0, "Normalized load should be non-negative");
    }
}
