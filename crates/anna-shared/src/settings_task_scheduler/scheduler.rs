// v0.0.610: Settings Task Scheduler - Scheduler (Phase 186)
// Task scheduler implementation

use std::collections::HashMap;

use super::task_definition::TaskDefinition;
use super::task_instance::TaskInstance;
use super::types::TaskState;

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
