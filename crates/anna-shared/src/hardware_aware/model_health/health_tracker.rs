//! Overall model health tracking (v0.0.434).
//!
//! Maintains health records for all models in a plan and provides persistence.

use super::health_record::ModelHealthRecord;
use super::model_status::{timestamp_now, InstalledBy, ModelStatus};
use crate::hardware_aware::model_plan::ModelPlan;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Overall model health tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    /// Per-model health records.
    pub models: HashMap<String, ModelHealthRecord>,
    /// Last full check time.
    pub last_check: Option<String>,
}

impl ModelHealth {
    /// Create empty health tracker.
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            last_check: None,
        }
    }

    /// Load from file.
    pub fn load(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save to file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    /// Get or create record for a model.
    pub fn get_or_create(&mut self, name: &str) -> &mut ModelHealthRecord {
        self.models
            .entry(name.to_string())
            .or_insert_with(|| ModelHealthRecord::missing(name))
    }

    /// Get status of a model.
    pub fn status(&self, name: &str) -> ModelStatus {
        self.models
            .get(name)
            .map(|r| r.status)
            .unwrap_or(ModelStatus::Missing)
    }

    /// Check if all plan models are usable.
    pub fn all_usable(&self, plan: &ModelPlan) -> bool {
        plan.model_names()
            .iter()
            .all(|name| self.status(name).is_usable())
    }

    /// Get missing models from plan.
    pub fn missing_models(&self, plan: &ModelPlan) -> Vec<String> {
        plan.model_names()
            .iter()
            .filter(|name| self.status(name) == ModelStatus::Missing)
            .map(|s| s.to_string())
            .collect()
    }

    /// Get broken models from plan.
    pub fn broken_models(&self, plan: &ModelPlan) -> Vec<String> {
        plan.model_names()
            .iter()
            .filter(|name| self.status(name) == ModelStatus::Broken)
            .map(|s| s.to_string())
            .collect()
    }

    /// Mark check complete.
    pub fn mark_checked(&mut self) {
        self.last_check = Some(timestamp_now());
    }

    /// Get models installed by Anna.
    pub fn anna_installed_models(&self) -> Vec<&str> {
        self.models
            .iter()
            .filter(|(_, r)| r.installed_by == InstalledBy::Anna)
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

impl Default for ModelHealth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware_aware::profile::CapabilityTier;

    fn mock_plan() -> ModelPlan {
        ModelPlan {
            catalog_version: 1,
            profile_version: 1,
            tier: CapabilityTier::Small,
            translator_model: "qwen3:0.6b".to_string(),
            junior_model: "qwen3:1.7b".to_string(),
            senior_model: "qwen3:4b".to_string(),
            prefer_small: false,
            estimated_disk_gb: 6,
            created_at: "0".to_string(),
            rationale: "Test plan".to_string(),
        }
    }

    #[test]
    fn test_health_tracker() {
        let mut health = ModelHealth::new();
        let plan = mock_plan();

        // Initially all missing
        assert_eq!(health.status("qwen3:0.6b"), ModelStatus::Missing);
        assert!(!health.all_usable(&plan));

        // Add records
        health.models.insert(
            "qwen3:0.6b".to_string(),
            ModelHealthRecord::installed_by_anna("qwen3:0.6b"),
        );
        health.models.insert(
            "qwen3:1.7b".to_string(),
            ModelHealthRecord::installed_by_anna("qwen3:1.7b"),
        );
        health.models.insert(
            "qwen3:4b".to_string(),
            ModelHealthRecord::installed_by_anna("qwen3:4b"),
        );

        // All unverified but usable
        assert!(health.all_usable(&plan));
    }

    #[test]
    fn test_missing_models() {
        let mut health = ModelHealth::new();
        let plan = mock_plan();

        let missing = health.missing_models(&plan);
        assert_eq!(missing.len(), 3);

        // Add one model
        health.models.insert(
            "qwen3:0.6b".to_string(),
            ModelHealthRecord::installed_by_anna("qwen3:0.6b"),
        );

        let missing = health.missing_models(&plan);
        assert_eq!(missing.len(), 2);
    }

    #[test]
    fn test_anna_installed_models() {
        let mut health = ModelHealth::new();

        health.models.insert(
            "model1".to_string(),
            ModelHealthRecord::installed_by_anna("model1"),
        );
        health
            .models
            .insert("model2".to_string(), ModelHealthRecord::detected("model2"));

        let anna_models = health.anna_installed_models();
        assert_eq!(anna_models.len(), 1);
        assert!(anna_models.contains(&"model1"));
    }
}
