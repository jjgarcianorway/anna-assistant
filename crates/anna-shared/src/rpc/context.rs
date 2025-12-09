//! Runtime context types (v0.0.220).

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

/// Hardware summary for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSummary {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub ram_gb: f64,
    pub gpu: Option<String>,
    pub gpu_vram_gb: Option<f64>,
}
