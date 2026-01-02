//! CanonicalIntent implementation methods.

use super::types::CanonicalIntent;

impl CanonicalIntent {
    /// Parse from string (fuzzy matching)
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        match lower.as_str() {
            // Memory
            "check_free_ram" | "free_ram" | "ram_usage" | "memory_usage" => Self::CheckFreeRam,
            "check_swap_presence" | "swap_presence" | "has_swap" => Self::CheckSwapPresence,
            "check_swap_usage" | "swap_usage" => Self::CheckSwapUsage,
            "list_top_memory_processes" | "top_memory" | "memory_hogs" => {
                Self::ListTopMemoryProcesses
            }

            // Storage
            "check_disk_usage" | "disk_usage" | "disk_space" => Self::CheckDiskUsage,
            "check_disk_health" | "disk_health" | "smart" => Self::CheckDiskHealth,
            "check_trim_service" | "trim_service" | "fstrim" => Self::CheckTrimService,
            "find_largest_files" | "largest_files" | "big_files" => Self::FindLargestFiles,

            // Services
            "check_failed_services" | "failed_services" | "failed_units" => {
                Self::CheckFailedServices
            }
            "check_service_status" | "service_status" => Self::CheckServiceStatus,
            "list_running_services" | "running_services" => Self::ListRunningServices,
            "check_timers" | "systemd_timers" => Self::CheckTimers,

            // Boot
            "check_boot_time" | "boot_time" | "startup_time" => Self::CheckBootTime,
            "diagnose_slow_boot" | "slow_boot" | "boot_slow" => Self::DiagnoseSlowBoot,
            "check_boot_errors" | "boot_errors" => Self::CheckBootErrors,

            // Network
            "check_network_connectivity" | "network_connectivity" | "ping" => {
                Self::CheckNetworkConnectivity
            }
            "check_dns_health" | "dns_health" | "dns" => Self::CheckDnsHealth,
            "check_listening_ports" | "listening_ports" | "open_ports" => Self::CheckListeningPorts,
            "check_firewall_status" | "firewall_status" | "firewall" => Self::CheckFirewallStatus,

            // Packages
            "check_package_installed" | "package_installed" | "is_installed" => {
                Self::CheckPackageInstalled
            }
            "list_installed_packages" | "installed_packages" => Self::ListInstalledPackages,
            "check_updates" | "updates" | "checkupdates" => Self::CheckUpdates,

            // Process
            "list_top_cpu_processes" | "top_cpu" | "cpu_hogs" => Self::ListTopCpuProcesses,
            "check_uptime" | "uptime" => Self::CheckUptime,
            "check_load_average" | "load_average" | "load" => Self::CheckLoadAverage,

            // Desktop
            "check_desktop_environment" | "desktop_environment" | "de" | "wm" => {
                Self::CheckDesktopEnvironment
            }
            "find_config_file" | "config_file" | "config" => Self::FindConfigFile,

            // Audio
            "check_audio_devices" | "audio_devices" | "sound_devices" => Self::CheckAudioDevices,
            "check_audio_server" | "audio_server" | "pipewire" | "pulseaudio" => {
                Self::CheckAudioServer
            }

            // GPU
            "check_gpu_drivers" | "gpu_drivers" | "gpu" => Self::CheckGpuDrivers,
            "check_display_info" | "display_info" | "monitors" => Self::CheckDisplayInfo,

            // Generic
            "explain_concept" | "explain" | "what_is" => Self::ExplainConcept,
            _ => Self::GeneralQuery,
        }
    }

    /// Get required probes for this intent
    pub fn required_probes(&self) -> Vec<&'static str> {
        match self {
            // Memory
            Self::CheckFreeRam => vec!["memory_info"],
            Self::CheckSwapPresence => vec!["swap_files"],
            Self::CheckSwapUsage => vec!["swap_files", "memory_info"],
            Self::ListTopMemoryProcesses => vec!["top_memory"],

            // Storage
            Self::CheckDiskUsage => vec!["disk_usage"],
            Self::CheckDiskHealth => vec!["disk_usage", "block_devices"],
            Self::CheckTrimService => vec!["systemd_timers", "fstrim_status"],
            Self::FindLargestFiles => vec!["largest_dirs"],

            // Services
            Self::CheckFailedServices => vec!["failed_services"],
            Self::CheckServiceStatus => vec!["running_services"],
            Self::ListRunningServices => vec!["running_services"],
            Self::CheckTimers => vec!["systemd_timers"],

            // Boot
            Self::CheckBootTime => vec!["boot_time"],
            Self::DiagnoseSlowBoot => vec!["boot_time", "boot_blame"],
            Self::CheckBootErrors => vec!["boot_time", "journal_errors"],

            // Network
            Self::CheckNetworkConnectivity => vec!["network_addrs", "ping_check"],
            Self::CheckDnsHealth => vec!["dns_servers"],
            Self::CheckListeningPorts => vec!["listening_ports"],
            Self::CheckFirewallStatus => vec!["firewall_status"],

            // Packages
            Self::CheckPackageInstalled => vec!["package_count"],
            Self::ListInstalledPackages => vec!["installed_packages"],
            Self::CheckUpdates => vec!["package_count"],

            // Process
            Self::ListTopCpuProcesses => vec!["top_cpu"],
            Self::CheckUptime => vec!["uptime"],
            Self::CheckLoadAverage => vec!["load_average"],

            // Desktop
            Self::CheckDesktopEnvironment => vec!["desktop_session"],
            Self::FindConfigFile => vec!["desktop_session"],

            // Audio
            Self::CheckAudioDevices => vec!["audio_devices"],
            Self::CheckAudioServer => vec!["audio_server"],

            // GPU
            Self::CheckGpuDrivers => vec!["gpu_info"],
            Self::CheckDisplayInfo => vec!["display_info"],

            // Generic
            Self::ExplainConcept => vec![],
            Self::GeneralQuery => vec![],
        }
    }

    /// Get knowledge topics for this intent
    pub fn knowledge_topics(&self) -> Vec<&'static str> {
        match self {
            // Memory
            Self::CheckFreeRam | Self::ListTopMemoryProcesses => vec!["ram_usage", "free_command"],
            Self::CheckSwapPresence | Self::CheckSwapUsage => {
                vec!["swap_configuration", "proc_swaps"]
            }

            // Storage
            Self::CheckDiskUsage => vec!["df_command", "disk_usage"],
            Self::CheckDiskHealth => vec!["smartctl", "disk_health"],
            Self::CheckTrimService => vec!["fstrim", "systemd_timers", "ssd_trim"],
            Self::FindLargestFiles => vec!["du_command", "disk_usage"],

            // Services
            Self::CheckFailedServices | Self::CheckServiceStatus | Self::ListRunningServices => {
                vec!["systemctl", "systemd_units"]
            }
            Self::CheckTimers => vec!["systemd_timers", "systemctl"],

            // Boot
            Self::CheckBootTime | Self::DiagnoseSlowBoot | Self::CheckBootErrors => {
                vec!["systemd_analyze", "boot_performance", "journalctl"]
            }

            // Network
            Self::CheckNetworkConnectivity => vec!["ip_command", "network_interfaces"],
            Self::CheckDnsHealth => vec!["dns_resolver", "resolvectl"],
            Self::CheckListeningPorts => vec!["ss_command", "listening_ports"],
            Self::CheckFirewallStatus => vec!["firewall", "ufw", "iptables"],

            // Packages
            Self::CheckPackageInstalled | Self::ListInstalledPackages | Self::CheckUpdates => {
                vec!["pacman", "package_management"]
            }

            // Process
            Self::ListTopCpuProcesses => vec!["ps_command", "top_command"],
            Self::CheckUptime | Self::CheckLoadAverage => vec!["uptime", "load_average"],

            // Desktop
            Self::CheckDesktopEnvironment | Self::FindConfigFile => {
                vec!["desktop_environment", "config_files"]
            }

            // Audio
            Self::CheckAudioDevices | Self::CheckAudioServer => {
                vec!["pipewire", "pulseaudio", "audio"]
            }

            // GPU
            Self::CheckGpuDrivers | Self::CheckDisplayInfo => vec!["gpu_drivers", "lspci"],

            // Generic
            Self::ExplainConcept | Self::GeneralQuery => vec![],
        }
    }

    /// Check if this intent can be answered by a recipe (deterministic)
    pub fn is_recipe_eligible(&self) -> bool {
        !matches!(self, Self::ExplainConcept | Self::GeneralQuery)
    }

    /// Get display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::CheckFreeRam => "Check Free RAM",
            Self::CheckSwapPresence => "Check Swap Presence",
            Self::CheckSwapUsage => "Check Swap Usage",
            Self::ListTopMemoryProcesses => "Top Memory Processes",
            Self::CheckDiskUsage => "Check Disk Usage",
            Self::CheckDiskHealth => "Check Disk Health",
            Self::CheckTrimService => "Check TRIM Service",
            Self::FindLargestFiles => "Find Largest Files",
            Self::CheckFailedServices => "Check Failed Services",
            Self::CheckServiceStatus => "Check Service Status",
            Self::ListRunningServices => "List Running Services",
            Self::CheckTimers => "Check Timers",
            Self::CheckBootTime => "Check Boot Time",
            Self::DiagnoseSlowBoot => "Diagnose Slow Boot",
            Self::CheckBootErrors => "Check Boot Errors",
            Self::CheckNetworkConnectivity => "Check Network",
            Self::CheckDnsHealth => "Check DNS",
            Self::CheckListeningPorts => "Check Listening Ports",
            Self::CheckFirewallStatus => "Check Firewall",
            Self::CheckPackageInstalled => "Check Package Installed",
            Self::ListInstalledPackages => "List Installed Packages",
            Self::CheckUpdates => "Check Updates",
            Self::ListTopCpuProcesses => "Top CPU Processes",
            Self::CheckUptime => "Check Uptime",
            Self::CheckLoadAverage => "Check Load Average",
            Self::CheckDesktopEnvironment => "Check Desktop Environment",
            Self::FindConfigFile => "Find Config File",
            Self::CheckAudioDevices => "Check Audio Devices",
            Self::CheckAudioServer => "Check Audio Server",
            Self::CheckGpuDrivers => "Check GPU Drivers",
            Self::CheckDisplayInfo => "Check Display Info",
            Self::ExplainConcept => "Explain Concept",
            Self::GeneralQuery => "General Query",
        }
    }
}
