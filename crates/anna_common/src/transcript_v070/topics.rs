//! Evidence Topic Abstraction for v0.0.70
//!
//! Maps internal tool names to human-friendly topic categories.
//! Used in Human Mode to show "hardware inventory snapshot" instead of "hw_snapshot_summary".

use serde::{Deserialize, Serialize};

/// Human-friendly evidence topic categories
/// Maps internal tool names to topics users can understand
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTopicV70 {
    HardwareInventory,
    SoftwareInventory,
    ServiceInventory,
    NetworkSnapshot,
    StorageSnapshot,
    AudioSnapshot,
    GraphicsSnapshot,
    BootTimeline,
    JournalErrorIndex,
    PackageHistory,
    SystemStatus,
    DaemonStatus,
    ConfigState,
    DiagnosticsSignals,
    DocumentationSearch,
    Custom(String),
}

impl EvidenceTopicV70 {
    /// Human-readable description for display (v0.0.71: shorter labels)
    pub fn human_description(&self) -> &str {
        match self {
            Self::HardwareInventory => "hardware inventory",
            Self::SoftwareInventory => "software/services inventory",
            Self::ServiceInventory => "service status",
            Self::NetworkSnapshot => "network link and routing signals",
            Self::StorageSnapshot => "storage status",
            Self::AudioSnapshot => "audio stack",
            Self::GraphicsSnapshot => "graphics status",
            Self::BootTimeline => "boot timeline",
            Self::JournalErrorIndex => "system error journal summary",
            Self::PackageHistory => "package changes",
            Self::SystemStatus => "system load",
            Self::DaemonStatus => "daemon health",
            Self::ConfigState => "config state",
            Self::DiagnosticsSignals => "diagnostics",
            Self::DocumentationSearch => "documentation",
            Self::Custom(s) => s,
        }
    }
}

/// Map tool name to evidence topic
pub fn tool_to_evidence_topic(tool_name: &str) -> EvidenceTopicV70 {
    match tool_name {
        // Hardware tools
        "hw_snapshot_summary"
        | "hw_snapshot_cpu"
        | "hw_snapshot_memory"
        | "hw_snapshot_gpu"
        | "hw_snapshot_disk"
        | "hardware_info" => EvidenceTopicV70::HardwareInventory,

        // Software tools
        "sw_snapshot_summary"
        | "sw_snapshot_packages"
        | "sw_snapshot_services"
        | "recent_installs"
        | "package_info"
        | "installed_packages" => EvidenceTopicV70::SoftwareInventory,

        // Service tools
        "service_status" | "service_logs" | "service_inventory" | "systemd_units" => {
            EvidenceTopicV70::ServiceInventory
        }

        // Network tools
        "network_status"
        | "network_interfaces"
        | "network_connectivity"
        | "dns_check"
        | "ping_check"
        | "wifi_status"
        | "ip_addresses" => EvidenceTopicV70::NetworkSnapshot,

        // Storage tools
        "disk_usage" | "disk_free" | "mount_info" | "storage_health" | "btrfs_status"
        | "fstab_check" => EvidenceTopicV70::StorageSnapshot,

        // Audio tools
        "audio_status" | "pipewire_status" | "pulseaudio_status" | "alsa_info" => {
            EvidenceTopicV70::AudioSnapshot
        }

        // Graphics tools
        "gpu_info" | "xorg_log" | "wayland_status" | "display_info" => {
            EvidenceTopicV70::GraphicsSnapshot
        }

        // Boot tools
        "boot_time" | "boot_timeline" | "boot_time_trend" | "systemd_analyze" => {
            EvidenceTopicV70::BootTimeline
        }

        // Journal tools
        "journal_errors" | "journal_warnings" | "recent_errors" | "dmesg_errors" => {
            EvidenceTopicV70::JournalErrorIndex
        }

        // Package tools
        "package_history" | "recent_updates" | "package_changelog" => {
            EvidenceTopicV70::PackageHistory
        }

        // System tools
        "uptime" | "load_average" | "memory_usage" | "cpu_usage" => EvidenceTopicV70::SystemStatus,

        // Daemon tools
        "anna_status" | "daemon_health" | "daemon_metrics" => EvidenceTopicV70::DaemonStatus,

        // Config tools
        "config_check" | "config_diff" | "env_vars" => EvidenceTopicV70::ConfigState,

        // Diagnostics
        "diagnostics" | "health_check" | "system_scan" => EvidenceTopicV70::DiagnosticsSignals,

        // Documentation
        "man_search" | "doc_search" | "knowledge_search" => EvidenceTopicV70::DocumentationSearch,

        // Default
        _ => EvidenceTopicV70::Custom(tool_name.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_to_topic_mapping() {
        assert_eq!(
            tool_to_evidence_topic("hw_snapshot_summary"),
            EvidenceTopicV70::HardwareInventory
        );
        assert_eq!(
            tool_to_evidence_topic("network_status"),
            EvidenceTopicV70::NetworkSnapshot
        );
        assert_eq!(
            tool_to_evidence_topic("unknown_tool"),
            EvidenceTopicV70::Custom("unknown_tool".to_string())
        );
    }

    #[test]
    fn test_human_description() {
        // v0.0.71: shorter labels, no "snapshot" suffix
        assert_eq!(
            EvidenceTopicV70::HardwareInventory.human_description(),
            "hardware inventory"
        );
        assert_eq!(
            EvidenceTopicV70::NetworkSnapshot.human_description(),
            "network link and routing signals"
        );
    }
}
