//! Model Registry v0.16.1
//!
//! Dynamic model selection based on a remote registry that can be updated
//! without releasing new Anna versions. The registry is fetched from GitHub
//! and cached locally.
//!
//! Registry URL: https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/models.json

use crate::hardware::HardwareTier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Registry fetch URL
pub const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/models.json";

/// Cache duration: 24 hours
pub const CACHE_DURATION_SECS: u64 = 86400;

/// Local cache path
pub fn cache_path() -> PathBuf {
    PathBuf::from("/var/lib/anna/model_registry.json")
}

/// Model registry schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub schema_version: String,
    pub last_updated: String,
    #[serde(default)]
    pub notes: String,
    pub recommended_by_tier: HashMap<String, TierRecommendation>,
    #[serde(default)]
    pub known_good_models: Vec<String>,
    #[serde(default)]
    pub model_families: HashMap<String, ModelFamily>,
    #[serde(default)]
    pub deprecated_models: Vec<String>,
    #[serde(default)]
    pub upgrade_suggestions: HashMap<String, String>,
}

/// Recommendation for a hardware tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRecommendation {
    #[serde(default)]
    pub description: String,
    pub senior: String,
    pub junior: String,
}

/// Model family info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFamily {
    #[serde(default = "default_priority")]
    pub priority: u8,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub sizes: Vec<String>,
}

fn default_priority() -> u8 {
    99
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::builtin()
    }
}

impl ModelRegistry {
    /// Built-in fallback registry (used if fetch fails)
    pub fn builtin() -> Self {
        let mut recommended = HashMap::new();

        recommended.insert(
            "datacenter".to_string(),
            TierRecommendation {
                description: "80GB+ VRAM".to_string(),
                senior: "qwen2.5:72b".to_string(),
                junior: "qwen3:8b".to_string(),
            },
        );
        recommended.insert(
            "datacenter_entry".to_string(),
            TierRecommendation {
                description: "32-48GB VRAM".to_string(),
                senior: "qwen3:32b".to_string(),
                junior: "qwen3:4b".to_string(),
            },
        );
        recommended.insert(
            "high_end_gpu".to_string(),
            TierRecommendation {
                description: "16-24GB VRAM".to_string(),
                senior: "qwen3:14b".to_string(),
                junior: "qwen3:4b".to_string(),
            },
        );
        recommended.insert(
            "mid_range_gpu".to_string(),
            TierRecommendation {
                description: "6-12GB VRAM".to_string(),
                senior: "qwen3:8b".to_string(),
                junior: "qwen3:1.7b".to_string(),
            },
        );
        recommended.insert(
            "low_gpu".to_string(),
            TierRecommendation {
                description: "<6GB VRAM".to_string(),
                senior: "qwen3:4b".to_string(),
                junior: "qwen3:0.6b".to_string(),
            },
        );
        recommended.insert(
            "high_cpu".to_string(),
            TierRecommendation {
                description: "32GB+ RAM".to_string(),
                senior: "qwen3:4b".to_string(),
                junior: "qwen3:1.7b".to_string(),
            },
        );
        recommended.insert(
            "mid_cpu".to_string(),
            TierRecommendation {
                description: "16GB+ RAM".to_string(),
                senior: "qwen3:1.7b".to_string(),
                junior: "qwen3:0.6b".to_string(),
            },
        );
        recommended.insert(
            "low_cpu".to_string(),
            TierRecommendation {
                description: "<16GB RAM".to_string(),
                senior: "qwen3:0.6b".to_string(),
                junior: "qwen3:0.6b".to_string(),
            },
        );

        Self {
            schema_version: "1.0.0".to_string(),
            last_updated: "2025-11-27".to_string(),
            notes: "Built-in fallback registry".to_string(),
            recommended_by_tier: recommended,
            known_good_models: vec![
                "qwen3:0.6b".to_string(),
                "qwen3:1.7b".to_string(),
                "qwen3:4b".to_string(),
                "qwen3:8b".to_string(),
                "qwen3:14b".to_string(),
                "qwen3:32b".to_string(),
                "llama3.2:3b".to_string(),
                "llama3.1:8b".to_string(),
            ],
            model_families: HashMap::new(),
            deprecated_models: vec!["llama2:*".to_string()],
            upgrade_suggestions: HashMap::new(),
        }
    }

    /// Get recommendation for a hardware tier
    pub fn get_recommendation(&self, tier: HardwareTier) -> Option<&TierRecommendation> {
        let key = tier.as_str();
        self.recommended_by_tier.get(key)
    }

    /// Check if a model is known good
    pub fn is_known_good(&self, model: &str) -> bool {
        // Check exact match
        if self.known_good_models.contains(&model.to_string()) {
            return true;
        }

        // Check family patterns (e.g., "qwen3:*" matches "qwen3:8b")
        for known in &self.known_good_models {
            if known.ends_with(":*") {
                let family = &known[..known.len() - 2];
                if model.starts_with(family) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a model is deprecated
    pub fn is_deprecated(&self, model: &str) -> bool {
        for deprecated in &self.deprecated_models {
            if deprecated.ends_with(":*") {
                let family = &deprecated[..deprecated.len() - 2];
                if model.starts_with(family) {
                    return true;
                }
            } else if deprecated == model {
                return true;
            }
        }
        false
    }

    /// Get upgrade suggestion for a model
    pub fn get_upgrade_suggestion(&self, model: &str) -> Option<&String> {
        self.upgrade_suggestions.get(model)
    }

    /// Load registry from cache or fetch from remote
    pub fn load() -> Self {
        // Try cache first
        if let Some(cached) = Self::load_from_cache() {
            return cached;
        }

        // Return builtin (async fetch should be done separately)
        Self::builtin()
    }

    /// Load from local cache if valid
    fn load_from_cache() -> Option<Self> {
        let path = cache_path();
        if !path.exists() {
            return None;
        }

        // Check cache age
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                let age = SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or(Duration::MAX);
                if age > Duration::from_secs(CACHE_DURATION_SECS) {
                    return None; // Cache expired
                }
            }
        }

        // Try to read and parse
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
    }

    /// Save registry to cache
    pub fn save_to_cache(&self) -> std::io::Result<()> {
        let path = cache_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }
}

/// Cache metadata for registry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistryCache {
    pub last_fetch: Option<i64>,
    pub fetch_error: Option<String>,
    pub registry_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_registry() {
        let registry = ModelRegistry::builtin();
        assert!(!registry.recommended_by_tier.is_empty());
        assert!(registry.get_recommendation(HardwareTier::MidRangeGpu).is_some());
    }

    #[test]
    fn test_is_known_good() {
        let registry = ModelRegistry::builtin();
        assert!(registry.is_known_good("qwen3:8b"));
        assert!(registry.is_known_good("llama3.1:8b"));
    }

    #[test]
    fn test_is_deprecated() {
        let registry = ModelRegistry::builtin();
        assert!(registry.is_deprecated("llama2:7b"));
        assert!(registry.is_deprecated("llama2:13b"));
        assert!(!registry.is_deprecated("qwen3:8b"));
    }

    #[test]
    fn test_tier_recommendation() {
        let registry = ModelRegistry::builtin();
        let rec = registry.get_recommendation(HardwareTier::MidRangeGpu).unwrap();
        assert_eq!(rec.senior, "qwen3:8b");
        assert_eq!(rec.junior, "qwen3:1.7b");
    }

    #[test]
    fn test_all_tiers_have_recommendation() {
        let registry = ModelRegistry::builtin();
        let tiers = [
            HardwareTier::Datacenter,
            HardwareTier::DatacenterEntry,
            HardwareTier::HighEndGpu,
            HardwareTier::MidRangeGpu,
            HardwareTier::LowGpu,
            HardwareTier::HighCpu,
            HardwareTier::MidCpu,
            HardwareTier::LowCpu,
        ];
        for tier in tiers {
            assert!(
                registry.get_recommendation(tier).is_some(),
                "Missing recommendation for {:?}",
                tier
            );
        }
    }
}
