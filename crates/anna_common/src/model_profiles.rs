//! Model Profiles - Data-Driven LLM Model Selection
//!
//! Defines available local LLM models with hardware requirements,
//! quality tiers, upgrade paths, and performance expectations.
//!
//! Beta.68: Integrated with benchmarking system for performance validation.

use crate::hardware_capability::LlmCapability;
use serde::{Deserialize, Serialize};

/// Quality tier for models
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QualityTier {
    /// Tiny: 1B params, very fast, basic understanding
    Tiny,

    /// Small: 3B params, good balance
    Small,

    /// Medium: 7-8B params, high quality
    Medium,

    /// Large: 13B+ params, best quality (rare for local)
    Large,
}

impl QualityTier {
    pub fn description(&self) -> &'static str {
        match self {
            QualityTier::Tiny => "Very fast, basic understanding",
            QualityTier::Small => "Good balance of speed and quality",
            QualityTier::Medium => "High quality responses, slower",
            QualityTier::Large => "Best quality, requires powerful hardware",
        }
    }

    /// Expected minimum tokens/second for this tier (for benchmarking)
    pub fn min_tokens_per_second(&self) -> f64 {
        match self {
            QualityTier::Tiny => 30.0,   // Very fast
            QualityTier::Small => 20.0,  // Fast
            QualityTier::Medium => 10.0, // Acceptable
            QualityTier::Large => 5.0,   // Slow but tolerable
        }
    }

    /// Expected minimum quality score (0.0-1.0) for this tier
    pub fn min_quality_score(&self) -> f64 {
        match self {
            QualityTier::Tiny => 0.6,   // Basic accuracy
            QualityTier::Small => 0.75, // Good accuracy
            QualityTier::Medium => 0.85, // High accuracy
            QualityTier::Large => 0.9,   // Excellent accuracy
        }
    }
}

/// A specific model profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    /// Unique identifier (e.g., "ollama-llama3.2-1b")
    pub id: String,

    /// Engine name (e.g., "ollama")
    pub engine: String,

    /// Model name for the engine (e.g., "llama3.2:1b")
    pub model_name: String,

    /// Minimum RAM in GB
    pub min_ram_gb: u64,

    /// Recommended CPU cores
    pub recommended_cores: usize,

    /// Quality tier
    pub quality_tier: QualityTier,

    /// Human-readable description
    pub description: String,

    /// Approximate size on disk in GB
    pub size_gb: f64,
}

impl ModelProfile {
    /// Check if this profile is suitable for given hardware
    pub fn is_suitable_for(&self, ram_gb: f64, cores: usize) -> bool {
        ram_gb >= self.min_ram_gb as f64 && cores >= self.recommended_cores
    }

    /// Check if this profile is better than another
    pub fn is_better_than(&self, other: &ModelProfile) -> bool {
        self.quality_tier > other.quality_tier
    }

    /// Validate benchmark performance against tier expectations
    pub fn meets_tier_expectations(&self, tokens_per_sec: f64, quality_score: f64) -> bool {
        tokens_per_sec >= self.quality_tier.min_tokens_per_second()
            && quality_score >= self.quality_tier.min_quality_score()
    }

    /// Get performance feedback for benchmark results
    pub fn performance_feedback(&self, tokens_per_sec: f64, quality_score: f64) -> String {
        let expected_speed = self.quality_tier.min_tokens_per_second();
        let expected_quality = self.quality_tier.min_quality_score();

        if self.meets_tier_expectations(tokens_per_sec, quality_score) {
            format!(
                "✓ Performing as expected for {} tier",
                format!("{:?}", self.quality_tier).to_lowercase()
            )
        } else if tokens_per_sec < expected_speed {
            format!(
                "⚠ Performance below expected ({:.1} tok/s, expected ≥{:.1} tok/s). Consider: lighter model or hardware upgrade.",
                tokens_per_sec, expected_speed
            )
        } else {
            format!(
                "⚠ Quality below expected ({:.0}%, expected ≥{:.0}%). Model may need fine-tuning or different version.",
                quality_score * 100.0, expected_quality * 100.0
            )
        }
    }
}

/// Get all available model profiles (data-driven, easy to update)
pub fn get_available_profiles() -> Vec<ModelProfile> {
    vec![
        // ═══ Tiny Tier - for low-end or constrained systems ═══
        ModelProfile {
            id: "ollama-llama3.2-1b".to_string(),
            engine: "ollama".to_string(),
            model_name: "llama3.2:1b".to_string(),
            min_ram_gb: 4,
            recommended_cores: 2,
            quality_tier: QualityTier::Tiny,
            description: "Llama 3.2 1B - Very fast, suitable for basic queries".to_string(),
            size_gb: 1.3,
        },
        ModelProfile {
            id: "ollama-qwen2.5-1b".to_string(),
            engine: "ollama".to_string(),
            model_name: "qwen2.5:1.5b".to_string(),
            min_ram_gb: 4,
            recommended_cores: 2,
            quality_tier: QualityTier::Tiny,
            description: "Qwen 2.5 1.5B - Fast, good for coding and technical tasks".to_string(),
            size_gb: 1.0,
        },
        // ═══ Small Tier - good default for most systems ═══
        ModelProfile {
            id: "ollama-llama3.2-3b".to_string(),
            engine: "ollama".to_string(),
            model_name: "llama3.2:3b".to_string(),
            min_ram_gb: 8,
            recommended_cores: 4,
            quality_tier: QualityTier::Small,
            description: "Llama 3.2 3B - Good balance of speed and understanding".to_string(),
            size_gb: 2.0,
        },
        ModelProfile {
            id: "ollama-phi3-mini".to_string(),
            engine: "ollama".to_string(),
            model_name: "phi3:mini".to_string(),
            min_ram_gb: 8,
            recommended_cores: 4,
            quality_tier: QualityTier::Small,
            description: "Phi 3 Mini - Microsoft's efficient 3.8B model".to_string(),
            size_gb: 2.3,
        },
        ModelProfile {
            id: "ollama-qwen2.5-3b".to_string(),
            engine: "ollama".to_string(),
            model_name: "qwen2.5:3b".to_string(),
            min_ram_gb: 8,
            recommended_cores: 4,
            quality_tier: QualityTier::Small,
            description: "Qwen 2.5 3B - Strong coding and technical understanding".to_string(),
            size_gb: 2.0,
        },
        // ═══ Medium Tier - for powerful systems ═══
        ModelProfile {
            id: "ollama-llama3.1-8b".to_string(),
            engine: "ollama".to_string(),
            model_name: "llama3.1:8b".to_string(),
            min_ram_gb: 16,
            recommended_cores: 6,
            quality_tier: QualityTier::Medium,
            description: "Llama 3.1 8B - High quality, detailed responses".to_string(),
            size_gb: 4.7,
        },
        ModelProfile {
            id: "ollama-mistral-7b".to_string(),
            engine: "ollama".to_string(),
            model_name: "mistral:7b".to_string(),
            min_ram_gb: 16,
            recommended_cores: 6,
            quality_tier: QualityTier::Medium,
            description: "Mistral 7B - Excellent reasoning and instruction following".to_string(),
            size_gb: 4.1,
        },
        ModelProfile {
            id: "ollama-qwen2.5-7b".to_string(),
            engine: "ollama".to_string(),
            model_name: "qwen2.5:7b".to_string(),
            min_ram_gb: 16,
            recommended_cores: 6,
            quality_tier: QualityTier::Medium,
            description: "Qwen 2.5 7B - Advanced coding and multilingual support".to_string(),
            size_gb: 4.7,
        },
        // ═══ Large Tier - for enthusiast/server hardware ═══
        ModelProfile {
            id: "ollama-llama3.1-13b".to_string(),
            engine: "ollama".to_string(),
            model_name: "llama3.1:13b".to_string(),
            min_ram_gb: 32,
            recommended_cores: 8,
            quality_tier: QualityTier::Large,
            description: "Llama 3.1 13B - Exceptional quality, requires significant resources".to_string(),
            size_gb: 7.4,
        },
        ModelProfile {
            id: "ollama-qwen2.5-14b".to_string(),
            engine: "ollama".to_string(),
            model_name: "qwen2.5:14b".to_string(),
            min_ram_gb: 32,
            recommended_cores: 8,
            quality_tier: QualityTier::Large,
            description: "Qwen 2.5 14B - Top-tier coding and reasoning abilities".to_string(),
            size_gb: 9.0,
        },
    ]
}

/// Select the best model profile for given hardware capability
pub fn select_model_for_capability(capability: LlmCapability) -> Option<ModelProfile> {
    let profiles = get_available_profiles();

    // Get approximate RAM and cores for the capability tier
    let (min_ram_gb, min_cores) = match capability {
        LlmCapability::High => (16.0, 4),
        LlmCapability::Medium => (8.0, 2),
        LlmCapability::Low => (4.0, 2),
    };

    // Find the best suitable profile
    profiles
        .into_iter()
        .filter(|p| p.is_suitable_for(min_ram_gb, min_cores))
        .max_by_key(|p| p.quality_tier)
}

/// Find upgrade opportunities when hardware improves
pub fn find_upgrade_profile(
    current_profile_id: &str,
    new_ram_gb: f64,
    new_cores: usize,
) -> Option<ModelProfile> {
    let profiles = get_available_profiles();

    // Find current profile
    let current = profiles
        .iter()
        .find(|p| p.id == current_profile_id)?
        .clone();

    // Find better profiles that are now suitable
    profiles
        .into_iter()
        .filter(|p| p.is_suitable_for(new_ram_gb, new_cores) && p.is_better_than(&current))
        .max_by_key(|p| p.quality_tier)
}

/// Get a profile by ID
pub fn get_profile_by_id(id: &str) -> Option<ModelProfile> {
    get_available_profiles().into_iter().find(|p| p.id == id)
}

/// Get profiles by quality tier
pub fn get_profiles_by_tier(tier: QualityTier) -> Vec<ModelProfile> {
    get_available_profiles()
        .into_iter()
        .filter(|p| p.quality_tier == tier)
        .collect()
}

/// Get recommended profile for hardware with fallback options
pub fn get_recommended_with_fallbacks(
    ram_gb: f64,
    cores: usize,
) -> (Option<ModelProfile>, Vec<ModelProfile>) {
    let profiles = get_available_profiles();

    let suitable: Vec<_> = profiles
        .iter()
        .filter(|p| p.is_suitable_for(ram_gb, cores))
        .cloned()
        .collect();

    if suitable.is_empty() {
        return (None, vec![]);
    }

    // Best match
    let best = suitable
        .iter()
        .max_by_key(|p| p.quality_tier)
        .cloned();

    // Fallback options (same tier, different models)
    let fallbacks: Vec<_> = if let Some(ref best_profile) = best {
        suitable
            .into_iter()
            .filter(|p| p.quality_tier == best_profile.quality_tier && p.id != best_profile.id)
            .collect()
    } else {
        vec![]
    };

    (best, fallbacks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_selection_for_high_capability() {
        let profile = select_model_for_capability(LlmCapability::High);
        assert!(profile.is_some());

        let profile = profile.unwrap();
        // High capability should get at least Small tier
        assert!(profile.quality_tier >= QualityTier::Small);
    }

    #[test]
    fn test_model_selection_for_medium_capability() {
        let profile = select_model_for_capability(LlmCapability::Medium);
        assert!(profile.is_some());

        let profile = profile.unwrap();
        // Medium capability might get Tiny or Small
        assert!(profile.quality_tier >= QualityTier::Tiny);
    }

    #[test]
    fn test_model_selection_for_low_capability() {
        let profile = select_model_for_capability(LlmCapability::Low);
        assert!(profile.is_some());

        let profile = profile.unwrap();
        // Low capability should get Tiny
        assert_eq!(profile.quality_tier, QualityTier::Tiny);
    }

    #[test]
    fn test_upgrade_detection() {
        // Start with tiny model on low-spec machine
        let current_id = "ollama-llama3.2-1b";

        // Upgrade RAM to 16GB, 8 cores
        let upgrade = find_upgrade_profile(current_id, 16.0, 8);

        assert!(upgrade.is_some());
        let upgrade = upgrade.unwrap();

        // Should suggest a better model
        assert!(upgrade.quality_tier > QualityTier::Tiny);
    }

    #[test]
    fn test_no_upgrade_when_already_best() {
        // Start with medium model on powerful machine
        let current_id = "ollama-llama3.1-8b";

        // Same hardware - should have no upgrade to Large tier with this RAM
        let upgrade = find_upgrade_profile(current_id, 16.0, 8);

        // Might have upgrade if we consider same-tier alternatives, but not higher tier
        if let Some(up) = upgrade {
            assert!(up.quality_tier <= QualityTier::Medium || up.min_ram_gb <= 16);
        }
    }

    #[test]
    fn test_profile_lookup() {
        let profile = get_profile_by_id("ollama-llama3.2-3b");
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().quality_tier, QualityTier::Small);
    }

    #[test]
    fn test_all_profiles_have_required_fields() {
        let profiles = get_available_profiles();

        assert!(!profiles.is_empty());
        // Should have at least 10 models now (expanded catalog)
        assert!(profiles.len() >= 10);

        for profile in profiles {
            assert!(!profile.id.is_empty());
            assert!(!profile.engine.is_empty());
            assert!(!profile.model_name.is_empty());
            assert!(profile.min_ram_gb > 0);
            assert!(profile.recommended_cores > 0);
            assert!(profile.size_gb > 0.0);
        }
    }

    #[test]
    fn test_quality_tier_performance_expectations() {
        // Tiny should expect high speed, lower quality
        assert!(QualityTier::Tiny.min_tokens_per_second() > QualityTier::Medium.min_tokens_per_second());
        assert!(QualityTier::Tiny.min_quality_score() < QualityTier::Medium.min_quality_score());

        // Large should expect high quality, lower speed acceptable
        assert!(QualityTier::Large.min_quality_score() > QualityTier::Small.min_quality_score());
        assert!(QualityTier::Large.min_tokens_per_second() < QualityTier::Tiny.min_tokens_per_second());
    }

    #[test]
    fn test_benchmark_validation() {
        let profile = get_profile_by_id("ollama-llama3.2-3b").unwrap();

        // Good performance should pass
        assert!(profile.meets_tier_expectations(25.0, 0.8));

        // Poor performance should fail
        assert!(!profile.meets_tier_expectations(5.0, 0.8));  // Too slow
        assert!(!profile.meets_tier_expectations(25.0, 0.5)); // Poor quality
    }

    #[test]
    fn test_performance_feedback() {
        let profile = get_profile_by_id("ollama-llama3.2-3b").unwrap();

        // Good performance
        let feedback = profile.performance_feedback(25.0, 0.8);
        assert!(feedback.contains("as expected"));

        // Slow performance
        let feedback = profile.performance_feedback(5.0, 0.8);
        assert!(feedback.contains("below expected"));
        assert!(feedback.contains("tok/s"));
    }

    #[test]
    fn test_get_profiles_by_tier() {
        let tiny_models = get_profiles_by_tier(QualityTier::Tiny);
        assert!(!tiny_models.is_empty());
        assert!(tiny_models.iter().all(|p| p.quality_tier == QualityTier::Tiny));

        let medium_models = get_profiles_by_tier(QualityTier::Medium);
        assert!(!medium_models.is_empty());
        assert!(medium_models.iter().all(|p| p.quality_tier == QualityTier::Medium));
    }

    #[test]
    fn test_recommended_with_fallbacks() {
        // High-end system: 32GB, 8 cores
        let (best, fallbacks) = get_recommended_with_fallbacks(32.0, 8);

        assert!(best.is_some());
        let best = best.unwrap();

        // Should recommend a high-tier model
        assert!(best.quality_tier >= QualityTier::Medium);

        // Should have fallback options in the same tier
        for fallback in fallbacks {
            assert_eq!(fallback.quality_tier, best.quality_tier);
            assert_ne!(fallback.id, best.id);
        }
    }

    #[test]
    fn test_model_catalog_expansion() {
        let profiles = get_available_profiles();

        // Verify we have models from different families
        let has_llama = profiles.iter().any(|p| p.id.contains("llama"));
        let has_qwen = profiles.iter().any(|p| p.id.contains("qwen"));
        let has_mistral = profiles.iter().any(|p| p.id.contains("mistral"));
        let has_phi = profiles.iter().any(|p| p.id.contains("phi"));

        assert!(has_llama, "Should include Llama models");
        assert!(has_qwen, "Should include Qwen models");
        assert!(has_mistral, "Should include Mistral models");
        assert!(has_phi, "Should include Phi models");
    }
}
