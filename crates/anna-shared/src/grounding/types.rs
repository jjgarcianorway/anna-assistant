//! Types for claim verification against evidence (v0.0.296).
//! v0.0.296: Added summary() and has_kind() for self-healing validation.

use crate::claims::Claim;
use crate::parsers::{DiskUsage, MemoryInfo, ParsedProbeData, ServiceStatus};
use crate::trace::EvidenceKind;
use serde::{Deserialize, Serialize};

/// Grounding verification report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundingReport {
    /// Total number of auditable claims extracted
    pub total_claims: u32,
    /// Number of claims verified against evidence
    pub verified_claims: u32,
    /// Grounding ratio: verified / total (0.0 when total == 0)
    pub grounding_ratio: f32,
    /// Individual claim verification results
    pub details: Vec<ClaimVerification>,
}

/// Result of verifying a single claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimVerification {
    /// The claim that was checked
    pub claim: Claim,
    /// Whether the claim was verified
    pub verified: bool,
    /// Reason for verification result
    pub reason: VerificationReason,
}

/// Why a claim was or wasn't verified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationReason {
    /// Claim matches evidence exactly
    ExactMatch,
    /// Evidence value differs from claim
    Mismatch { expected: String, actual: String },
    /// No evidence available for this claim key
    NoEvidence,
}

/// Collected evidence from parsed probe outputs.
#[derive(Debug, Clone, Default)]
pub struct ParsedEvidence {
    /// Memory info from free -h
    pub memory: Option<MemoryInfo>,
    /// Disk usage entries from df -h
    pub disks: Vec<DiskUsage>,
    /// Service statuses from systemctl
    pub services: Vec<ServiceStatus>,
}

impl ParsedEvidence {
    /// Build evidence from a list of parsed probe data.
    pub fn from_probes(probes: &[ParsedProbeData]) -> Self {
        let mut evidence = Self::default();

        for probe in probes {
            match probe {
                ParsedProbeData::Memory(m) => {
                    evidence.memory = Some(m.clone());
                }
                ParsedProbeData::Disk(d) => {
                    evidence.disks.extend(d.iter().cloned());
                }
                ParsedProbeData::Services(s) => {
                    evidence.services.extend(s.iter().cloned());
                }
                ParsedProbeData::Service(s) => {
                    evidence.services.push(s.clone());
                }
                ParsedProbeData::BlockDevices(_)
                | ParsedProbeData::Cpu(_)
                | ParsedProbeData::JournalErrors(_)
                | ParsedProbeData::JournalWarnings(_)
                | ParsedProbeData::BootTime(_)
                | ParsedProbeData::Tool(_)
                | ParsedProbeData::Package(_)
                | ParsedProbeData::Audio(_)
                | ParsedProbeData::Error(_)
                | ParsedProbeData::Unsupported => {}
            }
        }

        evidence
    }

    /// v0.0.296: Check if evidence contains a specific kind
    pub fn has_kind(&self, kind: &EvidenceKind) -> bool {
        match kind {
            EvidenceKind::Memory => self.memory.is_some(),
            EvidenceKind::Disk => !self.disks.is_empty(),
            EvidenceKind::Services => !self.services.is_empty(),
            // TODO: extend ParsedEvidence to include these
            _ => false,
        }
    }

    /// v0.0.296: Generate a human-readable summary for LLM prompts
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(mem) = &self.memory {
            let total_gb = mem.total_bytes as f64 / 1_073_741_824.0;
            let used_gb = mem.used_bytes as f64 / 1_073_741_824.0;
            let avail_gb = mem.available_bytes as f64 / 1_073_741_824.0;
            parts.push(format!(
                "Memory: {:.1}GB total, {:.1}GB used, {:.1}GB available",
                total_gb, used_gb, avail_gb
            ));
        }

        if !self.disks.is_empty() {
            for disk in &self.disks {
                let used_gb = disk.used_bytes as f64 / 1_073_741_824.0;
                let total_gb = disk.size_bytes as f64 / 1_073_741_824.0;
                parts.push(format!(
                    "Disk {}: {}% used ({:.1}GB of {:.1}GB)",
                    disk.mount, disk.percent_used, used_gb, total_gb
                ));
            }
        }

        if !self.services.is_empty() {
            for svc in &self.services {
                parts.push(format!("Service {}: {}", svc.name, svc.state));
            }
        }

        if parts.is_empty() {
            "No evidence collected.".to_string()
        } else {
            parts.join("\n")
        }
    }
}
