//! Hardware profiling and capability tiers (v0.0.434).
//!
//! Collects system information and maps to capability tiers.
//!
//! This module has been split into smaller submodules for maintainability.

// Re-export all public types and functions from sibling modules
pub use super::hardware_profile::HardwareProfile;
pub use super::types::{CapabilityTier, CpuInfo, GpuInfo, GpuVendor, OsInfo, StorageInfo};
