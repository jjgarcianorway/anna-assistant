//! Autonomous Recovery Supervisor (Phase 2)
//!
//! Self-healing task supervision with exponential backoff and circuit breakers.
//!
//! Features:
//! - Exponential backoff with jitter for failed tasks
//! - Circuit breaker pattern for repeated failures
//! - Metrics: anna_recovery_actions_total, anna_task_restart_total
//! - Configuration via /etc/anna/supervisor.yml

pub mod backoff;
pub mod circuit;
pub mod registry;

pub use backoff::{BackoffConfig, BackoffState};
pub use circuit::{CircuitBreaker, CircuitState};
pub use registry::{TaskRegistry, TaskStatus};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Supervisor for autonomous task recovery
pub struct Supervisor {
    registry: Arc<RwLock<TaskRegistry>>,
    backoff_config: BackoffConfig,
}

impl Supervisor {
    pub fn new(backoff_config: BackoffConfig) -> Self {
        Self {
            registry: Arc::new(RwLock::new(TaskRegistry::new())),
            backoff_config,
        }
    }

    /// Register a task for supervision
    pub async fn register_task(&self, task_id: String) {
        let mut registry = self.registry.write().await;
        registry.register(task_id, self.backoff_config.clone());
    }

    /// Record task failure and get backoff duration
    pub async fn record_failure(&self, task_id: &str) -> Option<std::time::Duration> {
        let mut registry = self.registry.write().await;
        registry.record_failure(task_id)
    }

    /// Record task success (resets backoff)
    pub async fn record_success(&self, task_id: &str) {
        let mut registry = self.registry.write().await;
        registry.record_success(task_id);
    }

    /// Check if circuit breaker is open for task
    pub async fn is_circuit_open(&self, task_id: &str) -> bool {
        let registry = self.registry.read().await;
        registry.is_circuit_open(task_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_supervisor_basic() {
        let config = BackoffConfig::default();
        let supervisor = Supervisor::new(config);

        supervisor.register_task("test_task".to_string()).await;

        // First failure
        let backoff1 = supervisor.record_failure("test_task").await;
        assert!(backoff1.is_some());

        // Second failure
        let backoff2 = supervisor.record_failure("test_task").await;
        assert!(backoff2.is_some());
        assert!(backoff2.unwrap() >= backoff1.unwrap());

        // Success resets
        supervisor.record_success("test_task").await;
        let backoff3 = supervisor.record_failure("test_task").await;
        assert!(backoff3.is_some());
        assert!(backoff3.unwrap() <= backoff2.unwrap());
    }
}
