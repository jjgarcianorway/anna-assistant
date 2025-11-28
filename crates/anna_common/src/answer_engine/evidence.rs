//! Evidence model for Anna v0.14.0
//!
//! Every probe result is wrapped in structured evidence.
//! LLM-A and LLM-B see only this structured evidence, never raw shell access.
//!
//! v0.14.0: Aligned catalog to reality - only 6 probes that actually exist.

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

/// v0.14.0 Probe catalog - ONLY the 6 probes that actually exist
///
/// IMPORTANT: Do not add probes here that don't have matching JSON files
/// in the probes/ directory. The LLM prompts are aligned to these 6 probes.
#[derive(Debug, Clone)]
pub struct ProbeCatalog {
    probes: HashMap<String, ProbeDefinitionV10>,
}

impl ProbeCatalog {
    /// Create the standard probe catalog with the 6 REAL probes
    ///
    /// v0.14.0: Shrunk from 14 to 6 probes to match reality
    pub fn standard() -> Self {
        let mut probes = HashMap::new();

        // CPU info - lscpu style JSON
        probes.insert(
            "cpu.info".to_string(),
            ProbeDefinitionV10 {
                probe_id: "cpu.info".to_string(),
                description: "CPU information (model, threads, flags) from lscpu".to_string(),
                commands: vec!["lscpu -J".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Memory info - /proc/meminfo text
        probes.insert(
            "mem.info".to_string(),
            ProbeDefinitionV10 {
                probe_id: "mem.info".to_string(),
                description: "Memory usage from /proc/meminfo (RAM total/free in kB)".to_string(),
                commands: vec!["cat /proc/meminfo".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Disk/block devices - lsblk JSON
        probes.insert(
            "disk.lsblk".to_string(),
            ProbeDefinitionV10 {
                probe_id: "disk.lsblk".to_string(),
                description: "Block device information (partitions, sizes) from lsblk".to_string(),
                commands: vec!["lsblk -J -b -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // GPU detection
        probes.insert(
            "hardware.gpu".to_string(),
            ProbeDefinitionV10 {
                probe_id: "hardware.gpu".to_string(),
                description: "GPU presence and basic model/vendor detection".to_string(),
                commands: vec!["lspci -v | grep -i vga".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // GPU drivers
        probes.insert(
            "drivers.gpu".to_string(),
            ProbeDefinitionV10 {
                probe_id: "drivers.gpu".to_string(),
                description: "GPU driver stack summary".to_string(),
                commands: vec!["lsmod | grep -E 'nvidia|amdgpu|i915|nouveau'".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // RAM summary
        probes.insert(
            "hardware.ram".to_string(),
            ProbeDefinitionV10 {
                probe_id: "hardware.ram".to_string(),
                description: "High level RAM summary (total capacity, slot info)".to_string(),
                commands: vec!["dmidecode -t memory 2>/dev/null || cat /proc/meminfo".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // v0.74.0: Additional probes for canonical questions

        // System/OS info (Q1 full answer)
        probes.insert(
            "system.os".to_string(),
            ProbeDefinitionV10 {
                probe_id: "system.os".to_string(),
                description: "Operating system info (distro, kernel version)".to_string(),
                commands: vec!["cat /etc/os-release && uname -r".to_string()],
                cost: ProbeCost::Cheap,
            },
        );

        // Annad service logs (Q3)
        probes.insert(
            "logs.annad".to_string(),
            ProbeDefinitionV10 {
                probe_id: "logs.annad".to_string(),
                description: "Anna daemon logs from journalctl (last 6 hours)".to_string(),
                commands: vec!["journalctl -u annad --since '6 hours ago' --no-pager -n 200".to_string()],
                cost: ProbeCost::Medium,
            },
        );

        // Pending system updates (Q4)
        probes.insert(
            "updates.pending".to_string(),
            ProbeDefinitionV10 {
                probe_id: "updates.pending".to_string(),
                description: "List of pending package updates (pacman or apt)".to_string(),
                commands: vec!["pacman -Qu 2>/dev/null || apt list --upgradable 2>/dev/null || echo 'No package manager found'".to_string()],
                cost: ProbeCost::Medium,
            },
        );

        // Anna self-health (Q5)
        probes.insert(
            "anna.self_health".to_string(),
            ProbeDefinitionV10 {
                probe_id: "anna.self_health".to_string(),
                description: "Anna self-diagnostic (daemon, Ollama, models, permissions)".to_string(),
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

    /// Get number of probes in catalog
    pub fn len(&self) -> usize {
        self.probes.len()
    }

    /// Check if catalog is empty
    pub fn is_empty(&self) -> bool {
        self.probes.is_empty()
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
        // v0.74.0: 10 probes total (6 original + 4 for canonical questions)
        assert_eq!(catalog.len(), 10);

        // Original 6 probes
        assert!(catalog.is_valid("cpu.info"));
        assert!(catalog.is_valid("mem.info"));
        assert!(catalog.is_valid("disk.lsblk"));
        assert!(catalog.is_valid("hardware.gpu"));
        assert!(catalog.is_valid("drivers.gpu"));
        assert!(catalog.is_valid("hardware.ram"));

        // v0.74.0: 4 new probes for canonical questions
        assert!(catalog.is_valid("system.os"));
        assert!(catalog.is_valid("logs.annad"));
        assert!(catalog.is_valid("updates.pending"));
        assert!(catalog.is_valid("anna.self_health"));

        // These should NOT exist
        assert!(!catalog.is_valid("net.links"));
        assert!(!catalog.is_valid("net.addr"));
        assert!(!catalog.is_valid("pkg.games"));
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
        // v0.74.0: 10 probes (6 original + 4 for canonical questions)
        assert_eq!(probes.len(), 10);
        assert!(probes.iter().any(|p| p.probe_id == "cpu.info"));
        assert!(probes.iter().any(|p| p.probe_id == "hardware.gpu"));
        assert!(probes.iter().any(|p| p.probe_id == "system.os"));
        assert!(probes.iter().any(|p| p.probe_id == "logs.annad"));
    }
}
