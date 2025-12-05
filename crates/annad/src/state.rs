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
use anna_shared::status_snapshot::{
    ConfigInfo, DaemonInfo, HelpersInfo, ModelsInfo, PermissionsInfo, RoleModelBinding,
    StatusSnapshot, UpdateInfo, UpdateResult, VersionInfo,
};
use anna_shared::{DEFAULT_UPDATE_CHECK_INTERVAL, VERSION};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::config::Config;

/// Probe cache TTL (30 seconds)
pub const PROBE_CACHE_TTL: Duration = Duration::from_secs(30);

/// Max number of latency records to keep per stage
pub const MAX_LATENCY_RECORDS: usize = 20;

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

/// Latency stats for a pipeline stage
#[derive(Debug, Clone, Default)]
pub struct LatencyStats {
    /// Last N latency samples in milliseconds
    pub samples: Vec<u64>,
}

impl LatencyStats {
    /// Add a latency sample
    pub fn add(&mut self, ms: u64) {
        self.samples.push(ms);
        if self.samples.len() > MAX_LATENCY_RECORDS {
            self.samples.remove(0);
        }
    }

    /// Average latency in ms
    pub fn avg_ms(&self) -> Option<u64> {
        if self.samples.is_empty() {
            None
        } else {
            Some(self.samples.iter().sum::<u64>() / self.samples.len() as u64)
        }
    }

    /// P50 (median) latency in ms (v0.0.36)
    pub fn p50_ms(&self) -> Option<u64> {
        self.percentile_ms(0.50)
    }

    /// P90 latency in ms (v0.0.36)
    pub fn p90_ms(&self) -> Option<u64> {
        self.percentile_ms(0.90)
    }

    /// P95 latency in ms
    pub fn p95_ms(&self) -> Option<u64> {
        self.percentile_ms(0.95)
    }

    /// Calculate percentile latency (v0.0.36)
    fn percentile_ms(&self, p: f64) -> Option<u64> {
        if self.samples.is_empty() {
            None
        } else {
            let mut sorted = self.samples.clone();
            sorted.sort_unstable();
            let idx = (sorted.len() as f64 * p).ceil() as usize - 1;
            Some(sorted[idx.min(sorted.len() - 1)])
        }
    }

    /// Min latency in ms (v0.0.36)
    pub fn min_ms(&self) -> Option<u64> {
        self.samples.iter().min().copied()
    }

    /// Max latency in ms (v0.0.36)
    pub fn max_ms(&self) -> Option<u64> {
        self.samples.iter().max().copied()
    }

    /// Number of samples collected
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

/// Per-stage latency tracking
#[derive(Debug, Clone, Default)]
pub struct PipelineLatency {
    pub translator: LatencyStats,
    pub probes: LatencyStats,
    pub specialist: LatencyStats,
    pub total: LatencyStats,
}

/// Shared daemon state
pub struct DaemonStateInner {
    pub state: DaemonState,
    pub pid: u32,
    pub started_at: Instant,
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
    /// Per-stage latency statistics
    pub latency: PipelineLatency,
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
            update: UpdateStateInner::default(),
            ollama: OllamaStatus::default(),
            llm: LlmStatus::default(),
            hardware: HardwareInfo::default(),
            ledger: Ledger::new(),
            last_error: None,
            probe_cache: HashMap::new(),
            progress_events: Vec::new(),
            config: Config::load(),
            latency: PipelineLatency::default(),
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
        use anna_shared::status::LatencyStatus;

        // v0.0.36: Include p50 and p90 percentiles
        let latency = if !self.latency.total.samples.is_empty() {
            Some(LatencyStatus {
                translator_avg_ms: self.latency.translator.avg_ms(),
                translator_p50_ms: self.latency.translator.p50_ms(),
                translator_p90_ms: self.latency.translator.p90_ms(),
                translator_p95_ms: self.latency.translator.p95_ms(),
                probes_avg_ms: self.latency.probes.avg_ms(),
                probes_p50_ms: self.latency.probes.p50_ms(),
                probes_p90_ms: self.latency.probes.p90_ms(),
                probes_p95_ms: self.latency.probes.p95_ms(),
                specialist_avg_ms: self.latency.specialist.avg_ms(),
                specialist_p50_ms: self.latency.specialist.p50_ms(),
                specialist_p90_ms: self.latency.specialist.p90_ms(),
                specialist_p95_ms: self.latency.specialist.p95_ms(),
                total_avg_ms: self.latency.total.avg_ms(),
                total_p50_ms: self.latency.total.p50_ms(),
                total_p90_ms: self.latency.total.p90_ms(),
                total_p95_ms: self.latency.total.p95_ms(),
                sample_count: self.latency.total.samples.len(),
            })
        } else {
            None
        };

        DaemonStatus {
            version: VERSION.to_string(),
            state: self.state.clone(),
            pid: Some(self.pid),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            debug_mode: self.config.debug_mode(),
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
            latency,
            teams: anna_shared::status::TeamRoster::new(),
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

    /// Build comprehensive status snapshot (v0.0.29)
    pub fn to_status_snapshot(&self) -> StatusSnapshot {
        use anna_shared::helpers::load_helpers;
        use anna_shared::specialists::SpecialistRole;
        use anna_shared::teams::Team;

        let captured_at_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Version info
        let versions = VersionInfo::new(VERSION).with_remote(
            self.update.available_version.clone(),
        );

        // Daemon info
        let daemon = DaemonInfo::running(self.pid, self.started_at.elapsed().as_secs())
            .with_error(self.last_error.clone().unwrap_or_default());

        // Permissions info (basic - can be enhanced)
        let perms = PermissionsInfo::current()
            .with_daemon_access(true)
            .with_data_dir_ok(true);

        // Update info
        let update = UpdateInfo {
            interval_s: self.update.check_interval_secs,
            last_check_ts: self.update.last_check.map(|dt| dt.timestamp() as u64),
            next_check_ts: self.update.next_check.map(|dt| dt.timestamp() as u64),
            last_result: if self.update.update_available {
                UpdateResult::UpdateAvailable {
                    version: self.update.available_version.clone().unwrap_or_default(),
                }
            } else if self.update.last_check.is_some() {
                UpdateResult::UpToDate
            } else {
                UpdateResult::NotChecked
            },
        };

        // Helpers info
        let helpers_registry = load_helpers();
        let helpers = HelpersInfo::from_registry(&helpers_registry);

        // Models info
        let models = ModelsInfo {
            ollama_present: self.ollama.installed,
            ollama_running: self.ollama.running,
            ollama_version: self.ollama.version.clone(),
            roles: self.llm.models.iter().map(|m| RoleModelBinding {
                team: Team::General,
                role: SpecialistRole::Junior,
                model_name: m.name.clone(),
                model_present: m.pulled,
            }).collect(),
            downloads: Vec::new(),
        };

        // Config info
        let config = ConfigInfo {
            debug_mode: self.config.debug_mode(),
            repl_clean_mode: !self.config.debug_mode(),
            autonomy_level: 0, // Conservative default
        };

        StatusSnapshot {
            captured_at_ts,
            versions,
            daemon,
            perms,
            update,
            helpers,
            models,
            config,
        }
    }
}

/// Thread-safe shared state handle
pub type SharedState = Arc<RwLock<DaemonStateInner>>;

pub fn create_shared_state() -> SharedState {
    Arc::new(RwLock::new(DaemonStateInner::new()))
}
