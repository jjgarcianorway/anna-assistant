//! Status types for Anna daemon.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ledger::LedgerSummary;

/// Overall daemon status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub version: String,
    pub state: DaemonState,
    pub uptime_seconds: u64,
    pub last_update_check: Option<DateTime<Utc>>,
    pub next_update_check: Option<DateTime<Utc>>,
    pub ollama: OllamaStatus,
    pub model: Option<ModelInfo>,
    pub hardware: HardwareInfo,
    pub ledger: LedgerSummary,
}

/// Daemon state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DaemonState {
    Starting,
    InstallingOllama,
    ProbingHardware,
    Benchmarking,
    PullingModel,
    Ready,
    Error,
}

impl std::fmt::Display for DaemonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonState::Starting => write!(f, "Starting"),
            DaemonState::InstallingOllama => write!(f, "Installing Ollama"),
            DaemonState::ProbingHardware => write!(f, "Probing hardware"),
            DaemonState::Benchmarking => write!(f, "Running benchmark"),
            DaemonState::PullingModel => write!(f, "Pulling model"),
            DaemonState::Ready => write!(f, "Ready"),
            DaemonState::Error => write!(f, "Error"),
        }
    }
}

/// Ollama service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
}

/// Information about the selected model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size_bytes: u64,
    pub quantization: Option<String>,
    pub pulled: bool,
}

/// Hardware information from probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu_cores: u32,
    pub cpu_model: String,
    pub ram_bytes: u64,
    pub gpu: Option<GpuInfo>,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: String,
    pub model: String,
    pub vram_bytes: u64,
}

impl Default for HardwareInfo {
    fn default() -> Self {
        Self {
            cpu_cores: 0,
            cpu_model: "Unknown".to_string(),
            ram_bytes: 0,
            gpu: None,
        }
    }
}

impl Default for OllamaStatus {
    fn default() -> Self {
        Self {
            installed: false,
            running: false,
            version: None,
        }
    }
}
