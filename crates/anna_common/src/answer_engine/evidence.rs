//! Evidence model for Anna v0.10.0
//!
//! Every probe result is wrapped in structured evidence.
//! LLM-A and LLM-B see only this structured evidence, never raw shell access.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Probe cost estimation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeCost {
    /// Fast probe, can run frequently (< 100ms)
    Cheap,
    /// Moderate cost probe (100ms - 1s)
    Medium,
    /// Expensive probe, should be cached (> 1s)
    Expensive,
}

impl ProbeCost {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProbeCost::Cheap => "cheap",
            ProbeCost::Medium => "medium",
            ProbeCost::Expensive => "expensive",
        }
    }
}

/// Probe definition in the tool catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeDefinitionV10 {
    /// Unique probe identifier (e.g., "cpu.info", "mem.info")
    pub probe_id: String,
    /// Human-readable description
    pub description: String,
    /// Underlying command(s) executed
    pub commands: Vec<String>,
    /// Estimated cost
    pub cost: ProbeCost,
}

/// Evidence from a probe execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEvidenceV10 {
    /// Probe identifier
    pub probe_id: String,
    /// Execution timestamp (RFC 3339)
    pub timestamp: String,
    /// Execution status
    pub status: EvidenceStatus,
    /// Command that was executed
    pub command: String,
    /// Raw stdout/stderr snippet (truncated if large)
    #[serde(default)]
    pub raw: Option<String>,
    /// Parsed/simplified JSON data if applicable
    #[serde(default)]
    pub parsed: Option<serde_json::Value>,
}

/// Evidence execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceStatus {
    Ok,
    Error,
    Timeout,
    NotFound,
}

impl EvidenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvidenceStatus::Ok => "ok",
            EvidenceStatus::Error => "error",
            EvidenceStatus::Timeout => "timeout",
            EvidenceStatus::NotFound => "not_found",
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, EvidenceStatus::Ok)
    }
}

/// v0.10.0 Probe catalog - registered probes only
#[derive(Debug, Clone)]
pub struct ProbeCatalog {
    probes: HashMap<String, ProbeDefinitionV10>,
}

impl ProbeCatalog {
    /// Create the standard probe catalog
    pub fn standard() -> Self {
        let mut probes = HashMap::new();

        // CPU info
        probes.insert(
            "cpu.info".to_string(),
            ProbeDefinitionV10 {
                probe_id: "cpu.info".to_string(),
                description: "CPU information from /proc/cpuinfo and lscpu".to_string(),
                commands: vec!["lscpu -J".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Memory info
        probes.insert(
            "mem.info".to_string(),
            ProbeDefinitionV10 {
                probe_id: "mem.info".to_string(),
                description: "Memory usage from /proc/meminfo".to_string(),
                commands: vec!["cat /proc/meminfo".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Disk/block devices
        probes.insert(
            "disk.lsblk".to_string(),
            ProbeDefinitionV10 {
                probe_id: "disk.lsblk".to_string(),
                description: "Block device information from lsblk".to_string(),
                commands: vec!["lsblk -J -b -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Filesystem usage
        probes.insert(
            "fs.usage_root".to_string(),
            ProbeDefinitionV10 {
                probe_id: "fs.usage_root".to_string(),
                description: "Filesystem usage for root partition".to_string(),
                commands: vec!["df -h /".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Network links
        probes.insert(
            "net.links".to_string(),
            ProbeDefinitionV10 {
                probe_id: "net.links".to_string(),
                description: "Network interface link status".to_string(),
                commands: vec!["ip -j link show".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Network addresses
        probes.insert(
            "net.addr".to_string(),
            ProbeDefinitionV10 {
                probe_id: "net.addr".to_string(),
                description: "Network interface addresses".to_string(),
                commands: vec!["ip -j addr show".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Network routes
        probes.insert(
            "net.routes".to_string(),
            ProbeDefinitionV10 {
                probe_id: "net.routes".to_string(),
                description: "Network routing table".to_string(),
                commands: vec!["ip -j route show".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // DNS configuration
        probes.insert(
            "dns.resolv".to_string(),
            ProbeDefinitionV10 {
                probe_id: "dns.resolv".to_string(),
                description: "DNS resolver configuration".to_string(),
                commands: vec!["cat /etc/resolv.conf".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Pacman updates
        probes.insert(
            "pkg.pacman_updates".to_string(),
            ProbeDefinitionV10 {
                probe_id: "pkg.pacman_updates".to_string(),
                description: "Available pacman package updates".to_string(),
                commands: vec!["checkupdates".to_string()],
                cost: ProbeCost::Medium,
            },
        );

        // AUR updates (yay)
        probes.insert(
            "pkg.yay_updates".to_string(),
            ProbeDefinitionV10 {
                probe_id: "pkg.yay_updates".to_string(),
                description: "Available AUR package updates via yay".to_string(),
                commands: vec!["yay -Qua".to_string()],
                cost: ProbeCost::Medium,
            },
        );

        // Games/Steam packages
        probes.insert(
            "pkg.games".to_string(),
            ProbeDefinitionV10 {
                probe_id: "pkg.games".to_string(),
                description: "Installed game-related packages".to_string(),
                commands: vec![
                    "pacman -Qs steam".to_string(),
                    "pacman -Qs lutris".to_string(),
                    "pacman -Qs wine".to_string(),
                ],
                cost: ProbeCost::Medium,
            },
        );

        // System kernel
        probes.insert(
            "system.kernel".to_string(),
            ProbeDefinitionV10 {
                probe_id: "system.kernel".to_string(),
                description: "Kernel and system information".to_string(),
                commands: vec!["uname -a".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Journal slice (recent logs)
        probes.insert(
            "system.journal_slice".to_string(),
            ProbeDefinitionV10 {
                probe_id: "system.journal_slice".to_string(),
                description: "Recent system journal entries".to_string(),
                commands: vec!["journalctl -n 50 --no-pager".to_string()],
                cost: ProbeCost::Medium,
            },
        );

        // Anna self-health
        probes.insert(
            "anna.self_health".to_string(),
            ProbeDefinitionV10 {
                probe_id: "anna.self_health".to_string(),
                description: "Anna daemon self-health check".to_string(),
                commands: vec!["internal:self_health".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        Self { probes }
    }

    /// Get a probe definition by ID
    pub fn get(&self, probe_id: &str) -> Option<&ProbeDefinitionV10> {
        self.probes.get(probe_id)
    }

    /// Check if a probe ID is valid
    pub fn is_valid(&self, probe_id: &str) -> bool {
        self.probes.contains_key(probe_id)
    }

    /// List all available probes
    pub fn list(&self) -> Vec<&ProbeDefinitionV10> {
        self.probes.values().collect()
    }

    /// Get available probes as serializable format for LLM
    pub fn available_probes(&self) -> Vec<AvailableProbe> {
        self.probes
            .values()
            .map(|p| AvailableProbe {
                probe_id: p.probe_id.clone(),
                description: p.description.clone(),
                cost: p.cost,
            })
            .collect()
    }
}

/// Simplified probe info for LLM context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableProbe {
    pub probe_id: String,
    pub description: String,
    pub cost: ProbeCost,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_catalog_standard() {
        let catalog = ProbeCatalog::standard();
        assert!(catalog.is_valid("cpu.info"));
        assert!(catalog.is_valid("mem.info"));
        assert!(catalog.is_valid("net.links"));
        assert!(catalog.is_valid("anna.self_health"));
        assert!(!catalog.is_valid("nonexistent.probe"));
    }

    #[test]
    fn test_evidence_status() {
        assert!(EvidenceStatus::Ok.is_ok());
        assert!(!EvidenceStatus::Error.is_ok());
        assert_eq!(EvidenceStatus::Timeout.as_str(), "timeout");
    }

    #[test]
    fn test_probe_cost() {
        assert_eq!(ProbeCost::Cheap.as_str(), "cheap");
        assert_eq!(ProbeCost::Expensive.as_str(), "expensive");
    }

    #[test]
    fn test_available_probes() {
        let catalog = ProbeCatalog::standard();
        let probes = catalog.available_probes();
        assert!(!probes.is_empty());
        assert!(probes.iter().any(|p| p.probe_id == "cpu.info"));
    }
}
