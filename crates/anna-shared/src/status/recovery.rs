//! Recovery metrics and health status for self-healing infrastructure.
//!
//! v0.3.36: Phase 8 - Proactive Monitoring and Auto-Healing
//!
//! Anna monitors her own health and attempts automatic recovery without
//! ever asking the user to run manual commands.

use serde::{Deserialize, Serialize};

/// Health state for a subsystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SubsystemHealth {
    /// Subsystem is operating normally
    #[default]
    Healthy,
    /// Subsystem has issues but is functional (degraded performance)
    Degraded,
    /// Subsystem is not functional
    Unavailable,
    /// Subsystem is recovering from a failure
    Recovering,
}

impl std::fmt::Display for SubsystemHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unavailable => write!(f, "unavailable"),
            Self::Recovering => write!(f, "recovering"),
        }
    }
}

/// Recovery attempt outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryOutcome {
    /// Recovery succeeded
    Success,
    /// Recovery failed with reason
    Failed(String),
    /// Recovery skipped (not needed)
    Skipped,
    /// Recovery requires privilege escalation
    NeedsEscalation,
}

/// Individual recovery event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryEvent {
    /// When the recovery was attempted (RFC3339)
    pub timestamp: String,
    /// Which subsystem was recovered
    pub subsystem: String,
    /// What triggered the recovery
    pub trigger: String,
    /// Outcome of the recovery attempt
    pub outcome: RecoveryOutcome,
    /// How long the recovery took (milliseconds)
    pub duration_ms: u64,
}

/// Metrics for a single subsystem's recovery history
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubsystemRecoveryMetrics {
    /// Current health state
    pub health: SubsystemHealth,
    /// Total recovery attempts since daemon start
    pub total_attempts: u32,
    /// Successful recovery attempts
    pub successful_recoveries: u32,
    /// Failed recovery attempts
    pub failed_recoveries: u32,
    /// Last recovery timestamp (RFC3339)
    pub last_recovery: Option<String>,
    /// Last recovery outcome
    pub last_outcome: Option<String>,
    /// Last error message (if degraded/unavailable)
    pub last_error: Option<String>,
    /// Consecutive failures (for circuit breaker)
    pub consecutive_failures: u32,
}

impl SubsystemRecoveryMetrics {
    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f32 {
        if self.total_attempts == 0 {
            100.0 // No attempts = healthy
        } else {
            (self.successful_recoveries as f32 / self.total_attempts as f32) * 100.0
        }
    }

    /// Record a successful recovery
    pub fn record_success(&mut self, timestamp: &str) {
        self.total_attempts += 1;
        self.successful_recoveries += 1;
        self.consecutive_failures = 0;
        self.last_recovery = Some(timestamp.to_string());
        self.last_outcome = Some("success".to_string());
        self.last_error = None;
        self.health = SubsystemHealth::Healthy;
    }

    /// Record a failed recovery
    pub fn record_failure(&mut self, timestamp: &str, error: &str) {
        self.total_attempts += 1;
        self.failed_recoveries += 1;
        self.consecutive_failures += 1;
        self.last_recovery = Some(timestamp.to_string());
        self.last_outcome = Some("failed".to_string());
        self.last_error = Some(error.to_string());

        // Update health based on consecutive failures
        self.health = if self.consecutive_failures >= 3 {
            SubsystemHealth::Unavailable
        } else {
            SubsystemHealth::Degraded
        };
    }

    /// Mark subsystem as recovering
    pub fn mark_recovering(&mut self) {
        self.health = SubsystemHealth::Recovering;
    }
}

/// Overall recovery status for all subsystems
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecoveryStatus {
    /// Daemon connection recovery metrics
    pub daemon: SubsystemRecoveryMetrics,
    /// Ollama service recovery metrics
    pub ollama: SubsystemRecoveryMetrics,
    /// Model loading recovery metrics
    pub models: SubsystemRecoveryMetrics,
    /// Wiki initialization recovery metrics
    pub wiki: SubsystemRecoveryMetrics,
    /// Permission recovery metrics
    pub permissions: SubsystemRecoveryMetrics,
    /// Recent recovery events (last 10)
    #[serde(default)]
    pub recent_events: Vec<RecoveryEvent>,
    /// Total auto-healing actions since daemon start
    #[serde(default)]
    pub total_auto_heals: u32,
}

impl RecoveryStatus {
    /// Calculate overall health (worst of all subsystems)
    pub fn overall_health(&self) -> SubsystemHealth {
        let subsystems = [
            &self.daemon,
            &self.ollama,
            &self.models,
            &self.wiki,
            &self.permissions,
        ];

        // Find worst health state
        let mut worst = SubsystemHealth::Healthy;
        for sub in &subsystems {
            match sub.health {
                SubsystemHealth::Unavailable => return SubsystemHealth::Unavailable,
                SubsystemHealth::Recovering => worst = SubsystemHealth::Recovering,
                SubsystemHealth::Degraded if worst == SubsystemHealth::Healthy => {
                    worst = SubsystemHealth::Degraded
                }
                _ => {}
            }
        }
        worst
    }

    /// Calculate overall success rate
    pub fn overall_success_rate(&self) -> f32 {
        let total_attempts = self.daemon.total_attempts
            + self.ollama.total_attempts
            + self.models.total_attempts
            + self.wiki.total_attempts
            + self.permissions.total_attempts;

        let total_successes = self.daemon.successful_recoveries
            + self.ollama.successful_recoveries
            + self.models.successful_recoveries
            + self.wiki.successful_recoveries
            + self.permissions.successful_recoveries;

        if total_attempts == 0 {
            100.0
        } else {
            (total_successes as f32 / total_attempts as f32) * 100.0
        }
    }

    /// Add a recovery event (keeps last 10)
    pub fn add_event(&mut self, event: RecoveryEvent) {
        self.recent_events.push(event);
        if self.recent_events.len() > 10 {
            self.recent_events.remove(0);
        }
        self.total_auto_heals += 1;
    }

    /// Get current timestamp as RFC3339 string
    pub fn now_rfc3339() -> String {
        chrono::Utc::now().to_rfc3339()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsystem_health_display() {
        assert_eq!(SubsystemHealth::Healthy.to_string(), "healthy");
        assert_eq!(SubsystemHealth::Degraded.to_string(), "degraded");
        assert_eq!(SubsystemHealth::Unavailable.to_string(), "unavailable");
        assert_eq!(SubsystemHealth::Recovering.to_string(), "recovering");
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut metrics = SubsystemRecoveryMetrics::default();
        assert_eq!(metrics.success_rate(), 100.0); // No attempts = 100%

        metrics.record_success("2026-01-13T12:00:00Z");
        assert_eq!(metrics.success_rate(), 100.0);

        metrics.record_failure("2026-01-13T12:01:00Z", "test error");
        assert_eq!(metrics.success_rate(), 50.0);
    }

    #[test]
    fn test_consecutive_failures_health() {
        let mut metrics = SubsystemRecoveryMetrics::default();
        assert_eq!(metrics.health, SubsystemHealth::Healthy);

        metrics.record_failure("t1", "e1");
        assert_eq!(metrics.health, SubsystemHealth::Degraded);
        assert_eq!(metrics.consecutive_failures, 1);

        metrics.record_failure("t2", "e2");
        assert_eq!(metrics.health, SubsystemHealth::Degraded);
        assert_eq!(metrics.consecutive_failures, 2);

        metrics.record_failure("t3", "e3");
        assert_eq!(metrics.health, SubsystemHealth::Unavailable);
        assert_eq!(metrics.consecutive_failures, 3);

        // Success resets
        metrics.record_success("t4");
        assert_eq!(metrics.health, SubsystemHealth::Healthy);
        assert_eq!(metrics.consecutive_failures, 0);
    }

    #[test]
    fn test_overall_health() {
        let mut status = RecoveryStatus::default();
        assert_eq!(status.overall_health(), SubsystemHealth::Healthy);

        status.ollama.health = SubsystemHealth::Degraded;
        assert_eq!(status.overall_health(), SubsystemHealth::Degraded);

        status.wiki.health = SubsystemHealth::Recovering;
        assert_eq!(status.overall_health(), SubsystemHealth::Recovering);

        status.daemon.health = SubsystemHealth::Unavailable;
        assert_eq!(status.overall_health(), SubsystemHealth::Unavailable);
    }

    #[test]
    fn test_recent_events_limit() {
        let mut status = RecoveryStatus::default();
        for i in 0..15 {
            status.add_event(RecoveryEvent {
                timestamp: format!("t{}", i),
                subsystem: "test".to_string(),
                trigger: "test".to_string(),
                outcome: RecoveryOutcome::Success,
                duration_ms: 100,
            });
        }
        assert_eq!(status.recent_events.len(), 10);
        assert_eq!(status.recent_events[0].timestamp, "t5"); // First 5 removed
    }
}
