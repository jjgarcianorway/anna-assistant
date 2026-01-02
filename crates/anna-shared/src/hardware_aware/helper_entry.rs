//! Helper tool catalog and definitions (v0.0.434).
//!
//! Defines helper tools like lm_sensors, smartmontools, etc. and the catalog of available helpers.

use serde::{Deserialize, Serialize};

/// A helper tool entry in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperEntry {
    /// Helper identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Purpose/description.
    pub purpose: String,
    /// Package names for Arch Linux.
    pub packages_arch: Vec<String>,
    /// Package names for Debian/Ubuntu.
    pub packages_debian: Vec<String>,
    /// Package names for Fedora/RHEL.
    pub packages_fedora: Vec<String>,
    /// Command to check if installed.
    pub check_command: String,
    /// Probes that benefit from this helper.
    pub benefits_probes: Vec<String>,
}

impl HelperEntry {
    /// Get packages for the current distro.
    pub fn packages_for_distro(&self, distro: &str) -> &[String] {
        let distro_lower = distro.to_lowercase();
        if distro_lower.contains("arch") {
            &self.packages_arch
        } else if distro_lower.contains("debian")
            || distro_lower.contains("ubuntu")
            || distro_lower.contains("mint")
        {
            &self.packages_debian
        } else if distro_lower.contains("fedora")
            || distro_lower.contains("rhel")
            || distro_lower.contains("centos")
        {
            &self.packages_fedora
        } else {
            &self.packages_arch // Default fallback
        }
    }

    /// Check if helper is installed.
    pub fn is_installed(&self) -> bool {
        use std::process::Command;

        // Try the check command
        let parts: Vec<&str> = self.check_command.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        Command::new(parts[0])
            .args(&parts[1..])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// The helper catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperCatalog {
    /// Catalog version.
    pub version: u32,
    /// Available helpers.
    pub helpers: Vec<HelperEntry>,
}

impl HelperCatalog {
    /// Create the default catalog.
    pub fn default_catalog() -> Self {
        Self {
            version: 1,
            helpers: vec![
                HelperEntry {
                    id: "lm_sensors".to_string(),
                    name: "lm-sensors".to_string(),
                    purpose: "Read CPU and system temperatures".to_string(),
                    packages_arch: vec!["lm_sensors".to_string()],
                    packages_debian: vec!["lm-sensors".to_string()],
                    packages_fedora: vec!["lm_sensors".to_string()],
                    check_command: "sensors --version".to_string(),
                    benefits_probes: vec!["temperature".to_string(), "cpu_temp".to_string()],
                },
                HelperEntry {
                    id: "smartmontools".to_string(),
                    name: "smartmontools".to_string(),
                    purpose: "Disk health monitoring (S.M.A.R.T.)".to_string(),
                    packages_arch: vec!["smartmontools".to_string()],
                    packages_debian: vec!["smartmontools".to_string()],
                    packages_fedora: vec!["smartmontools".to_string()],
                    check_command: "smartctl --version".to_string(),
                    benefits_probes: vec!["disk_health".to_string(), "smart".to_string()],
                },
                HelperEntry {
                    id: "nvme_cli".to_string(),
                    name: "nvme-cli".to_string(),
                    purpose: "Detailed NVMe drive statistics".to_string(),
                    packages_arch: vec!["nvme-cli".to_string()],
                    packages_debian: vec!["nvme-cli".to_string()],
                    packages_fedora: vec!["nvme-cli".to_string()],
                    check_command: "nvme version".to_string(),
                    benefits_probes: vec!["nvme".to_string(), "disk_health".to_string()],
                },
                HelperEntry {
                    id: "ethtool".to_string(),
                    name: "ethtool".to_string(),
                    purpose: "Detailed network interface statistics".to_string(),
                    packages_arch: vec!["ethtool".to_string()],
                    packages_debian: vec!["ethtool".to_string()],
                    packages_fedora: vec!["ethtool".to_string()],
                    check_command: "ethtool --version".to_string(),
                    benefits_probes: vec!["network".to_string(), "nic".to_string()],
                },
                HelperEntry {
                    id: "hdparm".to_string(),
                    name: "hdparm".to_string(),
                    purpose: "HDD/SSD parameters and benchmarking".to_string(),
                    packages_arch: vec!["hdparm".to_string()],
                    packages_debian: vec!["hdparm".to_string()],
                    packages_fedora: vec!["hdparm".to_string()],
                    check_command: "hdparm -V".to_string(),
                    benefits_probes: vec!["disk".to_string(), "disk_perf".to_string()],
                },
                HelperEntry {
                    id: "dmidecode".to_string(),
                    name: "dmidecode".to_string(),
                    purpose: "Hardware information from BIOS/UEFI".to_string(),
                    packages_arch: vec!["dmidecode".to_string()],
                    packages_debian: vec!["dmidecode".to_string()],
                    packages_fedora: vec!["dmidecode".to_string()],
                    check_command: "dmidecode --version".to_string(),
                    benefits_probes: vec!["hardware".to_string(), "memory".to_string()],
                },
                HelperEntry {
                    id: "lshw".to_string(),
                    name: "lshw".to_string(),
                    purpose: "Detailed hardware listing".to_string(),
                    packages_arch: vec!["lshw".to_string()],
                    packages_debian: vec!["lshw".to_string()],
                    packages_fedora: vec!["lshw".to_string()],
                    check_command: "lshw -version".to_string(),
                    benefits_probes: vec!["hardware".to_string(), "inventory".to_string()],
                },
            ],
        }
    }

    /// Get helper by ID.
    pub fn get(&self, id: &str) -> Option<&HelperEntry> {
        self.helpers.iter().find(|h| h.id == id)
    }

    /// Get helpers that benefit a probe.
    pub fn helpers_for_probe(&self, probe: &str) -> Vec<&HelperEntry> {
        self.helpers
            .iter()
            .filter(|h| h.benefits_probes.iter().any(|p| p == probe))
            .collect()
    }
}

impl Default for HelperCatalog {
    fn default() -> Self {
        Self::default_catalog()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_catalog() {
        let catalog = HelperCatalog::default_catalog();
        assert!(!catalog.helpers.is_empty());
        assert!(catalog.get("lm_sensors").is_some());
    }

    #[test]
    fn test_packages_for_distro() {
        let catalog = HelperCatalog::default_catalog();
        let lm = catalog.get("lm_sensors").unwrap();

        assert_eq!(lm.packages_for_distro("Arch Linux"), &["lm_sensors"]);
        assert_eq!(lm.packages_for_distro("Ubuntu 22.04"), &["lm-sensors"]);
    }

    #[test]
    fn test_helpers_for_probe() {
        let catalog = HelperCatalog::default_catalog();

        let temp_helpers = catalog.helpers_for_probe("temperature");
        assert!(!temp_helpers.is_empty());
        assert!(temp_helpers.iter().any(|h| h.id == "lm_sensors"));
    }
}
