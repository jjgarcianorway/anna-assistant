// v0.0.553: Model Config (Phase 129)
// Configurable LLM model settings per VISION.md

use serde::{Deserialize, Serialize};

/// Model size preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ModelSizePreference {
    Tiny,
    Small,
    #[default]
    Medium,
    Large,
    Largest,
}

impl std::fmt::Display for ModelSizePreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tiny => write!(f, "Tiny (fastest)"),
            Self::Small => write!(f, "Small"),
            Self::Medium => write!(f, "Medium"),
            Self::Large => write!(f, "Large"),
            Self::Largest => write!(f, "Largest (best quality)"),
        }
    }
}

/// Model quality vs speed preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum QualitySpeedBalance {
    Speed,
    #[default]
    Balanced,
    Quality,
}

impl std::fmt::Display for QualitySpeedBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Speed => write!(f, "Speed (fast responses)"),
            Self::Balanced => write!(f, "Balanced"),
            Self::Quality => write!(f, "Quality (best answers)"),
        }
    }
}

/// Model management mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ModelManagement {
    #[default]
    Automatic,
    Manual,
    Conservative,
}

impl std::fmt::Display for ModelManagement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Automatic => write!(f, "Automatic (Anna decides)"),
            Self::Manual => write!(f, "Manual (user controls)"),
            Self::Conservative => write!(f, "Conservative (minimal downloads)"),
        }
    }
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub size_preference: ModelSizePreference,
    pub quality_speed: QualitySpeedBalance,
    pub management: ModelManagement,
    pub auto_download: bool,
    pub auto_update_models: bool,
    pub use_gpu: bool,
    pub max_memory_mb: u64,
    pub max_context_tokens: u32,
    pub respect_hardware_limits: bool,
    pub fallback_to_smaller: bool,
    pub specialist_uses_senior: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            size_preference: ModelSizePreference::Medium,
            quality_speed: QualitySpeedBalance::Balanced,
            management: ModelManagement::Automatic,
            auto_download: true,
            auto_update_models: true,
            use_gpu: true,
            max_memory_mb: 8192,
            max_context_tokens: 4096,
            respect_hardware_limits: true,
            fallback_to_smaller: true,
            specialist_uses_senior: false,
        }
    }
}

impl ModelConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast preset - prioritize speed
    pub fn fast() -> Self {
        Self {
            size_preference: ModelSizePreference::Tiny,
            quality_speed: QualitySpeedBalance::Speed,
            management: ModelManagement::Automatic,
            auto_download: true,
            auto_update_models: true,
            use_gpu: true,
            max_memory_mb: 4096,
            max_context_tokens: 2048,
            respect_hardware_limits: true,
            fallback_to_smaller: true,
            specialist_uses_senior: false,
        }
    }

    /// Quality preset - prioritize answer quality
    pub fn quality() -> Self {
        Self {
            size_preference: ModelSizePreference::Large,
            quality_speed: QualitySpeedBalance::Quality,
            management: ModelManagement::Automatic,
            auto_download: true,
            auto_update_models: true,
            use_gpu: true,
            max_memory_mb: 16384,
            max_context_tokens: 8192,
            respect_hardware_limits: true,
            fallback_to_smaller: true,
            specialist_uses_senior: true,
        }
    }

    /// Minimal preset - conserve resources
    pub fn minimal() -> Self {
        Self {
            size_preference: ModelSizePreference::Tiny,
            quality_speed: QualitySpeedBalance::Speed,
            management: ModelManagement::Conservative,
            auto_download: false,
            auto_update_models: false,
            use_gpu: false,
            max_memory_mb: 2048,
            max_context_tokens: 1024,
            respect_hardware_limits: true,
            fallback_to_smaller: true,
            specialist_uses_senior: false,
        }
    }

    /// Is GPU enabled?
    pub fn is_gpu_enabled(&self) -> bool {
        self.use_gpu
    }

    /// Is auto-download enabled?
    pub fn is_auto_download(&self) -> bool {
        self.auto_download && self.management != ModelManagement::Manual
    }

    /// Should use senior models for specialists?
    pub fn should_use_senior(&self) -> bool {
        self.specialist_uses_senior
    }

    /// Get max memory in GB
    pub fn max_memory_gb(&self) -> f64 {
        self.max_memory_mb as f64 / 1024.0
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Preset changes
        if lower.contains("fast model") || lower.contains("quick response") || lower.contains("speed") {
            *self = Self::fast();
            return Some("Fast mode - prioritizing quick responses.".to_string());
        }
        if lower.contains("quality model") || lower.contains("best answer") || lower.contains("smart") {
            *self = Self::quality();
            return Some("Quality mode - using larger models for better answers.".to_string());
        }
        if lower.contains("minimal model") || lower.contains("conserve") || lower.contains("low resource") {
            *self = Self::minimal();
            return Some("Minimal mode - conserving system resources.".to_string());
        }

        // Size preference
        if lower.contains("tiny model") || lower.contains("smallest") {
            self.size_preference = ModelSizePreference::Tiny;
            return Some("Using tiny models.".to_string());
        }
        if lower.contains("small model") {
            self.size_preference = ModelSizePreference::Small;
            return Some("Using small models.".to_string());
        }
        if lower.contains("large model") || lower.contains("bigger") {
            self.size_preference = ModelSizePreference::Large;
            return Some("Using large models.".to_string());
        }

        // Feature toggles
        if lower.contains("enable gpu") || lower.contains("use gpu") {
            self.use_gpu = true;
            return Some("GPU acceleration enabled.".to_string());
        }
        if lower.contains("disable gpu") || lower.contains("cpu only") {
            self.use_gpu = false;
            return Some("GPU disabled - using CPU only.".to_string());
        }
        if lower.contains("auto download") || lower.contains("download model") {
            self.auto_download = true;
            return Some("Auto-download enabled.".to_string());
        }
        if lower.contains("no download") || lower.contains("manual download") {
            self.auto_download = false;
            return Some("Auto-download disabled.".to_string());
        }
        if lower.contains("use senior") || lower.contains("deep thinking") {
            self.specialist_uses_senior = true;
            return Some("Specialists will use senior (larger) models.".to_string());
        }
        if lower.contains("use junior") || lower.contains("lighter model") {
            self.specialist_uses_senior = false;
            return Some("Specialists will use junior (lighter) models.".to_string());
        }

        None
    }
}

/// Format model config
pub fn format_model_config(config: &ModelConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Model Configuration ===\n\n");

    output.push_str(&format!("Size Preference: {}\n", config.size_preference));
    output.push_str(&format!("Quality/Speed: {}\n", config.quality_speed));
    output.push_str(&format!("Management: {}\n", config.management));
    output.push_str(&format!("Auto Download: {}\n", config.auto_download));
    output.push_str(&format!("Auto Update Models: {}\n", config.auto_update_models));
    output.push_str(&format!("Use GPU: {}\n", config.use_gpu));
    output.push_str(&format!("Max Memory: {} MB\n", config.max_memory_mb));
    output.push_str(&format!("Max Context: {} tokens\n", config.max_context_tokens));
    output.push_str(&format!("Respect Hardware Limits: {}\n", config.respect_hardware_limits));
    output.push_str(&format!("Fallback to Smaller: {}\n", config.fallback_to_smaller));
    output.push_str(&format!("Specialist Uses Senior: {}\n", config.specialist_uses_senior));

    output
}

/// Check if query is model-related
pub fn is_model_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("model")
        || lower.contains("llm")
        || lower.contains("gpu")
        || lower.contains("ollama")
}

/// Fun fact about models
pub fn model_fun_fact() -> &'static str {
    "The first neural network was proposed in 1943, but LLMs only became practical 80 years later in 2023!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_preference_display() {
        assert_eq!(format!("{}", ModelSizePreference::Tiny), "Tiny (fastest)");
        assert_eq!(format!("{}", ModelSizePreference::Largest), "Largest (best quality)");
    }

    #[test]
    fn test_default_config() {
        let config = ModelConfig::default();
        assert_eq!(config.size_preference, ModelSizePreference::Medium);
        assert!(config.use_gpu);
    }

    #[test]
    fn test_fast_preset() {
        let config = ModelConfig::fast();
        assert_eq!(config.size_preference, ModelSizePreference::Tiny);
        assert_eq!(config.quality_speed, QualitySpeedBalance::Speed);
    }

    #[test]
    fn test_quality_preset() {
        let config = ModelConfig::quality();
        assert_eq!(config.size_preference, ModelSizePreference::Large);
        assert!(config.specialist_uses_senior);
    }

    #[test]
    fn test_minimal_preset() {
        let config = ModelConfig::minimal();
        assert!(!config.auto_download);
        assert!(!config.use_gpu);
    }

    #[test]
    fn test_is_gpu_enabled() {
        let config = ModelConfig::default();
        assert!(config.is_gpu_enabled());
        let minimal = ModelConfig::minimal();
        assert!(!minimal.is_gpu_enabled());
    }

    #[test]
    fn test_max_memory_gb() {
        let config = ModelConfig::default();
        assert!((config.max_memory_gb() - 8.0).abs() < 0.1);
    }

    #[test]
    fn test_apply_fast() {
        let mut config = ModelConfig::default();
        let result = config.apply_change("use fast models");
        assert!(result.is_some());
        assert_eq!(config.size_preference, ModelSizePreference::Tiny);
    }

    #[test]
    fn test_apply_gpu_toggle() {
        let mut config = ModelConfig::default();
        config.apply_change("disable gpu");
        assert!(!config.is_gpu_enabled());
    }

    #[test]
    fn test_is_model_query() {
        assert!(is_model_query("Configure models"));
        assert!(is_model_query("Enable GPU"));
        assert!(!is_model_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = model_fun_fact();
        assert!(fact.contains("1943"));
    }
}
