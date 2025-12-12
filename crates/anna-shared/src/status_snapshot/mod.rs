//! StatusSnapshot - single authoritative system state snapshot (v0.0.211).
//!
//! Provides comprehensive, deterministic system state for annactl status.
//! All fields are optional where discovery can fail - no fiction.
//!
//! v0.0.29: Initial implementation.
//! v0.0.211: Modularized into domain-focused submodules.
//! v0.0.454: Added teams_info for dynamic team availability.
//! v0.0.463: Enhanced permissions with folder info per VISION.md Phase 29.

mod config;
mod daemon;
mod helpers_info;
mod models;
mod permissions;
mod snapshot;
mod teams_info;
mod tests;
mod update;
mod version;

// Re-export all types and functions
pub use config::ConfigInfo;
pub use daemon::DaemonInfo;
pub use helpers_info::{HelperPackageLite, HelpersInfo};
pub use models::{ModelDownloadStatus, ModelsInfo, RoleModelBinding};
pub use permissions::{FolderPermission, PermissionsInfo};
pub use snapshot::StatusSnapshot;
pub use teams_info::{HiddenTeam, TeamsInfo};
pub use update::{UpdateInfo, UpdateResult};
pub use version::VersionInfo;
