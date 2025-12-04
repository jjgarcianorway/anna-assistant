//! Daemon state management.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anna_shared::ledger::Ledger;
use anna_shared::progress::ProgressEvent;
use anna_shared::rpc::ProbeResult;
use anna_shared::status::{
    BenchmarkResult, DaemonState, DaemonStatus, HardwareInfo, LlmState, LlmStatus, ModelInfo,
    OllamaStatus, ProgressInfo, UpdateStatus,
};
use anna_shared::{DEFAULT_UPDATE_CHECK_INTERVAL, VERSION};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::config::Config;

/// Probe cache TTL (30 seconds)
pub const PROBE_CACHE_TTL: Duration = Duration::from_secs(30);

/// Cached probe result with timestamp
#[derive(Debug, Clone)]
pub struct CachedProbe {
    pub result: ProbeResult,
    pub cached_at: Instant,
}

impl CachedProbe {
    pub fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < PROBE_CACHE_TTL
    }
}

/// Shared daemon state
pub struct DaemonStateInner {
    pub state: DaemonState,
    pub pid: u32,
    pub started_at: Instant,
    pub debug_mode: bool,
    pub update: UpdateStateInner,
    pub ollama: OllamaStatus,
    pub llm: LlmStatus,
    pub hardware: HardwareInfo,
    pub ledger: Ledger,
    pub last_error: Option<String>,
    /// Probe result cache (command -> cached result)
    pub probe_cache: HashMap<String, CachedProbe>,
    /// Progress events for current/last request (for polling)
    pub progress_events: Vec<ProgressEvent>,
    /// Configuration loaded from file
    pub config: Config,
}

/// Update state tracking
pub struct UpdateStateInner {
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub last_check: Option<DateTime<Utc>>,
    pub next_check: Option<DateTime<Utc>>,
    pub available_version: Option<String>,
    pub update_available: bool,
}

impl Default for UpdateStateInner {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: DEFAULT_UPDATE_CHECK_INTERVAL,
            last_check: None,
            next_check: None,
            available_version: None,
            update_available: false,
        }
    }
}

impl DaemonStateInner {
    pub fn new() -> Self {
        Self {
            state: DaemonState::Starting,
            pid: std::process::id(),
            started_at: Instant::now(),
            debug_mode: true, // Debug mode ON by default
            update: UpdateStateInner::default(),
            ollama: OllamaStatus::default(),
            llm: LlmStatus::default(),
            hardware: HardwareInfo::default(),
            ledger: Ledger::new(),
            last_error: None,
            probe_cache: HashMap::new(),
            progress_events: Vec::new(),
            config: Config::load(),
        }
    }

    /// Get cached probe result if still valid
    pub fn get_cached_probe(&self, command: &str) -> Option<ProbeResult> {
        self.probe_cache.get(command).and_then(|cached| {
            if cached.is_valid() {
                Some(cached.result.clone())
            } else {
                None
            }
        })
    }

    /// Cache a probe result
    pub fn cache_probe(&mut self, result: ProbeResult) {
        self.probe_cache.insert(
            result.command.clone(),
            CachedProbe {
                result,
                cached_at: Instant::now(),
            },
        );
    }

    /// Clean expired probe cache entries
    pub fn clean_probe_cache(&mut self) {
        self.probe_cache.retain(|_, cached| cached.is_valid());
    }

    pub fn to_status(&self) -> DaemonStatus {
        DaemonStatus {
            version: VERSION.to_string(),
            state: self.state.clone(),
            pid: Some(self.pid),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            debug_mode: self.debug_mode,
            update: UpdateStatus {
                enabled: self.update.enabled,
                check_interval_secs: self.update.check_interval_secs,
                last_check: self.update.last_check,
                next_check: self.update.next_check,
                available_version: self.update.available_version.clone(),
                update_available: self.update.update_available,
            },
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
