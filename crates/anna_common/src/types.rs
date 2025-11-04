//! Core data types for Anna Assistant

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Risk level for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Priority level for recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Critical security/driver issues
    Mandatory = 3,
    /// Significant improvements, quality of life
    Recommended = 2,
    /// Nice-to-have optimizations
    Optional = 1,
    /// Beautification, minor enhancements
    Cosmetic = 0,
}

/// Autonomy tier configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomyTier {
    /// Tier 0: Advise only, never execute
    AdviseOnly = 0,
    /// Tier 1: Auto-execute Low risk only
    SafeAutoApply = 1,
    /// Tier 2: Auto-execute Low + Medium risk
    SemiAutonomous = 2,
    /// Tier 3: Auto-execute all risk levels
    FullyAutonomous = 3,
}

impl AutonomyTier {
    /// Check if this tier allows auto-execution for a given risk level
    pub fn allows(&self, risk: RiskLevel) -> bool {
        match (self, risk) {
            (AutonomyTier::AdviseOnly, _) => false,
            (AutonomyTier::SafeAutoApply, RiskLevel::Low) => true,
            (AutonomyTier::SafeAutoApply, _) => false,
            (AutonomyTier::SemiAutonomous, RiskLevel::High) => false,
            (AutonomyTier::SemiAutonomous, _) => true,
            (AutonomyTier::FullyAutonomous, _) => true,
        }
    }
}

/// System facts collected by telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemFacts {
    pub timestamp: DateTime<Utc>,

    // Hardware
    pub hostname: String,
    pub kernel: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub total_memory_gb: f64,
    pub gpu_vendor: Option<String>,
    pub storage_devices: Vec<StorageDevice>,

    // Software & Packages
    pub installed_packages: usize,
    pub orphan_packages: Vec<String>,
    pub package_groups: Vec<String>, // detected groups: base-devel, gnome, kde, etc.

    // Network
    pub network_interfaces: Vec<String>,
    pub has_wifi: bool,
    pub has_ethernet: bool,

    // User Environment
    pub shell: String, // bash, zsh, fish
    pub desktop_environment: Option<String>, // GNOME, KDE, i3, etc.
    pub display_server: Option<String>, // X11, Wayland

    // User Behavior (learned from system)
    pub frequently_used_commands: Vec<CommandUsage>,
    pub dev_tools_detected: Vec<String>, // git, docker, vim, etc.
    pub media_usage: MediaUsageProfile,
    pub common_file_types: Vec<String>, // .py, .rs, .js, .mp4, etc.
}

/// Command usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandUsage {
    pub command: String,
    pub count: usize,
}

/// Media usage profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUsageProfile {
    pub has_video_files: bool,
    pub has_audio_files: bool,
    pub has_images: bool,
    pub video_player_installed: bool,
    pub audio_player_installed: bool,
    pub image_viewer_installed: bool,
}

/// Storage device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub name: String,
    pub filesystem: String,
    pub size_gb: f64,
    pub used_gb: f64,
    pub mount_point: String,
}

/// A single piece of advice from the recommendation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advice {
    pub id: String,
    pub title: String,
    pub reason: String,
    pub action: String,
    pub command: Option<String>,
    pub risk: RiskLevel,
    pub priority: Priority,
    pub wiki_refs: Vec<String>,
    pub category: String, // "security", "drivers", "development", "media", "beautification", etc.
}

/// An action to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub advice_id: String,
    pub command: String,
    pub executed_at: DateTime<Utc>,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Rollback token for reversing an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackToken {
    pub action_id: String,
    pub advice_id: String,
    pub executed_at: DateTime<Utc>,
    pub command: String,
    pub rollback_command: Option<String>,
    pub snapshot_before: Option<String>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action_type: String,
    pub details: String,
    pub success: bool,
}

/// RPC Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub params: serde_json::Value,
}

/// RPC Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}
