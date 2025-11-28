//! Daemon Watchdog v0.26.0
//!
//! Self-healing watchdog for daemon reliability.
//! Monitors health and initiates automatic recovery.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::protocol_v26::{
    HealingEvent, HealingEventType, HealingTrace, WatchdogConfig, WatchdogState,
};

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Maximum events to keep in history
pub const MAX_HISTORY_EVENTS: usize = 100;

/// Default check interval (seconds)
pub const DEFAULT_CHECK_INTERVAL: u64 = 30;

/// Default max failures before restart
pub const DEFAULT_MAX_FAILURES: u32 = 3;

// ============================================================================
// HEALTH CHECK
// ============================================================================

/// Health check result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthCheckResult {
    /// All checks passed
    Healthy,
    /// Some degradation detected
    Degraded { reason: String },
    /// Critical failure
    Failed { reason: String },
}

impl HealthCheckResult {
    /// Check if healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if failed
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

/// Health check target
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckTarget {
    /// Daemon process
    Daemon,
    /// Daemon API endpoint
    DaemonApi,
    /// LLM backend (Ollama)
    LlmBackend,
    /// IPC socket
    IpcSocket,
    /// Memory usage
    Memory,
    /// Disk space
    Disk,
}

/// A single health check definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check target
    pub target: CheckTarget,
    /// Timeout (seconds)
    pub timeout_secs: u64,
    /// Critical flag (failure = restart)
    pub critical: bool,
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            target: CheckTarget::DaemonApi,
            timeout_secs: 5,
            critical: true,
        }
    }
}

// ============================================================================
// WATCHDOG
// ============================================================================

/// Daemon watchdog for self-healing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonWatchdog {
    /// Configuration
    pub config: WatchdogConfig,
    /// Current state
    pub state: WatchdogState,
    /// Health checks to perform
    pub checks: Vec<HealthCheck>,
    /// Event history
    pub history: VecDeque<HealingEvent>,
    /// Active healing trace
    pub active_trace: Option<HealingTrace>,
    /// Last check results
    pub last_results: Vec<(CheckTarget, HealthCheckResult)>,
}

impl Default for DaemonWatchdog {
    fn default() -> Self {
        Self {
            config: WatchdogConfig::default(),
            state: WatchdogState::default(),
            checks: vec![
                HealthCheck {
                    target: CheckTarget::DaemonApi,
                    timeout_secs: 5,
                    critical: true,
                },
                HealthCheck {
                    target: CheckTarget::LlmBackend,
                    timeout_secs: 5,
                    critical: false,
                },
            ],
            history: VecDeque::new(),
            active_trace: None,
            last_results: Vec::new(),
        }
    }
}

impl DaemonWatchdog {
    /// Create with custom config
    pub fn with_config(config: WatchdogConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Check if watchdog is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Should perform health check now?
    pub fn should_check(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        let now = chrono::Utc::now().timestamp();
        match self.state.last_check {
            Some(last) => now - last >= self.config.check_interval_secs as i64,
            None => true,
        }
    }

    /// Record health check result
    pub fn record_check(&mut self, target: CheckTarget, result: HealthCheckResult) {
        // Update last results
        self.last_results.retain(|(t, _)| *t != target);
        self.last_results.push((target.clone(), result.clone()));

        // Update state
        if result.is_healthy() {
            self.state.record_success();
            self.add_event(HealingEventType::HealthCheck, &target, true, None);
        } else {
            self.state.record_failure();
            let reason = match &result {
                HealthCheckResult::Degraded { reason } => Some(reason.clone()),
                HealthCheckResult::Failed { reason } => Some(reason.clone()),
                _ => None,
            };
            let event_type = if result.is_failed() {
                HealingEventType::ComponentFailed
            } else {
                HealingEventType::ComponentDegraded
            };
            self.add_event(event_type, &target, false, reason);
        }
    }

    /// Check if restart is needed
    pub fn needs_restart(&self) -> bool {
        self.state.needs_restart(&self.config)
    }

    /// Can restart (respects rate limiting)
    pub fn can_restart(&self) -> bool {
        self.state.can_restart(&self.config)
    }

    /// Record restart attempt
    pub fn record_restart(&mut self, success: bool) {
        if success {
            self.state.record_restart();
            self.add_event(
                HealingEventType::RestartCompleted,
                &CheckTarget::Daemon,
                true,
                None,
            );
        } else {
            self.add_event(
                HealingEventType::RestartFailed,
                &CheckTarget::Daemon,
                false,
                None,
            );
        }

        // End active trace
        let now = chrono::Utc::now().timestamp();
        if let Some(ref mut trace) = self.active_trace {
            trace.completed_at = Some(now);
            trace.success = success;
        }
        self.active_trace = None;
    }

    /// Start healing trace
    pub fn start_healing_trace(&mut self, component: &str) {
        let trace_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        self.active_trace = Some(HealingTrace {
            trace_id,
            triggered_at: now,
            completed_at: None,
            component: component.to_string(),
            events: Vec::new(),
            success: false,
        });
    }

    /// Get active trace
    pub fn get_trace(&self) -> Option<&HealingTrace> {
        self.active_trace.as_ref()
    }

    /// Add event to history and active trace
    fn add_event(
        &mut self,
        event_type: HealingEventType,
        target: &CheckTarget,
        success: bool,
        details: Option<String>,
    ) {
        let event = HealingEvent {
            timestamp: chrono::Utc::now().timestamp(),
            event_type,
            component: format!("{:?}", target).to_lowercase(),
            action: None,
            success,
            details,
        };

        // Add to history
        self.history.push_back(event.clone());
        while self.history.len() > MAX_HISTORY_EVENTS {
            self.history.pop_front();
        }

        // Add to active trace
        if let Some(ref mut trace) = self.active_trace {
            trace.events.push(event);
        }
    }

    /// Get recent events
    pub fn recent_events(&self, count: usize) -> Vec<&HealingEvent> {
        self.history.iter().rev().take(count).collect()
    }

    /// Get overall health status
    pub fn overall_health(&self) -> HealthCheckResult {
        let critical_failed = self
            .last_results
            .iter()
            .any(|(_, r)| matches!(r, HealthCheckResult::Failed { .. }));

        let any_degraded = self
            .last_results
            .iter()
            .any(|(_, r)| matches!(r, HealthCheckResult::Degraded { .. }));

        if critical_failed {
            HealthCheckResult::Failed {
                reason: "Critical health check failed".to_string(),
            }
        } else if any_degraded {
            HealthCheckResult::Degraded {
                reason: "Some health checks degraded".to_string(),
            }
        } else if self.last_results.is_empty() {
            HealthCheckResult::Degraded {
                reason: "No health checks performed yet".to_string(),
            }
        } else {
            HealthCheckResult::Healthy
        }
    }

    /// Get restart statistics
    pub fn restart_stats(&self) -> RestartStats {
        RestartStats {
            total_restarts: self.history
                .iter()
                .filter(|e| e.event_type == HealingEventType::RestartCompleted)
                .count(),
            restarts_this_hour: self.state.restarts_this_hour as usize,
            last_restart: self.state.last_restart,
            consecutive_failures: self.state.consecutive_failures as usize,
        }
    }
}

/// Restart statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartStats {
    /// Total restarts since tracking started
    pub total_restarts: usize,
    /// Restarts in current hour
    pub restarts_this_hour: usize,
    /// Last restart timestamp
    pub last_restart: Option<i64>,
    /// Current consecutive failures
    pub consecutive_failures: usize,
}

// ============================================================================
// HEALTH CHECK IMPLEMENTATIONS
// ============================================================================

/// Check daemon API health
pub fn check_daemon_api(timeout_secs: u64) -> HealthCheckResult {
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = "127.0.0.1:7865";
    match TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        Duration::from_secs(timeout_secs),
    ) {
        Ok(_) => HealthCheckResult::Healthy,
        Err(e) => HealthCheckResult::Failed {
            reason: format!("Cannot connect to daemon: {}", e),
        },
    }
}

/// Check LLM backend health
pub fn check_llm_backend(timeout_secs: u64) -> HealthCheckResult {
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = "127.0.0.1:11434";
    match TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        Duration::from_secs(timeout_secs),
    ) {
        Ok(_) => HealthCheckResult::Healthy,
        Err(e) => HealthCheckResult::Failed {
            reason: format!("Cannot connect to LLM backend: {}", e),
        },
    }
}

/// Check IPC socket exists
pub fn check_ipc_socket() -> HealthCheckResult {
    let socket_path = std::path::Path::new("/run/anna/annad.sock");
    if socket_path.exists() {
        HealthCheckResult::Healthy
    } else {
        HealthCheckResult::Failed {
            reason: "IPC socket not found".to_string(),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_default() {
        let watchdog = DaemonWatchdog::default();
        assert!(watchdog.config.enabled);
        assert_eq!(watchdog.checks.len(), 2);
    }

    #[test]
    fn test_watchdog_should_check() {
        let mut watchdog = DaemonWatchdog::default();

        // First check should always run
        assert!(watchdog.should_check());

        // Record a check
        watchdog.state.last_check = Some(chrono::Utc::now().timestamp());
        assert!(!watchdog.should_check());

        // After interval passes
        watchdog.state.last_check =
            Some(chrono::Utc::now().timestamp() - watchdog.config.check_interval_secs as i64 - 1);
        assert!(watchdog.should_check());
    }

    #[test]
    fn test_watchdog_record_check() {
        let mut watchdog = DaemonWatchdog::default();

        // Record healthy check
        watchdog.record_check(CheckTarget::DaemonApi, HealthCheckResult::Healthy);
        assert_eq!(watchdog.state.consecutive_failures, 0);
        assert!(!watchdog.history.is_empty());

        // Record failed check
        watchdog.record_check(
            CheckTarget::DaemonApi,
            HealthCheckResult::Failed {
                reason: "test".to_string(),
            },
        );
        assert_eq!(watchdog.state.consecutive_failures, 1);
    }

    #[test]
    fn test_watchdog_needs_restart() {
        let mut watchdog = DaemonWatchdog::default();
        watchdog.config.max_failures = 3;
        watchdog.config.restart_cooldown_secs = 0;

        // Not enough failures
        watchdog.state.consecutive_failures = 2;
        assert!(!watchdog.needs_restart());

        // Enough failures
        watchdog.state.consecutive_failures = 3;
        assert!(watchdog.needs_restart());
    }

    #[test]
    fn test_watchdog_restart_stats() {
        let mut watchdog = DaemonWatchdog::default();
        watchdog.state.restarts_this_hour = 2;
        watchdog.state.consecutive_failures = 1;

        let stats = watchdog.restart_stats();
        assert_eq!(stats.restarts_this_hour, 2);
        assert_eq!(stats.consecutive_failures, 1);
    }

    #[test]
    fn test_overall_health() {
        let mut watchdog = DaemonWatchdog::default();

        // No checks yet
        assert!(matches!(
            watchdog.overall_health(),
            HealthCheckResult::Degraded { .. }
        ));

        // Healthy check
        watchdog.last_results.push((CheckTarget::DaemonApi, HealthCheckResult::Healthy));
        assert!(watchdog.overall_health().is_healthy());

        // Failed check
        watchdog.last_results.push((
            CheckTarget::LlmBackend,
            HealthCheckResult::Failed {
                reason: "test".to_string(),
            },
        ));
        assert!(watchdog.overall_health().is_failed());
    }

    #[test]
    fn test_healing_trace() {
        let mut watchdog = DaemonWatchdog::default();

        watchdog.start_healing_trace("daemon");
        assert!(watchdog.active_trace.is_some());

        watchdog.record_restart(true);
        assert!(watchdog.active_trace.is_none());
        assert_eq!(watchdog.state.restarts_this_hour, 1);
    }

    #[test]
    fn test_recent_events() {
        let mut watchdog = DaemonWatchdog::default();

        for _ in 0..5 {
            watchdog.record_check(CheckTarget::DaemonApi, HealthCheckResult::Healthy);
        }

        let recent = watchdog.recent_events(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_health_check_result() {
        assert!(HealthCheckResult::Healthy.is_healthy());
        assert!(!HealthCheckResult::Healthy.is_failed());

        let failed = HealthCheckResult::Failed {
            reason: "test".to_string(),
        };
        assert!(!failed.is_healthy());
        assert!(failed.is_failed());
    }
}
