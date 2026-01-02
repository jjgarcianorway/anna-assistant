//! Hardware capability types

use serde::{Deserialize, Serialize};

/// Hardware category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HardwareCategory {
    #[default]
    Network,
    Audio,
    Video,
    Storage,
    Input,
    Usb,
    Bluetooth,
    Wireless,
    Power,
    Cpu,
    Memory,
    Other,
}

impl HardwareCategory {
    pub fn name(&self) -> &'static str {
        match self {
            HardwareCategory::Network => "Network",
            HardwareCategory::Audio => "Audio",
            HardwareCategory::Video => "Video",
            HardwareCategory::Storage => "Storage",
            HardwareCategory::Input => "Input",
            HardwareCategory::Usb => "USB",
            HardwareCategory::Bluetooth => "Bluetooth",
            HardwareCategory::Wireless => "Wireless",
            HardwareCategory::Power => "Power",
            HardwareCategory::Cpu => "CPU",
            HardwareCategory::Memory => "Memory",
            HardwareCategory::Other => "Other",
        }
    }
}

/// Hardware status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HardwareStatus {
    #[default]
    Detected,
    NotDetected,
    Disabled,
    Error,
    Unknown,
}

impl HardwareStatus {
    pub fn name(&self) -> &'static str {
        match self {
            HardwareStatus::Detected => "Detected",
            HardwareStatus::NotDetected => "Not Detected",
            HardwareStatus::Disabled => "Disabled",
            HardwareStatus::Error => "Error",
            HardwareStatus::Unknown => "Unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            HardwareStatus::Detected => "✓",
            HardwareStatus::NotDetected => "-",
            HardwareStatus::Disabled => "x",
            HardwareStatus::Error => "!",
            HardwareStatus::Unknown => "?",
        }
    }
}
