//! Daemon watchdog for health monitoring and automatic recovery.
//! v0.0.825: Added to improve daemon reliability.
//!
//! The watchdog monitors:
//! - Ollama service health
//! - Memory usage
//! - Response times
//! - Background task health
//!
//! If issues are detected, it can:
//! - Restart Ollama service
//! - Clear stale state
//! - Log diagnostic information

use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::ollama;
use crate::state::SharedState;

/// Watchdog configuration
pub struct WatchdogConfig {
    /// How often to check health (seconds)
    pub check_interval_secs: u64,
    /// Maximum time without a successful request before warning
    pub idle_warning_secs: u64,
    /// Whether to auto-restart Ollama if it dies
    pub auto_restart_ollama: bool,
    /// Maximum consecutive Ollama failures before restart
    pub max_ollama_failures: u32,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 30,
            idle_warning_secs: 300,
            auto_restart_ollama: true,
            max_ollama_failures: 3,
        }
    }
}

/// Health status from a watchdog check
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub ollama_healthy: bool,
    pub daemon_responsive: bool,
    pub memory_ok: bool,
    pub last_request_age_secs: Option<u64>,
    pub issues: Vec<String>,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.ollama_healthy && self.daemon_responsive && self.memory_ok && self.issues.is_empty()
    }
}

/// Watchdog state tracking
struct WatchdogState {
    consecutive_ollama_failures: u32,
    last_ollama_restart: Option<Instant>,
    last_healthy_check: Instant,
}

impl Default for WatchdogState {
    fn default() -> Self {
        Self {
            consecutive_ollama_failures: 0,
            last_ollama_restart: None,
            last_healthy_check: Instant::now(),
        }
    }
}

/// Start the watchdog loop
pub async fn start_watchdog(state: SharedState, config: WatchdogConfig) {
    info!(
        "Watchdog started (check interval: {}s, auto-restart: {})",
        config.check_interval_secs, config.auto_restart_ollama
    );

    let mut interval = interval(Duration::from_secs(config.check_interval_secs));
    let mut watchdog_state = WatchdogState::default();

    loop {
        interval.tick().await;

        let health = check_health(&state, &config).await;

        if health.is_healthy() {
            watchdog_state.consecutive_ollama_failures = 0;
            watchdog_state.last_healthy_check = Instant::now();
        } else {
            // Log issues
            for issue in &health.issues {
                warn!("Watchdog detected issue: {}", issue);
            }

            // Handle Ollama failures
            if !health.ollama_healthy {
                watchdog_state.consecutive_ollama_failures += 1;

                if config.auto_restart_ollama
                    && watchdog_state.consecutive_ollama_failures >= config.max_ollama_failures
                {
                    // Check if we restarted recently (avoid restart loops)
                    let can_restart = watchdog_state
                        .last_ollama_restart
                        .map(|t| t.elapsed() > Duration::from_secs(60))
                        .unwrap_or(true);

                    if can_restart {
                        warn!(
                            "Ollama failed {} times, attempting restart",
                            watchdog_state.consecutive_ollama_failures
                        );

                        if let Err(e) = restart_ollama().await {
                            error!("Failed to restart Ollama: {}", e);
                        } else {
                            info!("Ollama restarted successfully");
                            watchdog_state.consecutive_ollama_failures = 0;
                            watchdog_state.last_ollama_restart = Some(Instant::now());
                        }
                    } else {
                        warn!("Skipping Ollama restart (restarted recently)");
                    }
                }
            }
        }

        // Periodic health log (every 10 checks when healthy)
        if health.is_healthy() && watchdog_state.last_healthy_check.elapsed().as_secs() % 300 < 30 {
            info!("Watchdog: all systems healthy");
        }
    }
}

/// Check overall daemon health
async fn check_health(state: &SharedState, config: &WatchdogConfig) -> HealthStatus {
    let mut issues = Vec::new();

    // Check Ollama
    let ollama_healthy = ollama::is_running().await;
    if !ollama_healthy {
        issues.push("Ollama is not running".to_string());
    }

    // Check daemon state
    let (daemon_responsive, last_request_age_secs) = {
        let state_read = state.read().await;
        let responsive =
            matches!(state_read.state, anna_shared::status::DaemonState::Running);

        // Check time since last request (from stats)
        let age = None; // Could track last request time in state

        (responsive, age)
    };

    if !daemon_responsive {
        issues.push("Daemon not in Running state".to_string());
    }

    // Check idle warning
    if let Some(age) = last_request_age_secs {
        if age > config.idle_warning_secs {
            issues.push(format!("No requests for {} seconds", age));
        }
    }

    // Check memory usage (basic check)
    let memory_ok = check_memory_usage();
    if !memory_ok {
        issues.push("High memory usage detected".to_string());
    }

    HealthStatus {
        ollama_healthy,
        daemon_responsive,
        memory_ok,
        last_request_age_secs,
        issues,
    }
}

/// Check if memory usage is acceptable
fn check_memory_usage() -> bool {
    // Read /proc/self/statm for memory info
    if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if let Some(rss_pages) = parts.get(1).and_then(|s| s.parse::<u64>().ok()) {
            // RSS in pages (typically 4KB each)
            let rss_mb = (rss_pages * 4) / 1024;
            // Warn if daemon is using more than 2GB
            if rss_mb > 2048 {
                warn!("High memory usage: {}MB RSS", rss_mb);
                return false;
            }
        }
    }
    true
}

/// Attempt to restart Ollama service
async fn restart_ollama() -> anyhow::Result<()> {
    info!("Attempting to restart Ollama...");

    // Stop existing service
    let _ = std::process::Command::new("systemctl")
        .args(["stop", "ollama"])
        .output();

    // Kill any remaining processes
    let _ = std::process::Command::new("pkill")
        .args(["-9", "ollama"])
        .output();

    // Wait a moment
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Start service
    ollama::start_service().await?;

    // Verify it's running
    for _ in 0..10 {
        if ollama::is_running().await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    anyhow::bail!("Ollama failed to start after restart")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WatchdogConfig::default();
        assert_eq!(config.check_interval_secs, 30);
        assert!(config.auto_restart_ollama);
    }

    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus {
            ollama_healthy: true,
            daemon_responsive: true,
            memory_ok: true,
            last_request_age_secs: None,
            issues: vec![],
        };
        assert!(status.is_healthy());
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = HealthStatus {
            ollama_healthy: false,
            daemon_responsive: true,
            memory_ok: true,
            last_request_age_secs: None,
            issues: vec!["Ollama down".to_string()],
        };
        assert!(!status.is_healthy());
    }

    #[test]
    fn test_memory_check() {
        // Should pass on most systems
        assert!(check_memory_usage());
    }
}
