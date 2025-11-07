//! Core data types for Anna Assistant

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Risk level for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum RiskLevel {
    #[default]
    Low = 0,
    Medium = 1,
    High = 2,
}

/// System requirements for advice applicability (RC.6)
/// Ensures advice is only shown when it makes sense for the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Requirement {
    /// Requires any display server (X11 or Wayland)
    DisplayServer,
    /// Requires Wayland specifically
    Wayland,
    /// Requires X11 specifically
    X11,
    /// Requires any audio system (PulseAudio, PipeWire, ALSA)
    AudioSystem,
    /// Requires PulseAudio specifically
    PulseAudio,
    /// Requires PipeWire specifically
    PipeWire,
    /// Requires specific GPU vendor
    GpuVendor(String), // "nvidia", "amd", "intel"
    /// Requires any GPU (not just integrated)
    DedicatedGpu,
    /// Requires Bluetooth hardware
    Bluetooth,
    /// Requires WiFi hardware
    WiFi,
    /// Requires Ethernet connection
    Ethernet,
    /// Requires specific package to be installed
    Package(String),
    /// Requires desktop environment (any)
    DesktopEnvironment,
    /// Requires window manager (any)
    WindowManager,
    /// Requires laptop hardware (battery present)
    Laptop,
    /// Requires gaming-related setup
    Gaming,
    /// Requires development environment
    Development,
}

impl Requirement {
    /// Check if this requirement is met by the given system facts
    pub fn is_met(&self, facts: &SystemFacts) -> bool {
        match self {
            Requirement::DisplayServer => {
                facts.display_server.is_some()
            }
            Requirement::Wayland => {
                facts.display_server.as_ref().map(|s| s.to_lowercase().contains("wayland")).unwrap_or(false)
            }
            Requirement::X11 => {
                facts.display_server.as_ref().map(|s| s.to_lowercase().contains("x11")).unwrap_or(false)
            }
            Requirement::AudioSystem => {
                facts.audio_system.is_some() && facts.audio_server_running
            }
            Requirement::PulseAudio => {
                facts.audio_system.as_ref().map(|s| s.to_lowercase().contains("pulse")).unwrap_or(false)
                    && facts.audio_server_running
            }
            Requirement::PipeWire => {
                facts.audio_system.as_ref().map(|s| s.to_lowercase().contains("pipewire")).unwrap_or(false)
                    && facts.audio_server_running
            }
            Requirement::GpuVendor(vendor) => {
                match vendor.to_lowercase().as_str() {
                    "nvidia" => facts.is_nvidia,
                    "amd" => facts.is_amd_gpu,
                    "intel" => facts.is_intel_gpu,
                    _ => false,
                }
            }
            Requirement::DedicatedGpu => {
                facts.gpu_vendor.is_some()
            }
            Requirement::Bluetooth => {
                facts.bluetooth_status.available
            }
            Requirement::WiFi => facts.has_wifi,
            Requirement::Ethernet => facts.has_ethernet,
            Requirement::Package(pkg) => {
                // Check if package exists in system
                // For now, we'll assume it's met - can be enhanced later
                std::process::Command::new("pacman")
                    .arg("-Q")
                    .arg(pkg)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }
            Requirement::DesktopEnvironment => facts.desktop_environment.is_some(),
            Requirement::WindowManager => facts.window_manager.is_some(),
            Requirement::Laptop => facts.battery_info.is_some(),
            Requirement::Gaming => {
                // Has gaming profile or gaming-related packages
                facts.gaming_profile.steam_installed || facts.gaming_profile.game_count > 0
            }
            Requirement::Development => {
                // Has dev tools or dev profile
                !facts.dev_tools_detected.is_empty() || !facts.development_environment.languages.is_empty()
            }
        }
    }
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
    pub desktop_environment: Option<String>, // GNOME, KDE, XFCE, etc.
    pub window_manager: Option<String>, // Hyprland, i3, sway, bspwm, etc.
    pub compositor: Option<String>, // Hyprland, picom, etc.
    pub display_server: Option<String>, // X11, Wayland

    // GPU Detection (beta.41+)
    pub is_nvidia: bool, // Whether system has Nvidia GPU
    pub nvidia_driver_version: Option<String>, // Nvidia driver version if present
    pub has_wayland_nvidia_support: bool, // Whether Nvidia+Wayland is properly configured
    pub is_intel_gpu: bool, // Whether system has Intel integrated graphics
    pub is_amd_gpu: bool, // Whether system has AMD/ATI GPU
    pub amd_driver_version: Option<String>, // AMD driver version (amdgpu or radeon)

    // Enhanced GPU Telemetry (beta.43+)
    #[serde(default)]
    pub gpu_model: Option<String>, // Specific GPU model name (e.g., "NVIDIA GeForce RTX 4090")
    #[serde(default)]
    pub gpu_vram_mb: Option<u32>, // VRAM size in MB
    #[serde(default)]
    pub vulkan_support: bool, // Whether Vulkan API is available
    #[serde(default)]
    pub nvidia_cuda_support: bool, // Whether NVIDIA CUDA is available

    // Performance Score (RC.9.7+) - "Potato computer, potato tools"
    #[serde(default)]
    pub performance_score: u32, // 0-100: system capability score for resource-aware recommendations
    #[serde(default)]
    pub resource_tier: String, // "Efficient", "Balanced", or "Performance"

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

    // Enhanced Telemetry (beta.35+)
    pub hardware_monitoring: HardwareMonitoring,
    pub disk_health: Vec<DiskHealthInfo>,
    pub system_health_metrics: SystemHealthMetrics,
    pub performance_metrics: PerformanceMetrics,
    pub predictive_insights: PredictiveInsights,

    // Extended Telemetry (beta.43+)
    #[serde(default)]
    pub microcode_status: MicrocodeStatus, // CPU microcode update status
    pub battery_info: Option<BatteryInfo>, // Battery status for laptops
    #[serde(default)]
    pub backup_systems: Vec<String>, // timeshift, rsync, borg, restic, etc.
    #[serde(default)]
    pub bluetooth_status: BluetoothStatus,
    #[serde(default)]
    pub ssd_info: Vec<SSDInfo>, // SSD-specific info and TRIM status
    #[serde(default)]
    pub swap_config: SwapConfiguration,
    #[serde(default)]
    pub locale_info: LocaleInfo, // timezone, language settings
    #[serde(default)]
    pub pacman_hooks: Vec<String>, // installed pacman hooks

    // Audio System (beta.43+)
    #[serde(default)]
    pub audio_system: Option<String>, // PulseAudio, PipeWire, ALSA
    #[serde(default)]
    pub audio_server_running: bool, // Audio server is active
    #[serde(default)]
    pub pipewire_session_manager: Option<String>, // WirePlumber, pipewire-media-session
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
    pub has_ssh_client_keys: bool, // User has SSH keys in ~/.ssh/
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

/// Hardware monitoring data (beta.35+)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwareMonitoring {
    pub cpu_temperature_celsius: Option<f64>,
    pub cpu_load_1min: Option<f64>,
    pub cpu_load_5min: Option<f64>,
    pub cpu_load_15min: Option<f64>,
    pub memory_used_gb: f64,
    pub memory_available_gb: f64,
    pub swap_used_gb: f64,
    pub swap_total_gb: f64,
    pub battery_health: Option<BatteryHealth>,
}

/// Battery health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryHealth {
    pub percentage: u8,
    pub status: String, // Charging, Discharging, Full
    pub health_percentage: Option<u8>, // 0-100, capacity vs design capacity
    pub cycles: Option<u32>,
    pub is_critical: bool, // < 20%
}

/// Disk health information from SMART data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskHealthInfo {
    pub device: String, // /dev/sda, /dev/nvme0n1
    pub health_status: String, // PASSED, FAILING, UNKNOWN
    pub temperature_celsius: Option<u8>,
    pub power_on_hours: Option<u64>,
    pub wear_leveling: Option<u8>, // 0-100 for SSDs
    pub reallocated_sectors: Option<u64>,
    pub pending_sectors: Option<u64>,
    pub has_errors: bool,
}

/// System health metrics from journal and systemd
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemHealthMetrics {
    pub journal_errors_last_24h: usize,
    pub journal_warnings_last_24h: usize,
    pub critical_events: Vec<CriticalEvent>,
    pub degraded_services: Vec<String>, // services in degraded state
    pub recent_crashes: Vec<ServiceCrash>,
    pub oom_events_last_week: usize, // Out of memory kills
    pub kernel_errors: Vec<String>, // Recent kernel errors
}

/// Critical system event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalEvent {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub unit: Option<String>, // systemd unit involved
    pub severity: String, // error, critical, emergency
}

/// Service crash information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCrash {
    pub service_name: String,
    pub timestamp: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub signal: Option<String>,
}

/// Performance metrics trends
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    pub cpu_usage_avg_percent: f64,
    pub memory_usage_avg_percent: f64,
    pub disk_io_read_mb_s: f64,
    pub disk_io_write_mb_s: f64,
    pub network_rx_mb_s: f64,
    pub network_tx_mb_s: f64,
    pub high_cpu_processes: Vec<ProcessInfo>,
    pub high_memory_processes: Vec<ProcessInfo>,
    // Network quality metrics
    #[serde(default)]
    pub average_latency_ms: Option<f64>, // Ping to 8.8.8.8 or gateway
    #[serde(default)]
    pub packet_loss_percent: Option<f64>, // Packet loss percentage
}

/// Process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub cpu_percent: f64,
    pub memory_mb: f64,
}

/// Predictive insights based on trends
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PredictiveInsights {
    pub disk_full_prediction: Option<DiskPrediction>,
    pub temperature_trend: TemperatureTrend,
    pub service_reliability: Vec<ServiceReliability>,
    pub boot_time_trend: BootTimeTrend,
    pub memory_pressure_risk: RiskLevel,
}

/// Disk space prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPrediction {
    pub mount_point: String,
    pub days_until_full: Option<u32>, // None if not trending toward full
    pub current_growth_gb_per_day: f64,
}

/// Temperature trend analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemperatureTrend {
    pub cpu_trend: TrendDirection,
    pub is_concerning: bool, // true if trending up significantly
    pub average_temp_celsius: Option<f64>,
    pub max_temp_celsius: Option<f64>,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TrendDirection {
    #[default]
    Stable,
    Rising,
    Falling,
    Unknown,
}

/// Service reliability tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceReliability {
    pub service_name: String,
    pub reliability_score: f64, // 0.0-1.0
    pub failures_last_30_days: usize,
    pub is_unreliable: bool, // true if score < 0.8
}

/// Boot time trend
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BootTimeTrend {
    pub current_seconds: Option<f64>,
    pub trend: TrendDirection,
    pub is_degrading: bool, // true if boot time increasing
}

/// Microcode update status (beta.43+)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MicrocodeStatus {
    pub microcode_installed: bool, // intel-ucode or amd-ucode package
    pub vendor: String, // "Intel", "AMD", or "Unknown"
    pub current_version: Option<String>,
    pub needs_update: bool,
}

/// Battery information for laptops (beta.43+)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub present: bool,
    pub capacity_percent: Option<f64>, // Current charge percentage
    pub health_percent: Option<f64>, // Battery health (capacity vs design capacity)
    pub status: String, // "Charging", "Discharging", "Full", "Unknown"
    pub time_to_empty: Option<f64>, // Minutes remaining (if discharging)
    pub time_to_full: Option<f64>, // Minutes to full charge (if charging)
    pub cycle_count: Option<u32>, // Number of charge cycles
}

/// Bluetooth status (beta.43+)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BluetoothStatus {
    pub available: bool, // Bluetooth hardware present
    pub enabled: bool, // Bluetooth service running
    pub connected_devices: Vec<String>, // List of connected device names
}

/// SSD-specific information (beta.43+)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSDInfo {
    pub device: String, // e.g., /dev/sda, /dev/nvme0n1
    pub model: String,
    pub trim_enabled: bool, // TRIM/discard enabled
    pub wear_leveling_count: Option<u32>, // From SMART data
    pub total_bytes_written: Option<u64>, // Lifetime writes
    pub health_percent: Option<f64>, // Overall SSD health
}

/// Swap configuration (beta.43+)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwapConfiguration {
    pub swap_enabled: bool,
    pub swap_type: String, // "partition", "file", "zram", "none"
    pub swap_size_gb: f64,
    pub swap_usage_percent: f64,
    pub swappiness: u32, // /proc/sys/vm/swappiness value
    pub zram_enabled: bool,
}

/// Locale and timezone information (beta.43+)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocaleInfo {
    pub timezone: String, // e.g., "America/New_York"
    pub locale: String, // e.g., "en_US.UTF-8"
    pub keymap: String, // e.g., "us", "de", "fr"
    pub language: String, // Primary language code
}

/// Alternative software option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub name: String,
    pub description: String,
    pub install_command: String,
}

/// Default popularity score for advice (50 = common)
fn default_popularity() -> u8 {
    50
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
    #[serde(default)]
    pub satisfies: Vec<String>, // IDs of advice that are satisfied/replaced by this advice (e.g., bundles that replace individual packages)
    /// Popularity score (0-100): how commonly this recommendation is needed/applied
    /// 90-100: Essential (microcode, drivers, security basics)
    /// 70-89: Very common (dev tools, popular packages)
    /// 50-69: Common (nice-to-have tools)
    /// 30-49: Niche (specific use cases)
    /// 0-29: Rare (very specialized)
    #[serde(default = "default_popularity")]
    pub popularity: u8,
    /// System requirements that must be met for this advice to be shown (RC.6)
    /// Empty = no requirements, always show
    #[serde(default)]
    pub requires: Vec<Requirement>,
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
            satisfies: Vec::new(),
            popularity: 50, // Default: common
            requires: Vec::new(), // Default: no requirements
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

    /// Set advice IDs that this advice satisfies/replaces
    pub fn with_satisfies(mut self, satisfies: Vec<String>) -> Self {
        self.satisfies = satisfies;
        self
    }

    /// Set popularity score (0-100)
    pub fn with_popularity(mut self, popularity: u8) -> Self {
        self.popularity = popularity.min(100); // Cap at 100
        self
    }

    /// Set system requirements (RC.6)
    /// Only show this advice if all requirements are met
    pub fn with_requirements(mut self, requires: Vec<Requirement>) -> Self {
        self.requires = requires;
        self
    }

    /// Check if all requirements are met for this advice
    pub fn requirements_met(&self, facts: &SystemFacts) -> bool {
        // Empty requirements = always met
        if self.requires.is_empty() {
            return true;
        }

        // All requirements must be met
        self.requires.iter().all(|req| req.is_met(facts))
    }

    /// Get popularity as stars (★★★★★ or ☆☆☆☆☆)
    pub fn popularity_stars(&self) -> String {
        let stars = (self.popularity as f32 / 20.0).round() as u8; // 0-5 stars
        let filled = "★".repeat(stars as usize);
        let empty = "☆".repeat((5 - stars) as usize);
        format!("{}{}", filled, empty)
    }

    /// Get popularity label
    pub fn popularity_label(&self) -> &'static str {
        match self.popularity {
            90..=100 => "Essential",
            70..=89 => "Very Common",
            50..=69 => "Common",
            30..=49 => "Niche",
            _ => "Rare",
        }
    }

    /// Calculate composite recommendation score (0-1000)
    /// Combines priority, risk, and popularity into a single sortable value
    /// Higher score = more important recommendation
    pub fn compute_score(&self) -> u32 {
        // Priority weight: 400 points (most important factor)
        let priority_score = match self.priority {
            Priority::Mandatory => 400,
            Priority::Recommended => 250,
            Priority::Optional => 100,
            Priority::Cosmetic => 50,
        };

        // Risk weight: 300 points (security/stability concerns)
        let risk_score = match self.risk {
            RiskLevel::High => 300,    // High risk items need attention
            RiskLevel::Medium => 150,
            RiskLevel::Low => 50,
        };

        // Popularity weight: 300 points (community adoption)
        // Scale popularity (0-100) to (0-300)
        let popularity_score = self.popularity as u32 * 3;

        priority_score + risk_score + popularity_score
    }

    /// Get score category label
    pub fn score_category(&self) -> &'static str {
        let score = self.compute_score();
        match score {
            900..=1000 => "Critical",
            750..=899 => "High Priority",
            500..=749 => "Recommended",
            300..=499 => "Beneficial",
            _ => "Optional",
        }
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
    /// Rollback command to undo this action (Beta.89+)
    #[serde(default)]
    pub rollback_command: Option<String>,
    /// Whether this action can be safely rolled back
    #[serde(default)]
    pub can_rollback: bool,
    /// Reason why rollback is not available (if can_rollback is false)
    #[serde(default)]
    pub rollback_unavailable_reason: Option<String>,
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

/// Feedback event when user interacts with advice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEvent {
    pub advice_id: String,
    pub advice_category: String,
    pub event_type: FeedbackType,
    pub timestamp: DateTime<Utc>,
    pub username: String,
}

/// Type of feedback event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    Applied,     // User applied the advice
    Dismissed,   // User explicitly dismissed/ignored
    Viewed,      // User viewed but took no action
}

/// User feedback log for learning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserFeedbackLog {
    pub events: Vec<FeedbackEvent>,
}

impl UserFeedbackLog {
    /// Get path to feedback log
    pub fn log_path() -> std::path::PathBuf {
        std::path::PathBuf::from("/var/log/anna/feedback.jsonl")
    }

    /// Load feedback log from disk
    pub fn load() -> Result<Self, std::io::Error> {
        let path = Self::log_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let mut events = Vec::new();

        for line in content.lines() {
            if let Ok(event) = serde_json::from_str::<FeedbackEvent>(line) {
                events.push(event);
            }
        }

        Ok(Self { events })
    }

    /// Save feedback log to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::log_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        for event in &self.events {
            if let Ok(json) = serde_json::to_string(event) {
                content.push_str(&json);
                content.push('\n');
            }
        }

        std::fs::write(path, content)
    }

    /// Record a feedback event
    pub fn record(&mut self, event: FeedbackEvent) {
        self.events.push(event);
    }

    /// Get recent feedback events
    pub fn recent(&self, count: usize) -> Vec<&FeedbackEvent> {
        self.events.iter().rev().take(count).collect()
    }

    /// Count how many times an advice category was dismissed
    pub fn dismissal_count(&self, category: &str) -> usize {
        self.events
            .iter()
            .filter(|e| e.advice_category == category && e.event_type == FeedbackType::Dismissed)
            .count()
    }

    /// Count how many times an advice category was applied
    pub fn application_count(&self, category: &str) -> usize {
        self.events
            .iter()
            .filter(|e| e.advice_category == category && e.event_type == FeedbackType::Applied)
            .count()
    }

    /// Check if a specific advice was dismissed
    pub fn was_dismissed(&self, advice_id: &str) -> bool {
        self.events
            .iter()
            .any(|e| e.advice_id == advice_id && e.event_type == FeedbackType::Dismissed)
    }

    /// Check if a specific advice was applied
    pub fn was_applied(&self, advice_id: &str) -> bool {
        self.events
            .iter()
            .any(|e| e.advice_id == advice_id && e.event_type == FeedbackType::Applied)
    }
}

/// Learned preferences from user behavior
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnedPreferences {
    pub prefers_categories: Vec<String>,        // Categories user applies most
    pub dismisses_categories: Vec<String>,      // Categories user dismisses most
    pub avg_response_time_minutes: f64,         // How long before user acts
    pub prefers_low_risk: bool,                 // Tends to apply only low-risk items
    pub power_user_level: u8,                   // 0-10 based on complexity of applied items
    pub last_updated: DateTime<Utc>,
}

impl LearnedPreferences {
    /// Calculate preferences from feedback log
    pub fn from_feedback(log: &UserFeedbackLog) -> Self {
        let mut category_applications: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut category_dismissals: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for event in &log.events {
            match event.event_type {
                FeedbackType::Applied => {
                    *category_applications.entry(event.advice_category.clone()).or_insert(0) += 1;
                }
                FeedbackType::Dismissed => {
                    *category_dismissals.entry(event.advice_category.clone()).or_insert(0) += 1;
                }
                _ => {}
            }
        }

        // Get top 5 most applied categories
        let mut prefers: Vec<_> = category_applications.iter().collect();
        prefers.sort_by(|a, b| b.1.cmp(a.1));
        let prefers_categories: Vec<String> = prefers.iter().take(5).map(|(k, _)| (*k).clone()).collect();

        // Get top 5 most dismissed categories
        let mut dismisses: Vec<_> = category_dismissals.iter().collect();
        dismisses.sort_by(|a, b| b.1.cmp(a.1));
        let dismisses_categories: Vec<String> = dismisses.iter().take(5).map(|(k, _)| (*k).clone()).collect();

        // Calculate average response time (time between viewing and applying)
        let mut response_times: Vec<i64> = Vec::new();
        let mut sorted_events = log.events.clone();
        sorted_events.sort_by_key(|e| e.timestamp);

        for window in sorted_events.windows(2) {
            if window[0].event_type == FeedbackType::Viewed &&
               window[1].event_type == FeedbackType::Applied &&
               window[0].advice_id == window[1].advice_id {
                let duration = window[1].timestamp - window[0].timestamp;
                response_times.push(duration.num_minutes());
            }
        }

        let avg_response_time_minutes = if !response_times.is_empty() {
            response_times.iter().sum::<i64>() as f64 / response_times.len() as f64
        } else {
            0.0
        };

        // Infer risk preference from applied categories
        // Security/maintenance categories typically have lower risk
        let low_risk_categories = ["security", "maintenance", "cleanup", "updates"];
        let low_risk_applications: usize = category_applications.iter()
            .filter(|(cat, _)| low_risk_categories.contains(&cat.as_str()))
            .map(|(_, count)| count)
            .sum();
        let total_applications: usize = category_applications.values().sum();

        let prefers_low_risk = if total_applications > 5 {
            (low_risk_applications as f64 / total_applications as f64) > 0.6
        } else {
            false // Not enough data
        };

        // Calculate power user level based on variety and volume of applications
        let categories_used = category_applications.len();
        let power_user_level = if total_applications > 50 && categories_used > 10 {
            9 // Power user
        } else if total_applications > 30 && categories_used > 7 {
            7 // Advanced
        } else if total_applications > 15 && categories_used > 5 {
            6 // Intermediate
        } else if total_applications > 5 {
            5 // Regular user
        } else {
            3 // New user
        };

        Self {
            prefers_categories,
            dismisses_categories,
            avg_response_time_minutes,
            prefers_low_risk,
            power_user_level,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// System health score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthScore {
    pub overall_score: u8,              // 0-100
    pub security_score: u8,             // 0-100
    pub performance_score: u8,          // 0-100
    pub maintenance_score: u8,          // 0-100
    pub timestamp: DateTime<Utc>,
    pub issues_count: usize,            // Total pending recommendations
    pub critical_issues: usize,         // Mandatory priority items
    pub health_trend: HealthTrend,      // Improving, stable, or declining
    pub security_details: Vec<String>,  // Why security score is what it is
    pub performance_details: Vec<String>, // Why performance score is what it is
    pub maintenance_details: Vec<String>, // Why maintenance score is what it is
}

/// Health trend indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthTrend {
    Improving,
    Stable,
    Declining,
}

impl SystemHealthScore {
    /// Calculate health score from system facts and advice
    pub fn calculate(facts: &SystemFacts, advice: &[Advice]) -> Self {
        let critical_issues = advice.iter().filter(|a| matches!(a.priority, Priority::Mandatory)).count();
        let total_issues = advice.len();

        // Security score based on security-related advice
        let security_issues = advice.iter().filter(|a| a.category == "Security & Privacy").count();
        let mut security_details = Vec::new();

        let security_score = if security_issues == 0 {
            security_details.push("✓ No security issues detected".to_string());
            security_details.push("✓ All security recommendations applied".to_string());
            100
        } else if security_issues < 3 {
            security_details.push(format!("• {} security recommendation{} pending", security_issues, if security_issues == 1 { "" } else { "s" }));
            security_details.push("  These are minor security improvements".to_string());
            80
        } else if security_issues < 5 {
            security_details.push(format!("⚠ {} security issues found", security_issues));
            security_details.push("  Please address these when possible".to_string());
            60
        } else {
            security_details.push(format!("⚠ {} security issues detected!", security_issues));
            security_details.push("  Your system needs security attention".to_string());
            40
        };

        // Performance score based on system resource usage
        let mut performance_details = Vec::new();
        let mut perf_penalties = Vec::new();

        // Check disk usage
        for disk in &facts.storage_devices {
            let usage = (disk.used_gb / disk.size_gb * 100.0) as u8;
            if usage > 90 {
                performance_details.push(format!("⚠ Disk {} is {}% full ({} GB free)", disk.name, usage, disk.size_gb - disk.used_gb));
                perf_penalties.push(40);
            } else if usage > 80 {
                performance_details.push(format!("• Disk {} is {}% full ({} GB free)", disk.name, usage, disk.size_gb - disk.used_gb));
                perf_penalties.push(10);
            }
        }

        // Check orphan packages
        if facts.orphan_packages.len() > 20 {
            performance_details.push(format!("• {} orphaned packages (wasting disk space)", facts.orphan_packages.len()));
            perf_penalties.push(20);
        } else if facts.orphan_packages.len() > 5 {
            performance_details.push(format!("• {} orphaned packages (run cleanup when convenient)", facts.orphan_packages.len()));
            perf_penalties.push(10);
        }

        // Check performance recommendations
        let perf_advice = advice.iter().filter(|a| a.category == "Performance Optimization").count();
        if perf_advice > 0 {
            performance_details.push(format!("• {} performance optimization{} available", perf_advice, if perf_advice == 1 { "" } else { "s" }));
            perf_penalties.push((perf_advice as u8).min(15)); // Deduct for pending optimizations
        }

        let performance_score = 100 - perf_penalties.iter().sum::<u8>().min(50);

        if performance_details.is_empty() {
            performance_details.push("✓ System performance is optimal".to_string());
            performance_details.push("✓ No storage issues detected".to_string());
        }

        // Maintenance score based on orphans, old kernels, etc.
        let maintenance_issues = advice.iter().filter(|a| a.category == "System Maintenance" || a.category == "Package Management").count();
        let mut maintenance_details = Vec::new();

        let maintenance_score = if maintenance_issues == 0 {
            maintenance_details.push("✓ System is well-maintained".to_string());
            maintenance_details.push("✓ No cleanup needed".to_string());
            100
        } else if maintenance_issues < 5 {
            maintenance_details.push(format!("• {} maintenance task{} pending", maintenance_issues, if maintenance_issues == 1 { "" } else { "s" }));
            maintenance_details.push("  System maintenance is mostly up-to-date".to_string());
            80
        } else {
            maintenance_details.push(format!("⚠ {} maintenance tasks pending", maintenance_issues));
            maintenance_details.push("  System could use some cleanup".to_string());
            60
        };

        // Add specific maintenance info
        if facts.orphan_packages.len() > 0 {
            maintenance_details.push(format!("  - {} orphaned package{}", facts.orphan_packages.len(), if facts.orphan_packages.len() == 1 { "" } else { "s" }));
        }
        if facts.package_cache_size_gb > 1.0 {
            maintenance_details.push(format!("  - Package cache: {:.1} GB", facts.package_cache_size_gb));
        }

        // Overall score is weighted average
        let overall_score = (security_score as f64 * 0.4 + performance_score as f64 * 0.3 + maintenance_score as f64 * 0.3) as u8;

        // Calculate health trend based on current state
        // Future enhancement: track historical scores for accurate trend analysis
        let health_trend = if critical_issues > 0 {
            // Critical issues present = system declining
            HealthTrend::Declining
        } else if overall_score >= 90 && security_issues == 0 && maintenance_issues < 3 {
            // High score, no security issues, minimal maintenance = improving or stable
            if total_issues == 0 {
                HealthTrend::Stable
            } else {
                HealthTrend::Improving
            }
        } else if overall_score >= 70 {
            // Good score but some issues = stable
            HealthTrend::Stable
        } else if overall_score >= 50 {
            // Below good but not critical = could be declining
            if security_issues > 3 || maintenance_issues > 8 {
                HealthTrend::Declining
            } else {
                HealthTrend::Stable
            }
        } else {
            // Low score = declining
            HealthTrend::Declining
        };

        Self {
            overall_score,
            security_score,
            performance_score,
            maintenance_score,
            timestamp: chrono::Utc::now(),
            issues_count: total_issues,
            critical_issues,
            health_trend,
            security_details,
            performance_details,
            maintenance_details,
        }
    }

    /// Get color for display based on score
    pub fn get_color_code(&self) -> &'static str {
        match self.overall_score {
            90..=100 => "\x1b[92m", // Green
            70..=89 => "\x1b[93m",  // Yellow
            50..=69 => "\x1b[33m",  // Orange
            _ => "\x1b[91m",        // Red
        }
    }

    /// Get grade letter
    pub fn get_grade(&self) -> &'static str {
        match self.overall_score {
            95..=100 => "A+",
            90..=94 => "A",
            85..=89 => "A-",
            80..=84 => "B+",
            75..=79 => "B",
            70..=74 => "B-",
            65..=69 => "C+",
            60..=64 => "C",
            55..=59 => "C-",
            50..=54 => "D",
            _ => "F",
        }
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

/// History entry for an applied recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub advice_id: String,
    pub advice_title: String,
    pub category: String,
    pub applied_at: DateTime<Utc>,
    pub applied_by: String,
    pub command_run: Option<String>,
    pub success: bool,
    pub output: String,
    pub health_score_before: Option<u8>,
    pub health_score_after: Option<u8>,
}

/// Application history log
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApplicationHistory {
    pub entries: Vec<HistoryEntry>,
}

impl ApplicationHistory {
    /// Get path to history file
    pub fn history_path() -> std::path::PathBuf {
        std::path::PathBuf::from("/var/log/anna/application_history.jsonl")
    }

    /// Load history from disk
    pub fn load() -> Result<Self, std::io::Error> {
        let path = Self::history_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(line) {
                entries.push(entry);
            }
        }

        Ok(Self { entries })
    }

    /// Save history to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::history_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        for entry in &self.entries {
            if let Ok(json) = serde_json::to_string(entry) {
                content.push_str(&json);
                content.push('\n');
            }
        }

        std::fs::write(path, content)?;
        Ok(())
    }

    /// Record an application
    pub fn record(&mut self, entry: HistoryEntry) {
        self.entries.push(entry);
    }

    /// Get recent applications
    pub fn recent(&self, count: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    /// Get applications from the last N days
    pub fn last_n_days(&self, days: i64) -> Vec<&HistoryEntry> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        self.entries
            .iter()
            .filter(|e| e.applied_at > cutoff)
            .collect()
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        let successful = self.entries.iter().filter(|e| e.success).count();
        (successful as f64 / self.entries.len() as f64) * 100.0
    }

    /// Get most applied categories
    pub fn top_categories(&self, count: usize) -> Vec<(String, usize)> {
        let mut category_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for entry in &self.entries {
            *category_counts.entry(entry.category.clone()).or_insert(0) += 1;
        }

        let mut counts: Vec<_> = category_counts.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1));
        counts.into_iter().take(count).collect()
    }

    /// Calculate average health improvement
    pub fn average_health_improvement(&self) -> Option<f64> {
        let mut improvements = Vec::new();

        for entry in &self.entries {
            if let (Some(before), Some(after)) = (entry.health_score_before, entry.health_score_after) {
                improvements.push((after as i16 - before as i16) as f64);
            }
        }

        if improvements.is_empty() {
            return None;
        }

        Some(improvements.iter().sum::<f64>() / improvements.len() as f64)
    }

    /// Get statistics for a time period
    pub fn period_stats(&self, days: i64) -> PeriodStats {
        let entries = self.last_n_days(days);
        
        let total = entries.len();
        let successful = entries.iter().filter(|e| e.success).count();
        let failed = total - successful;

        let mut category_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for entry in &entries {
            *category_counts.entry(entry.category.clone()).or_insert(0) += 1;
        }

        let top_category = category_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(cat, count)| (cat, count));

        PeriodStats {
            total_applications: total,
            successful_applications: successful,
            failed_applications: failed,
            success_rate: if total > 0 { (successful as f64 / total as f64) * 100.0 } else { 0.0 },
            top_category,
            days,
        }
    }
}

/// Statistics for a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodStats {
    pub total_applications: usize,
    pub successful_applications: usize,
    pub failed_applications: usize,
    pub success_rate: f64,
    pub top_category: Option<(String, usize)>,
    pub days: i64,
}
