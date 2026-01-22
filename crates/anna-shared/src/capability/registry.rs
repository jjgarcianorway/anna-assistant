//! Capability Registry - Static, canonical capability definitions.
//!
//! No dynamic registration. No inference. Unknown capabilities are rejected.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Unique identifier for a capability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

impl CapabilityId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Capability execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityMode {
    /// Can execute: gathers facts, produces output, no system mutation.
    ReadOnly,
    /// Cannot execute yet: produces plan proposal, abstains from execution.
    Mutating,
}

impl CapabilityMode {
    /// Whether this mode allows execution.
    pub fn can_execute(&self) -> bool {
        matches!(self, CapabilityMode::ReadOnly)
    }

    /// Whether this mode requires execution gate approval (always blocked for now).
    pub fn requires_gate(&self) -> bool {
        matches!(self, CapabilityMode::Mutating)
    }
}

/// A registered capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Unique identifier.
    pub id: CapabilityId,
    /// One-sentence description.
    pub description: String,
    /// Execution mode.
    pub mode: CapabilityMode,
    /// Warning categories this capability is relevant to.
    pub relevant_warnings: Vec<WarningCategory>,
    /// Low-risk operations skip confirmation (reversible, no data loss).
    pub low_risk: bool,
}

/// Categories of warnings for noise containment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningCategory {
    /// Status, identity, permission-related.
    StatusIdentity,
    /// Display, graphics, UI-related.
    Display,
    /// Network, connectivity-related.
    Network,
    /// Storage, disk-related.
    Storage,
    /// Service, systemd-related.
    Service,
    /// Package, update-related.
    Package,
    /// Security-related.
    Security,
    /// All warnings (for meta capabilities like status).
    All,
}

/// Static capability registry.
pub struct CapabilityRegistry {
    capabilities: HashMap<CapabilityId, Capability>,
}

impl CapabilityRegistry {
    /// Look up a capability by ID.
    pub fn get(&self, id: &CapabilityId) -> Option<&Capability> {
        self.capabilities.get(id)
    }

    /// Check if a capability exists.
    pub fn exists(&self, id: &CapabilityId) -> bool {
        self.capabilities.contains_key(id)
    }

    /// List all registered capabilities.
    pub fn list(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.values()
    }

    /// Count of registered capabilities.
    pub fn count(&self) -> usize {
        self.capabilities.len()
    }
}

/// The canonical capability registry. Static, immutable, no dynamic registration.
pub static CAPABILITY_REGISTRY: LazyLock<CapabilityRegistry> = LazyLock::new(|| {
    let mut capabilities = HashMap::new();

    // =========================================================================
    // STATUS CAPABILITIES (ReadOnly)
    // =========================================================================

    capabilities.insert(
        CapabilityId::new("status.system"),
        Capability {
            id: CapabilityId::new("status.system"),
            description: "Report overall system status including warnings and baseline state.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::All],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("status.disk"),
        Capability {
            id: CapabilityId::new("status.disk"),
            description: "Report disk usage and storage status.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::Storage],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("status.memory"),
        Capability {
            id: CapabilityId::new("status.memory"),
            description: "Report memory and swap usage.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::StatusIdentity],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("status.network"),
        Capability {
            id: CapabilityId::new("status.network"),
            description: "Report network connectivity and interface status.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::Network],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("status.services"),
        Capability {
            id: CapabilityId::new("status.services"),
            description: "Report systemd service status and failures.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::Service],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("status.identity"),
        Capability {
            id: CapabilityId::new("status.identity"),
            description: "Report user identity, groups, and permission state.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::StatusIdentity, WarningCategory::Security],
            low_risk: false,
        },
    );

    // =========================================================================
    // DISPLAY CAPABILITIES (ReadOnly for now)
    // =========================================================================

    // Phase 31: GDM scaling is MUTATING but LOW RISK - just copies a config file
    capabilities.insert(
        CapabilityId::new("display.scale.gdm"),
        Capability {
            id: CapabilityId::new("display.scale.gdm"),
            description: "Configure GDM login screen scaling by propagating monitors.xml.".to_string(),
            mode: CapabilityMode::Mutating,
            relevant_warnings: vec![WarningCategory::Display],
            low_risk: true,  // Reversible, no data loss
        },
    );

    capabilities.insert(
        CapabilityId::new("display.scale.xorg"),
        Capability {
            id: CapabilityId::new("display.scale.xorg"),
            description: "Analyze Xorg display scaling configuration.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::Display],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("display.scale.wayland"),
        Capability {
            id: CapabilityId::new("display.scale.wayland"),
            description: "Analyze Wayland display scaling configuration.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::Display],
            low_risk: false,
        },
    );

    // =========================================================================
    // PACKAGE CAPABILITIES (Mutating - blocked)
    // =========================================================================

    capabilities.insert(
        CapabilityId::new("package.install"),
        Capability {
            id: CapabilityId::new("package.install"),
            description: "Install packages via pacman (execution blocked).".to_string(),
            mode: CapabilityMode::Mutating,
            relevant_warnings: vec![WarningCategory::Package],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("package.remove"),
        Capability {
            id: CapabilityId::new("package.remove"),
            description: "Remove packages via pacman (execution blocked).".to_string(),
            mode: CapabilityMode::Mutating,
            relevant_warnings: vec![WarningCategory::Package],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("package.update"),
        Capability {
            id: CapabilityId::new("package.update"),
            description: "Update system packages via pacman (execution blocked).".to_string(),
            mode: CapabilityMode::Mutating,
            relevant_warnings: vec![WarningCategory::Package],
            low_risk: false,
        },
    );

    // =========================================================================
    // SERVICE CAPABILITIES (Mutating - blocked)
    // =========================================================================

    capabilities.insert(
        CapabilityId::new("service.start"),
        Capability {
            id: CapabilityId::new("service.start"),
            description: "Start a systemd service (execution blocked).".to_string(),
            mode: CapabilityMode::Mutating,
            relevant_warnings: vec![WarningCategory::Service],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("service.stop"),
        Capability {
            id: CapabilityId::new("service.stop"),
            description: "Stop a systemd service (execution blocked).".to_string(),
            mode: CapabilityMode::Mutating,
            relevant_warnings: vec![WarningCategory::Service],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("service.restart"),
        Capability {
            id: CapabilityId::new("service.restart"),
            description: "Restart a systemd service (execution blocked).".to_string(),
            mode: CapabilityMode::Mutating,
            relevant_warnings: vec![WarningCategory::Service],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("service.enable"),
        Capability {
            id: CapabilityId::new("service.enable"),
            description: "Enable a systemd service (execution blocked).".to_string(),
            mode: CapabilityMode::Mutating,
            relevant_warnings: vec![WarningCategory::Service],
            low_risk: false,
        },
    );

    // =========================================================================
    // CONFIG CAPABILITIES (Mutating - blocked)
    // =========================================================================

    capabilities.insert(
        CapabilityId::new("config.edit"),
        Capability {
            id: CapabilityId::new("config.edit"),
            description: "Edit a configuration file (execution blocked).".to_string(),
            mode: CapabilityMode::Mutating,
            relevant_warnings: vec![WarningCategory::StatusIdentity, WarningCategory::Security],
            low_risk: false,
        },
    );

    // =========================================================================
    // CONFIG REVIEW CAPABILITIES (ReadOnly)
    // =========================================================================

    capabilities.insert(
        CapabilityId::new("config.review.group_change"),
        Capability {
            id: CapabilityId::new("config.review.group_change"),
            description: "Review changes to /etc/group and provide restore instructions.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::StatusIdentity, WarningCategory::Security],
            low_risk: false,
        },
    );

    capabilities.insert(
        CapabilityId::new("config.review.passwd_change"),
        Capability {
            id: CapabilityId::new("config.review.passwd_change"),
            description: "Review changes to /etc/passwd and explain what changed.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::StatusIdentity, WarningCategory::Security],
            low_risk: false,
        },
    );

    // =========================================================================
    // POWER CAPABILITIES (Phase 33) - LOW RISK, reversible config changes
    // =========================================================================

    capabilities.insert(
        CapabilityId::new("power.inhibit.sleep"),
        Capability {
            id: CapabilityId::new("power.inhibit.sleep"),
            description: "Configure lid close, idle, and suspend key behavior.".to_string(),
            mode: CapabilityMode::Mutating,
            relevant_warnings: vec![WarningCategory::StatusIdentity],
            low_risk: true,  // Reversible, no data loss
        },
    );

    // =========================================================================
    // SYSTEM CAPABILITIES (Phase 33)
    // =========================================================================

    capabilities.insert(
        CapabilityId::new("system.thermal.status"),
        Capability {
            id: CapabilityId::new("system.thermal.status"),
            description: "Report CPU/GPU temperatures and fan speeds.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::StatusIdentity],
            low_risk: false,
        },
    );

    // =========================================================================
    // AUDIO CAPABILITIES (Phase 33)
    // =========================================================================

    capabilities.insert(
        CapabilityId::new("audio.stack.detect"),
        Capability {
            id: CapabilityId::new("audio.stack.detect"),
            description: "Detect PipeWire vs PulseAudio and audio configuration.".to_string(),
            mode: CapabilityMode::ReadOnly,
            relevant_warnings: vec![WarningCategory::StatusIdentity],
            low_risk: false,
        },
    );

    CapabilityRegistry { capabilities }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_is_static() {
        // Registry exists and has capabilities
        assert!(CAPABILITY_REGISTRY.count() > 0);
    }

    #[test]
    fn test_capability_lookup() {
        let id = CapabilityId::new("display.scale.gdm");
        let cap = CAPABILITY_REGISTRY.get(&id);
        assert!(cap.is_some());
        // Phase 31: display.scale.gdm is now Mutating (changes system files)
        assert_eq!(cap.unwrap().mode, CapabilityMode::Mutating);
    }

    #[test]
    fn test_unknown_capability_returns_none() {
        let id = CapabilityId::new("nonexistent.capability");
        assert!(CAPABILITY_REGISTRY.get(&id).is_none());
    }

    #[test]
    fn test_mutating_requires_gate() {
        let id = CapabilityId::new("package.install");
        let cap = CAPABILITY_REGISTRY.get(&id).unwrap();
        assert_eq!(cap.mode, CapabilityMode::Mutating);
        assert!(cap.mode.requires_gate());
        assert!(!cap.mode.can_execute());
    }

    #[test]
    fn test_readonly_can_execute() {
        let id = CapabilityId::new("status.system");
        let cap = CAPABILITY_REGISTRY.get(&id).unwrap();
        assert_eq!(cap.mode, CapabilityMode::ReadOnly);
        assert!(cap.mode.can_execute());
        assert!(!cap.mode.requires_gate());
    }
}
