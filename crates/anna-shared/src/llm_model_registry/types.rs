// v0.0.531: LLM Model Registry Types
// Tracks installed LLM models and their assignments to specialists per VISION.md

use serde::{Deserialize, Serialize};

/// Model capability level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ModelCapability {
    Light,      // Fast, low resource (for juniors)
    Standard,   // Balanced
    Heavy,      // Deep thinking (for seniors)
    Multimodal, // Vision/audio capable
}

impl Default for ModelCapability {
    fn default() -> Self {
        Self::Standard
    }
}

impl std::fmt::Display for ModelCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "Light"),
            Self::Standard => write!(f, "Standard"),
            Self::Heavy => write!(f, "Heavy"),
            Self::Multimodal => write!(f, "Multimodal"),
        }
    }
}

/// Model installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ModelStatus {
    #[default]
    Available,
    Downloading,
    Installing,
    Ready,
    Failed,
    Removed,
}

impl std::fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Available => write!(f, "Available"),
            Self::Downloading => write!(f, "Downloading"),
            Self::Installing => write!(f, "Installing"),
            Self::Ready => write!(f, "Ready"),
            Self::Failed => write!(f, "Failed"),
            Self::Removed => write!(f, "Removed"),
        }
    }
}

/// Who installed the model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InstalledBy {
    #[default]
    User,
    Anna,
    System,
}

impl std::fmt::Display for InstalledBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "User"),
            Self::Anna => write!(f, "Anna"),
            Self::System => write!(f, "System"),
        }
    }
}

/// Individual model record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecord {
    pub name: String,
    pub capability: ModelCapability,
    pub status: ModelStatus,
    pub installed_by: InstalledBy,
    pub size_gb: f64,
    pub vram_required_gb: f64,
    pub assigned_specialists: Vec<String>,
    pub usage_count: u32,
    pub avg_response_ms: u64,
    pub installed_at: Option<String>,
}

impl ModelRecord {
    /// Create a new model record
    pub fn new(name: &str, capability: ModelCapability, size_gb: f64, vram_gb: f64) -> Self {
        Self {
            name: name.to_string(),
            capability,
            status: ModelStatus::Available,
            installed_by: InstalledBy::User,
            size_gb,
            vram_required_gb: vram_gb,
            assigned_specialists: Vec::new(),
            usage_count: 0,
            avg_response_ms: 0,
            installed_at: None,
        }
    }

    /// Install model
    pub fn install(&mut self, by: InstalledBy, timestamp: &str) {
        self.status = ModelStatus::Ready;
        self.installed_by = by;
        self.installed_at = Some(timestamp.to_string());
    }

    /// Assign to specialist
    pub fn assign(&mut self, specialist_id: &str) {
        if !self.assigned_specialists.contains(&specialist_id.to_string()) {
            self.assigned_specialists.push(specialist_id.to_string());
        }
    }

    /// Unassign from specialist
    pub fn unassign(&mut self, specialist_id: &str) {
        self.assigned_specialists.retain(|s| s != specialist_id);
    }

    /// Record usage
    pub fn record_use(&mut self, response_ms: u64) {
        self.usage_count += 1;
        let total = self.avg_response_ms * (self.usage_count - 1) as u64 + response_ms;
        self.avg_response_ms = total / self.usage_count as u64;
    }

    /// Is model ready for use?
    pub fn is_ready(&self) -> bool {
        self.status == ModelStatus::Ready
    }
}
