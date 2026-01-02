// v0.0.610: Settings Task Scheduler - Task Definition (Phase 186)
// Task definition structure and builder

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{TaskFrequency, TaskType};

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
