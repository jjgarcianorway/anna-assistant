//! Domain classification for specialist routing.
//!
//! Maps IntentSubjects to specialist domains for deterministic dispatch.

use crate::translator::intent::IntentSubject;
use serde::{Deserialize, Serialize};

/// Domains group related IntentSubjects for specialist routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    /// System administration: services, processes, memory, CPU
    System,
    /// Storage: disks, partitions, mounts
    Storage,
    /// Network: connectivity, DNS, routing
    Network,
    /// Package management: install, update, remove
    Packages,
    /// Desktop: editors, shells, window managers
    Desktop,
    /// Hardware: GPU, drivers, peripherals
    Hardware,
    /// Audio: sound, pipewire, pulseaudio
    Audio,
    /// Security: permissions, SSH, encryption
    Security,
    /// General fallback
    General,
}

impl Domain {
    /// Map IntentSubject to Domain.
    pub fn from_subject(subject: &IntentSubject) -> Self {
        match subject {
            // System domain
            IntentSubject::CpuUsage
            | IntentSubject::MemoryUsage
            | IntentSubject::ProcessInfo
            | IntentSubject::ServiceStatus
            | IntentSubject::SystemInfo => Domain::System,

            // Storage domain
            IntentSubject::DiskUsage => Domain::Storage,

            // Network domain
            IntentSubject::NetworkStatus | IntentSubject::NetworkConfig => Domain::Network,

            // Package domain
            IntentSubject::PackageInfo
            | IntentSubject::PackageInstall
            | IntentSubject::PackageRemove
            | IntentSubject::PackageSearch
            | IntentSubject::PackageUpdate => Domain::Packages,

            // Configure subjects route by context
            IntentSubject::ServiceControl => Domain::System,
            IntentSubject::FileEdit => Domain::Desktop,
            IntentSubject::PermissionChange => Domain::Security,

            // Troubleshoot subjects
            IntentSubject::ErrorDiagnosis => Domain::System,
            IntentSubject::PerformanceIssue => Domain::System,
            IntentSubject::ConnectivityIssue => Domain::Network,
            IntentSubject::BootIssue => Domain::System,

            // Help subjects default to General
            IntentSubject::HowTo | IntentSubject::Explanation | IntentSubject::ManPage => {
                Domain::General
            }

            // Generic subjects route via keyword detection
            IntentSubject::Generic(_) => Domain::General,
        }
    }

    /// Map department name to Domain.
    pub fn from_department(dept: &str) -> Self {
        match dept.to_lowercase().as_str() {
            "system" => Domain::System,
            "storage" => Domain::Storage,
            "network" => Domain::Network,
            "packages" => Domain::Packages,
            "desktop" => Domain::Desktop,
            "hardware" => Domain::Hardware,
            "audio" => Domain::Audio,
            "security" => Domain::Security,
            _ => Domain::General,
        }
    }

    /// Get department name for display.
    pub fn department_name(&self) -> &'static str {
        match self {
            Domain::System => "System",
            Domain::Storage => "Storage",
            Domain::Network => "Network",
            Domain::Packages => "Packages",
            Domain::Desktop => "Desktop",
            Domain::Hardware => "Hardware",
            Domain::Audio => "Audio",
            Domain::Security => "Security",
            Domain::General => "General",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_from_subject_system() {
        assert_eq!(Domain::from_subject(&IntentSubject::CpuUsage), Domain::System);
        assert_eq!(Domain::from_subject(&IntentSubject::MemoryUsage), Domain::System);
        assert_eq!(Domain::from_subject(&IntentSubject::ServiceStatus), Domain::System);
    }

    #[test]
    fn test_domain_from_subject_storage() {
        assert_eq!(Domain::from_subject(&IntentSubject::DiskUsage), Domain::Storage);
    }

    #[test]
    fn test_domain_from_subject_network() {
        assert_eq!(Domain::from_subject(&IntentSubject::NetworkStatus), Domain::Network);
        assert_eq!(Domain::from_subject(&IntentSubject::ConnectivityIssue), Domain::Network);
    }

    #[test]
    fn test_domain_from_subject_packages() {
        assert_eq!(Domain::from_subject(&IntentSubject::PackageInstall), Domain::Packages);
        assert_eq!(Domain::from_subject(&IntentSubject::PackageRemove), Domain::Packages);
    }

    #[test]
    fn test_domain_from_department() {
        assert_eq!(Domain::from_department("System"), Domain::System);
        assert_eq!(Domain::from_department("NETWORK"), Domain::Network);
        assert_eq!(Domain::from_department("unknown"), Domain::General);
    }

    #[test]
    fn test_department_name() {
        assert_eq!(Domain::System.department_name(), "System");
        assert_eq!(Domain::Network.department_name(), "Network");
    }
}
