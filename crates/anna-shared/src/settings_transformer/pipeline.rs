// v0.0.598: Settings Transform Pipeline (Phase 174)
// Pipeline for applying transforms

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{TransformDef, TransformDirection};

/// Transform pipeline
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformPipeline {
    /// Transforms
    pub(crate) transforms: Vec<TransformDef>,
}

impl TransformPipeline {
    /// Create new pipeline
    pub fn new() -> Self {
        Self::default()
    }

    /// Add transform
    pub fn add(&mut self, transform: TransformDef) {
        self.transforms.push(transform);
        self.transforms.sort_by_key(|t| t.priority);
    }

    /// Remove transform
    pub fn remove(&mut self, id: &str) -> Option<TransformDef> {
        if let Some(pos) = self.transforms.iter().position(|t| t.id == id) {
            Some(self.transforms.remove(pos))
        } else {
            None
        }
    }

    /// Get transform
    pub fn get(&self, id: &str) -> Option<&TransformDef> {
        self.transforms.iter().find(|t| t.id == id)
    }

    /// Enable transform
    pub fn enable(&mut self, id: &str) -> bool {
        if let Some(t) = self.transforms.iter_mut().find(|t| t.id == id) {
            t.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable transform
    pub fn disable(&mut self, id: &str) -> bool {
        if let Some(t) = self.transforms.iter_mut().find(|t| t.id == id) {
            t.enabled = false;
            true
        } else {
            false
        }
    }

    /// Get transforms for category and direction
    pub fn for_category_dir(
        &self,
        category: SettingsCategory,
        dir: TransformDirection,
    ) -> Vec<&TransformDef> {
        self.transforms
            .iter()
            .filter(|t| t.enabled && t.applies_to(category) && t.applies_to_direction(dir))
            .collect()
    }

    /// Count transforms
    pub fn count(&self) -> usize {
        self.transforms.len()
    }

    /// Count enabled
    pub fn enabled_count(&self) -> usize {
        self.transforms.iter().filter(|t| t.enabled).count()
    }
}
