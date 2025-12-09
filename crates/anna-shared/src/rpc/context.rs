//! Runtime context types (v0.0.260).
//!
//! v0.0.260: Added OS info to hardware summary.

use serde::{Deserialize, Serialize};

/// Runtime context injected into every LLM request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeContext {
    pub version: String,
    pub daemon_running: bool,
    pub capabilities: Capabilities,
    pub hardware: HardwareSummary,
    pub probes: std::collections::HashMap<String, String>,
}

/// Capability flags for the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub can_read_system_info: bool,
    pub can_run_probes: bool,
    pub can_modify_files: bool,
    pub can_install_packages: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            can_read_system_info: true,
            can_run_probes: true,
            can_modify_files: false,
            can_install_packages: false,
        }
    }
}

/// Hardware summary for context (v0.0.260: added OS info)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareSummary {
    #[serde(default)]
    pub cpu_model: String,
    #[serde(default)]
    pub cpu_cores: u32,
    #[serde(default)]
    pub ram_gb: f64,
    #[serde(default)]
    pub gpu: Option<String>,
    #[serde(default)]
    pub gpu_vram_gb: Option<f64>,
    /// v0.0.260: OS name (e.g., "Linux")
    #[serde(default)]
    pub os_name: String,
    /// v0.0.260: Kernel version (e.g., "6.17.9-arch1-1")
    #[serde(default)]
    pub kernel: String,
    /// v0.0.260: Distribution (e.g., "Arch Linux")
    #[serde(default)]
    pub distro: String,
}
