//! State machine types for Anna 1.0
//!
//! Six explicit states determine available commands and behavior.
//!
//! Citation: [archwiki:installation_guide]

use serde::{Deserialize, Serialize};
use std::fmt;

/// System state determines available commands
///
/// States are mutually exclusive and detected in order of precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemState {
    /// Running from Arch ISO - no managed state exists
    /// Detection: /run/archiso exists
    /// Citation: [archwiki:installation_guide:boot_the_live_environment]
    IsoLive,

    /// Installed Linux found but unhealthy or unbootable
    /// Detection: Mountable Linux root with /etc/os-release but not current boot
    /// Citation: [archwiki:chroot]
    RecoveryCandidate,

    /// Fresh Arch with base only - no Anna state
    /// Detection: /etc/arch-release exists but no /etc/anna/state.json
    /// Citation: [archwiki:installation_guide:configure_the_system]
    PostInstallMinimal,

    /// Managed host with valid state file and health checks passing
    /// Detection: /etc/anna/state.json exists, readable, valid JSON, health OK
    /// Citation: [archwiki:system_maintenance]
    Configured,

    /// Managed host with failing health checks
    /// Detection: /etc/anna/state.json exists but health check fails
    /// Citation: [archwiki:system_maintenance#check_for_errors]
    Degraded,

    /// Unable to determine state - discovery mode only
    /// Detection: None of the above matched
    Unknown,
}

impl SystemState {
    /// Get Wiki citation for this state's detection logic
    pub fn citation(&self) -> &'static str {
        match self {
            SystemState::IsoLive => "[archwiki:installation_guide:boot_the_live_environment]",
            SystemState::RecoveryCandidate => "[archwiki:chroot]",
            SystemState::PostInstallMinimal => "[archwiki:installation_guide:configure_the_system]",
            SystemState::Configured => "[archwiki:system_maintenance]",
            SystemState::Degraded => "[archwiki:system_maintenance#check_for_errors]",
            SystemState::Unknown => "[archwiki:installation_guide]",
        }
    }

    /// Human-readable description of this state
    pub fn description(&self) -> &'static str {
        match self {
            SystemState::IsoLive => "Running from Arch ISO - installation available",
            SystemState::RecoveryCandidate => "Broken system detected - rescue tools available",
            SystemState::PostInstallMinimal => "Fresh Arch installation - setup required",
            SystemState::Configured => "Managed system - all operations available",
            SystemState::Degraded => "System health issues detected - recovery recommended",
            SystemState::Unknown => "Unable to determine system state - limited operations",
        }
    }
}

impl fmt::Display for SystemState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemState::IsoLive => write!(f, "iso_live"),
            SystemState::RecoveryCandidate => write!(f, "recovery_candidate"),
            SystemState::PostInstallMinimal => write!(f, "post_install_minimal"),
            SystemState::Configured => write!(f, "configured"),
            SystemState::Degraded => write!(f, "degraded"),
            SystemState::Unknown => write!(f, "unknown"),
        }
    }
}

/// Detection result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDetection {
    /// Detected state
    pub state: SystemState,

    /// When detection occurred
    pub detected_at: String,

    /// Additional detection metadata
    pub details: StateDetails,

    /// Wiki citation for detection logic
    pub citation: String,
}

/// Additional metadata about detected state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDetails {
    /// Running under UEFI (vs BIOS)
    pub uefi: bool,

    /// Detected block devices
    pub disks: Vec<String>,

    /// Network connectivity status
    pub network: NetworkStatus,

    /// Anna state file present
    pub state_file_present: bool,

    /// Health check passed (if applicable)
    pub health_ok: Option<bool>,
}

/// Network connectivity metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Has active network interface
    pub has_interface: bool,

    /// Has default route
    pub has_route: bool,

    /// Can resolve DNS
    pub can_resolve: bool,
}

impl Default for NetworkStatus {
    fn default() -> Self {
        NetworkStatus {
            has_interface: false,
            has_route: false,
            can_resolve: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_citations() {
        assert_eq!(
            SystemState::IsoLive.citation(),
            "[archwiki:installation_guide:boot_the_live_environment]"
        );
        assert_eq!(
            SystemState::Configured.citation(),
            "[archwiki:system_maintenance]"
        );
    }

    #[test]
    fn test_state_display() {
        assert_eq!(SystemState::IsoLive.to_string(), "iso_live");
        assert_eq!(SystemState::Configured.to_string(), "configured");
    }

    #[test]
    fn test_state_serialization() {
        let state = SystemState::Configured;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"configured\"");

        let deserialized: SystemState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SystemState::Configured);
    }
}
