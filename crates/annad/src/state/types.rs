//! Core daemon state type definitions.
//! v0.0.825: Use tokio::sync::Mutex for async-safe streaming events.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anna_shared::ledger::Ledger;
use anna_shared::progress::ProgressEvent;
use anna_shared::recipe_index::RecipeIndex;
use anna_shared::stats::GlobalStats;
use anna_shared::status::{DaemonState, HardwareInfo, LlmStatus, OllamaStatus};
use anna_shared::truth_ledger::TruthLedger;
use anna_shared::{DEFAULT_UPDATE_CHECK_INTERVAL, VERSION};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::state_types::{CachedProbe, PipelineLatency};

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
    /// v0.0.825: Use tokio::sync::Mutex for async-safe access
    pub streaming_events: Arc<tokio::sync::Mutex<Vec<ProgressEvent>>>,
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
            streaming_events: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            config: Config::load(),
            latency: PipelineLatency::default(),
            stats: GlobalStats::new(),
            truthfulness_score: 1.0, // Initialize truthfulness score
            recipe_index: RecipeIndex::build_from_disk(),
        }
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
}
