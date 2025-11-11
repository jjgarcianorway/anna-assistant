//! Installation types and data structures
//!
//! Phase 0.8: System Installer types
//! Citation: [archwiki:Installation_guide]

use serde::{Deserialize, Serialize};
use std::fmt;

/// Installation environment state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallationState {
    /// Running from Arch ISO (live environment)
    IsoLive,
    /// Base system installed but not configured
    PostInstallMinimal,
    /// Fully configured system
    Configured,
    /// Unknown state
    Unknown,
}

/// Disk setup mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiskSetupMode {
    /// Manual partitioning (user provides partition layout)
    Manual {
        root_partition: String,
        boot_partition: String,
        swap_partition: Option<String>,
    },
    /// Automatic with btrfs subvolumes
    AutoBtrfs {
        target_disk: String,
        create_swap: bool,
        swap_size_gb: u32,
    },
}

/// Bootloader type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootloaderType {
    SystemdBoot,
    Grub,
}

impl fmt::Display for BootloaderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootloaderType::SystemdBoot => write!(f, "systemd-boot"),
            BootloaderType::Grub => write!(f, "GRUB"),
        }
    }
}

/// Installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Disk setup mode
    pub disk_setup: DiskSetupMode,

    /// Bootloader type
    pub bootloader: BootloaderType,

    /// System hostname
    pub hostname: String,

    /// Username to create
    pub username: String,

    /// Timezone (e.g., "America/New_York")
    pub timezone: String,

    /// Locale (e.g., "en_US.UTF-8")
    pub locale: String,

    /// Additional packages to install
    pub extra_packages: Vec<String>,

    /// Enable multilib repository
    pub enable_multilib: bool,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            disk_setup: DiskSetupMode::Manual {
                root_partition: String::new(),
                boot_partition: String::new(),
                swap_partition: None,
            },
            bootloader: BootloaderType::SystemdBoot,
            hostname: "archlinux".to_string(),
            username: "user".to_string(),
            timezone: "UTC".to_string(),
            locale: "en_US.UTF-8".to_string(),
            extra_packages: vec![],
            enable_multilib: false,
        }
    }
}

/// Result of a single installation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallStep {
    /// Step name (e.g., "disk_setup")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Whether step succeeded
    pub success: bool,

    /// Detailed output or error message
    pub details: String,

    /// Arch Wiki citation
    pub citation: String,
}

/// Overall installation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    /// Whether installation succeeded
    pub success: bool,

    /// Individual step results
    pub steps: Vec<InstallStep>,

    /// Summary message
    pub message: String,

    /// Arch Wiki citation
    pub citation: String,
}

/// Step execution result
pub type StepResult = anyhow::Result<String>;
