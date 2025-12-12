//! Deterministic Intent Map (Part B) - v0.0.439.
//!
//! Maps intent → department + default probes.
//! This is NOT per-question. It is per-intent.
//! The taxonomy is hardcoded, not the answers.

use std::collections::HashMap;

use super::intent_schema::{CanonicalIntent, Department};

/// Mapping entry for an intent.
#[derive(Debug, Clone)]
pub struct IntentMapping {
    /// The canonical intent.
    pub intent: CanonicalIntent,
    /// Department that owns this intent.
    pub department: Department,
    /// Required probes (must succeed for direct answer).
    pub required_probes: Vec<&'static str>,
    /// Optional probes (nice to have).
    pub optional_probes: Vec<&'static str>,
    /// Whether this intent can be answered directly from probes.
    pub can_answer_from_probes: bool,
    /// Description of what this intent covers.
    pub description: &'static str,
}

impl IntentMapping {
    /// Create a new mapping.
    const fn new(
        intent: CanonicalIntent,
        department: Department,
        required: &'static [&'static str],
        optional: &'static [&'static str],
        direct_answer: bool,
        desc: &'static str,
    ) -> Self {
        Self {
            intent,
            department,
            required_probes: Vec::new(), // Will be populated in build
            optional_probes: Vec::new(), // Will be populated in build
            can_answer_from_probes: direct_answer,
            description: desc,
        }
    }
}

/// The deterministic intent mapping table.
pub struct IntentMapTable {
    mappings: HashMap<CanonicalIntent, IntentMapping>,
}

impl IntentMapTable {
    /// Build the canonical intent map.
    pub fn build() -> Self {
        let mut mappings = HashMap::new();

        // ========== PERFORMANCE ==========
        mappings.insert(CanonicalIntent::BootPerf, IntentMapping {
            intent: CanonicalIntent::BootPerf,
            department: Department::Performance,
            required_probes: vec![
                "systemd_analyze",
                "systemd_blame",
            ],
            optional_probes: vec![
                "systemd_critical_chain",
                "journalctl_boot_errors",
            ],
            can_answer_from_probes: true, // Boot time is a fact from systemd-analyze
            description: "Boot performance analysis",
        });

        mappings.insert(CanonicalIntent::MemStatus, IntentMapping {
            intent: CanonicalIntent::MemStatus,
            department: Department::Performance,
            required_probes: vec!["free_h"],
            optional_probes: vec!["meminfo", "vmstat"],
            can_answer_from_probes: true, // RAM available is a direct fact
            description: "Memory status and usage",
        });

        mappings.insert(CanonicalIntent::CpuLoad, IntentMapping {
            intent: CanonicalIntent::CpuLoad,
            department: Department::Performance,
            required_probes: vec!["uptime", "top_cpu"],
            optional_probes: vec!["mpstat", "ps_aux_cpu"],
            can_answer_from_probes: true, // Load average is a direct fact
            description: "CPU load and top consumers",
        });

        mappings.insert(CanonicalIntent::IoWait, IntentMapping {
            intent: CanonicalIntent::IoWait,
            department: Department::Performance,
            required_probes: vec!["iostat", "vmstat"],
            optional_probes: vec!["iotop_snapshot"],
            can_answer_from_probes: true,
            description: "I/O wait analysis",
        });

        // ========== STORAGE ==========
        mappings.insert(CanonicalIntent::DiskUsage, IntentMapping {
            intent: CanonicalIntent::DiskUsage,
            department: Department::Storage,
            required_probes: vec!["df_h"],
            optional_probes: vec!["lsblk", "du_top_dirs"],
            can_answer_from_probes: true, // Disk % is a direct fact
            description: "Disk usage and free space",
        });

        mappings.insert(CanonicalIntent::MountHealth, IntentMapping {
            intent: CanonicalIntent::MountHealth,
            department: Department::Storage,
            required_probes: vec!["mount", "findmnt"],
            optional_probes: vec!["fstab_check"],
            can_answer_from_probes: true,
            description: "Mount point health",
        });

        mappings.insert(CanonicalIntent::SmartStatus, IntentMapping {
            intent: CanonicalIntent::SmartStatus,
            department: Department::Storage,
            required_probes: vec!["smartctl_health"],
            optional_probes: vec!["smartctl_attributes"],
            can_answer_from_probes: true,
            description: "SMART disk health",
        });

        mappings.insert(CanonicalIntent::BtrfsHealth, IntentMapping {
            intent: CanonicalIntent::BtrfsHealth,
            department: Department::Storage,
            required_probes: vec!["btrfs_fi_show", "btrfs_device_stats"],
            optional_probes: vec!["btrfs_scrub_status"],
            can_answer_from_probes: true,
            description: "Btrfs filesystem health",
        });

        // ========== SERVICES ==========
        mappings.insert(CanonicalIntent::SvcFailed, IntentMapping {
            intent: CanonicalIntent::SvcFailed,
            department: Department::Services,
            required_probes: vec!["systemctl_failed"],
            optional_probes: vec!["journalctl_failed_units"],
            can_answer_from_probes: true, // List of failed services is a fact
            description: "Failed systemd services",
        });

        mappings.insert(CanonicalIntent::SvcHealth, IntentMapping {
            intent: CanonicalIntent::SvcHealth,
            department: Department::Services,
            required_probes: vec!["systemctl_status_all"],
            optional_probes: vec!["systemctl_list_units"],
            can_answer_from_probes: false, // Needs synthesis for "health"
            description: "Overall service health",
        });

        mappings.insert(CanonicalIntent::SvcStatus, IntentMapping {
            intent: CanonicalIntent::SvcStatus,
            department: Department::Services,
            required_probes: vec!["systemctl_status"], // Needs service name
            optional_probes: vec!["journalctl_unit"],
            can_answer_from_probes: true,
            description: "Specific service status",
        });

        mappings.insert(CanonicalIntent::LogsRecentErrors, IntentMapping {
            intent: CanonicalIntent::LogsRecentErrors,
            department: Department::Services,
            required_probes: vec!["journalctl_errors_20"],
            optional_probes: vec!["dmesg_errors"],
            can_answer_from_probes: true,
            description: "Recent error logs",
        });

        mappings.insert(CanonicalIntent::TimerStatus, IntentMapping {
            intent: CanonicalIntent::TimerStatus,
            department: Department::Services,
            required_probes: vec!["systemctl_list_timers"],
            optional_probes: vec![],
            can_answer_from_probes: true,
            description: "Systemd timer status",
        });

        // ========== NETWORK ==========
        mappings.insert(CanonicalIntent::NetHealth, IntentMapping {
            intent: CanonicalIntent::NetHealth,
            department: Department::Network,
            required_probes: vec!["ip_addr", "ip_route"],
            optional_probes: vec!["nmcli_status", "ss_listen"],
            can_answer_from_probes: false, // "Health" needs synthesis
            description: "Network health overview",
        });

        mappings.insert(CanonicalIntent::DnsHealth, IntentMapping {
            intent: CanonicalIntent::DnsHealth,
            department: Department::Network,
            required_probes: vec!["resolvectl_status"],
            optional_probes: vec!["dig_test", "getent_hosts_test"],
            can_answer_from_probes: true,
            description: "DNS resolution health",
        });

        mappings.insert(CanonicalIntent::WifiStatus, IntentMapping {
            intent: CanonicalIntent::WifiStatus,
            department: Department::Network,
            required_probes: vec!["iw_link", "nmcli_wifi"],
            optional_probes: vec!["iwconfig"],
            can_answer_from_probes: true,
            description: "WiFi connection status",
        });

        mappings.insert(CanonicalIntent::RouteStatus, IntentMapping {
            intent: CanonicalIntent::RouteStatus,
            department: Department::Network,
            required_probes: vec!["ip_route", "ip_rule"],
            optional_probes: vec!["traceroute_gateway"],
            can_answer_from_probes: true,
            description: "Routing table status",
        });

        // ========== HARDWARE ==========
        mappings.insert(CanonicalIntent::GpuInfo, IntentMapping {
            intent: CanonicalIntent::GpuInfo,
            department: Department::Hardware,
            required_probes: vec!["lspci_gpu"],
            optional_probes: vec!["glxinfo", "nvidia_smi"],
            can_answer_from_probes: true,
            description: "GPU information",
        });

        mappings.insert(CanonicalIntent::GpuDriver, IntentMapping {
            intent: CanonicalIntent::GpuDriver,
            department: Department::Hardware,
            required_probes: vec!["lspci_k_gpu", "lsmod_gpu"],
            optional_probes: vec!["modinfo_gpu", "dmesg_gpu"],
            can_answer_from_probes: true,
            description: "GPU driver status",
        });

        mappings.insert(CanonicalIntent::HardwareSensors, IntentMapping {
            intent: CanonicalIntent::HardwareSensors,
            department: Department::Hardware,
            required_probes: vec!["sensors"],
            optional_probes: vec!["hwinfo_temps"],
            can_answer_from_probes: true,
            description: "Hardware temperature sensors",
        });

        mappings.insert(CanonicalIntent::CpuInfo, IntentMapping {
            intent: CanonicalIntent::CpuInfo,
            department: Department::Hardware,
            required_probes: vec!["lscpu"],
            optional_probes: vec!["cpuinfo"],
            can_answer_from_probes: true,
            description: "CPU hardware information",
        });

        mappings.insert(CanonicalIntent::AudioHealth, IntentMapping {
            intent: CanonicalIntent::AudioHealth,
            department: Department::Hardware,
            required_probes: vec!["pactl_info", "aplay_l"],
            optional_probes: vec!["pipewire_status", "alsa_info"],
            can_answer_from_probes: false, // Audio "health" needs synthesis
            description: "Audio subsystem health",
        });

        mappings.insert(CanonicalIntent::UsbDevices, IntentMapping {
            intent: CanonicalIntent::UsbDevices,
            department: Department::Hardware,
            required_probes: vec!["lsusb"],
            optional_probes: vec!["usb_devices"],
            can_answer_from_probes: true,
            description: "USB device listing",
        });

        mappings.insert(CanonicalIntent::PciDevices, IntentMapping {
            intent: CanonicalIntent::PciDevices,
            department: Department::Hardware,
            required_probes: vec!["lspci"],
            optional_probes: vec!["lspci_v"],
            can_answer_from_probes: true,
            description: "PCI device listing",
        });

        // ========== DESKTOP ==========
        mappings.insert(CanonicalIntent::SessionDesktop, IntentMapping {
            intent: CanonicalIntent::SessionDesktop,
            department: Department::Desktop,
            required_probes: vec!["echo_xdg_session", "loginctl_session"],
            optional_probes: vec!["echo_desktop_session"],
            can_answer_from_probes: true,
            description: "Desktop session info",
        });

        mappings.insert(CanonicalIntent::EditorConfig, IntentMapping {
            intent: CanonicalIntent::EditorConfig,
            department: Department::Desktop,
            required_probes: vec![], // No probes, config lookup
            optional_probes: vec![],
            can_answer_from_probes: false,
            description: "Editor configuration",
        });

        mappings.insert(CanonicalIntent::ShellConfig, IntentMapping {
            intent: CanonicalIntent::ShellConfig,
            department: Department::Desktop,
            required_probes: vec!["echo_shell"],
            optional_probes: vec![],
            can_answer_from_probes: false,
            description: "Shell configuration",
        });

        mappings.insert(CanonicalIntent::ThemeConfig, IntentMapping {
            intent: CanonicalIntent::ThemeConfig,
            department: Department::Desktop,
            required_probes: vec![],
            optional_probes: vec!["gsettings_theme"],
            can_answer_from_probes: false,
            description: "Theme configuration",
        });

        // ========== SECURITY ==========
        mappings.insert(CanonicalIntent::SecurityFirewall, IntentMapping {
            intent: CanonicalIntent::SecurityFirewall,
            department: Department::Security,
            required_probes: vec!["firewall_status"],
            optional_probes: vec!["iptables_l", "nft_list"],
            can_answer_from_probes: true,
            description: "Firewall status",
        });

        mappings.insert(CanonicalIntent::PermissionCheck, IntentMapping {
            intent: CanonicalIntent::PermissionCheck,
            department: Department::Security,
            required_probes: vec![], // Needs path
            optional_probes: vec![],
            can_answer_from_probes: false,
            description: "Permission check",
        });

        mappings.insert(CanonicalIntent::VulnCheck, IntentMapping {
            intent: CanonicalIntent::VulnCheck,
            department: Department::Security,
            required_probes: vec!["arch_audit"],
            optional_probes: vec![],
            can_answer_from_probes: false, // Needs synthesis
            description: "Vulnerability check",
        });

        // ========== PACKAGES ==========
        mappings.insert(CanonicalIntent::PkgInventory, IntentMapping {
            intent: CanonicalIntent::PkgInventory,
            department: Department::Services, // Package management is a service
            required_probes: vec!["pacman_q_count"],
            optional_probes: vec!["pacman_qe"],
            can_answer_from_probes: true,
            description: "Package inventory",
        });

        mappings.insert(CanonicalIntent::PkgUpdates, IntentMapping {
            intent: CanonicalIntent::PkgUpdates,
            department: Department::Services,
            required_probes: vec!["checkupdates"],
            optional_probes: vec![],
            can_answer_from_probes: true,
            description: "Available package updates",
        });

        mappings.insert(CanonicalIntent::PkgSearch, IntentMapping {
            intent: CanonicalIntent::PkgSearch,
            department: Department::Services,
            required_probes: vec![], // Needs package name
            optional_probes: vec![],
            can_answer_from_probes: false,
            description: "Package search",
        });

        // ========== UNKNOWN ==========
        mappings.insert(CanonicalIntent::Unknown, IntentMapping {
            intent: CanonicalIntent::Unknown,
            department: Department::Services, // Default to services
            required_probes: vec![],
            optional_probes: vec![],
            can_answer_from_probes: false,
            description: "Unknown intent - needs clarification",
        });

        Self { mappings }
    }

    /// Get mapping for an intent.
    pub fn get(&self, intent: CanonicalIntent) -> Option<&IntentMapping> {
        self.mappings.get(&intent)
    }

    /// Get the correct department for an intent.
    pub fn get_department(&self, intent: CanonicalIntent) -> Department {
        self.mappings
            .get(&intent)
            .map(|m| m.department)
            .unwrap_or(Department::Services)
    }

    /// Get required probes for an intent.
    pub fn get_required_probes(&self, intent: CanonicalIntent) -> Vec<&str> {
        self.mappings
            .get(&intent)
            .map(|m| m.required_probes.clone())
            .unwrap_or_default()
    }

    /// Get optional probes for an intent.
    pub fn get_optional_probes(&self, intent: CanonicalIntent) -> Vec<&str> {
        self.mappings
            .get(&intent)
            .map(|m| m.optional_probes.clone())
            .unwrap_or_default()
    }

    /// Check if intent can be answered directly from probes.
    pub fn can_answer_directly(&self, intent: CanonicalIntent) -> bool {
        self.mappings
            .get(&intent)
            .map(|m| m.can_answer_from_probes)
            .unwrap_or(false)
    }

    /// List all intents for a department.
    pub fn intents_for_department(&self, dept: Department) -> Vec<CanonicalIntent> {
        self.mappings
            .values()
            .filter(|m| m.department == dept)
            .map(|m| m.intent)
            .collect()
    }
}

impl Default for IntentMapTable {
    fn default() -> Self {
        Self::build()
    }
}

/// Global intent map (lazy static alternative).
pub fn get_intent_map() -> IntentMapTable {
    IntentMapTable::build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_perf_maps_to_performance() {
        let map = IntentMapTable::build();
        assert_eq!(map.get_department(CanonicalIntent::BootPerf), Department::Performance);
    }

    #[test]
    fn test_gpu_maps_to_hardware() {
        let map = IntentMapTable::build();
        assert_eq!(map.get_department(CanonicalIntent::GpuInfo), Department::Hardware);
        assert_eq!(map.get_department(CanonicalIntent::GpuDriver), Department::Hardware);
    }

    #[test]
    fn test_disk_maps_to_storage() {
        let map = IntentMapTable::build();
        assert_eq!(map.get_department(CanonicalIntent::DiskUsage), Department::Storage);
    }

    #[test]
    fn test_ram_maps_to_performance() {
        let map = IntentMapTable::build();
        assert_eq!(map.get_department(CanonicalIntent::MemStatus), Department::Performance);
    }

    #[test]
    fn test_required_probes_for_boot_perf() {
        let map = IntentMapTable::build();
        let probes = map.get_required_probes(CanonicalIntent::BootPerf);
        assert!(probes.contains(&"systemd_analyze"));
        assert!(probes.contains(&"systemd_blame"));
    }

    #[test]
    fn test_can_answer_directly() {
        let map = IntentMapTable::build();
        // Facts can be answered directly
        assert!(map.can_answer_directly(CanonicalIntent::MemStatus));
        assert!(map.can_answer_directly(CanonicalIntent::DiskUsage));
        // "Health" synthesis cannot
        assert!(!map.can_answer_directly(CanonicalIntent::SvcHealth));
        assert!(!map.can_answer_directly(CanonicalIntent::NetHealth));
    }

    #[test]
    fn test_intents_for_department() {
        let map = IntentMapTable::build();
        let perf_intents = map.intents_for_department(Department::Performance);
        assert!(perf_intents.contains(&CanonicalIntent::BootPerf));
        assert!(perf_intents.contains(&CanonicalIntent::MemStatus));
        assert!(!perf_intents.contains(&CanonicalIntent::DiskUsage)); // Storage
    }
}
