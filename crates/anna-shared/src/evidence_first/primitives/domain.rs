//! Domain and parser type definitions for probe categorization.

use serde::{Deserialize, Serialize};

/// Domain for probe categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    /// Boot and startup.
    Boot,
    /// Systemd services.
    Services,
    /// System logs.
    Logs,
    /// Memory.
    Memory,
    /// Disk and storage.
    Disk,
    /// Network.
    Network,
    /// Hardware.
    Hardware,
    /// Performance.
    Performance,
    /// Desktop environment.
    Desktop,
    /// Packages.
    Packages,
    /// Security.
    Security,
}

impl Domain {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Boot => "boot",
            Self::Services => "services",
            Self::Logs => "logs",
            Self::Memory => "memory",
            Self::Disk => "disk",
            Self::Network => "network",
            Self::Hardware => "hardware",
            Self::Performance => "performance",
            Self::Desktop => "desktop",
            Self::Packages => "packages",
            Self::Security => "security",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "boot" | "startup" => Some(Self::Boot),
            "services" | "systemd" => Some(Self::Services),
            "logs" | "journal" => Some(Self::Logs),
            "memory" | "ram" => Some(Self::Memory),
            "disk" | "storage" => Some(Self::Disk),
            "network" | "net" => Some(Self::Network),
            "hardware" | "hw" => Some(Self::Hardware),
            "performance" | "perf" => Some(Self::Performance),
            "desktop" | "gui" => Some(Self::Desktop),
            "packages" | "pkg" => Some(Self::Packages),
            "security" | "sec" => Some(Self::Security),
            _ => None,
        }
    }
}

/// Parser identifier for probe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParserId {
    /// Raw text, no parsing.
    Raw,
    /// Key-value pairs.
    KeyValue,
    /// Table format.
    Table,
    /// JSON output.
    Json,
    /// Time/duration values.
    TimeDuration,
    /// Numeric values.
    Numeric,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_labels() {
        assert_eq!(Domain::Boot.label(), "boot");
        assert_eq!(Domain::Services.label(), "services");
    }

    #[test]
    fn test_domain_from_str() {
        assert_eq!(Domain::from_str("boot"), Some(Domain::Boot));
        assert_eq!(Domain::from_str("services"), Some(Domain::Services));
        assert_eq!(Domain::from_str("systemd"), Some(Domain::Services));
        assert_eq!(Domain::from_str("unknown"), None);
    }
}
