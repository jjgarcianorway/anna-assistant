//! StatusSnapshot - single authoritative system state snapshot (v0.0.211).
//!
//! Provides comprehensive, deterministic system state for annactl status.
//! All fields are optional where discovery can fail - no fiction.
//!
//! v0.0.29: Initial implementation.
//! v0.0.211: Modularized into domain-focused submodules.

mod config;
mod daemon;
mod helpers_info;
mod models;
mod permissions;
mod snapshot;
mod tests;
mod update;
mod version;

// Re-export all types and functions
pub use config::ConfigInfo;
pub use daemon::DaemonInfo;
pub use helpers_info::{HelperPackageLite, HelpersInfo};
pub use models::{ModelDownloadStatus, ModelsInfo, RoleModelBinding};
pub use permissions::PermissionsInfo;
pub use snapshot::StatusSnapshot;
pub use update::{UpdateInfo, UpdateResult};
pub use version::VersionInfo;
