//! Core types and enums for the recipe engine.

use serde::{Deserialize, Serialize};

/// Recipe kind - what type of solution this provides
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeKind {
    /// Read-only probe execution (no changes)
    ProbeOnly,
    /// Configuration change
    Configure,
    /// System inspection/diagnosis
    Inspect,
    /// Problem diagnosis with suggested fixes
    Diagnose,
    /// Generate a report/summary
    Report,
}

impl Default for RecipeKind {
    fn default() -> Self {
        Self::ProbeOnly
    }
}

impl std::fmt::Display for RecipeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProbeOnly => write!(f, "probe_only"),
            Self::Configure => write!(f, "configure"),
            Self::Inspect => write!(f, "inspect"),
            Self::Diagnose => write!(f, "diagnose"),
            Self::Report => write!(f, "report"),
        }
    }
}

/// Evidence requirement - what data a recipe needs to run
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRequirement {
    None,
    Meminfo,
    Swaps,
    DfRoot,
    SystemdFailed,
    PacmanList,
    JournalErrors,
    NetworkInterfaces,
    GpuInfo,
    AudioDevices,
    /// Extensible custom requirement
    Custom(String),
}

impl std::fmt::Display for EvidenceRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Meminfo => write!(f, "meminfo"),
            Self::Swaps => write!(f, "swaps"),
            Self::DfRoot => write!(f, "df_root"),
            Self::SystemdFailed => write!(f, "systemd_failed"),
            Self::PacmanList => write!(f, "pacman_list"),
            Self::JournalErrors => write!(f, "journal_errors"),
            Self::NetworkInterfaces => write!(f, "network_interfaces"),
            Self::GpuInfo => write!(f, "gpu_info"),
            Self::AudioDevices => write!(f, "audio_devices"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// Recipe step type - what kind of action this step performs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStepType {
    /// Run an existing probe by ID
    RunProbe,
    /// Run an explicit shell command
    RunCommand,
    /// Check a condition on previous outputs
    CheckCondition,
    /// Edit a file (templated)
    EditFile,
    /// Render the final answer
    RenderAnswer,
    /// Call another recipe (composition)
    Subrecipe,
}

impl std::fmt::Display for RecipeStepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RunProbe => write!(f, "run_probe"),
            Self::RunCommand => write!(f, "run_command"),
            Self::CheckCondition => write!(f, "check_condition"),
            Self::EditFile => write!(f, "edit_file"),
            Self::RenderAnswer => write!(f, "render_answer"),
            Self::Subrecipe => write!(f, "subrecipe"),
        }
    }
}

/// Risk level for recipe execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// No system changes
    ReadOnly,
    /// Minor changes, easily reversible
    Low,
    /// Significant changes
    Medium,
    /// Potentially destructive
    High,
}

/// A recipe parameter (variable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeParameter {
    /// Parameter name (e.g., "service_name")
    pub name: String,
    /// Description
    pub description: String,
    /// How to extract from query
    pub extraction_hint: String,
    /// Optional default value
    pub default: Option<String>,
    /// Whether required
    pub required: bool,
}
