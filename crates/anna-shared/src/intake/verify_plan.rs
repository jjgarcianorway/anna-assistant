//! Verification planning for clarification answers (v0.0.180).

use serde::{Deserialize, Serialize};

/// Plan for verifying a user's clarification answer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerifyPlan {
    /// No verification needed
    None,
    /// Verify binary exists: `command -v <binary>` or `which <binary>`
    BinaryExists { binary: String },
    /// Verify systemd unit exists
    UnitExists { unit: String },
    /// Verify mount point exists (check df/lsblk output)
    MountExists { mount: String },
    /// Verify network interface exists (check ip link output)
    InterfaceExists { iface: String },
    /// Verify file exists
    FileExists { path: String },
    /// Verify directory exists
    DirectoryExists { path: String },
    /// Verify from existing probe evidence (key in evidence map)
    FromEvidence { key: String },
}

impl VerifyPlan {
    /// Get the probe command for this verification plan
    pub fn probe_command(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::BinaryExists { binary } => Some(format!("command -v {}", binary)),
            Self::UnitExists { unit } => Some(format!("systemctl list-unit-files {}", unit)),
            Self::MountExists { .. } => Some("df -h".to_string()), // Parse output
            Self::InterfaceExists { .. } => Some("ip link show".to_string()), // Parse output
            Self::FileExists { path } => Some(format!("test -f {} && echo exists", path)),
            Self::DirectoryExists { path } => Some(format!("test -d {} && echo exists", path)),
            Self::FromEvidence { .. } => None, // Use existing evidence
        }
    }

    /// Check if this plan requires running a probe
    pub fn needs_probe(&self) -> bool {
        !matches!(self, Self::None | Self::FromEvidence { .. })
    }
}
