//! Status types for Anna daemon.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ledger::LedgerSummary;
use crate::teams::Team;

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
    /// Per-stage latency statistics (populated in debug mode)
    #[serde(default)]
    pub latency: Option<LatencyStatus>,
    /// Team roster with active teams (v0.0.25)
    #[serde(default)]
    pub teams: TeamRoster,
}

/// Team roster showing which teams are active
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamRoster {
    /// List of team information
    pub teams: Vec<TeamInfo>,
}

/// Information about a team's status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInfo {
    /// Team type
    pub team: Team,
    /// Whether the team is active
    pub active: bool,
    /// Model used for junior reviews
    pub junior_model: String,
    /// Model used for senior reviews (if escalation enabled)
    pub senior_model: String,
}

impl TeamRoster {
    /// Create a new team roster with all teams in default state
    pub fn new() -> Self {
        Self {
            teams: vec![
                TeamInfo::new(Team::Desktop),
                TeamInfo::new(Team::Storage),
                TeamInfo::new(Team::Network),
                TeamInfo::new(Team::Performance),
                TeamInfo::new(Team::Services),
                TeamInfo::new(Team::Security),
                TeamInfo::new(Team::Hardware),
                TeamInfo::new(Team::General),
            ],
        }
    }

    /// Get info for a specific team
    pub fn get_team(&self, team: Team) -> Option<&TeamInfo> {
        self.teams.iter().find(|t| t.team == team)
    }

    /// Count active teams
    pub fn active_count(&self) -> usize {
        self.teams.iter().filter(|t| t.active).count()
    }
}

impl TeamInfo {
    /// Create default team info
    pub fn new(team: Team) -> Self {
        Self {
            team,
            active: true,
            junior_model: "local-default".to_string(),
            senior_model: "local-default".to_string(),
        }
    }

    /// Create inactive team info
    pub fn inactive(team: Team) -> Self {
        Self {
            team,
            active: false,
            junior_model: String::new(),
            senior_model: String::new(),
        }
    }
}

/// Per-stage latency statistics (v0.0.36: added p50, p90)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatencyStatus {
    pub translator_avg_ms: Option<u64>,
    pub translator_p50_ms: Option<u64>,
    pub translator_p90_ms: Option<u64>,
    pub translator_p95_ms: Option<u64>,
    pub probes_avg_ms: Option<u64>,
    pub probes_p50_ms: Option<u64>,
    pub probes_p90_ms: Option<u64>,
    pub probes_p95_ms: Option<u64>,
    pub specialist_avg_ms: Option<u64>,
    pub specialist_p50_ms: Option<u64>,
    pub specialist_p90_ms: Option<u64>,
    pub specialist_p95_ms: Option<u64>,
    pub total_avg_ms: Option<u64>,
    pub total_p50_ms: Option<u64>,
    pub total_p90_ms: Option<u64>,
    pub total_p95_ms: Option<u64>,
    pub sample_count: usize,
}

/// Update subsystem status
/// v0.0.72: Renamed fields for clarity - never confuse installed vs latest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    /// Whether auto-update is enabled
    pub enabled: bool,
    /// How often we check (seconds)
    pub check_interval_secs: u64,
    /// v0.0.72: When we last attempted a check (success or failure)
    pub last_check_at: Option<DateTime<Utc>>,
    /// When we'll next check
    pub next_check_at: Option<DateTime<Utc>>,
    /// v0.0.72: The latest version from GitHub (preserved on failure)
    pub latest_version: Option<String>,
    /// v0.0.72: When we last successfully fetched latest_version
    pub latest_checked_at: Option<DateTime<Utc>>,
    /// Whether latest_version > installed_version
    pub update_available: bool,
    /// v0.0.72: State of last update check
    pub check_state: UpdateCheckState,
}

/// v0.0.72: State of the update checker
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UpdateCheckState {
    /// Never checked yet
    #[default]
    NeverChecked,
    /// Last check succeeded
    Success,
    /// Last check failed (but we keep last known version)
    Failed,
    /// Currently checking
    Checking,
}

impl std::fmt::Display for UpdateCheckState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateCheckState::NeverChecked => write!(f, "NEVER_CHECKED"),
            UpdateCheckState::Success => write!(f, "OK"),
            UpdateCheckState::Failed => write!(f, "FAILED"),
            UpdateCheckState::Checking => write!(f, "CHECKING"),
        }
    }
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
            last_check_at: None,
            next_check_at: None,
            latest_version: None,
            latest_checked_at: None,
            update_available: false,
            check_state: UpdateCheckState::NeverChecked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_roster_new() {
        let roster = TeamRoster::new();
        assert_eq!(roster.teams.len(), 8);
        assert_eq!(roster.active_count(), 8);
    }

    #[test]
    fn test_team_roster_get_team() {
        let roster = TeamRoster::new();
        let storage = roster.get_team(Team::Storage).unwrap();
        assert_eq!(storage.team, Team::Storage);
        assert!(storage.active);
    }

    #[test]
    fn test_team_info_inactive() {
        let info = TeamInfo::inactive(Team::Network);
        assert!(!info.active);
        assert!(info.junior_model.is_empty());
    }
}
