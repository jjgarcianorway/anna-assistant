//! Side Effect Predictor types and enums.

use serde::{Deserialize, Serialize};

/// A predicted side effect of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    /// What type of side effect
    pub effect_type: SideEffectType,
    /// Targets (files, services, packages, etc.)
    pub targets: Vec<String>,
    /// Confidence in this prediction (0.0-1.0)
    pub confidence: f32,
    /// Is this reversible?
    pub reversible: bool,
    /// Description
    pub description: String,
}

/// Types of side effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideEffectType {
    /// File creation
    FileCreate,
    /// File modification
    FileModify,
    /// File deletion
    FileDelete,
    /// Directory creation
    DirCreate,
    /// Directory deletion
    DirDelete,
    /// Permission change
    PermissionChange,
    /// Package installation
    PackageInstall,
    /// Package removal
    PackageRemove,
    /// Package upgrade
    PackageUpgrade,
    /// Service start
    ServiceStart,
    /// Service stop
    ServiceStop,
    /// Service restart
    ServiceRestart,
    /// Service enable
    ServiceEnable,
    /// Service disable
    ServiceDisable,
    /// Network change
    NetworkChange,
    /// Firewall rule
    FirewallRule,
    /// Mount operation
    MountOperation,
    /// User/group change
    UserChange,
    /// Kernel module
    KernelModule,
    /// System reboot
    SystemReboot,
    /// Unknown
    Unknown,
}

impl SideEffectType {
    /// Get risk level (0.0-1.0)
    pub fn risk_level(&self) -> f32 {
        match self {
            SideEffectType::FileCreate => 0.1,
            SideEffectType::FileModify => 0.3,
            SideEffectType::FileDelete => 0.5,
            SideEffectType::DirCreate => 0.1,
            SideEffectType::DirDelete => 0.6,
            SideEffectType::PermissionChange => 0.4,
            SideEffectType::PackageInstall => 0.3,
            SideEffectType::PackageRemove => 0.5,
            SideEffectType::PackageUpgrade => 0.4,
            SideEffectType::ServiceStart => 0.2,
            SideEffectType::ServiceStop => 0.4,
            SideEffectType::ServiceRestart => 0.3,
            SideEffectType::ServiceEnable => 0.2,
            SideEffectType::ServiceDisable => 0.4,
            SideEffectType::NetworkChange => 0.5,
            SideEffectType::FirewallRule => 0.5,
            SideEffectType::MountOperation => 0.6,
            SideEffectType::UserChange => 0.5,
            SideEffectType::KernelModule => 0.7,
            SideEffectType::SystemReboot => 0.8,
            SideEffectType::Unknown => 0.5,
        }
    }

    /// Is this type generally reversible?
    pub fn is_reversible(&self) -> bool {
        match self {
            SideEffectType::FileCreate => true,
            SideEffectType::FileModify => false, // Without backup
            SideEffectType::FileDelete => false, // Without backup
            SideEffectType::DirCreate => true,
            SideEffectType::DirDelete => false,
            SideEffectType::PermissionChange => true,
            SideEffectType::PackageInstall => true,
            SideEffectType::PackageRemove => true,
            SideEffectType::PackageUpgrade => false, // Downgrade is complex
            SideEffectType::ServiceStart => true,
            SideEffectType::ServiceStop => true,
            SideEffectType::ServiceRestart => true,
            SideEffectType::ServiceEnable => true,
            SideEffectType::ServiceDisable => true,
            SideEffectType::NetworkChange => true,
            SideEffectType::FirewallRule => true,
            SideEffectType::MountOperation => true,
            SideEffectType::UserChange => true,
            SideEffectType::KernelModule => true,
            SideEffectType::SystemReboot => false,
            SideEffectType::Unknown => false,
        }
    }
}
