//! Type definitions for helper tracking

use serde::{Deserialize, Serialize};

/// Who installed the helper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstallerSource {
    Anna,
    User,
    System,
    Unknown,
}

impl InstallerSource {
    pub fn name(&self) -> &'static str {
        match self {
            InstallerSource::Anna => "Anna",
            InstallerSource::User => "User",
            InstallerSource::System => "System",
            InstallerSource::Unknown => "Unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            InstallerSource::Anna => "A",
            InstallerSource::User => "U",
            InstallerSource::System => "S",
            InstallerSource::Unknown => "?",
        }
    }
}

/// Purpose of the helper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HelperPurpose {
    SystemInfo,
    NetworkDiag,
    DiskUtil,
    ProcessMon,
    LogAnalysis,
    Security,
    Performance,
    Development,
    Multimedia,
    General,
}

impl HelperPurpose {
    pub fn name(&self) -> &'static str {
        match self {
            HelperPurpose::SystemInfo => "System Info",
            HelperPurpose::NetworkDiag => "Network Diagnostics",
            HelperPurpose::DiskUtil => "Disk Utilities",
            HelperPurpose::ProcessMon => "Process Monitoring",
            HelperPurpose::LogAnalysis => "Log Analysis",
            HelperPurpose::Security => "Security",
            HelperPurpose::Performance => "Performance",
            HelperPurpose::Development => "Development",
            HelperPurpose::Multimedia => "Multimedia",
            HelperPurpose::General => "General",
        }
    }
}

/// A helper (tool) record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperRecord {
    /// Helper name/command
    pub name: String,
    /// Package name (may differ from command)
    pub package_name: Option<String>,
    /// Who installed it
    pub installed_by: InstallerSource,
    /// Purpose category
    pub purpose: HelperPurpose,
    /// Description of what it does
    pub description: String,
    /// When it was first detected/installed
    pub installed_at: u64,
    /// Times Anna has used this helper
    pub usage_count: u64,
    /// Last time used
    pub last_used: Option<u64>,
    /// Whether it's currently available
    pub available: bool,
    /// Why Anna installed it (if Anna-installed)
    pub install_reason: Option<String>,
    /// Ticket ID that triggered installation
    pub ticket_id: Option<String>,
}
