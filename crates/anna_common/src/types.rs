//! Core data types for Anna Assistant

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Risk level for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low = 0,
    Medium = 1,
    High = 2,
}

/// Priority level for recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Critical security/driver issues
    Mandatory = 3,
    /// Significant improvements, quality of life
    Recommended = 2,
    /// Nice-to-have optimizations
    Optional = 1,
    /// Beautification, minor enhancements
    Cosmetic = 0,
}

/// Autonomy tier configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomyTier {
    /// Tier 0: Advise only, never execute
    AdviseOnly = 0,
    /// Tier 1: Auto-execute Low risk only
    SafeAutoApply = 1,
    /// Tier 2: Auto-execute Low + Medium risk
    SemiAutonomous = 2,
    /// Tier 3: Auto-execute all risk levels
    FullyAutonomous = 3,
}

impl AutonomyTier {
    /// Check if this tier allows auto-execution for a given risk level
    pub fn allows(&self, risk: RiskLevel) -> bool {
        match (self, risk) {
            (AutonomyTier::AdviseOnly, _) => false,
            (AutonomyTier::SafeAutoApply, RiskLevel::Low) => true,
            (AutonomyTier::SafeAutoApply, _) => false,
            (AutonomyTier::SemiAutonomous, RiskLevel::High) => false,
            (AutonomyTier::SemiAutonomous, _) => true,
            (AutonomyTier::FullyAutonomous, _) => true,
        }
    }
}

/// Category metadata with Arch Wiki alignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    pub id: String,
    pub display_name: String,
    pub wiki_url: String,
    pub description: String,
}

impl CategoryInfo {
    pub fn get_all() -> Vec<Self> {
        vec![
            Self {
                id: "security".to_string(),
                display_name: "Security & Privacy".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/Security".to_string(),
                description: "Protect your system from threats".to_string(),
            },
            Self {
                id: "performance".to_string(),
                display_name: "Performance & Optimization".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/Improving_performance".to_string(),
                description: "Make your system faster".to_string(),
            },
            Self {
                id: "hardware".to_string(),
                display_name: "Hardware Support".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/Hardware".to_string(),
                description: "Drivers and hardware configuration".to_string(),
            },
            Self {
                id: "networking".to_string(),
                display_name: "Network Configuration".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/Network_configuration".to_string(),
                description: "WiFi, Ethernet, VPN setup".to_string(),
            },
            Self {
                id: "desktop".to_string(),
                display_name: "Desktop Environment".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/Desktop_environment".to_string(),
                description: "GUI and window managers".to_string(),
            },
            Self {
                id: "development".to_string(),
                display_name: "Development Tools".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/List_of_applications/Development".to_string(),
                description: "Programming and build tools".to_string(),
            },
            Self {
                id: "gaming".to_string(),
                display_name: "Gaming & Entertainment".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/Gaming".to_string(),
                description: "Steam, emulators, and games".to_string(),
            },
            Self {
                id: "multimedia".to_string(),
                display_name: "Multimedia & Graphics".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/List_of_applications/Multimedia".to_string(),
                description: "Video, audio, and image tools".to_string(),
            },
            Self {
                id: "maintenance".to_string(),
                display_name: "System Maintenance".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/System_maintenance".to_string(),
                description: "Keep your system healthy".to_string(),
            },
            Self {
                id: "beautification".to_string(),
                display_name: "Terminal & CLI Tools".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/Command-line_shell".to_string(),
                description: "Modern command-line experience".to_string(),
            },
            Self {
                id: "power".to_string(),
                display_name: "Power Management".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/Power_management".to_string(),
                description: "Battery and energy saving".to_string(),
            },
            Self {
                id: "system".to_string(),
                display_name: "System Configuration".to_string(),
                wiki_url: "https://wiki.archlinux.org/title/System_configuration".to_string(),
                description: "Core system settings".to_string(),
            },
        ]
    }

    pub fn get_by_id(id: &str) -> Option<Self> {
        Self::get_all().into_iter().find(|c| c.id == id)
    }
}

/// System facts collected by telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemFacts {
    pub timestamp: DateTime<Utc>,

    // Hardware
    pub hostname: String,
    pub kernel: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub total_memory_gb: f64,
    pub gpu_vendor: Option<String>,
    pub storage_devices: Vec<StorageDevice>,

    // Software & Packages
    pub installed_packages: usize,
    pub orphan_packages: Vec<String>,
    pub package_groups: Vec<String>, // detected groups: base-devel, gnome, kde, etc.

    // Network
    pub network_interfaces: Vec<String>,
    pub has_wifi: bool,
    pub has_ethernet: bool,

    // User Environment
    pub shell: String, // bash, zsh, fish
    pub desktop_environment: Option<String>, // GNOME, KDE, i3, etc.
    pub display_server: Option<String>, // X11, Wayland

    // User Behavior (learned from system)
    pub frequently_used_commands: Vec<CommandUsage>,
    pub dev_tools_detected: Vec<String>, // git, docker, vim, etc.
    pub media_usage: MediaUsageProfile,
    pub common_file_types: Vec<String>, // .py, .rs, .js, .mp4, etc.

    // Boot Performance
    pub boot_time_seconds: Option<f64>,
    pub slow_services: Vec<SystemdService>, // services taking >5s to start
    pub failed_services: Vec<String>,

    // Package Management
    pub aur_packages: usize,
    pub aur_helper: Option<String>, // yay, paru, aurutils, etc.
    pub package_cache_size_gb: f64,
    pub last_system_upgrade: Option<DateTime<Utc>>,

    // Kernel & Boot Parameters
    pub kernel_parameters: Vec<String>,

    // Advanced Telemetry for Better Understanding
    pub recently_installed_packages: Vec<PackageInstallation>, // last 30 days
    pub active_services: Vec<String>, // currently running systemd services
    pub enabled_services: Vec<String>, // services enabled on boot
    pub disk_usage_trend: DiskUsageTrend,
    pub session_info: SessionInfo,
    pub development_environment: DevelopmentProfile,
    pub gaming_profile: GamingProfile,
    pub network_profile: NetworkProfile,
    pub system_age_days: u64, // days since installation
    pub user_preferences: UserPreferences, // detected preferences
}

/// Package installation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInstallation {
    pub name: String,
    pub installed_at: DateTime<Utc>,
    pub from_aur: bool,
}

/// Disk usage trend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageTrend {
    pub total_gb: f64,
    pub used_gb: f64,
    pub largest_directories: Vec<DirectorySize>, // top 10 space consumers
    pub cache_size_gb: f64, // total cache size
    pub log_size_gb: f64, // /var/log size
}

/// Directory size information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorySize {
    pub path: String,
    pub size_gb: f64,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub current_user: String,
    pub login_count_last_30_days: usize,
    pub average_session_hours: f64,
    pub last_login: Option<DateTime<Utc>>,
    pub multiple_users: bool, // more than one user account
}

/// Development environment profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentProfile {
    pub languages: Vec<LanguageUsage>, // detected languages with project counts
    pub ides_installed: Vec<String>, // vscode, vim, emacs, intellij, etc.
    pub active_projects: Vec<ProjectInfo>, // detected project directories
    pub uses_containers: bool, // Docker/Podman usage
    pub uses_virtualization: bool, // QEMU/VirtualBox/VMware
    pub git_repos_count: usize,
}

/// Programming language usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageUsage {
    pub language: String,
    pub project_count: usize,
    pub file_count: usize,
    pub has_lsp: bool, // language server installed
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub path: String,
    pub language: String,
    pub last_modified: DateTime<Utc>,
    pub has_git: bool,
}

/// Gaming profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingProfile {
    pub steam_installed: bool,
    pub lutris_installed: bool,
    pub wine_installed: bool,
    pub proton_ge_installed: bool,
    pub mangohud_installed: bool,
    pub game_count: usize, // detected games
    pub uses_gamepad: bool, // gamepad detected or drivers installed
}

/// Network profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProfile {
    pub vpn_configured: bool, // WireGuard, OpenVPN, etc.
    pub firewall_active: bool,
    pub ssh_server_running: bool,
    pub has_static_ip: bool,
    pub dns_configuration: String, // systemd-resolved, dnsmasq, etc.
    pub uses_network_share: bool, // NFS, Samba mounts
}

/// User preferences detected from system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub prefers_cli_over_gui: bool, // based on command usage
    pub is_power_user: bool, // based on tool complexity
    pub values_aesthetics: bool, // has beautification tools
    pub is_gamer: bool,
    pub is_developer: bool,
    pub is_content_creator: bool, // multimedia tools
    pub uses_laptop: bool, // based on hardware
    pub prefers_minimalism: bool, // based on package count and choices
}

/// Systemd service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdService {
    pub name: String,
    pub time_seconds: f64,
}

/// Command usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandUsage {
    pub command: String,
    pub count: usize,
}

/// Media usage profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUsageProfile {
    pub has_video_files: bool,
    pub has_audio_files: bool,
    pub has_images: bool,
    pub video_player_installed: bool,
    pub audio_player_installed: bool,
    pub image_viewer_installed: bool,
}

/// Storage device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub name: String,
    pub filesystem: String,
    pub size_gb: f64,
    pub used_gb: f64,
    pub mount_point: String,
}

/// Alternative software option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub name: String,
    pub description: String,
    pub install_command: String,
}

/// A single piece of advice from the recommendation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advice {
    pub id: String,
    pub title: String,
    pub reason: String,
    pub action: String,
    pub command: Option<String>,
    pub risk: RiskLevel,
    pub priority: Priority,
    pub wiki_refs: Vec<String>,
    pub category: String, // "security", "drivers", "development", "media", "beautification", etc.
    #[serde(default)]
    pub alternatives: Vec<Alternative>,
    #[serde(default)]
    pub depends_on: Vec<String>, // IDs of advice that should be applied first
    #[serde(default)]
    pub related_to: Vec<String>, // IDs of related advice (suggestions, not dependencies)
    #[serde(default)]
    pub bundle: Option<String>, // Workflow bundle name (e.g., "Python Dev Stack")
}

impl Advice {
    /// Create a builder for constructing Advice with optional alternatives
    pub fn new(
        id: String,
        title: String,
        reason: String,
        action: String,
        command: Option<String>,
        risk: RiskLevel,
        priority: Priority,
        wiki_refs: Vec<String>,
        category: String,
    ) -> Self {
        Self {
            id,
            title,
            reason,
            action,
            command,
            risk,
            priority,
            wiki_refs,
            category,
            alternatives: Vec::new(),
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
        }
    }

    /// Add alternatives to this advice
    pub fn with_alternatives(mut self, alternatives: Vec<Alternative>) -> Self {
        self.alternatives = alternatives;
        self
    }

    /// Add dependencies (advice IDs that should be applied first)
    pub fn with_dependencies(mut self, depends_on: Vec<String>) -> Self {
        self.depends_on = depends_on;
        self
    }

    /// Add related advice IDs
    pub fn with_related(mut self, related_to: Vec<String>) -> Self {
        self.related_to = related_to;
        self
    }

    /// Set workflow bundle
    pub fn with_bundle(mut self, bundle: String) -> Self {
        self.bundle = Some(bundle);
        self
    }
}

/// An action to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub advice_id: String,
    pub command: String,
    pub executed_at: DateTime<Utc>,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Rollback token for reversing an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackToken {
    pub action_id: String,
    pub advice_id: String,
    pub executed_at: DateTime<Utc>,
    pub command: String,
    pub rollback_command: Option<String>,
    pub snapshot_before: Option<String>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action_type: String,
    pub details: String,
    pub success: bool,
}

/// Bundle installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BundleStatus {
    Completed,
    Partial,
    Failed,
}

/// Bundle installation history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleHistoryEntry {
    pub bundle_name: String,
    pub installed_items: Vec<String>, // advice IDs
    pub installed_at: DateTime<Utc>,
    pub installed_by: String, // username
    pub status: BundleStatus,
    pub rollback_available: bool,
}

/// Bundle history storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundleHistory {
    pub entries: Vec<BundleHistoryEntry>,
}

/// Arch Wiki cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiCacheEntry {
    pub page_title: String,
    pub url: String,
    pub content: String, // Simplified markdown content
    pub cached_at: DateTime<Utc>,
    pub checksum: String, // To detect updates
}

/// Wiki cache storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikiCache {
    pub entries: Vec<WikiCacheEntry>,
    pub last_updated: Option<DateTime<Utc>>,
}

impl WikiCache {
    /// Path to wiki cache directory
    pub fn cache_dir() -> std::path::PathBuf {
        std::path::PathBuf::from("/var/lib/anna/wiki_cache")
    }

    /// Path to wiki cache index
    pub fn index_path() -> std::path::PathBuf {
        Self::cache_dir().join("index.json")
    }

    /// Load wiki cache from disk
    pub fn load() -> Result<Self, std::io::Error> {
        let path = Self::index_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })
    }

    /// Save wiki cache to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::index_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }

    /// Get cached page by URL
    pub fn get_by_url(&self, url: &str) -> Option<&WikiCacheEntry> {
        self.entries.iter().find(|e| e.url == url)
    }

    /// Get cached page by title
    pub fn get_by_title(&self, title: &str) -> Option<&WikiCacheEntry> {
        self.entries.iter().find(|e| e.page_title == title)
    }

    /// Add or update cache entry
    pub fn upsert(&mut self, entry: WikiCacheEntry) {
        // Remove existing entry with same URL
        self.entries.retain(|e| e.url != entry.url);
        self.entries.push(entry);
        self.last_updated = Some(chrono::Utc::now());
    }

    /// Check if cache needs refresh (older than 7 days)
    pub fn needs_refresh(&self) -> bool {
        if let Some(last_updated) = self.last_updated {
            let age = chrono::Utc::now() - last_updated;
            return age.num_days() > 7;
        }
        true
    }
}

/// Autonomy action record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyAction {
    pub action_type: String, // "clean_orphans", "clean_cache", "rotate_logs"
    pub executed_at: DateTime<Utc>,
    pub description: String,
    pub command_run: String,
    pub success: bool,
    pub output: String,
    pub can_undo: bool,
    pub undo_command: Option<String>,
}

/// Autonomy log storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutonomyLog {
    pub actions: Vec<AutonomyAction>,
}

impl AutonomyLog {
    /// Path to autonomy log
    pub fn log_path() -> std::path::PathBuf {
        std::path::PathBuf::from("/var/lib/anna/autonomy_log.json")
    }

    /// Load autonomy log
    pub fn load() -> Result<Self, std::io::Error> {
        let path = Self::log_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })
    }

    /// Save autonomy log
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::log_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }

    /// Record an action
    pub fn record(&mut self, action: AutonomyAction) {
        self.actions.push(action);
        // Keep last 1000 actions only
        if self.actions.len() > 1000 {
            self.actions.drain(0..self.actions.len() - 1000);
        }
    }

    /// Get recent actions (last N)
    pub fn recent(&self, count: usize) -> Vec<&AutonomyAction> {
        self.actions.iter().rev().take(count).collect()
    }
}

impl BundleHistory {
    /// Path to bundle history file
    pub fn history_path() -> std::path::PathBuf {
        std::path::PathBuf::from("/var/lib/anna/bundle_history.json")
    }

    /// Load bundle history from disk
    pub fn load() -> Result<Self, std::io::Error> {
        let path = Self::history_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })
    }

    /// Save bundle history to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::history_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }

    /// Add a new bundle installation entry
    pub fn add_entry(&mut self, entry: BundleHistoryEntry) {
        self.entries.push(entry);
    }

    /// Get the most recent installation of a bundle
    pub fn get_latest(&self, bundle_name: &str) -> Option<&BundleHistoryEntry> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.bundle_name == bundle_name && e.status == BundleStatus::Completed)
    }
}

/// RPC Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub params: serde_json::Value,
}

/// RPC Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}
