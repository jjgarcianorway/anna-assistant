// v0.0.678: Settings Partitioner Registry
// Registry for managing multiple partitioners

use std::collections::HashMap;
use super::partitioner::SettingsPartitioner;

/// Partitioner registry
#[derive(Debug, Clone, Default)]
pub struct PartitionerRegistry {
    /// Partitioners by ID
    partitioners: HashMap<String, SettingsPartitioner>,
}

impl PartitionerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register partitioner
    pub fn register(&mut self, id: impl Into<String>, partitioner: SettingsPartitioner) {
        self.partitioners.insert(id.into(), partitioner);
    }

    /// Unregister partitioner
    pub fn unregister(&mut self, id: &str) -> bool {
        self.partitioners.remove(id).is_some()
    }

    /// Get partitioner
    pub fn get(&self, id: &str) -> Option<&SettingsPartitioner> {
        self.partitioners.get(id)
    }

    /// Get partitioner mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPartitioner> {
        self.partitioners.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.partitioners.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_partitioner::config::PartitionerConfig;

    #[test]
    fn test_registry_new() {
        let r = PartitionerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PartitionerRegistry::new();
        r.register("p1", SettingsPartitioner::new(PartitionerConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
