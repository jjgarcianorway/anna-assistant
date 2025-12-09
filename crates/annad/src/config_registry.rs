//! Model registry configuration for domain-specific specialist models.
//!
//! Extracted from config.rs (v0.0.162) for modularization.
//! Added in v0.0.76 for Qwen3-VL support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model registry configuration (v0.0.76)
/// Maps domain and seniority tier to specific models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistryConfig {
    /// Translator model (fast classification)
    #[serde(default = "default_registry_translator")]
    pub translator: String,

    /// Default specialist model (fallback for all domains)
    #[serde(default = "default_registry_specialist")]
    pub specialist_default: String,

    /// Domain-specific specialist overrides
    /// Key format: "domain" or "domain:tier" (e.g., "network", "performance:senior")
    #[serde(default)]
    pub specialist_overrides: HashMap<String, String>,

    /// Preferred model family (for auto-selection when multiple available)
    /// Options: "qwen3-vl", "qwen2.5", "llama3.2", "auto"
    #[serde(default = "default_preferred_family")]
    pub preferred_family: String,
}

fn default_registry_translator() -> String {
    "qwen2.5:0.5b-instruct".to_string()
}

fn default_registry_specialist() -> String {
    "qwen2.5:7b-instruct".to_string()
}

fn default_preferred_family() -> String {
    // v0.0.76: Prefer Qwen3-VL when available, but default to Qwen2.5 for now
    "qwen2.5".to_string()
}

impl Default for ModelRegistryConfig {
    fn default() -> Self {
        Self {
            translator: default_registry_translator(),
            specialist_default: default_registry_specialist(),
            specialist_overrides: HashMap::new(),
            preferred_family: default_preferred_family(),
        }
    }
}

impl ModelRegistryConfig {
    /// Get specialist model for a domain and tier
    /// Lookup order: domain:tier -> domain -> specialist_default
    pub fn get_specialist(&self, domain: &str, tier: Option<&str>) -> &str {
        // Try domain:tier first
        if let Some(t) = tier {
            let key = format!("{}:{}", domain.to_lowercase(), t.to_lowercase());
            if let Some(model) = self.specialist_overrides.get(&key) {
                return model;
            }
        }

        // Try domain only
        let domain_key = domain.to_lowercase();
        if let Some(model) = self.specialist_overrides.get(&domain_key) {
            return model;
        }

        // Fall back to default
        &self.specialist_default
    }

    /// Check if Qwen3-VL family is preferred
    pub fn prefers_qwen3_vl(&self) -> bool {
        self.preferred_family.to_lowercase() == "qwen3-vl"
    }

    /// Get list of all configured models (for pulling)
    pub fn all_models(&self) -> Vec<String> {
        let mut models = vec![self.translator.clone(), self.specialist_default.clone()];
        for model in self.specialist_overrides.values() {
            if !models.contains(model) {
                models.push(model.clone());
            }
        }
        models.sort();
        models.dedup();
        models
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_default() {
        let registry = ModelRegistryConfig::default();
        assert_eq!(registry.translator, "qwen2.5:0.5b-instruct");
        assert_eq!(registry.specialist_default, "qwen2.5:7b-instruct");
        assert_eq!(registry.preferred_family, "qwen2.5");
        assert!(registry.specialist_overrides.is_empty());
    }

    #[test]
    fn test_model_registry_get_specialist_default() {
        let registry = ModelRegistryConfig::default();
        assert_eq!(
            registry.get_specialist("network", None),
            "qwen2.5:7b-instruct"
        );
        assert_eq!(
            registry.get_specialist("performance", Some("senior")),
            "qwen2.5:7b-instruct"
        );
    }

    #[test]
    fn test_model_registry_get_specialist_with_overrides() {
        let mut registry = ModelRegistryConfig::default();
        registry
            .specialist_overrides
            .insert("network".to_string(), "qwen3-vl:4b".to_string());
        registry
            .specialist_overrides
            .insert("security:senior".to_string(), "qwen3-vl:8b".to_string());

        // Domain override
        assert_eq!(registry.get_specialist("network", None), "qwen3-vl:4b");
        assert_eq!(
            registry.get_specialist("network", Some("frontline")),
            "qwen3-vl:4b"
        );

        // Domain:tier override
        assert_eq!(
            registry.get_specialist("security", Some("senior")),
            "qwen3-vl:8b"
        );
        // Fall back to default for security without tier match
        assert_eq!(
            registry.get_specialist("security", Some("frontline")),
            "qwen2.5:7b-instruct"
        );
    }

    #[test]
    fn test_model_registry_all_models() {
        let mut registry = ModelRegistryConfig::default();
        registry
            .specialist_overrides
            .insert("network".to_string(), "custom:4b".to_string());

        let models = registry.all_models();
        assert!(models.contains(&"qwen2.5:0.5b-instruct".to_string()));
        assert!(models.contains(&"qwen2.5:7b-instruct".to_string()));
        assert!(models.contains(&"custom:4b".to_string()));
    }
}
