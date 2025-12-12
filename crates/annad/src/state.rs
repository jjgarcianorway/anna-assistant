//! Daemon state management.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use anna_shared::ledger::Ledger;
use anna_shared::progress::ProgressEvent;
use anna_shared::recipe_index::RecipeIndex;
use anna_shared::rpc::ProbeResult;
use anna_shared::stats::GlobalStats;
use anna_shared::status::{
    BenchmarkResult, DaemonState, DaemonStatus, HardwareInfo, LlmState, LlmStatus, ModelInfo,
    OllamaStatus, ProgressInfo, UpdateStatus,
};
use anna_shared::status_snapshot::{
    ConfigInfo, DaemonInfo, HelpersInfo, ModelsInfo, PermissionsInfo, RoleModelBinding,
    StatusSnapshot, UpdateInfo, UpdateResult, VersionInfo,
};
use anna_shared::truth_ledger::{TruthLedger, Veracity};
use anna_shared::{DEFAULT_UPDATE_CHECK_INTERVAL, VERSION};
use chrono::{DateTime, Utc};
use std::sync::Mutex;
use tokio::sync::RwLock;
use tracing::info;

use crate::state_types::{CachedProbe, PipelineLatency};

use crate::config::Config;

pub const TRUTH_LEDGER_PATH: &str = "/var/lib/anna/truth_ledger.json";

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
    pub truth_ledger: TruthLedger,
    pub last_error: Option<String>,
    /// Probe result cache (command -> cached result)
    pub probe_cache: HashMap<String, CachedProbe>,
    /// Progress events for current/last request (for polling)
    pub progress_events: Vec<ProgressEvent>,
    /// v0.0.247: Live streaming events (shared with ProgressTracker for real-time access)
    pub streaming_events: Arc<Mutex<Vec<ProgressEvent>>>,
    /// Configuration loaded from file
    pub config: Config,
    /// Per-stage latency statistics
    pub latency: PipelineLatency,
    /// v0.0.79: Global statistics (requests, fast-path hits, etc.)
    pub stats: GlobalStats,
    /// Overall truthfulness score of the system (0.0 - 1.0)
    pub truthfulness_score: f64,
    /// v0.0.101: Recipe index for fast path recipe matching
    pub recipe_index: RecipeIndex,
}

/// Thread-safe shared state handle
pub type SharedState = Arc<RwLock<DaemonStateInner>>;

/// Update state tracking
/// v0.0.72: Field names match UpdateStatus for clarity
pub struct UpdateStateInner {
    pub enabled: bool,
    pub check_interval_secs: u64,
    /// When we last attempted a check (success or failure)
    pub last_check_at: Option<DateTime<Utc>>,
    /// When we'll next check
    pub next_check_at: Option<DateTime<Utc>>,
    /// The latest version from GitHub (preserved on failure)
    pub latest_version: Option<String>,
    /// When we last successfully fetched latest_version
    pub latest_checked_at: Option<DateTime<Utc>>,
    /// Whether latest_version > installed_version
    pub update_available: bool,
    /// State of last update check
    pub check_state: anna_shared::status::UpdateCheckState,
}

impl Default for UpdateStateInner {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: DEFAULT_UPDATE_CHECK_INTERVAL,
            last_check_at: None,
            next_check_at: None,
            latest_version: None,
            latest_checked_at: None,
            update_available: false,
            check_state: anna_shared::status::UpdateCheckState::NeverChecked,
        }
    }
}

impl Default for DaemonStateInner {
    fn default() -> Self {
        Self::new()
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
            ledger: Ledger::new(),            // Initialize empty
            truth_ledger: TruthLedger::new(), // Initialize empty
            last_error: None,
            probe_cache: HashMap::new(),
            progress_events: Vec::new(),
            streaming_events: Arc::new(Mutex::new(Vec::new())),
            config: Config::load(),
            latency: PipelineLatency::default(),
            stats: GlobalStats::new(),
            truthfulness_score: 1.0, // Initialize truthfulness score
            recipe_index: RecipeIndex::build_from_disk(),
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

    /// Calculate the system's overall truthfulness score based on the TruthLedger.
    /// Score is between 0.0 and 1.0. 1.0 means perfect truthfulness.
    pub fn calculate_truthfulness_score(&self) -> f64 {
        let claims = self.truth_ledger.get_all_claims();
        if claims.is_empty() {
            return 1.0; // Perfect score if no claims
        }

        let mut verified_count = 0;
        let mut disputed_count = 0;

        for claim in claims {
            match claim.veracity {
                Veracity::Verified => verified_count += 1,
                Veracity::Disputed => disputed_count += 1,
                _ => {} // Ignore Unknown or Pending for score calculation for now
            }
        }

        let total_assessable_claims = (verified_count + disputed_count) as f64;
        if total_assessable_claims == 0.0 {
            return 1.0; // Still perfect if no assessable claims
        }

        // Simple score: (verified - disputed) / total_assessable_claims
        // This gives a range from -1.0 (all disputed) to 1.0 (all verified)
        let score = (verified_count as f64 - disputed_count as f64) / total_assessable_claims;

        // Normalize to 0.0 - 1.0 range: (score + 1.0) / 2.0
        (score + 1.0) / 2.0
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
                last_check_at: self.update.last_check_at,
                next_check_at: self.update.next_check_at,
                latest_version: self.update.latest_version.clone(),
                latest_checked_at: self.update.latest_checked_at,
                update_available: false,
                check_state: anna_shared::status::UpdateCheckState::NeverChecked,
            },
            llm: self.llm.clone(),
            hardware: self.hardware.clone(),
            ledger: self.ledger.summary(),
            last_error: self.last_error.clone(),
            latency,
            teams: anna_shared::status::TeamRoster::new(),
            truthfulness_score: self.truthfulness_score,
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

    /// v0.0.79: Record request outcome details in stats
    /// v00.248: No longer increments total_requests - that's done at start via record_request_received
    pub fn record_request(
        &mut self,
        fast_path: bool,
        translator_timeout: bool,
        specialist_timeout: bool,
    ) {
        if fast_path {
            self.stats.record_fast_path_hit();
        }
        // Note: total_requests is now incremented at request start, not here
        if translator_timeout {
            self.stats.record_translator_timeout();
        }
        if specialist_timeout {
            self.stats.record_specialist_timeout();
        }
    }

    /// Build comprehensive status snapshot (v0.0.29)
    pub fn to_status_snapshot(&self) -> StatusSnapshot {
        use anna_shared::helpers::{known_helpers, load_helpers, HelperPackage, InstallSource};
        use anna_shared::specialists::SpecialistRole;
        use anna_shared::teams::Team;

        let captured_at_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Version info
        let versions = VersionInfo::new(VERSION).with_remote(self.update.latest_version.clone());

        // Daemon info
        let mut daemon = DaemonInfo::running(self.pid, self.started_at.elapsed().as_secs());
        if let Some(err) = &self.last_error {
            daemon = daemon.with_error(err.clone());
        }

        // Permissions info (basic - can be enhanced)
        let (user, groups) = current_user_and_groups();
        let mut perms = PermissionsInfo::current()
            .with_groups(groups)
            .with_daemon_access(Path::new(anna_shared::SOCKET_PATH).exists())
            .with_data_dir_ok(Path::new("/var/lib/anna").is_dir());
        perms.user = user;

        // Update info
        let update = UpdateInfo {
            interval_s: self.update.check_interval_secs,
            last_check_ts: self.update.last_check_at.map(|dt| dt.timestamp() as u64),
            next_check_ts: self.update.next_check_at.map(|dt| dt.timestamp() as u64),
            last_result: if self.update.update_available {
                UpdateResult::UpdateAvailable {
                    version: self.update.latest_version.clone().unwrap_or_default(),
                }
            } else if self.update.last_check_at.is_some() {
                UpdateResult::UpToDate
            } else {
                UpdateResult::NotChecked
            },
        };

        // Helpers info
        let mut helpers_registry = load_helpers();
        for pkg in known_helpers().packages {
            if helpers_registry.get(&pkg.id).is_none() {
                helpers_registry.register(pkg);
            }
        }
        if let Some(ollama_pkg) = helpers_registry.get_mut("ollama") {
            ollama_pkg.available = self.ollama.installed;
            if ollama_pkg.install_source == InstallSource::Unknown && self.ollama.installed {
                ollama_pkg.install_source = InstallSource::User;
            }
        } else {
            let mut pkg = HelperPackage::new("ollama", "Ollama").required();
            pkg.available = self.ollama.installed;
            pkg.install_source = if self.ollama.installed {
                InstallSource::User
            } else {
                InstallSource::Unknown
            };
            helpers_registry.register(pkg);
        }
        let helpers = HelpersInfo::from_registry(&helpers_registry);

        // Models info
        let models = ModelsInfo {
            ollama_present: self.ollama.installed,
            ollama_running: self.ollama.running,
            ollama_version: self.ollama.version.clone(),
            roles: self
                .llm
                .models
                .iter()
                .map(|m| RoleModelBinding {
                    team: Team::General,
                    role: match m.role.as_str() {
                        "translator" => SpecialistRole::Translator,
                        "supervisor" => SpecialistRole::Senior,
                        _ => SpecialistRole::Junior,
                    },
                    model_name: m.name.clone(),
                    model_present: m.pulled,
                })
                .collect(),
            downloads: Vec::new(),
        };

        // Config info - v0.0.449: Enhanced per VISION.md
        let config = ConfigInfo {
            debug_mode: self.config.debug_mode(),
            repl_clean_mode: !self.config.debug_mode(),
            autonomy_level: 0, // Conservative default
            auto_update: self.config.daemon.auto_update,
            learning_mode: false, // TODO: Add to config when implemented
            fast_path_enabled: self.config.daemon.fast_path_enabled,
            internal_comms: false, // TODO: Add to config when implemented
            request_timeout_secs: self.config.daemon.request_timeout_secs,
            update_check_interval_secs: self.config.daemon.update_interval,
        };

        // Teams info - v0.0.454: Dynamic team availability per VISION.md
        let teams = anna_shared::status_snapshot::TeamsInfo::detect();

        StatusSnapshot {
            captured_at_ts,
            versions,
            daemon,
            perms,
            update,
            helpers,
            models,
            config,
            teams,
            truthfulness_score: self.truthfulness_score,
        }
    }
}

fn current_user_and_groups() -> (String, Vec<String>) {
    let user = std::env::var("SUDO_USER")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown".to_string());

    let groups = Command::new("id")
        .args(["-Gn", &user])
        .output()
        .ok()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    (user, groups)
}

pub async fn load_initial_state() -> SharedState {
    let shared_state = Arc::new(RwLock::new(DaemonStateInner::new()));
    {
        let mut state_write = shared_state.write().await;
        // Load existing ledger if available
        if let Ok(ledger) = Ledger::load() {
            state_write.ledger = ledger;
            info!("Loaded existing ledger");
        }
        // Load existing truth ledger if available
        if let Ok(truth_ledger) = TruthLedger::load(TRUTH_LEDGER_PATH) {
            state_write.truth_ledger = truth_ledger;
            info!("Loaded existing truth ledger");
        }
    }
    shared_state
}
