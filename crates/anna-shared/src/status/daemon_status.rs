//! Main DaemonStatus struct.
//! v0.3.36: Added recovery_status for self-healing metrics

use serde::{Deserialize, Serialize};

use super::config_snapshot::{ConfigSnapshot, ModelMapping};
use super::errors::ErrorSummary;
use super::helpers::{BackupStatus, HelperInfo, LearningStatus};
use super::permissions::PermissionsAudit;
use super::recovery::RecoveryStatus;
use super::rpg::RpgStats;
use super::socket::{BuildMetadata, SocketHealth};
use super::tickets::{TeamRoster, TicketTracker};
use super::types::{default_true, DaemonState, UpdateCheckState};

/// Overall daemon status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub state: DaemonState,
    pub version: String,
    pub ollama_running: bool,
    pub model: Option<String>,
    pub uptime_secs: u64,
    pub gpu: Option<String>,
    pub vram_mb: Option<u64>,
    /// v0.0.924: Number of experiences in memory
    #[serde(default)]
    pub memory_experiences: usize,
    /// v0.0.924: Memory health issues (if any)
    #[serde(default)]
    pub memory_health_issues: Vec<String>,
    /// v0.1.0: Update check timing
    #[serde(default)]
    pub update_check_interval_secs: u64,
    /// v0.1.0: Last update check timestamp (RFC3339)
    #[serde(default)]
    pub last_update_check: Option<String>,
    /// v0.1.0: Next update check timestamp (RFC3339)
    #[serde(default)]
    pub next_update_check: Option<String>,
    /// v0.1.0: Latest available version from GitHub
    #[serde(default)]
    pub latest_version: Option<String>,
    /// v0.1.0: Update check state
    #[serde(default)]
    pub update_state: UpdateCheckState,
    /// v0.3.25: Whether auto-update is enabled
    #[serde(default = "default_true")]
    pub auto_update_enabled: bool,
    /// v0.1.0: Number of active patterns
    #[serde(default)]
    pub pattern_count: usize,
    /// v0.1.0: Number of learned recipes
    #[serde(default)]
    pub recipe_count: usize,
    /// v0.2.7: RPG stats for gamification
    #[serde(default)]
    pub rpg_stats: RpgStats,
    /// v0.3.3: Ticket tracking
    #[serde(default)]
    pub ticket_tracker: TicketTracker,
    /// v0.3.3: Team roster (specialists per department)
    #[serde(default)]
    pub team_roster: TeamRoster,
    /// v0.3.20: Ollama version
    #[serde(default)]
    pub ollama_version: Option<String>,
    /// v0.3.20: User permissions audit
    #[serde(default)]
    pub permissions: PermissionsAudit,
    /// v0.3.20: Escalated tickets count (for stats)
    #[serde(default)]
    pub escalated_tickets_count: u64,
    /// v0.3.20: Questions solved without LLM (instant + memory)
    #[serde(default)]
    pub solved_alone_count: u64,
    /// v0.3.21: Build metadata (git sha, build time, integrity)
    #[serde(default)]
    pub build_info: BuildMetadata,
    /// v0.3.21: Socket health status
    #[serde(default)]
    pub socket_health: SocketHealth,
    /// v0.3.21: Recent errors and warnings
    #[serde(default)]
    pub error_summary: ErrorSummary,
    /// v0.3.21: Full config snapshot
    #[serde(default)]
    pub config_snapshot: ConfigSnapshot,
    /// v0.3.21: Model role mappings
    #[serde(default)]
    pub model_mappings: Vec<ModelMapping>,
    /// v0.3.21: Installed helpers with source
    #[serde(default)]
    pub helpers: Vec<HelperInfo>,
    /// v0.3.24: Backup status
    #[serde(default)]
    pub backup_info: BackupStatus,
    /// v0.3.27: Skill learning status
    #[serde(default)]
    pub learning_status: LearningStatus,
    /// v0.3.36: Self-healing recovery metrics
    #[serde(default)]
    pub recovery_status: RecoveryStatus,
    /// v0.3.211: Current init step (empty when ready)
    #[serde(default)]
    pub init_status: String,
    /// v0.3.211: Last init error (None when no error)
    #[serde(default)]
    pub last_error: Option<String>,
}
