//! Hardware capability struct

use serde::{Deserialize, Serialize};

use super::types::{HardwareCategory, HardwareStatus};

/// A hardware capability record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapability {
    /// Capability name (e.g., "ethernet", "wifi", "sound")
    pub name: String,
    /// Category
    pub category: HardwareCategory,
    /// Status
    pub status: HardwareStatus,
    /// Device name/model if detected
    pub device: Option<String>,
    /// When last checked
    pub last_check: u64,
    /// Relevant helpers for this capability
    pub relevant_helpers: Vec<String>,
}
