// v0.0.618: Settings Coordinator (Phase 194)
// Coordinate settings operations across components

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Coordinator state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CoordinatorState {
    /// Initializing
    Initializing,
    /// Ready
    #[default]
    Ready,
    /// Coordinating
    Coordinating,
    /// Paused
    Paused,
    /// Shutdown
    Shutdown,
}

impl std::fmt::Display for CoordinatorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initializing => write!(f, "initializing"),
            Self::Ready => write!(f, "ready"),
            Self::Coordinating => write!(f, "coordinating"),
            Self::Paused => write!(f, "paused"),
            Self::Shutdown => write!(f, "shutdown"),
        }
    }
}

/// Component type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    /// Queue component
    Queue,
    /// Worker component
    Worker,
    /// Executor component
    Executor,
    /// Pipeline component
    Pipeline,
    /// Handler component
    Handler,
    /// Dispatcher component
    Dispatcher,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queue => write!(f, "queue"),
            Self::Worker => write!(f, "worker"),
            Self::Executor => write!(f, "executor"),
            Self::Pipeline => write!(f, "pipeline"),
            Self::Handler => write!(f, "handler"),
            Self::Dispatcher => write!(f, "dispatcher"),
        }
    }
}

/// Component status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ComponentStatus {
    /// Unknown
    #[default]
    Unknown,
    /// Healthy
    Healthy,
    /// Degraded
    Degraded,
    /// Unhealthy
    Unhealthy,
    /// Offline
    Offline,
}

impl std::fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
            Self::Offline => write!(f, "offline"),
        }
    }
}

/// Component info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    /// Component type
    pub component_type: ComponentType,
    /// Status
    pub status: ComponentStatus,
    /// Last check
    pub last_check: u64,
    /// Message
    pub message: Option<String>,
}

impl ComponentInfo {
    /// Create new info
    pub fn new(component_type: ComponentType) -> Self {
        Self {
            component_type,
            status: ComponentStatus::Unknown,
            last_check: 0,
            message: None,
        }
    }

    /// Set status
    pub fn status(mut self, status: ComponentStatus) -> Self {
        self.status = status;
        self
    }

    /// Set message
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Update check
    pub fn update(&mut self, status: ComponentStatus, timestamp: u64) {
        self.status = status;
        self.last_check = timestamp;
    }

    /// Is healthy
    pub fn is_healthy(&self) -> bool {
        self.status == ComponentStatus::Healthy
    }
}

/// Coordination task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationTask {
    /// Unique ID
    pub id: String,
    /// Description
    pub description: String,
    /// Components involved
    pub components: Vec<ComponentType>,
    /// Started timestamp
    pub started_at: u64,
    /// Completed timestamp
    pub completed_at: Option<u64>,
    /// Success
    pub success: Option<bool>,
}

impl CoordinationTask {
    /// Create new task
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            components: Vec::new(),
            started_at: 0,
            completed_at: None,
            success: None,
        }
    }

    /// Add component
    pub fn component(mut self, component: ComponentType) -> Self {
        self.components.push(component);
        self
    }

    /// Start task
    pub fn start(&mut self, timestamp: u64) {
        self.started_at = timestamp;
    }

    /// Complete task
    pub fn complete(&mut self, timestamp: u64, success: bool) {
        self.completed_at = Some(timestamp);
        self.success = Some(success);
    }

    /// Is complete
    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some()
    }
}

/// Settings coordinator
#[derive(Debug, Clone, Default)]
pub struct SettingsCoordinator {
    /// State
    state: CoordinatorState,
    /// Components
    components: HashMap<ComponentType, ComponentInfo>,
    /// Active tasks
    active_tasks: HashMap<String, CoordinationTask>,
    /// Task history
    history: Vec<CoordinationTask>,
    /// Max history
    max_history: usize,
}

impl SettingsCoordinator {
    /// Create new coordinator
    pub fn new() -> Self {
        Self {
            max_history: 100,
            ..Default::default()
        }
    }

    /// Get state
    pub fn state(&self) -> CoordinatorState {
        self.state
    }

    /// Start coordinating
    pub fn start(&mut self) {
        self.state = CoordinatorState::Coordinating;
    }

    /// Pause
    pub fn pause(&mut self) {
        self.state = CoordinatorState::Paused;
    }

    /// Resume
    pub fn resume(&mut self) {
        if self.state == CoordinatorState::Paused {
            self.state = CoordinatorState::Coordinating;
        }
    }

    /// Shutdown
    pub fn shutdown(&mut self) {
        self.state = CoordinatorState::Shutdown;
    }

    /// Register component
    pub fn register_component(&mut self, info: ComponentInfo) {
        self.components.insert(info.component_type, info);
    }

    /// Update component
    pub fn update_component(&mut self, component_type: ComponentType, status: ComponentStatus, timestamp: u64) {
        if let Some(info) = self.components.get_mut(&component_type) {
            info.update(status, timestamp);
        }
    }

    /// Get component
    pub fn get_component(&self, component_type: ComponentType) -> Option<&ComponentInfo> {
        self.components.get(&component_type)
    }

    /// Start task
    pub fn start_task(&mut self, task: CoordinationTask) {
        self.active_tasks.insert(task.id.clone(), task);
    }

    /// Complete task
    pub fn complete_task(&mut self, id: &str, timestamp: u64, success: bool) {
        if let Some(mut task) = self.active_tasks.remove(id) {
            task.complete(timestamp, success);
            self.history.push(task);
            while self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
    }

    /// Component count
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Healthy count
    pub fn healthy_count(&self) -> usize {
        self.components.values().filter(|c| c.is_healthy()).count()
    }

    /// Active task count
    pub fn active_task_count(&self) -> usize {
        self.active_tasks.len()
    }
}

/// Format coordinator
pub fn format_coordinator(coordinator: &SettingsCoordinator) -> String {
    let mut output = String::new();
    output.push_str("Settings Coordinator:\n");
    output.push_str(&format!("  State: {}\n", coordinator.state()));
    output.push_str(&format!("  Components: {}\n", coordinator.component_count()));
    output.push_str(&format!("  Healthy: {}\n", coordinator.healthy_count()));
    output.push_str(&format!("  Active Tasks: {}\n", coordinator.active_task_count()));
    output
}

/// Check if query is about coordinator
pub fn is_coordinator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("coordinator")
        || lower.contains("coordinate")
        || lower.contains("orchestrate")
}

/// Fun fact about coordinator
pub fn coordinator_fun_fact() -> &'static str {
    "Anna's coordinator ensures all settings components work together harmoniously!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", CoordinatorState::Ready), "ready");
        assert_eq!(format!("{}", CoordinatorState::Coordinating), "coordinating");
    }

    #[test]
    fn test_component_type_display() {
        assert_eq!(format!("{}", ComponentType::Queue), "queue");
        assert_eq!(format!("{}", ComponentType::Worker), "worker");
    }

    #[test]
    fn test_component_status_display() {
        assert_eq!(format!("{}", ComponentStatus::Healthy), "healthy");
        assert_eq!(format!("{}", ComponentStatus::Degraded), "degraded");
    }

    #[test]
    fn test_component_info_new() {
        let i = ComponentInfo::new(ComponentType::Queue);
        assert_eq!(i.status, ComponentStatus::Unknown);
    }

    #[test]
    fn test_component_info_healthy() {
        let i = ComponentInfo::new(ComponentType::Worker).status(ComponentStatus::Healthy);
        assert!(i.is_healthy());
    }

    #[test]
    fn test_task_new() {
        let t = CoordinationTask::new("t1", "Test task");
        assert!(!t.is_complete());
    }

    #[test]
    fn test_task_lifecycle() {
        let mut t = CoordinationTask::new("t1", "Test");
        t.start(100);
        t.complete(200, true);
        assert!(t.is_complete());
    }

    #[test]
    fn test_coordinator_new() {
        let c = SettingsCoordinator::new();
        assert_eq!(c.state(), CoordinatorState::Ready);
    }

    #[test]
    fn test_coordinator_lifecycle() {
        let mut c = SettingsCoordinator::new();
        c.start();
        assert_eq!(c.state(), CoordinatorState::Coordinating);
        c.pause();
        assert_eq!(c.state(), CoordinatorState::Paused);
    }

    #[test]
    fn test_coordinator_register_component() {
        let mut c = SettingsCoordinator::new();
        c.register_component(ComponentInfo::new(ComponentType::Queue));
        assert_eq!(c.component_count(), 1);
    }

    #[test]
    fn test_is_coordinator_query() {
        assert!(is_coordinator_query("settings coordinator"));
        assert!(!is_coordinator_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = coordinator_fun_fact();
        assert!(fact.contains("coordinator"));
    }
}
