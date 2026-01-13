//! Daemon status types.
//! v0.0.924: Added memory health fields
//! v0.1.0: Added update timing and extended status fields
//! v0.2.7: Added RPG stats
//! v0.3.3: Added per-specialist stats and ticket tracking
//! v0.3.20: Added permissions audit, ollama version, escalated tickets count
//! v0.3.21: Full status contract - build info, socket health, config, helpers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::AnnaConfig;
use crate::deps;
use crate::version::BuildInfo;

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
}

/// v0.3.24: Backup status for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackupStatus {
    /// Backup directory path
    pub directory: String,
    /// Number of backups stored
    pub backup_count: usize,
    /// Last backup timestamp (RFC3339 or "none")
    pub last_backup: Option<String>,
    /// Total size of all backups in bytes
    pub total_size_bytes: u64,
    /// Retention policy description
    pub retention_policy: String,
}

/// v0.2.7: RPG-style statistics for gamification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RpgStats {
    /// Experience points (0-100, non-linear scaling)
    pub xp: u32,
    /// Current title based on XP
    pub title: String,
    /// Total questions asked
    pub total_questions: u64,
    /// Questions answered without LLM (fast-path/instant)
    pub instant_answers: u64,
    /// Questions answered from memory/recipes
    pub memory_answers: u64,
    /// Questions that needed full LLM processing
    pub llm_answers: u64,
    /// Average response time in milliseconds
    pub avg_response_ms: u64,
    /// Fastest response time in milliseconds
    pub fastest_response_ms: u64,
    /// Slowest response time in milliseconds
    pub slowest_response_ms: u64,
    /// Number of recipes learned
    pub recipes_learned: u32,
    /// Reliability score (0.0-1.0)
    pub reliability: f32,
    /// When Anna was first installed
    pub installed_at: Option<String>,
    /// Total uptime since installation (seconds)
    pub total_uptime_secs: u64,
}

impl RpgStats {
    /// Calculate XP from stats (non-linear, 0-100)
    pub fn calculate_xp(&mut self) {
        // XP formula: weighted combination of activity metrics
        // - Questions answered: logarithmic scaling (each doubling adds ~10 XP)
        // - Memory efficiency: bonus for not needing LLM
        // - Recipes learned: linear bonus
        // - Reliability: multiplier

        let questions_xp = if self.total_questions > 0 {
            (self.total_questions as f64).log2() * 10.0
        } else {
            0.0
        };

        let efficiency = if self.total_questions > 0 {
            (self.instant_answers + self.memory_answers) as f64 / self.total_questions as f64
        } else {
            0.0
        };
        let efficiency_bonus = efficiency * 20.0; // Up to 20 XP for 100% efficiency

        let recipe_bonus = (self.recipes_learned as f64).min(20.0); // Max 20 XP from recipes

        let reliability_mult = 0.5 + (self.reliability as f64 * 0.5); // 0.5 - 1.0 multiplier

        let raw_xp = (questions_xp + efficiency_bonus + recipe_bonus) * reliability_mult;
        self.xp = (raw_xp as u32).min(100);
        self.title = Self::get_title(self.xp);
    }

    /// Get title based on XP level (fun RPG-style progression)
    pub fn get_title(xp: u32) -> String {
        match xp {
            0..=4 => "Novice Apprentice".to_string(),
            5..=9 => "Eager Learner".to_string(),
            10..=19 => "Junior Technician".to_string(),
            20..=29 => "Curious Explorer".to_string(),
            30..=39 => "Competent Assistant".to_string(),
            40..=49 => "Skilled Operator".to_string(),
            50..=59 => "Senior Specialist".to_string(),
            60..=69 => "Expert Analyst".to_string(),
            70..=79 => "Master Troubleshooter".to_string(),
            80..=89 => "IT Sage".to_string(),
            90..=94 => "System Whisperer".to_string(),
            95..=99 => "Arch Wizard".to_string(),
            100 => "Omniscient Oracle".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Get XP bar visualization (ASCII only)
    pub fn xp_bar(&self) -> String {
        let filled = (self.xp as usize) / 5; // 20 character bar
        let empty = 20 - filled;
        format!(
            "[{}{}] {}%",
            "=".repeat(filled),
            "-".repeat(empty),
            self.xp
        )
    }
}

/// v0.3.3: Per-specialist statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistStats {
    /// Specialist name (e.g., "Marcus", "Elena")
    pub name: String,
    /// Department (e.g., "System Administration", "Network Operations")
    pub department: String,
    /// Whether this is a senior specialist
    pub is_senior: bool,
    /// Total tickets handled
    pub tickets_handled: u64,
    /// Successfully resolved tickets
    pub tickets_resolved: u64,
    /// Tickets escalated to senior
    pub tickets_escalated: u64,
    /// Average resolution time in milliseconds
    pub avg_resolution_ms: u64,
    /// Topics this specialist excels at
    pub top_topics: Vec<String>,
    /// Current status (available, busy, offline)
    pub current_status: SpecialistStatus,
}

/// v0.3.3: Specialist availability status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpecialistStatus {
    #[default]
    Available,
    Busy,
    Offline,
}

impl std::fmt::Display for SpecialistStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialistStatus::Available => write!(f, "available"),
            SpecialistStatus::Busy => write!(f, "busy"),
            SpecialistStatus::Offline => write!(f, "offline"),
        }
    }
}

/// v0.3.3: Ticket tracking for numbered tickets
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketTracker {
    /// Next ticket number to assign
    pub next_number: u64,
    /// Tickets created today
    pub today_count: u64,
    /// Current date (DDMMYYYY format)
    pub current_date: String,
    /// Active tickets (not yet resolved)
    pub active_tickets: Vec<ActiveTicket>,
    /// Statistics by department
    pub dept_stats: HashMap<String, DepartmentTicketStats>,
}

impl TicketTracker {
    /// Generate next ticket ID in format CN-XXXX-DDMMYYYY
    pub fn next_ticket_id(&mut self) -> String {
        let today = chrono::Local::now().format("%d%m%Y").to_string();

        // Reset counter if new day
        if self.current_date != today {
            self.current_date = today.clone();
            self.today_count = 0;
        }

        self.today_count += 1;
        self.next_number += 1;

        format!("CN-{:04}-{}", self.today_count, today)
    }
}

/// v0.3.3: Active ticket info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTicket {
    /// Ticket ID (CN-XXXX-DDMMYYYY)
    pub id: String,
    /// Short summary
    pub summary: String,
    /// Assigned specialist
    pub assigned_to: Option<String>,
    /// Department handling this
    pub department: String,
    /// Created timestamp
    pub created_at: String,
    /// Current status
    pub status: TicketStatus,
}

/// v0.3.3: Ticket status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TicketStatus {
    #[default]
    Open,
    InProgress,
    Escalated,
    Resolved,
    Failed,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketStatus::Open => write!(f, "open"),
            TicketStatus::InProgress => write!(f, "in-progress"),
            TicketStatus::Escalated => write!(f, "escalated"),
            TicketStatus::Resolved => write!(f, "resolved"),
            TicketStatus::Failed => write!(f, "failed"),
        }
    }
}

/// v0.3.3: Department-level ticket statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DepartmentTicketStats {
    /// Total tickets received
    pub total_received: u64,
    /// Successfully resolved
    pub resolved: u64,
    /// Average resolution time in milliseconds
    pub avg_resolution_ms: u64,
    /// Escalations to other departments
    pub escalations_out: u64,
    /// Escalations received from other departments
    pub escalations_in: u64,
}

/// v0.3.3: Team roster for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamRoster {
    /// All specialists by department
    pub specialists: HashMap<String, Vec<SpecialistStats>>,
    /// Total team size
    pub total_specialists: usize,
    /// Currently available specialists
    pub available_count: usize,
}

/// v0.3.20: Permissions audit for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionsAudit {
    /// Current user
    pub user: String,
    /// User groups
    pub groups: Vec<String>,
    /// Whether user has sudo access
    pub has_sudo: bool,
    /// Whether user is in wheel group
    pub in_wheel: bool,
    /// Running as root
    pub is_root: bool,
    /// Relevant groups for system administration
    pub admin_groups: Vec<String>,
}

impl PermissionsAudit {
    /// Check current user permissions
    pub fn check() -> Self {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let is_root = user == "root" || std::env::var("EUID").map(|e| e == "0").unwrap_or(false);

        // Get groups
        let groups: Vec<String> = std::process::Command::new("groups")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().split_whitespace().map(|g| g.to_string()).collect())
            .unwrap_or_default();

        let in_wheel = groups.iter().any(|g| g == "wheel");
        let has_sudo = in_wheel || is_root || groups.iter().any(|g| g == "sudo");

        // Filter admin-relevant groups
        let admin_groups: Vec<String> = groups
            .iter()
            .filter(|g| {
                matches!(
                    g.as_str(),
                    "wheel" | "sudo" | "root" | "docker" | "libvirt" | "kvm" | "video" | "audio"
                )
            })
            .cloned()
            .collect();

        Self {
            user,
            groups,
            has_sudo,
            in_wheel,
            is_root,
            admin_groups,
        }
    }
}

/// Daemon state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DaemonState {
    #[default]
    Starting,
    Ready,
    Error,
}

impl std::fmt::Display for DaemonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonState::Starting => write!(f, "STARTING"),
            DaemonState::Ready => write!(f, "READY"),
            DaemonState::Error => write!(f, "ERROR"),
        }
    }
}

/// Update check state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UpdateCheckState {
    #[default]
    NeverChecked,
    Success,
    Failed,
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

// =============================================================================
// v0.3.21: Full Status Contract Types
// =============================================================================

/// v0.3.21: Build metadata from compile time
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildMetadata {
    /// Version string
    pub version: String,
    /// Git commit SHA (short)
    pub git_sha: String,
    /// Whether there were uncommitted changes at build
    pub git_dirty: bool,
    /// Build timestamp (RFC3339)
    pub build_time: String,
    /// Version file integrity check result
    pub integrity_ok: bool,
    /// Integrity error message if any
    pub integrity_error: Option<String>,
}

impl BuildMetadata {
    /// Create from BuildInfo
    pub fn from_build_info() -> Self {
        let info = BuildInfo::get();
        let integrity = crate::version::verify_version_integrity();
        Self {
            version: info.version.to_string(),
            git_sha: info.git_sha.to_string(),
            git_dirty: info.git_dirty,
            build_time: info.build_time.to_string(),
            integrity_ok: integrity.is_ok(),
            integrity_error: integrity.err(),
        }
    }

    /// Format as display string
    pub fn display(&self) -> String {
        let dirty = if self.git_dirty { "*" } else { "" };
        format!("{}+{}{}", self.version, self.git_sha, dirty)
    }
}

/// v0.3.21: Socket health status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocketHealth {
    /// Socket path
    pub path: String,
    /// Whether socket file exists
    pub exists: bool,
    /// Socket status
    pub status: SocketStatus,
    /// Last successful ping timestamp
    pub last_ping: Option<String>,
    /// Last error message if any
    pub last_error: Option<String>,
}

/// v0.3.21: Socket status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SocketStatus {
    #[default]
    Unknown,
    Healthy,
    Unresponsive,
    NotFound,
    PermissionDenied,
}

impl std::fmt::Display for SocketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketStatus::Unknown => write!(f, "UNKNOWN"),
            SocketStatus::Healthy => write!(f, "HEALTHY"),
            SocketStatus::Unresponsive => write!(f, "UNRESPONSIVE"),
            SocketStatus::NotFound => write!(f, "NOT_FOUND"),
            SocketStatus::PermissionDenied => write!(f, "PERMISSION_DENIED"),
        }
    }
}

impl SocketHealth {
    /// Check socket health
    pub fn check(path: &str) -> Self {
        let socket_path = std::path::Path::new(path);
        let exists = socket_path.exists();

        let status = if !exists {
            SocketStatus::NotFound
        } else {
            // Check if we can access the socket
            match std::fs::metadata(socket_path) {
                Ok(meta) => {
                    if meta.permissions().readonly() {
                        SocketStatus::PermissionDenied
                    } else {
                        // Basic existence check passed, connection test needed
                        SocketStatus::Unknown
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    SocketStatus::PermissionDenied
                }
                Err(_) => SocketStatus::Unknown,
            }
        };

        Self {
            path: path.to_string(),
            exists,
            status,
            last_ping: None,
            last_error: None,
        }
    }

    /// Mark as healthy after successful connection
    pub fn mark_healthy(&mut self) {
        self.status = SocketStatus::Healthy;
        self.last_ping = Some(chrono::Utc::now().to_rfc3339());
        self.last_error = None;
    }

    /// Mark as unresponsive with error
    pub fn mark_unresponsive(&mut self, error: &str) {
        self.status = SocketStatus::Unresponsive;
        self.last_error = Some(error.to_string());
    }
}

/// v0.3.21: Recent error/warning summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorSummary {
    /// Total error count since daemon start
    pub error_count: u64,
    /// Total warning count since daemon start
    pub warning_count: u64,
    /// Most recent errors (last 5)
    pub recent_errors: Vec<ErrorEntry>,
    /// Most recent warnings (last 5)
    pub recent_warnings: Vec<ErrorEntry>,
}

/// v0.3.21: Single error/warning entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Source component
    pub source: Option<String>,
    /// Timestamp (RFC3339)
    pub timestamp: String,
    /// Whether error is recoverable
    pub recoverable: bool,
}

impl ErrorSummary {
    /// Add an error
    pub fn add_error(&mut self, code: &str, message: &str, source: Option<&str>, recoverable: bool) {
        self.error_count += 1;
        let entry = ErrorEntry {
            code: code.to_string(),
            message: message.to_string(),
            source: source.map(|s| s.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            recoverable,
        };
        self.recent_errors.push(entry);
        // Keep only last 5
        if self.recent_errors.len() > 5 {
            self.recent_errors.remove(0);
        }
    }

    /// Add a warning
    pub fn add_warning(&mut self, code: &str, message: &str, source: Option<&str>) {
        self.warning_count += 1;
        let entry = ErrorEntry {
            code: code.to_string(),
            message: message.to_string(),
            source: source.map(|s| s.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            recoverable: true,
        };
        self.recent_warnings.push(entry);
        // Keep only last 5
        if self.recent_warnings.len() > 5 {
            self.recent_warnings.remove(0);
        }
    }
}

/// v0.3.21: Full config snapshot for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// Debug mode enabled
    pub debug_mode: bool,
    /// Auto-install helpers enabled
    pub auto_install_helpers: bool,
    /// Ask clarification enabled
    pub ask_clarification: bool,
    /// Use Ralph loop
    pub use_ralph_loop: bool,
    /// Ollama URL
    pub ollama_url: String,
    /// Ollama model
    pub ollama_model: String,
    /// Max iterations
    pub max_iterations: u32,
    /// LLM timeout (seconds)
    pub llm_timeout_secs: u64,
    /// Command timeout (seconds)
    pub command_timeout_secs: u64,
    /// Wiki cache path
    pub wiki_cache_path: String,
    /// Use embeddings
    pub use_embeddings: bool,
    /// High confidence threshold
    pub high_confidence_threshold: f32,
}

impl ConfigSnapshot {
    /// Create from AnnaConfig
    pub fn from_config(config: &AnnaConfig) -> Self {
        Self {
            debug_mode: config.debug_mode,
            auto_install_helpers: config.auto_install_helpers,
            ask_clarification: config.ask_clarification,
            use_ralph_loop: config.use_ralph_loop,
            ollama_url: config.ollama.url.clone(),
            ollama_model: config.ollama.model.clone(),
            max_iterations: config.performance.max_iterations,
            llm_timeout_secs: config.performance.llm_timeout_secs,
            command_timeout_secs: config.performance.command_timeout_secs,
            wiki_cache_path: config.wiki.cache_path.display().to_string(),
            use_embeddings: config.wiki.use_embeddings,
            high_confidence_threshold: config.performance.high_confidence_threshold,
        }
    }

    /// Create from current config
    pub fn current() -> Self {
        match AnnaConfig::load() {
            Ok(config) => Self::from_config(&config),
            Err(_) => Self::default(),
        }
    }
}

/// v0.3.21: Model role mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMapping {
    /// Role name (e.g., "intent", "command", "validation", "answer")
    pub role: String,
    /// Model used for this role
    pub model: String,
    /// Whether this is the default model
    pub is_default: bool,
}

impl ModelMapping {
    /// Get default mappings (all roles use the configured model)
    pub fn defaults(model: &str) -> Vec<Self> {
        vec![
            Self { role: "intent".to_string(), model: model.to_string(), is_default: true },
            Self { role: "command".to_string(), model: model.to_string(), is_default: true },
            Self { role: "validation".to_string(), model: model.to_string(), is_default: true },
            Self { role: "answer".to_string(), model: model.to_string(), is_default: true },
            Self { role: "clarification".to_string(), model: model.to_string(), is_default: true },
        ]
    }
}

/// v0.3.21: Helper tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperInfo {
    /// Tool/command name
    pub name: String,
    /// Description
    pub description: String,
    /// Whether the tool is installed
    pub installed: bool,
    /// Installation source
    pub source: HelperSource,
}

/// v0.3.21: Where a helper was installed from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HelperSource {
    #[default]
    Unknown,
    /// Installed by user before Anna
    User,
    /// Installed by Anna
    Anna,
    /// System package (pre-installed)
    System,
}

impl std::fmt::Display for HelperSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HelperSource::Unknown => write!(f, "unknown"),
            HelperSource::User => write!(f, "user"),
            HelperSource::Anna => write!(f, "anna"),
            HelperSource::System => write!(f, "system"),
        }
    }
}

impl HelperInfo {
    /// Check all diagnostic tools and their sources
    pub fn check_all() -> Vec<Self> {
        let anna_installed = deps::read_installed_packages().unwrap_or_default();

        deps::DIAGNOSTIC_TOOLS
            .iter()
            .map(|(name, desc)| {
                let installed = deps::command_exists(name);
                let source = if anna_installed.contains(&name.to_string()) {
                    HelperSource::Anna
                } else if installed {
                    // Was installed before Anna tracked it
                    HelperSource::User
                } else {
                    HelperSource::Unknown
                };

                Self {
                    name: name.to_string(),
                    description: desc.to_string(),
                    installed,
                    source,
                }
            })
            .collect()
    }
}

/// v0.3.25: Default true for serde
fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_state_display() {
        assert_eq!(DaemonState::Ready.to_string(), "READY");
        assert_eq!(DaemonState::Starting.to_string(), "STARTING");
        assert_eq!(DaemonState::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_rpg_stats_xp_bar() {
        let mut stats = RpgStats::default();
        stats.xp = 0;
        assert!(stats.xp_bar().contains("0%"));

        stats.xp = 50;
        assert!(stats.xp_bar().contains("50%"));

        stats.xp = 100;
        assert!(stats.xp_bar().contains("100%"));
    }

    #[test]
    fn test_rpg_stats_title_progression() {
        // Verify titles progress correctly
        assert_eq!(RpgStats::get_title(0), "Novice Apprentice");
        assert_eq!(RpgStats::get_title(10), "Junior Technician");
        assert_eq!(RpgStats::get_title(50), "Senior Specialist");
        assert_eq!(RpgStats::get_title(100), "Omniscient Oracle");
    }

    #[test]
    fn test_rpg_stats_calculate_xp() {
        let mut stats = RpgStats::default();
        stats.total_questions = 100;
        stats.instant_answers = 50;
        stats.memory_answers = 25;
        stats.llm_answers = 25;
        stats.recipes_learned = 10;
        stats.reliability = 0.9;

        stats.calculate_xp();

        assert!(stats.xp > 0);
        assert!(stats.xp <= 100);
        assert!(!stats.title.is_empty());
    }

    #[test]
    fn test_socket_status_display() {
        assert_eq!(SocketStatus::Healthy.to_string(), "HEALTHY");
        assert_eq!(SocketStatus::NotFound.to_string(), "NOT_FOUND");
        assert_eq!(SocketStatus::PermissionDenied.to_string(), "PERMISSION_DENIED");
    }

    #[test]
    fn test_ticket_status_display() {
        assert_eq!(TicketStatus::Open.to_string(), "open");
        assert_eq!(TicketStatus::InProgress.to_string(), "in-progress");
        assert_eq!(TicketStatus::Resolved.to_string(), "resolved");
    }

    #[test]
    fn test_build_metadata_display() {
        let mut meta = BuildMetadata::default();
        meta.version = "0.3.22".to_string();
        meta.git_sha = "abc1234".to_string();
        meta.git_dirty = false;

        let display = meta.display();
        assert!(display.contains("0.3.22"));
        assert!(display.contains("abc1234"));

        meta.git_dirty = true;
        let display = meta.display();
        assert!(display.contains("*")); // dirty marker
    }

    #[test]
    fn test_error_summary_add_error() {
        let mut summary = ErrorSummary::default();

        summary.add_error("E001", "Test error", Some("test"), true);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.recent_errors.len(), 1);

        // Add 6 errors, should only keep 5
        for i in 2..=6 {
            summary.add_error(&format!("E{:03}", i), &format!("Error {}", i), None, false);
        }
        assert_eq!(summary.error_count, 6);
        assert_eq!(summary.recent_errors.len(), 5);
    }

    #[test]
    fn test_permissions_audit_check() {
        // This test just verifies the function runs without panic
        let perms = PermissionsAudit::check();
        assert!(!perms.user.is_empty());
    }
}
