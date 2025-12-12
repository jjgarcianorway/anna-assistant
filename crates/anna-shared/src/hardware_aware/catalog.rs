//! Model catalog and entries (v0.0.434).
//!
//! Static, versioned catalog of candidate models with requirements.

use super::CapabilityTier;
use serde::{Deserialize, Serialize};

/// Role a model can fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelRole {
    /// Fast classification and light reasoning.
    Translator,
    /// Quick answers for simple queries.
    Junior,
    /// Deep reasoning for complex queries.
    Senior,
}

impl ModelRole {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Translator => "translator",
            Self::Junior => "junior",
            Self::Senior => "senior",
        }
    }
}

/// A model entry in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Ollama model name (e.g., "qwen3:2b").
    pub name: String,
    /// Role this model is suitable for.
    pub role: ModelRole,
    /// Minimum capability tier required.
    pub min_tier: CapabilityTier,
    /// Minimum RAM in GB.
    pub min_ram_gb: u32,
    /// Approximate disk usage in GB.
    pub disk_usage_gb: u32,
    /// Suggested max tokens for context.
    pub max_tokens_hint: u32,
    /// Brief description.
    pub description: String,
    /// Priority (higher = preferred when multiple fit).
    pub priority: u32,
}

/// The model catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalog {
    /// Catalog version.
    pub version: u32,
    /// Available models.
    pub models: Vec<ModelEntry>,
}

impl ModelCatalog {
    /// Create the default catalog.
    pub fn default_catalog() -> Self {
        Self {
            version: super::CATALOG_VERSION,
            models: vec![
                // === Translator models (smallest, fastest) ===
                ModelEntry {
                    name: "qwen3:0.6b".to_string(),
                    role: ModelRole::Translator,
                    min_tier: CapabilityTier::Tiny,
                    min_ram_gb: 2,
                    disk_usage_gb: 1,
                    max_tokens_hint: 512,
                    description: "Tiny translator for classification".to_string(),
                    priority: 10,
                },
                ModelEntry {
                    name: "qwen3:1.7b".to_string(),
                    role: ModelRole::Translator,
                    min_tier: CapabilityTier::Small,
                    min_ram_gb: 4,
                    disk_usage_gb: 2,
                    max_tokens_hint: 1024,
                    description: "Small translator with better accuracy".to_string(),
                    priority: 20,
                },
                // === Junior models (responsive, balanced) ===
                ModelEntry {
                    name: "qwen3:1.7b".to_string(),
                    role: ModelRole::Junior,
                    min_tier: CapabilityTier::Tiny,
                    min_ram_gb: 4,
                    disk_usage_gb: 2,
                    max_tokens_hint: 2048,
                    description: "Tiny junior for basic queries".to_string(),
                    priority: 10,
                },
                ModelEntry {
                    name: "qwen3:4b".to_string(),
                    role: ModelRole::Junior,
                    min_tier: CapabilityTier::Small,
                    min_ram_gb: 6,
                    disk_usage_gb: 3,
                    max_tokens_hint: 4096,
                    description: "Small junior with good balance".to_string(),
                    priority: 20,
                },
                ModelEntry {
                    name: "qwen2.5:7b-instruct".to_string(),
                    role: ModelRole::Junior,
                    min_tier: CapabilityTier::Medium,
                    min_ram_gb: 10,
                    disk_usage_gb: 5,
                    max_tokens_hint: 4096,
                    description: "Medium junior for complex queries".to_string(),
                    priority: 30,
                },
                // === Senior models (powerful, deep reasoning) ===
                ModelEntry {
                    name: "qwen3:4b".to_string(),
                    role: ModelRole::Senior,
                    min_tier: CapabilityTier::Tiny,
                    min_ram_gb: 6,
                    disk_usage_gb: 3,
                    max_tokens_hint: 4096,
                    description: "Minimal senior for constrained systems".to_string(),
                    priority: 10,
                },
                ModelEntry {
                    name: "qwen2.5:7b-instruct".to_string(),
                    role: ModelRole::Senior,
                    min_tier: CapabilityTier::Small,
                    min_ram_gb: 10,
                    disk_usage_gb: 5,
                    max_tokens_hint: 8192,
                    description: "Balanced senior for most systems".to_string(),
                    priority: 20,
                },
                ModelEntry {
                    name: "qwen2.5:14b-instruct".to_string(),
                    role: ModelRole::Senior,
                    min_tier: CapabilityTier::Medium,
                    min_ram_gb: 16,
                    disk_usage_gb: 10,
                    max_tokens_hint: 8192,
                    description: "Powerful senior for complex analysis".to_string(),
                    priority: 30,
                },
                ModelEntry {
                    name: "deepseek-r1:7b".to_string(),
                    role: ModelRole::Senior,
                    min_tier: CapabilityTier::Medium,
                    min_ram_gb: 12,
                    disk_usage_gb: 6,
                    max_tokens_hint: 8192,
                    description: "Reasoning-focused senior model".to_string(),
                    priority: 35,
                },
                ModelEntry {
                    name: "deepseek-r1:14b".to_string(),
                    role: ModelRole::Senior,
                    min_tier: CapabilityTier::Large,
                    min_ram_gb: 20,
                    disk_usage_gb: 12,
                    max_tokens_hint: 16384,
                    description: "Large reasoning model".to_string(),
                    priority: 40,
                },
            ],
        }
    }

    /// Get models for a specific role.
    pub fn models_for_role(&self, role: ModelRole) -> Vec<&ModelEntry> {
        self.models.iter().filter(|m| m.role == role).collect()
    }

    /// Find the best model for a role given constraints.
    pub fn select_model(
        &self,
        role: ModelRole,
        tier: CapabilityTier,
        available_ram_gb: f32,
        prefer_small: bool,
    ) -> Option<&ModelEntry> {
        let mut candidates: Vec<_> = self
            .models
            .iter()
            .filter(|m| m.role == role)
            .filter(|m| tier_rank(tier) >= tier_rank(m.min_tier))
            .filter(|m| available_ram_gb >= m.min_ram_gb as f32)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Sort by priority (or inverse if prefer_small)
        if prefer_small {
            candidates.sort_by(|a, b| a.priority.cmp(&b.priority));
        } else {
            candidates.sort_by(|a, b| b.priority.cmp(&a.priority));
        }

        candidates.first().copied()
    }

    /// Get a model by name.
    pub fn get_model(&self, name: &str) -> Option<&ModelEntry> {
        self.models.iter().find(|m| m.name == name)
    }

    /// Get unique model names (deduplicated).
    pub fn unique_model_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.models.iter().map(|m| m.name.clone()).collect();
        names.sort();
        names.dedup();
        names
    }

    /// Total disk usage for a set of models.
    pub fn total_disk_usage(&self, model_names: &[&str]) -> u32 {
        let mut seen = std::collections::HashSet::new();
        let mut total = 0u32;

        for name in model_names {
            if !seen.contains(*name) {
                if let Some(model) = self.get_model(name) {
                    total += model.disk_usage_gb;
                    seen.insert(*name);
                }
            }
        }

        total
    }
}

impl Default for ModelCatalog {
    fn default() -> Self {
        Self::default_catalog()
    }
}

/// Convert tier to numeric rank for comparison.
fn tier_rank(tier: CapabilityTier) -> u32 {
    match tier {
        CapabilityTier::Tiny => 0,
        CapabilityTier::Small => 1,
        CapabilityTier::Medium => 2,
        CapabilityTier::Large => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_catalog() {
        let catalog = ModelCatalog::default_catalog();
        assert_eq!(catalog.version, 1);
        assert!(!catalog.models.is_empty());
    }

    #[test]
    fn test_models_for_role() {
        let catalog = ModelCatalog::default_catalog();

        let translators = catalog.models_for_role(ModelRole::Translator);
        assert!(!translators.is_empty());
        assert!(translators.iter().all(|m| m.role == ModelRole::Translator));

        let juniors = catalog.models_for_role(ModelRole::Junior);
        assert!(!juniors.is_empty());

        let seniors = catalog.models_for_role(ModelRole::Senior);
        assert!(!seniors.is_empty());
    }

    #[test]
    fn test_select_model_tiny_tier() {
        let catalog = ModelCatalog::default_catalog();

        // Tiny tier with 4GB RAM
        let translator =
            catalog.select_model(ModelRole::Translator, CapabilityTier::Tiny, 4.0, false);
        assert!(translator.is_some());
        assert_eq!(translator.unwrap().min_tier, CapabilityTier::Tiny);

        let junior = catalog.select_model(ModelRole::Junior, CapabilityTier::Tiny, 4.0, false);
        assert!(junior.is_some());
    }

    #[test]
    fn test_select_model_prefer_small() {
        let catalog = ModelCatalog::default_catalog();

        // Medium tier with lots of RAM, but prefer small
        let senior_big =
            catalog.select_model(ModelRole::Senior, CapabilityTier::Medium, 32.0, false);
        let senior_small =
            catalog.select_model(ModelRole::Senior, CapabilityTier::Medium, 32.0, true);

        assert!(senior_big.is_some());
        assert!(senior_small.is_some());

        // Prefer small should give lower priority model
        assert!(senior_small.unwrap().priority <= senior_big.unwrap().priority);
    }

    #[test]
    fn test_unique_model_names() {
        let catalog = ModelCatalog::default_catalog();
        let names = catalog.unique_model_names();

        // Should have no duplicates
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(names.len(), sorted.len());
    }

    #[test]
    fn test_disk_usage_dedup() {
        let catalog = ModelCatalog::default_catalog();

        // Same model twice should only count once
        let usage = catalog.total_disk_usage(&["qwen3:1.7b", "qwen3:1.7b"]);
        let single = catalog.total_disk_usage(&["qwen3:1.7b"]);

        assert_eq!(usage, single);
    }
}
