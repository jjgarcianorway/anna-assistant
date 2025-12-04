//! Human Labels for Evidence Topics v0.0.71
//!
//! Shorter, more natural labels for evidence topics in human mode.
//! Also provides one-line summary templates for common query types.

use serde::{Deserialize, Serialize};

/// Short human label for evidence topic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanLabel {
    HardwareInventory,
    SoftwareServices,
    ServiceStatus,
    NetworkSignals,
    StorageStatus,
    AudioStack,
    GraphicsStatus,
    BootTimeline,
    ErrorJournal,
    PackageChanges,
    SystemLoad,
    DaemonHealth,
    ConfigState,
    Diagnostics,
    Documentation,
    Custom(String),
}

impl HumanLabel {
    /// Short human-readable name (no "snapshot" suffix)
    pub fn short_name(&self) -> &str {
        match self {
            Self::HardwareInventory => "hardware inventory",
            Self::SoftwareServices => "software/services inventory",
            Self::ServiceStatus => "service status",
            Self::NetworkSignals => "network link and routing signals",
            Self::StorageStatus => "storage status",
            Self::AudioStack => "audio stack",
            Self::GraphicsStatus => "graphics status",
            Self::BootTimeline => "boot timeline",
            Self::ErrorJournal => "system error journal summary",
            Self::PackageChanges => "package changes",
            Self::SystemLoad => "system load",
            Self::DaemonHealth => "daemon health",
            Self::ConfigState => "config state",
            Self::Diagnostics => "diagnostics",
            Self::Documentation => "documentation",
            Self::Custom(s) => s,
        }
    }

    /// From legacy EvidenceTopicV70 human_description
    pub fn from_legacy(description: &str) -> Self {
        match description {
            "hardware inventory snapshot" => Self::HardwareInventory,
            "software and services inventory snapshot" => Self::SoftwareServices,
            "service inventory" => Self::ServiceStatus,
            "network status snapshot" => Self::NetworkSignals,
            "storage status snapshot" => Self::StorageStatus,
            "audio stack snapshot" => Self::AudioStack,
            "graphics status snapshot" => Self::GraphicsStatus,
            "boot timing history" => Self::BootTimeline,
            "journal error index" => Self::ErrorJournal,
            "package change history" => Self::PackageChanges,
            "system status" => Self::SystemLoad,
            "Anna daemon status snapshot" => Self::DaemonHealth,
            "configuration state" => Self::ConfigState,
            "diagnostics signals" => Self::Diagnostics,
            "documentation search" => Self::Documentation,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// One-line evidence summary generator
pub struct EvidenceSummary;

impl EvidenceSummary {
    /// Generate CPU summary from evidence
    pub fn cpu(model: &str) -> String {
        format!("hardware inventory reports {}", model)
    }

    /// Generate memory summary from evidence
    pub fn memory(total_gb: f64, used_percent: u8) -> String {
        format!(
            "hardware inventory reports {:.1} GiB total, {}% used",
            total_gb, used_percent
        )
    }

    /// Generate disk summary from evidence
    pub fn disk_free(mount: &str, free_gib: f64, total_gib: f64) -> String {
        format!(
            "storage status reports {:.1} GiB free on {} ({:.1} GiB total)",
            free_gib, mount, total_gib
        )
    }

    /// Generate network summary from evidence
    pub fn network(interfaces: usize, manager_present: bool, link_status: &str) -> String {
        let manager_str = if manager_present {
            "manager present"
        } else {
            "no manager"
        };
        format!(
            "network signals show {} interface(s) detected, {}, {}",
            interfaces, manager_str, link_status
        )
    }

    /// Generate service status summary from evidence
    pub fn service(name: &str, state: &str, substate: &str) -> String {
        format!(
            "service status reports {} is {} ({})",
            name, state, substate
        )
    }

    /// Generate systemd running summary
    pub fn systemd_running(running: bool, version: &str) -> String {
        if running {
            format!("systemd {} is running as PID 1", version)
        } else {
            "systemd status could not be determined".to_string()
        }
    }

    /// Generate boot time summary
    pub fn boot_time(seconds: f64) -> String {
        format!(
            "boot timeline shows system started in {:.1} seconds",
            seconds
        )
    }

    /// Generate audio summary
    pub fn audio(server: &str, sinks: usize) -> String {
        format!("audio stack reports {} with {} sink(s)", server, sinks)
    }

    /// Generate "evidence missing" message
    pub fn missing(topic: &str) -> String {
        format!("no {} evidence available in current snapshots", topic)
    }

    /// Generate "evidence incomplete" message
    pub fn incomplete(topic: &str, reason: &str) -> String {
        format!("{} evidence incomplete: {}", topic, reason)
    }
}

/// Convert raw evidence data to human-friendly one-liner
pub fn humanize_evidence(topic: &HumanLabel, raw_data: &str) -> String {
    // Try to parse common patterns
    match topic {
        HumanLabel::HardwareInventory => {
            // Try to extract CPU model
            if raw_data.contains("model name") {
                if let Some(model) = extract_after(raw_data, "model name:") {
                    return EvidenceSummary::cpu(model.trim());
                }
            }
            format!("hardware inventory: {}", truncate(raw_data, 60))
        }
        HumanLabel::StorageStatus => {
            // Try to extract disk free
            if raw_data.contains("Avail") || raw_data.contains("free") {
                return format!("storage status: {}", truncate(raw_data, 60));
            }
            format!("storage status: {}", truncate(raw_data, 60))
        }
        HumanLabel::NetworkSignals => {
            format!("network signals: {}", truncate(raw_data, 60))
        }
        _ => {
            format!("{}: {}", topic.short_name(), truncate(raw_data, 60))
        }
    }
}

fn extract_after<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    text.find(prefix).map(|i| {
        let start = i + prefix.len();
        let end = text[start..]
            .find('\n')
            .map(|j| start + j)
            .unwrap_or(text.len());
        &text[start..end]
    })
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_label_short_names() {
        assert_eq!(
            HumanLabel::HardwareInventory.short_name(),
            "hardware inventory"
        );
        assert_eq!(
            HumanLabel::NetworkSignals.short_name(),
            "network link and routing signals"
        );
        assert_eq!(
            HumanLabel::ErrorJournal.short_name(),
            "system error journal summary"
        );
    }

    #[test]
    fn test_evidence_summary_cpu() {
        let summary = EvidenceSummary::cpu("Intel i9-14900HX");
        assert_eq!(summary, "hardware inventory reports Intel i9-14900HX");
    }

    #[test]
    fn test_evidence_summary_disk() {
        let summary = EvidenceSummary::disk_free("/", 150.5, 500.0);
        assert!(summary.contains("150.5 GiB free"));
        assert!(summary.contains("/"));
    }

    #[test]
    fn test_evidence_summary_network() {
        let summary = EvidenceSummary::network(1, true, "link up");
        assert!(summary.contains("1 interface"));
        assert!(summary.contains("manager present"));
        assert!(summary.contains("link up"));
    }

    #[test]
    fn test_evidence_missing() {
        let summary = EvidenceSummary::missing("disk");
        assert!(summary.contains("no disk evidence available"));
    }

    #[test]
    fn test_from_legacy() {
        assert_eq!(
            HumanLabel::from_legacy("hardware inventory snapshot"),
            HumanLabel::HardwareInventory
        );
        assert_eq!(
            HumanLabel::from_legacy("network status snapshot"),
            HumanLabel::NetworkSignals
        );
    }
}
