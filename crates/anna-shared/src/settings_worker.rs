// v0.0.612: Settings Worker (Phase 188)
// Worker pool for settings operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Worker state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorkerState {
    /// Idle
    #[default]
    Idle,
    /// Busy
    Busy,
    /// Paused
    Paused,
    /// Stopped
    Stopped,
    /// Error
    Error,
}

impl std::fmt::Display for WorkerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Busy => write!(f, "busy"),
            Self::Paused => write!(f, "paused"),
            Self::Stopped => write!(f, "stopped"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Worker type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerType {
    /// Read worker
    Reader,
    /// Write worker
    Writer,
    /// Sync worker
    Syncer,
    /// Backup worker
    Backup,
    /// General worker
    General,
}

impl std::fmt::Display for WorkerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reader => write!(f, "reader"),
            Self::Writer => write!(f, "writer"),
            Self::Syncer => write!(f, "syncer"),
            Self::Backup => write!(f, "backup"),
            Self::General => write!(f, "general"),
        }
    }
}

/// Worker config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Worker type
    pub worker_type: WorkerType,
    /// Max concurrent tasks
    pub max_concurrent: usize,
    /// Timeout ms
    pub timeout_ms: u64,
    /// Retry count
    pub max_retries: u32,
    /// Enabled
    pub enabled: bool,
}

impl WorkerConfig {
    /// Create new config
    pub fn new(worker_type: WorkerType) -> Self {
        Self {
            worker_type,
            max_concurrent: 4,
            timeout_ms: 30000,
            max_retries: 3,
            enabled: true,
        }
    }

    /// Set max concurrent
    pub fn max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Set timeout
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Set max retries
    pub fn max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }
}

/// Worker instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    /// Unique ID
    pub id: String,
    /// Config
    pub config: WorkerConfig,
    /// State
    pub state: WorkerState,
    /// Current task ID
    pub current_task: Option<String>,
    /// Tasks completed
    pub tasks_completed: usize,
    /// Tasks failed
    pub tasks_failed: usize,
    /// Started timestamp
    pub started_at: u64,
    /// Last activity
    pub last_activity: u64,
}

impl Worker {
    /// Create new worker
    pub fn new(id: impl Into<String>, config: WorkerConfig) -> Self {
        Self {
            id: id.into(),
            config,
            state: WorkerState::Idle,
            current_task: None,
            tasks_completed: 0,
            tasks_failed: 0,
            started_at: 0,
            last_activity: 0,
        }
    }

    /// Assign task
    pub fn assign(&mut self, task_id: impl Into<String>) {
        self.state = WorkerState::Busy;
        self.current_task = Some(task_id.into());
    }

    /// Complete task
    pub fn complete(&mut self) {
        self.state = WorkerState::Idle;
        self.current_task = None;
        self.tasks_completed += 1;
    }

    /// Fail task
    pub fn fail(&mut self) {
        self.state = WorkerState::Error;
        self.current_task = None;
        self.tasks_failed += 1;
    }

    /// Pause
    pub fn pause(&mut self) {
        self.state = WorkerState::Paused;
    }

    /// Resume
    pub fn resume(&mut self) {
        if self.state == WorkerState::Paused {
            self.state = WorkerState::Idle;
        }
    }

    /// Stop
    pub fn stop(&mut self) {
        self.state = WorkerState::Stopped;
        self.current_task = None;
    }

    /// Is available
    pub fn is_available(&self) -> bool {
        self.config.enabled && self.state == WorkerState::Idle
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.tasks_completed + self.tasks_failed;
        if total == 0 {
            1.0
        } else {
            self.tasks_completed as f64 / total as f64
        }
    }
}

/// Worker pool
#[derive(Debug, Clone, Default)]
pub struct WorkerPool {
    /// Workers
    workers: HashMap<String, Worker>,
    /// Max workers
    max_workers: usize,
}

impl WorkerPool {
    /// Create new pool
    pub fn new() -> Self {
        Self {
            max_workers: 16,
            ..Default::default()
        }
    }

    /// Add worker
    pub fn add(&mut self, worker: Worker) -> bool {
        if self.workers.len() >= self.max_workers {
            return false;
        }
        self.workers.insert(worker.id.clone(), worker);
        true
    }

    /// Remove worker
    pub fn remove(&mut self, id: &str) -> Option<Worker> {
        self.workers.remove(id)
    }

    /// Get worker
    pub fn get(&self, id: &str) -> Option<&Worker> {
        self.workers.get(id)
    }

    /// Get worker mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Worker> {
        self.workers.get_mut(id)
    }

    /// Get available worker
    pub fn available(&self) -> Option<&Worker> {
        self.workers.values().find(|w| w.is_available())
    }

    /// Get available by type
    pub fn available_by_type(&self, worker_type: WorkerType) -> Option<&Worker> {
        self.workers.values()
            .find(|w| w.is_available() && w.config.worker_type == worker_type)
    }

    /// Worker count
    pub fn count(&self) -> usize {
        self.workers.len()
    }

    /// Busy count
    pub fn busy_count(&self) -> usize {
        self.workers.values().filter(|w| w.state == WorkerState::Busy).count()
    }

    /// Idle count
    pub fn idle_count(&self) -> usize {
        self.workers.values().filter(|w| w.state == WorkerState::Idle).count()
    }

    /// Total completed
    pub fn total_completed(&self) -> usize {
        self.workers.values().map(|w| w.tasks_completed).sum()
    }
}

/// Format worker pool
pub fn format_worker_pool(pool: &WorkerPool) -> String {
    let mut output = String::new();
    output.push_str("Worker Pool:\n");
    output.push_str(&format!("  Workers: {}\n", pool.count()));
    output.push_str(&format!("  Busy: {}\n", pool.busy_count()));
    output.push_str(&format!("  Idle: {}\n", pool.idle_count()));
    output.push_str(&format!("  Completed: {}\n", pool.total_completed()));
    output
}

/// Check if query is about worker
pub fn is_worker_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("worker")
        || lower.contains("worker pool")
        || lower.contains("background job")
}

/// Fun fact about worker
pub fn worker_fun_fact() -> &'static str {
    "Anna uses a pool of workers to process settings operations concurrently!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", WorkerState::Idle), "idle");
        assert_eq!(format!("{}", WorkerState::Busy), "busy");
    }

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", WorkerType::Reader), "reader");
        assert_eq!(format!("{}", WorkerType::Writer), "writer");
    }

    #[test]
    fn test_config_new() {
        let c = WorkerConfig::new(WorkerType::General);
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = WorkerConfig::new(WorkerType::Syncer)
            .max_concurrent(8)
            .timeout(60000);
        assert_eq!(c.max_concurrent, 8);
    }

    #[test]
    fn test_worker_new() {
        let w = Worker::new("w1", WorkerConfig::new(WorkerType::Reader));
        assert!(w.is_available());
    }

    #[test]
    fn test_worker_lifecycle() {
        let mut w = Worker::new("w1", WorkerConfig::new(WorkerType::Writer));
        w.assign("task1");
        assert_eq!(w.state, WorkerState::Busy);
        w.complete();
        assert_eq!(w.state, WorkerState::Idle);
        assert_eq!(w.tasks_completed, 1);
    }

    #[test]
    fn test_worker_success_rate() {
        let mut w = Worker::new("w1", WorkerConfig::new(WorkerType::General));
        w.tasks_completed = 9;
        w.tasks_failed = 1;
        assert!((w.success_rate() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_pool_new() {
        let p = WorkerPool::new();
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_pool_add() {
        let mut p = WorkerPool::new();
        p.add(Worker::new("w1", WorkerConfig::new(WorkerType::Reader)));
        assert_eq!(p.count(), 1);
    }

    #[test]
    fn test_pool_available() {
        let mut p = WorkerPool::new();
        p.add(Worker::new("w1", WorkerConfig::new(WorkerType::Reader)));
        assert!(p.available().is_some());
    }

    #[test]
    fn test_is_worker_query() {
        assert!(is_worker_query("worker pool"));
        assert!(!is_worker_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = worker_fun_fact();
        assert!(fact.contains("worker"));
    }
}
