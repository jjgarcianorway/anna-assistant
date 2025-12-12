//! Hardware-aware model selection and helper management (v0.0.434).
//!
//! Provides:
//! - Hardware profiling and capability tiers
//! - Model catalog and selection policy
//! - Model installation, verification, and health
//! - Helper tool policy and lifecycle
//! - Honest reflection in status and stats

pub mod catalog;
pub mod helper_config;
pub mod helpers;
pub mod integration;
pub mod model_config;
pub mod model_health;
pub mod model_plan;
pub mod profile;
pub mod status;
pub mod tests;

pub use catalog::{ModelCatalog, ModelEntry, ModelRole};
pub use helper_config::{HelperConfig, HelperInstallPolicy};
pub use helpers::{HelperCatalog, HelperEntry, HelperManager, HelperState};
pub use integration::{ProbeHelper, SpecialistHelper};
pub use model_config::{AutoInstallPolicy, ModelConfig};
pub use model_health::{ModelHealth, ModelStatus, ModelVerifier};
pub use model_plan::{ModelPlan, ModelPlanner};
pub use profile::{CapabilityTier, CpuInfo, GpuInfo, GpuVendor, HardwareProfile, StorageInfo};
pub use status::{HardwareStatus, HelperStatusSection, LlmSection};

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
