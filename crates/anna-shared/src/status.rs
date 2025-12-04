//! Status types for Anna daemon.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ledger::LedgerSummary;

/// Overall daemon status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub version: String,
    pub state: DaemonState,
    pub pid: Option<u32>,
    pub uptime_seconds: u64,
    pub debug_mode: bool,
    pub update: UpdateStatus,
    pub llm: LlmStatus,
    pub hardware: HardwareInfo,
    pub ledger: LedgerSummary,
    pub last_error: Option<String>,
}

/// Update subsystem status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub last_check: Option<DateTime<Utc>>,
    pub next_check: Option<DateTime<Utc>>,
    pub available_version: Option<String>,
    pub update_available: bool,
}

/// LLM subsystem status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStatus {
    pub state: LlmState,
    pub provider: String,
    pub phase: Option<String>,
    pub progress: Option<ProgressInfo>,
    pub benchmark: Option<BenchmarkResult>,
    pub models: Vec<ModelInfo>,
}

/// LLM state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LlmState {
    Bootstrapping,
    Ready,
    Error,
}

impl std::fmt::Display for LlmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmState::Bootstrapping => write!(f, "BOOTSTRAPPING"),
            LlmState::Ready => write!(f, "READY"),
            LlmState::Error => write!(f, "ERROR"),
        }
    }
}

/// Progress information for downloads/operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub current_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: u64,
}

impl ProgressInfo {
    pub fn percent(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            self.current_bytes as f32 / self.total_bytes as f32
        }
    }
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub cpu: String,
    pub ram: String,
    pub gpu: String,
}

/// Daemon state (simplified)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DaemonState {
    Starting,
    Running,
    Error,
}

impl std::fmt::Display for DaemonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonState::Starting => write!(f, "STARTING"),
            DaemonState::Running => write!(f, "RUNNING"),
            DaemonState::Error => write!(f, "ERROR"),
        }
    }
}

/// Ollama service status (kept for internal use)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
}

/// Information about a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub role: String,
    pub size_bytes: u64,
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

impl Default for LlmStatus {
    fn default() -> Self {
        Self {
            state: LlmState::Bootstrapping,
            provider: "ollama".to_string(),
            phase: None,
            progress: None,
            benchmark: None,
            models: Vec::new(),
        }
    }
}

impl Default for UpdateStatus {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: crate::DEFAULT_UPDATE_CHECK_INTERVAL,
            last_check: None,
            next_check: None,
            available_version: None,
            update_available: false,
        }
    }
}
