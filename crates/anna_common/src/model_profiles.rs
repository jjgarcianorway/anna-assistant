//! Model Profiles - Data-Driven LLM Model Selection
//!
//! Defines available local LLM models with hardware requirements,
//! quality tiers, and upgrade paths.

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
}

/// Get all available model profiles (data-driven, easy to update)
pub fn get_available_profiles() -> Vec<ModelProfile> {
    vec![
        // Tiny tier - for low-end or constrained systems
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
        // Small tier - good default for most systems
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
        // Medium tier - for powerful systems
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

        // Same hardware
        let upgrade = find_upgrade_profile(current_id, 16.0, 8);

        // No upgrade available
        assert!(upgrade.is_none());
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

        for profile in profiles {
            assert!(!profile.id.is_empty());
            assert!(!profile.engine.is_empty());
            assert!(!profile.model_name.is_empty());
            assert!(profile.min_ram_gb > 0);
            assert!(profile.recommended_cores > 0);
            assert!(profile.size_gb > 0.0);
        }
    }
}
