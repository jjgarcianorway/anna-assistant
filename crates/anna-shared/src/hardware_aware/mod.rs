//! Hardware-aware model selection and helper management (v0.0.434).
//!
//! Provides:
//! - Hardware profiling and capability tiers
//! - Model catalog and selection policy
//! - Model installation, verification, and health
//! - Helper tool policy and lifecycle
//! - Honest reflection in status and stats

pub mod catalog;
mod detection;
mod hardware_profile;
pub mod helper_config;
pub mod helper_entry;
pub mod helper_error;
pub mod helper_manager;
pub mod helper_state;
pub mod integration;
pub mod model_config;
pub mod model_health;
pub mod model_plan;
pub mod profile;
pub mod status;
mod types;

#[cfg(test)]
mod test_fixtures;
#[cfg(test)]
mod test_core;
#[cfg(test)]
mod test_integration;

pub use catalog::{ModelCatalog, ModelEntry, ModelRole};
pub use helper_config::{HelperConfig, HelperInstallPolicy};
pub use helper_entry::{HelperCatalog, HelperEntry};
pub use helper_error::HelperError;
pub use helper_manager::HelperManager;
pub use helper_state::{HelperInstalledBy, HelperState};
pub use integration::{
    HelperSuggestion, ModelAvailability, ProbeCommand, ProbeHelper, SpecialistHelper,
};
pub use model_config::{AutoInstallPolicy, ModelConfig};
pub use model_health::{ModelHealth, ModelStatus, ModelVerifier};
pub use model_plan::{ModelPlan, ModelPlanner};
pub use profile::{CapabilityTier, CpuInfo, GpuInfo, GpuVendor, HardwareProfile, OsInfo, StorageInfo};
pub use status::{
    HardwareStatus, HelperStatusEntry, HelperStatusSection, HelperUsage, HelperUsageStats,
    LlmSection, ModelError, ModelStatusEntry, ModelUsage, ModelUsageStats, SystemProfileSection,
};

/// Current catalog version.
pub const CATALOG_VERSION: u32 = 1;

/// Current profile version.
pub const PROFILE_VERSION: u32 = 1;

/// Default model storage limit in GB.
pub const DEFAULT_MAX_MODEL_DISK_GB: u32 = 25;

/// Anna data directory.
pub const ANNA_DATA_DIR: &str = "/var/lib/anna";

/// Anna user config directory.
pub const ANNA_CONFIG_DIR: &str = ".anna";
