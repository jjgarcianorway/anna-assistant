//! Evidence types for tool, package, and audio parsing (v0.0.173).

use serde::{Deserialize, Serialize};

/// Method used to check tool existence (v0.45.7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolExistsMethod {
    /// `command -v <name>` (POSIX)
    CommandV,
    /// `which <name>` (less portable)
    Which,
    /// `type <name>` (bash builtin)
    Type,
}

/// Tool existence evidence (v0.45.7)
/// Note: exit code 1 is VALID NEGATIVE EVIDENCE, not an error!
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolExists {
    /// Name of the tool/binary
    pub name: String,
    /// Whether the tool exists (false = valid negative evidence)
    pub exists: bool,
    /// Method used to check
    pub method: ToolExistsMethod,
    /// Path if found (from stdout)
    pub path: Option<String>,
}

/// Package installation evidence (v0.45.7)
/// Note: exit code 1 is VALID NEGATIVE EVIDENCE, not an error!
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageInstalled {
    /// Name of the package
    pub name: String,
    /// Whether the package is installed (false = valid negative evidence)
    pub installed: bool,
    /// Version if installed
    pub version: Option<String>,
}

/// Audio device from lspci or pactl (v0.45.8)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Device description (e.g., "Intel Corporation Cannon Lake PCH cAVS")
    pub description: String,
    /// PCI slot if from lspci (e.g., "00:1f.3")
    pub pci_slot: Option<String>,
    /// Vendor name extracted from description
    pub vendor: Option<String>,
}

/// Audio devices evidence (v0.45.8)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioDevices {
    /// List of detected audio devices
    pub devices: Vec<AudioDevice>,
    /// Source of the information (lspci, pactl)
    pub source: String,
}
