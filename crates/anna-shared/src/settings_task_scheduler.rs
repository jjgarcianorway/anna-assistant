// v0.0.610: Settings Task Scheduler (Phase 186)
// Advanced task scheduling for settings operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Task frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskFrequency {
    /// Run once
    #[default]
    Once,
    /// Every minute
    Minutely,
    /// Every hour
    Hourly,
    /// Every day
    Daily,
    /// Every week
    Weekly,
    /// Every month
    Monthly,
}

impl std::fmt::Display for TaskFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Once => write!(f, "once"),
            Self::Minutely => write!(f, "minutely"),
            Self::Hourly => write!(f, "hourly"),
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Monthly => write!(f, "monthly"),
        }
    }
}

/// Task type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// Backup task
    Backup,
    /// Sync task
    Sync,
    /// Validation task
    Validation,
    /// Report task
    Report,
    /// Cleanup task
    Cleanup,
    /// Custom task
    Custom,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Backup => write!(f, "backup"),
            Self::Sync => write!(f, "sync"),
            Self::Validation => write!(f, "validation"),
            Self::Report => write!(f, "report"),
            Self::Cleanup => write!(f, "cleanup"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskState {
    /// Pending
    #[default]
    Pending,
    /// Running
    Running,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Scheduled task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    /// Unique ID
    pub id: String,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Task type
    pub task_type: TaskType,
    /// Frequency
    pub frequency: TaskFrequency,
    /// Categories
    pub categories: Vec<SettingsCategory>,
    /// Enabled
    pub enabled: bool,
    /// Priority (lower is higher)
    pub priority: u32,
}

impl TaskDefinition {
    /// Create new definition
    pub fn new(id: impl Into<String>, task_type: TaskType) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            description: String::new(),
            task_type,
            frequency: TaskFrequency::Once,
            categories: Vec::new(),
            enabled: true,
            priority: 100,
        }
    }

    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set frequency
    pub fn frequency(mut self, freq: TaskFrequency) -> Self {
        self.frequency = freq;
        self
    }

    /// Add category
    pub fn category(mut self, cat: SettingsCategory) -> Self {
        self.categories.push(cat);
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

/// Task instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInstance {
    /// Instance ID
    pub instance_id: String,
    /// Definition ID
    pub definition_id: String,
    /// State
    pub state: TaskState,
    /// Scheduled time
    pub scheduled_at: u64,
    /// Started time
    pub started_at: Option<u64>,
    /// Completed time
    pub completed_at: Option<u64>,
    /// Result message
    pub result: Option<String>,
}

impl TaskInstance {
    /// Create new instance
    pub fn new(instance_id: impl Into<String>, definition_id: impl Into<String>) -> Self {
        Self {
            instance_id: instance_id.into(),
            definition_id: definition_id.into(),
            state: TaskState::Pending,
            scheduled_at: 0,
            started_at: None,
            completed_at: None,
            result: None,
        }
    }

    /// Set scheduled time
    pub fn scheduled_at(mut self, ts: u64) -> Self {
        self.scheduled_at = ts;
        self
    }

    /// Start task
    pub fn start(&mut self, ts: u64) {
        self.state = TaskState::Running;
        self.started_at = Some(ts);
    }

    /// Complete task
    pub fn complete(&mut self, ts: u64, result: impl Into<String>) {
        self.state = TaskState::Completed;
        self.completed_at = Some(ts);
        self.result = Some(result.into());
    }

    /// Fail task
    pub fn fail(&mut self, ts: u64, error: impl Into<String>) {
        self.state = TaskState::Failed;
        self.completed_at = Some(ts);
        self.result = Some(error.into());
    }

    /// Cancel task
    pub fn cancel(&mut self) {
        self.state = TaskState::Cancelled;
    }
}

/// Task scheduler
#[derive(Debug, Clone, Default)]
pub struct SettingsTaskScheduler {
    /// Definitions
    definitions: HashMap<String, TaskDefinition>,
    /// Instances
    instances: Vec<TaskInstance>,
    /// Max instances
    max_instances: usize,
}

impl SettingsTaskScheduler {
    /// Create new scheduler
    pub fn new() -> Self {
        Self {
            max_instances: 500,
            ..Default::default()
        }
    }

    /// Add definition
    pub fn add_definition(&mut self, def: TaskDefinition) {
        self.definitions.insert(def.id.clone(), def);
    }

    /// Remove definition
    pub fn remove_definition(&mut self, id: &str) -> Option<TaskDefinition> {
        self.definitions.remove(id)
    }

    /// Get definition
    pub fn get_definition(&self, id: &str) -> Option<&TaskDefinition> {
        self.definitions.get(id)
    }

    /// Schedule instance
    pub fn schedule(&mut self, instance: TaskInstance) {
        self.instances.push(instance);
        while self.instances.len() > self.max_instances {
            self.instances.remove(0);
        }
    }

    /// Get pending instances
    pub fn pending(&self) -> Vec<&TaskInstance> {
        self.instances.iter().filter(|i| i.state == TaskState::Pending).collect()
    }

    /// Get running instances
    pub fn running(&self) -> Vec<&TaskInstance> {
        self.instances.iter().filter(|i| i.state == TaskState::Running).collect()
    }

    /// Definition count
    pub fn definition_count(&self) -> usize {
        self.definitions.len()
    }

    /// Instance count
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// Get instance mut
    pub fn get_instance_mut(&mut self, id: &str) -> Option<&mut TaskInstance> {
        self.instances.iter_mut().find(|i| i.instance_id == id)
    }
}

/// Format scheduler
pub fn format_task_scheduler(scheduler: &SettingsTaskScheduler) -> String {
    let mut output = String::new();
    output.push_str("Settings Task Scheduler:\n");
    output.push_str(&format!("  Definitions: {}\n", scheduler.definition_count()));
    output.push_str(&format!("  Instances: {}\n", scheduler.instance_count()));
    output.push_str(&format!("  Pending: {}\n", scheduler.pending().len()));
    output.push_str(&format!("  Running: {}\n", scheduler.running().len()));
    output
}

/// Check if query is about task scheduler
pub fn is_task_scheduler_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("task schedule")
        || lower.contains("scheduled task")
        || lower.contains("task queue")
        || lower.contains("background task")
}

/// Fun fact about task scheduler
pub fn task_scheduler_fun_fact() -> &'static str {
    "Anna can schedule background tasks to maintain your settings automatically!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_display() {
        assert_eq!(format!("{}", TaskFrequency::Daily), "daily");
        assert_eq!(format!("{}", TaskFrequency::Hourly), "hourly");
    }

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", TaskType::Backup), "backup");
        assert_eq!(format!("{}", TaskType::Sync), "sync");
    }

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", TaskState::Running), "running");
        assert_eq!(format!("{}", TaskState::Completed), "completed");
    }

    #[test]
    fn test_definition_new() {
        let d = TaskDefinition::new("d1", TaskType::Backup);
        assert!(d.enabled);
    }

    #[test]
    fn test_definition_builder() {
        let d = TaskDefinition::new("d1", TaskType::Sync)
            .name("Daily Sync")
            .frequency(TaskFrequency::Daily)
            .priority(10);
        assert_eq!(d.priority, 10);
    }

    #[test]
    fn test_instance_new() {
        let i = TaskInstance::new("i1", "d1");
        assert_eq!(i.state, TaskState::Pending);
    }

    #[test]
    fn test_instance_lifecycle() {
        let mut i = TaskInstance::new("i1", "d1");
        i.start(100);
        assert_eq!(i.state, TaskState::Running);
        i.complete(200, "Done");
        assert_eq!(i.state, TaskState::Completed);
    }

    #[test]
    fn test_scheduler_new() {
        let s = SettingsTaskScheduler::new();
        assert_eq!(s.definition_count(), 0);
    }

    #[test]
    fn test_scheduler_add_definition() {
        let mut s = SettingsTaskScheduler::new();
        s.add_definition(TaskDefinition::new("d1", TaskType::Backup));
        assert_eq!(s.definition_count(), 1);
    }

    #[test]
    fn test_scheduler_schedule() {
        let mut s = SettingsTaskScheduler::new();
        s.schedule(TaskInstance::new("i1", "d1"));
        assert_eq!(s.instance_count(), 1);
    }

    #[test]
    fn test_is_task_scheduler_query() {
        assert!(is_task_scheduler_query("scheduled task"));
        assert!(!is_task_scheduler_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = task_scheduler_fun_fact();
        assert!(fact.contains("schedule"));
    }
}
