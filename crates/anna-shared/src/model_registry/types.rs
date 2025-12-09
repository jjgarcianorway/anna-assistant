//! Model registry types (v0.0.201).

use crate::specialists::SpecialistRole;
use crate::teams::Team;
use serde::{Deserialize, Serialize};

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
