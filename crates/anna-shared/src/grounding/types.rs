//! Types for claim verification against evidence (v0.0.195).

use crate::claims::Claim;
use crate::parsers::{DiskUsage, MemoryInfo, ParsedProbeData, ServiceStatus};
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
}
