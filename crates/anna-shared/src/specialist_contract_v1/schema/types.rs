//! Core enums for SRC v1.

use serde::{Deserialize, Serialize};

/// Department that handled the case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SrcDepartment {
    Performance,
    Storage,
    Services,
    Network,
    Security,
    Hardware,
    Desktop,
}

impl SrcDepartment {
    /// Parse from string.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "performance" | "perf" => Some(Self::Performance),
            "storage" | "disk" => Some(Self::Storage),
            "services" | "service" | "svc" => Some(Self::Services),
            "network" | "net" => Some(Self::Network),
            "security" | "sec" => Some(Self::Security),
            "hardware" | "hw" => Some(Self::Hardware),
            "desktop" | "de" => Some(Self::Desktop),
            _ => None,
        }
    }

    /// Label for serialization.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Performance => "Performance",
            Self::Storage => "Storage",
            Self::Services => "Services",
            Self::Network => "Network",
            Self::Security => "Security",
            Self::Hardware => "Hardware",
            Self::Desktop => "Desktop",
        }
    }
}

/// Risk level for proposed actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SrcRisk {
    /// Read-only operations (probes, status checks).
    ReadOnly,
    /// Safe changes (enabling a service, changing a setting).
    SafeChange,
    /// Risky changes (format, delete, kernel updates).
    RiskyChange,
}

impl SrcRisk {
    /// Label for serialization.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::SafeChange => "safe_change",
            Self::RiskyChange => "risky_change",
        }
    }
}

impl Default for SrcRisk {
    fn default() -> Self {
        Self::ReadOnly
    }
}

/// Action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SrcActionType {
    /// Run a probe for more information.
    Probe,
    /// Explain something to the user.
    Explain,
    /// Make a change to the system.
    Change,
}

/// Citation source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SrcCitationSource {
    /// Man page.
    Man,
    /// Arch Wiki.
    ArchWiki,
    /// --help output.
    Help,
    /// Local documentation.
    LocalDoc,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_src_department_parse() {
        assert_eq!(
            SrcDepartment::from_str_loose("Performance"),
            Some(SrcDepartment::Performance)
        );
        assert_eq!(
            SrcDepartment::from_str_loose("hardware"),
            Some(SrcDepartment::Hardware)
        );
        assert_eq!(SrcDepartment::from_str_loose("bogus"), None);
    }
}
