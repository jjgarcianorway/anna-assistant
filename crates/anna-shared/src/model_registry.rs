//! Model registry for role-model bindings and hardware-aware selection.
//!
//! Tracks which models are assigned to which roles (team + specialist role).
//! Provides hardware-aware model selection based on system capabilities.
//!
//! v0.0.29: Initial implementation.

use crate::specialists::SpecialistRole;
use crate::teams::Team;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Model specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelSpec {
    /// Model name (e.g., "llama3.2:3b", "qwen2.5:1.5b")
    pub name: String,
    /// Estimated size in GB (for selection guidance)
    pub size_hint_gb: Option<f32>,
    /// Quantization level if known (e.g., "Q4_K_M")
    pub quant: Option<String>,
}

impl ModelSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size_hint_gb: None,
            quant: None,
        }
    }

    pub fn with_size(mut self, size_gb: f32) -> Self {
        self.size_hint_gb = Some(size_gb);
        self
    }

    pub fn with_quant(mut self, quant: impl Into<String>) -> Self {
        self.quant = Some(quant.into());
        self
    }
}

/// Role binding - maps team + role to a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleBinding {
    /// Team this binding applies to
    pub team: Team,
    /// Role within the team
    pub role: SpecialistRole,
    /// Assigned model
    pub model: ModelSpec,
    /// Reason for this selection
    pub selection_reason: String,
}

impl RoleBinding {
    pub fn new(team: Team, role: SpecialistRole, model: ModelSpec) -> Self {
        Self {
            team,
            role,
            model,
            selection_reason: "default".to_string(),
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.selection_reason = reason.into();
        self
    }
}

/// Model state from Ollama
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelState {
    /// Whether model is present locally
    pub present: bool,
    /// Model digest if known
    pub digest: Option<String>,
    /// Last time model was seen (epoch seconds)
    pub last_seen_ts: Option<u64>,
    /// Model size in bytes if known
    pub size_bytes: Option<u64>,
}

/// Hardware tier for model selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareTier {
    /// Low-end: < 4GB RAM or < 4 cores
    Low,
    /// Medium: 4-8GB RAM, 4-8 cores
    Medium,
    /// High: 8-16GB RAM, 8+ cores
    High,
    /// Very High: > 16GB RAM, 8+ cores, GPU
    VeryHigh,
}

impl HardwareTier {
    /// Determine tier from hardware specs
    pub fn from_specs(ram_gb: f32, cpu_cores: u32, has_gpu: bool) -> Self {
        if has_gpu && ram_gb >= 16.0 {
            Self::VeryHigh
        } else if ram_gb >= 8.0 && cpu_cores >= 8 {
            Self::High
        } else if ram_gb >= 4.0 && cpu_cores >= 4 {
            Self::Medium
        } else {
            Self::Low
        }
    }
}

impl std::fmt::Display for HardwareTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::VeryHigh => write!(f, "very_high"),
        }
    }
}

/// Model recommendations by hardware tier
/// Pinned mapping table - deterministic selection
pub fn recommended_model_for_tier(tier: HardwareTier) -> ModelSpec {
    match tier {
        HardwareTier::Low => ModelSpec::new("qwen2.5:0.5b")
            .with_size(0.4)
            .with_quant("Q4_K_M"),
        HardwareTier::Medium => ModelSpec::new("qwen2.5:1.5b")
            .with_size(1.0)
            .with_quant("Q4_K_M"),
        HardwareTier::High => ModelSpec::new("llama3.2:3b")
            .with_size(2.0)
            .with_quant("Q4_K_M"),
        HardwareTier::VeryHigh => ModelSpec::new("llama3.2:3b")
            .with_size(2.0)
            .with_quant("Q8_0"),
    }
}

/// Model registry containing all role bindings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelRegistry {
    /// Role-model bindings
    pub bindings: Vec<RoleBinding>,
    /// Detected hardware tier
    pub hardware_tier: Option<HardwareTier>,
    /// Model states from Ollama
    pub states: Vec<(String, ModelState)>,
    /// Last benchmark result (epoch seconds)
    pub last_benchmark_ts: Option<u64>,
}

impl ModelRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Create registry with default bindings for a hardware tier
    pub fn with_defaults(tier: HardwareTier) -> Self {
        let model = recommended_model_for_tier(tier);
        let reason = format!("hardware_tier={}", tier);

        let teams = [
            Team::Desktop,
            Team::Storage,
            Team::Network,
            Team::Performance,
            Team::Services,
            Team::Security,
            Team::Hardware,
            Team::General,
        ];

        let roles = [
            SpecialistRole::Translator,
            SpecialistRole::Junior,
            SpecialistRole::Senior,
        ];

        let mut bindings = Vec::new();
        for team in teams {
            for role in roles {
                bindings.push(
                    RoleBinding::new(team, role, model.clone()).with_reason(&reason),
                );
            }
        }

        Self {
            bindings,
            hardware_tier: Some(tier),
            states: Vec::new(),
            last_benchmark_ts: None,
        }
    }

    /// Get binding for a team and role
    pub fn get_binding(&self, team: Team, role: SpecialistRole) -> Option<&RoleBinding> {
        self.bindings
            .iter()
            .find(|b| b.team == team && b.role == role)
    }

    /// Get model name for a team and role
    pub fn get_model_name(&self, team: Team, role: SpecialistRole) -> Option<&str> {
        self.get_binding(team, role).map(|b| b.model.name.as_str())
    }

    /// Update or add a binding
    pub fn set_binding(&mut self, binding: RoleBinding) {
        if let Some(existing) = self
            .bindings
            .iter_mut()
            .find(|b| b.team == binding.team && b.role == binding.role)
        {
            *existing = binding;
        } else {
            self.bindings.push(binding);
        }
    }

    /// Update model state
    pub fn update_state(&mut self, model_name: &str, state: ModelState) {
        if let Some((_, existing)) = self.states.iter_mut().find(|(n, _)| n == model_name) {
            *existing = state;
        } else {
            self.states.push((model_name.to_string(), state));
        }
    }

    /// Get model state
    pub fn get_state(&self, model_name: &str) -> Option<&ModelState> {
        self.states
            .iter()
            .find(|(n, _)| n == model_name)
            .map(|(_, s)| s)
    }

    /// Check if model is present
    pub fn is_model_present(&self, model_name: &str) -> bool {
        self.get_state(model_name).map(|s| s.present).unwrap_or(false)
    }

    /// Get all unique model names from bindings
    pub fn required_models(&self) -> Vec<&str> {
        let mut models: Vec<&str> = self.bindings.iter().map(|b| b.model.name.as_str()).collect();
        models.sort();
        models.dedup();
        models
    }

    /// Get missing models (required but not present)
    pub fn missing_models(&self) -> Vec<&str> {
        self.required_models()
            .into_iter()
            .filter(|m| !self.is_model_present(m))
            .collect()
    }

    /// Check if all required models are present
    pub fn all_models_present(&self) -> bool {
        self.missing_models().is_empty()
    }

    /// Clear all bindings and states (for reset)
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.states.clear();
        self.hardware_tier = None;
        self.last_benchmark_ts = None;
    }
}

/// Path to model registry file
pub fn model_registry_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".anna").join("model_registry.json")
}

/// Load model registry from disk
pub fn load_model_registry() -> ModelRegistry {
    let path = model_registry_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(registry) = serde_json::from_str(&content) {
                return registry;
            }
        }
    }
    ModelRegistry::new()
}

/// Save model registry to disk
pub fn save_model_registry(registry: &ModelRegistry) -> std::io::Result<()> {
    let path = model_registry_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(registry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(&path, content)
}

/// Parse ollama list output to extract model states
pub fn parse_ollama_list(output: &str) -> Vec<(String, ModelState)> {
    let mut states = Vec::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Skip header line
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0].to_string();
            // Try to parse size (e.g., "2.0 GB" or "1.5GB")
            let size_bytes = if parts.len() >= 3 {
                parse_size_string(parts[2])
            } else {
                None
            };

            states.push((
                name,
                ModelState {
                    present: true,
                    digest: None,
                    last_seen_ts: Some(now),
                    size_bytes,
                },
            ));
        }
    }

    states
}

/// Parse size string like "2.0 GB" or "1.5GB" to bytes
fn parse_size_string(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();
    let (num_str, multiplier) = if s.ends_with("GB") {
        (s.trim_end_matches("GB").trim(), 1024 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (s.trim_end_matches("MB").trim(), 1024 * 1024)
    } else if s.ends_with("KB") {
        (s.trim_end_matches("KB").trim(), 1024)
    } else {
        return None;
    };

    num_str.parse::<f64>().ok().map(|n| (n * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_spec_new() {
        let spec = ModelSpec::new("llama3.2:3b").with_size(2.0).with_quant("Q4_K_M");
        assert_eq!(spec.name, "llama3.2:3b");
        assert_eq!(spec.size_hint_gb, Some(2.0));
        assert_eq!(spec.quant, Some("Q4_K_M".to_string()));
    }

    #[test]
    fn test_hardware_tier_from_specs() {
        assert_eq!(HardwareTier::from_specs(2.0, 2, false), HardwareTier::Low);
        assert_eq!(HardwareTier::from_specs(4.0, 4, false), HardwareTier::Medium);
        assert_eq!(HardwareTier::from_specs(8.0, 8, false), HardwareTier::High);
        assert_eq!(HardwareTier::from_specs(16.0, 8, true), HardwareTier::VeryHigh);
    }

    #[test]
    fn test_recommended_model_for_tier() {
        let low = recommended_model_for_tier(HardwareTier::Low);
        assert!(low.name.contains("0.5b"));

        let high = recommended_model_for_tier(HardwareTier::High);
        assert!(high.name.contains("3b"));
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = ModelRegistry::with_defaults(HardwareTier::Medium);

        // 8 teams × 3 roles = 24 bindings
        assert_eq!(registry.bindings.len(), 24);
        assert_eq!(registry.hardware_tier, Some(HardwareTier::Medium));

        // All bindings should use the same model
        let model_name = &registry.bindings[0].model.name;
        assert!(registry.bindings.iter().all(|b| &b.model.name == model_name));
    }

    #[test]
    fn test_registry_get_binding() {
        let registry = ModelRegistry::with_defaults(HardwareTier::High);
        let binding = registry.get_binding(Team::Storage, SpecialistRole::Junior).unwrap();
        assert_eq!(binding.team, Team::Storage);
        assert_eq!(binding.role, SpecialistRole::Junior);
    }

    #[test]
    fn test_registry_model_presence() {
        let mut registry = ModelRegistry::with_defaults(HardwareTier::Medium);

        // Initially no states
        assert!(registry.missing_models().len() > 0);
        assert!(!registry.all_models_present());

        // Add state for the model
        let model_name = registry.bindings[0].model.name.clone();
        registry.update_state(
            &model_name,
            ModelState {
                present: true,
                digest: None,
                last_seen_ts: None,
                size_bytes: None,
            },
        );

        assert!(registry.is_model_present(&model_name));
        assert!(registry.all_models_present());
    }

    #[test]
    fn test_parse_ollama_list() {
        let output = "NAME              ID              SIZE      MODIFIED\n\
                      llama3.2:3b       abc123def456    2.0 GB    2 days ago\n\
                      qwen2.5:1.5b      789xyz012345    1.0 GB    1 week ago";

        let states = parse_ollama_list(output);
        assert_eq!(states.len(), 2);
        assert_eq!(states[0].0, "llama3.2:3b");
        assert!(states[0].1.present);
        assert_eq!(states[1].0, "qwen2.5:1.5b");
    }

    #[test]
    fn test_parse_size_string() {
        assert_eq!(parse_size_string("2.0 GB"), Some(2 * 1024 * 1024 * 1024));
        assert_eq!(parse_size_string("1.5GB"), Some((1.5 * 1024.0 * 1024.0 * 1024.0) as u64));
        assert_eq!(parse_size_string("512 MB"), Some(512 * 1024 * 1024));
        assert_eq!(parse_size_string("invalid"), None);
    }
}
