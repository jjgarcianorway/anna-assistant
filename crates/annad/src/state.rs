//! Daemon state management.

use std::sync::Arc;
use std::time::Instant;

use anna_shared::ledger::Ledger;
use anna_shared::status::{
    BenchmarkResult, DaemonState, DaemonStatus, HardwareInfo, LlmState, LlmStatus,
    ModelInfo, OllamaStatus, ProgressInfo,
};
use anna_shared::VERSION;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

/// Shared daemon state
pub struct DaemonStateInner {
    pub state: DaemonState,
    pub pid: u32,
    pub started_at: Instant,
    pub debug_mode: bool,
    pub auto_update: bool,
    pub last_update_check: Option<DateTime<Utc>>,
    pub next_update_check: Option<DateTime<Utc>>,
    pub ollama: OllamaStatus,
    pub llm: LlmStatus,
    pub hardware: HardwareInfo,
    pub ledger: Ledger,
    pub last_error: Option<String>,
}

impl DaemonStateInner {
    pub fn new() -> Self {
        Self {
            state: DaemonState::Starting,
            pid: std::process::id(),
            started_at: Instant::now(),
            debug_mode: true,
            auto_update: true,
            last_update_check: None,
            next_update_check: None,
            ollama: OllamaStatus::default(),
            llm: LlmStatus::default(),
            hardware: HardwareInfo::default(),
            ledger: Ledger::new(),
            last_error: None,
        }
    }

    pub fn to_status(&self) -> DaemonStatus {
        DaemonStatus {
            version: VERSION.to_string(),
            state: self.state.clone(),
            pid: Some(self.pid),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            debug_mode: self.debug_mode,
            auto_update: self.auto_update,
            last_update_check: self.last_update_check,
            next_update_check: self.next_update_check,
            llm: self.llm.clone(),
            hardware: self.hardware.clone(),
            ledger: self.ledger.summary(),
            last_error: self.last_error.clone(),
        }
    }

    pub fn set_llm_phase(&mut self, phase: &str) {
        self.llm.phase = Some(phase.to_string());
    }

    #[allow(dead_code)]
    pub fn set_llm_progress(&mut self, current: u64, total: u64, speed: u64, eta: u64) {
        self.llm.progress = Some(ProgressInfo {
            current_bytes: current,
            total_bytes: total,
            speed_bytes_per_sec: speed,
            eta_seconds: eta,
        });
    }

    #[allow(dead_code)]
    pub fn clear_llm_progress(&mut self) {
        self.llm.progress = None;
    }

    pub fn set_llm_ready(&mut self) {
        self.llm.state = LlmState::Ready;
        self.llm.phase = None;
        self.llm.progress = None;
        self.state = DaemonState::Running;
    }

    pub fn set_benchmark_result(&mut self, cpu: &str, ram: &str, gpu: &str) {
        self.llm.benchmark = Some(BenchmarkResult {
            cpu: cpu.to_string(),
            ram: ram.to_string(),
            gpu: gpu.to_string(),
        });
    }

    pub fn add_model(&mut self, name: &str, role: &str, size: u64) {
        self.llm.models.push(ModelInfo {
            name: name.to_string(),
            role: role.to_string(),
            size_bytes: size,
            pulled: true,
        });
    }
}

/// Thread-safe shared state handle
pub type SharedState = Arc<RwLock<DaemonStateInner>>;

pub fn create_shared_state() -> SharedState {
    Arc::new(RwLock::new(DaemonStateInner::new()))
}
