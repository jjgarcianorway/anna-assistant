// Self-Healing Framework
// Phase 3.1/3.2: Autonomous Service Recovery
//
// Provides detection, recovery, and logging for failed services.
// Foundation for future autonomous operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceHealth {
    /// Service is running normally
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service has failed
    Failed,
    /// Service status unknown
    Unknown,
}

/// Recovery action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Restart the service
    Restart,
    /// Reload configuration
    Reload,
    /// Stop and start (cold restart)
    StopStart,
    /// Manual intervention required
    Manual,
}

/// Recovery attempt outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryOutcome {
    /// Recovery succeeded
    Success,
    /// Recovery failed
    Failure,
    /// Recovery partially succeeded
    Partial,
    /// Recovery skipped (manual intervention required)
    Skipped,
}

/// Service recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRecoveryConfig {
    /// Service name (e.g., "prometheus", "grafana")
    pub name: String,

    /// Maximum restart attempts before giving up
    pub max_restart_attempts: u32,

    /// Cooldown period between restarts (seconds)
    pub restart_cooldown_secs: u64,

    /// Whether to attempt automatic recovery
    pub auto_recover: bool,

    /// Recovery action to take
    pub recovery_action: RecoveryAction,

    /// Critical service (system won't function without it)
    pub is_critical: bool,

    /// Dependencies (services that must be running first)
    pub dependencies: Vec<String>,
}

impl ServiceRecoveryConfig {
    /// Create default config for a service
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            max_restart_attempts: 3,
            restart_cooldown_secs: 60,
            auto_recover: true,
            recovery_action: RecoveryAction::Restart,
            is_critical: false,
            dependencies: Vec::new(),
        }
    }

    /// Mark service as critical
    pub fn critical(mut self) -> Self {
        self.is_critical = true;
        self
    }

    /// Set maximum restart attempts
    pub fn with_max_attempts(mut self, max: u32) -> Self {
        self.max_restart_attempts = max;
        self
    }

    /// Add dependency
    pub fn depends_on(mut self, service: impl Into<String>) -> Self {
        self.dependencies.push(service.into());
        self
    }

    /// Disable automatic recovery
    pub fn manual_recovery(mut self) -> Self {
        self.auto_recover = false;
        self.recovery_action = RecoveryAction::Manual;
        self
    }
}

/// Recovery attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    /// Unique ID for this attempt
    pub id: String,

    /// Service name
    pub service: String,

    /// Timestamp of attempt
    pub timestamp: DateTime<Utc>,

    /// Action taken
    pub action: RecoveryAction,

    /// Outcome of the attempt
    pub outcome: RecoveryOutcome,

    /// Duration of recovery attempt (milliseconds)
    pub duration_ms: u64,

    /// Error message if recovery failed
    pub error: Option<String>,

    /// Attempt number (1, 2, 3, ...)
    pub attempt_number: u32,
}

impl RecoveryAttempt {
    /// Create new recovery attempt
    pub fn new(service: impl Into<String>, action: RecoveryAction, attempt_number: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            service: service.into(),
            timestamp: Utc::now(),
            action,
            outcome: RecoveryOutcome::Success,
            duration_ms: 0,
            error: None,
            attempt_number,
        }
    }

    /// Mark attempt as failed
    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.outcome = RecoveryOutcome::Failure;
        self.error = Some(error.into());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

/// Service healing manager
pub struct SelfHealingManager {
    /// Service configurations
    configs: HashMap<String, ServiceRecoveryConfig>,

    /// Recent recovery attempts
    recovery_history: Vec<RecoveryAttempt>,

    /// Maximum history size
    max_history_size: usize,
}

impl SelfHealingManager {
    /// Create new self-healing manager
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            recovery_history: Vec::new(),
            max_history_size: 1000,
        }
    }

    /// Register a service for monitoring
    pub fn register_service(&mut self, config: ServiceRecoveryConfig) {
        self.configs.insert(config.name.clone(), config);
    }

    /// Get service configuration
    pub fn get_config(&self, service: &str) -> Option<&ServiceRecoveryConfig> {
        self.configs.get(service)
    }

    /// Check if service should be auto-recovered
    pub fn should_auto_recover(&self, service: &str) -> bool {
        self.configs
            .get(service)
            .map(|c| c.auto_recover)
            .unwrap_or(false)
    }

    /// Get recent attempts for a service
    pub fn get_recent_attempts(&self, service: &str, limit: usize) -> Vec<&RecoveryAttempt> {
        self.recovery_history
            .iter()
            .rev()
            .filter(|a| a.service == service)
            .take(limit)
            .collect()
    }

    /// Record recovery attempt
    pub fn record_attempt(&mut self, attempt: RecoveryAttempt) {
        self.recovery_history.push(attempt);

        // Trim history if too large
        if self.recovery_history.len() > self.max_history_size {
            self.recovery_history.drain(0..100);
        }
    }

    /// Get recovery success rate for a service
    pub fn get_success_rate(&self, service: &str) -> f64 {
        let attempts: Vec<_> = self
            .recovery_history
            .iter()
            .filter(|a| a.service == service)
            .collect();

        if attempts.is_empty() {
            return 0.0;
        }

        let successes = attempts
            .iter()
            .filter(|a| a.outcome == RecoveryOutcome::Success)
            .count();

        (successes as f64 / attempts.len() as f64) * 100.0
    }

    /// Check if service is in cooldown period
    pub fn is_in_cooldown(&self, service: &str) -> bool {
        if let Some(config) = self.configs.get(service) {
            if let Some(last_attempt) = self.get_recent_attempts(service, 1).first() {
                let elapsed = Utc::now()
                    .signed_duration_since(last_attempt.timestamp)
                    .num_seconds() as u64;
                return elapsed < config.restart_cooldown_secs;
            }
        }
        false
    }

    /// Check if max attempts reached
    pub fn max_attempts_reached(&self, service: &str) -> bool {
        if let Some(config) = self.configs.get(service) {
            let recent_failures = self
                .get_recent_attempts(service, config.max_restart_attempts as usize)
                .iter()
                .filter(|a| a.outcome == RecoveryOutcome::Failure)
                .count();

            return recent_failures >= config.max_restart_attempts as usize;
        }
        false
    }

    /// Get all registered services
    pub fn list_services(&self) -> Vec<&str> {
        self.configs.keys().map(|s| s.as_str()).collect()
    }

    /// Get all critical services
    pub fn list_critical_services(&self) -> Vec<&str> {
        self.configs
            .values()
            .filter(|c| c.is_critical)
            .map(|c| c.name.as_str())
            .collect()
    }
}

impl Default for SelfHealingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Default service configurations for common services
pub fn default_service_configs() -> Vec<ServiceRecoveryConfig> {
    vec![
        // Anna daemon - critical
        ServiceRecoveryConfig::new("annad")
            .critical()
            .with_max_attempts(5),
        // Prometheus - monitoring
        ServiceRecoveryConfig::new("prometheus").with_max_attempts(3),
        // Grafana - visualization
        ServiceRecoveryConfig::new("grafana")
            .with_max_attempts(3)
            .depends_on("prometheus"),
        // Systemd services
        ServiceRecoveryConfig::new("systemd-resolved").critical(),
        ServiceRecoveryConfig::new("systemd-networkd"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_recovery_config() {
        let config = ServiceRecoveryConfig::new("test-service")
            .critical()
            .with_max_attempts(5)
            .depends_on("other-service");

        assert_eq!(config.name, "test-service");
        assert!(config.is_critical);
        assert_eq!(config.max_restart_attempts, 5);
        assert_eq!(config.dependencies, vec!["other-service"]);
    }

    #[test]
    fn test_recovery_attempt() {
        let attempt =
            RecoveryAttempt::new("test-service", RecoveryAction::Restart, 1).with_duration(1500);

        assert_eq!(attempt.service, "test-service");
        assert_eq!(attempt.action, RecoveryAction::Restart);
        assert_eq!(attempt.attempt_number, 1);
        assert_eq!(attempt.duration_ms, 1500);
        assert_eq!(attempt.outcome, RecoveryOutcome::Success);
    }

    #[test]
    fn test_self_healing_manager() {
        let mut manager = SelfHealingManager::new();

        let config = ServiceRecoveryConfig::new("test-service");
        manager.register_service(config);

        assert!(manager.should_auto_recover("test-service"));
        assert!(!manager.should_auto_recover("unknown-service"));

        // Record attempts
        let attempt1 = RecoveryAttempt::new("test-service", RecoveryAction::Restart, 1);
        manager.record_attempt(attempt1);

        let attempt2 =
            RecoveryAttempt::new("test-service", RecoveryAction::Restart, 2).failed("Test error");
        manager.record_attempt(attempt2);

        let recent = manager.get_recent_attempts("test-service", 10);
        assert_eq!(recent.len(), 2);

        let success_rate = manager.get_success_rate("test-service");
        assert_eq!(success_rate, 50.0);
    }

    #[test]
    fn test_max_attempts_check() {
        let mut manager = SelfHealingManager::new();

        let config = ServiceRecoveryConfig::new("test-service").with_max_attempts(3);
        manager.register_service(config);

        // Record 3 failures
        for i in 1..=3 {
            let attempt =
                RecoveryAttempt::new("test-service", RecoveryAction::Restart, i).failed("Error");
            manager.record_attempt(attempt);
        }

        assert!(manager.max_attempts_reached("test-service"));
    }

    #[test]
    fn test_default_configs() {
        let configs = default_service_configs();
        assert!(!configs.is_empty());

        let annad_config = configs.iter().find(|c| c.name == "annad").unwrap();
        assert!(annad_config.is_critical);
        assert_eq!(annad_config.max_restart_attempts, 5);
    }
}
