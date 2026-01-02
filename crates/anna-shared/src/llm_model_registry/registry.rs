// v0.0.531: LLM Model Registry
// Main registry implementation for managing LLM models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{InstalledBy, ModelCapability, ModelRecord};

/// LLM Model Registry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmModelRegistry {
    models: HashMap<String, ModelRecord>,
}

impl LlmModelRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    /// Register a model
    pub fn register(&mut self, model: ModelRecord) {
        self.models.insert(model.name.clone(), model);
    }

    /// Get model by name
    pub fn get(&self, name: &str) -> Option<&ModelRecord> {
        self.models.get(name)
    }

    /// Get mutable model
    pub fn get_mut(&mut self, name: &str) -> Option<&mut ModelRecord> {
        self.models.get_mut(name)
    }

    /// Get ready models
    pub fn ready(&self) -> Vec<&ModelRecord> {
        self.models.values().filter(|m| m.is_ready()).collect()
    }

    /// Get models by capability
    pub fn by_capability(&self, cap: ModelCapability) -> Vec<&ModelRecord> {
        self.models
            .values()
            .filter(|m| m.capability == cap && m.is_ready())
            .collect()
    }

    /// Get models installed by Anna
    pub fn installed_by_anna(&self) -> Vec<&ModelRecord> {
        self.models
            .values()
            .filter(|m| m.installed_by == InstalledBy::Anna)
            .collect()
    }

    /// Get model for specialist
    pub fn for_specialist(&self, specialist_id: &str) -> Vec<&ModelRecord> {
        self.models
            .values()
            .filter(|m| m.assigned_specialists.contains(&specialist_id.to_string()))
            .collect()
    }

    /// Get best available model for capability
    pub fn best_for(&self, cap: ModelCapability) -> Option<&ModelRecord> {
        self.by_capability(cap)
            .into_iter()
            .min_by_key(|m| m.avg_response_ms)
    }

    /// Total VRAM used by ready models
    pub fn total_vram_gb(&self) -> f64 {
        self.ready().iter().map(|m| m.vram_required_gb).sum()
    }

    /// Total disk used
    pub fn total_disk_gb(&self) -> f64 {
        self.ready().iter().map(|m| m.size_gb).sum()
    }

    /// Model count
    pub fn total(&self) -> usize {
        self.models.len()
    }

    /// Ready count
    pub fn ready_count(&self) -> usize {
        self.ready().len()
    }

    /// All models
    pub fn all(&self) -> Vec<&ModelRecord> {
        self.models.values().collect()
    }
}
