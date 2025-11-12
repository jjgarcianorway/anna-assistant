//! Task registry for supervisor

use super::{BackoffConfig, BackoffState, CircuitBreaker};
use std::collections::HashMap;
use std::time::Duration;

/// Task status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is healthy
    Healthy,
    /// Task is backing off after failure
    BackingOff,
    /// Task circuit breaker is open
    CircuitOpen,
}

/// Entry for a supervised task
#[derive(Debug, Clone)]
struct TaskEntry {
    backoff: BackoffState,
    circuit: CircuitBreaker,
    status: TaskStatus,
}

/// Registry of supervised tasks
pub struct TaskRegistry {
    tasks: HashMap<String, TaskEntry>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    /// Register a new task
    pub fn register(&mut self, task_id: String, backoff_config: BackoffConfig) {
        self.tasks.insert(
            task_id,
            TaskEntry {
                backoff: BackoffState::new(backoff_config),
                circuit: CircuitBreaker::default(),
                status: TaskStatus::Healthy,
            },
        );
    }

    /// Record task failure and return backoff duration
    pub fn record_failure(&mut self, task_id: &str) -> Option<Duration> {
        let entry = self.tasks.get_mut(task_id)?;

        entry.circuit.record_failure();

        if entry.circuit.is_open() {
            entry.status = TaskStatus::CircuitOpen;
            return None; // Circuit open, don't retry
        }

        entry.status = TaskStatus::BackingOff;
        Some(entry.backoff.next_backoff())
    }

    /// Record task success
    pub fn record_success(&mut self, task_id: &str) {
        if let Some(entry) = self.tasks.get_mut(task_id) {
            entry.backoff.reset();
            entry.circuit.record_success();
            entry.status = TaskStatus::Healthy;
        }
    }

    /// Check if circuit breaker is open for task
    pub fn is_circuit_open(&self, task_id: &str) -> bool {
        self.tasks
            .get(task_id)
            .map(|e| e.status == TaskStatus::CircuitOpen)
            .unwrap_or(false)
    }

    /// Get task status
    pub fn status(&self, task_id: &str) -> Option<TaskStatus> {
        self.tasks.get(task_id).map(|e| e.status.clone())
    }

    /// Get all registered task IDs
    pub fn task_ids(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_registry() {
        let mut registry = TaskRegistry::new();
        let config = BackoffConfig::default();

        registry.register("task1".to_string(), config);

        assert_eq!(registry.status("task1"), Some(TaskStatus::Healthy));

        // Record failure
        let backoff = registry.record_failure("task1");
        assert!(backoff.is_some());
        assert_eq!(registry.status("task1"), Some(TaskStatus::BackingOff));

        // Record success
        registry.record_success("task1");
        assert_eq!(registry.status("task1"), Some(TaskStatus::Healthy));
    }

    #[test]
    fn test_circuit_breaker_integration() {
        let mut registry = TaskRegistry::new();
        let config = BackoffConfig::default();

        registry.register("task2".to_string(), config);

        // Record many failures to open circuit
        for _ in 0..5 {
            registry.record_failure("task2");
        }

        assert_eq!(registry.status("task2"), Some(TaskStatus::CircuitOpen));
        assert!(registry.is_circuit_open("task2"));

        // Further failures return None (circuit open)
        let backoff = registry.record_failure("task2");
        assert!(backoff.is_none());
    }
}
