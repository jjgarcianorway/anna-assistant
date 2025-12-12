//! Helper tool catalog and management (v0.0.434).
//!
//! Manages system helper tools like lm_sensors, smartmontools, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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

/// Who installed the helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelperInstalledBy {
    /// Anna installed this helper.
    Anna,
    /// User installed this helper.
    User,
}

impl HelperInstalledBy {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Anna => "anna",
            Self::User => "user",
        }
    }
}

/// State of a helper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperState {
    /// Helper ID.
    pub id: String,
    /// Who installed it.
    pub installed_by: HelperInstalledBy,
    /// When installed/detected.
    pub installed_at: String,
    /// Number of times used.
    pub use_count: u64,
    /// Last used timestamp.
    pub last_used: Option<String>,
}

impl HelperState {
    /// Create for Anna-installed helper.
    pub fn installed_by_anna(id: &str) -> Self {
        Self {
            id: id.to_string(),
            installed_by: HelperInstalledBy::Anna,
            installed_at: timestamp_now(),
            use_count: 0,
            last_used: None,
        }
    }

    /// Create for user-installed helper.
    pub fn detected_user(id: &str) -> Self {
        Self {
            id: id.to_string(),
            installed_by: HelperInstalledBy::User,
            installed_at: timestamp_now(),
            use_count: 0,
            last_used: None,
        }
    }

    /// Record usage.
    pub fn record_use(&mut self) {
        self.use_count += 1;
        self.last_used = Some(timestamp_now());
    }
}

/// Helper manager that tracks installed helpers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperManager {
    /// Tracked helpers.
    pub helpers: HashMap<String, HelperState>,
    /// Last scan time.
    pub last_scan: Option<String>,
}

impl HelperManager {
    /// Create empty manager.
    pub fn new() -> Self {
        Self {
            helpers: HashMap::new(),
            last_scan: None,
        }
    }

    /// Load from file.
    pub fn load(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save to file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    /// Check if a helper is tracked.
    pub fn is_tracked(&self, id: &str) -> bool {
        self.helpers.contains_key(id)
    }

    /// Get helper state.
    pub fn get(&self, id: &str) -> Option<&HelperState> {
        self.helpers.get(id)
    }

    /// Record helper as installed by Anna.
    pub fn record_anna_install(&mut self, id: &str) {
        self.helpers
            .insert(id.to_string(), HelperState::installed_by_anna(id));
    }

    /// Record helper as detected (user-installed).
    pub fn record_detected(&mut self, id: &str) {
        if !self.helpers.contains_key(id) {
            self.helpers
                .insert(id.to_string(), HelperState::detected_user(id));
        }
    }

    /// Record usage of a helper.
    pub fn record_use(&mut self, id: &str) {
        if let Some(state) = self.helpers.get_mut(id) {
            state.record_use();
        }
    }

    /// Remove a helper record.
    pub fn remove(&mut self, id: &str) {
        self.helpers.remove(id);
    }

    /// Get helpers installed by Anna.
    pub fn anna_installed(&self) -> Vec<&str> {
        self.helpers
            .iter()
            .filter(|(_, s)| s.installed_by == HelperInstalledBy::Anna)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Get helpers installed by user.
    pub fn user_installed(&self) -> Vec<&str> {
        self.helpers
            .iter()
            .filter(|(_, s)| s.installed_by == HelperInstalledBy::User)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Scan and sync with actual system state.
    pub fn sync_with_system(&mut self, catalog: &HelperCatalog) {
        for helper in &catalog.helpers {
            if helper.is_installed() {
                self.record_detected(&helper.id);
            } else {
                // If we think it's installed but it's not, update
                if let Some(state) = self.helpers.get(&helper.id) {
                    if state.installed_by == HelperInstalledBy::Anna {
                        // Anna installed it but it's gone - remove record
                        self.helpers.remove(&helper.id);
                    }
                }
            }
        }
        self.last_scan = Some(timestamp_now());
    }

    /// Install a helper.
    pub fn install(&mut self, helper: &HelperEntry, distro: &str) -> Result<(), HelperError> {
        let packages = helper.packages_for_distro(distro);
        if packages.is_empty() {
            return Err(HelperError::NoPackages(helper.id.clone()));
        }

        // Determine package manager
        let (pm_cmd, install_args) = get_package_manager(distro)?;

        use std::process::Command;
        let mut cmd = Command::new("sudo");
        cmd.arg(pm_cmd);
        for arg in install_args {
            cmd.arg(arg);
        }
        for pkg in packages {
            cmd.arg(pkg);
        }

        let output = cmd
            .output()
            .map_err(|e| HelperError::InstallFailed(format!("Failed to run: {}", e)))?;

        if !output.status.success() {
            return Err(HelperError::InstallFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        self.record_anna_install(&helper.id);
        Ok(())
    }

    /// Uninstall a helper (only if Anna-installed).
    pub fn uninstall(&mut self, helper: &HelperEntry, distro: &str) -> Result<(), HelperError> {
        // Only uninstall Anna-installed helpers
        if let Some(state) = self.helpers.get(&helper.id) {
            if state.installed_by != HelperInstalledBy::Anna {
                return Err(HelperError::NotAnnaInstalled(helper.id.clone()));
            }
        } else {
            return Err(HelperError::NotInstalled(helper.id.clone()));
        }

        let packages = helper.packages_for_distro(distro);
        if packages.is_empty() {
            return Err(HelperError::NoPackages(helper.id.clone()));
        }

        let (pm_cmd, remove_args) = get_package_manager_remove(distro)?;

        use std::process::Command;
        let mut cmd = Command::new("sudo");
        cmd.arg(pm_cmd);
        for arg in remove_args {
            cmd.arg(arg);
        }
        for pkg in packages {
            cmd.arg(pkg);
        }

        let output = cmd
            .output()
            .map_err(|e| HelperError::UninstallFailed(format!("Failed to run: {}", e)))?;

        if !output.status.success() {
            return Err(HelperError::UninstallFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        self.remove(&helper.id);
        Ok(())
    }
}

impl Default for HelperManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper error.
#[derive(Debug, Clone)]
pub enum HelperError {
    /// No packages defined for this distro.
    NoPackages(String),
    /// Helper not installed.
    NotInstalled(String),
    /// Helper was not installed by Anna.
    NotAnnaInstalled(String),
    /// Installation failed.
    InstallFailed(String),
    /// Uninstallation failed.
    UninstallFailed(String),
    /// Unknown package manager.
    UnknownPackageManager(String),
}

impl std::fmt::Display for HelperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPackages(id) => write!(f, "No packages defined for helper: {}", id),
            Self::NotInstalled(id) => write!(f, "Helper not installed: {}", id),
            Self::NotAnnaInstalled(id) => write!(f, "Helper {} was not installed by Anna", id),
            Self::InstallFailed(msg) => write!(f, "Installation failed: {}", msg),
            Self::UninstallFailed(msg) => write!(f, "Uninstallation failed: {}", msg),
            Self::UnknownPackageManager(distro) => {
                write!(f, "Unknown package manager for distro: {}", distro)
            }
        }
    }
}

impl std::error::Error for HelperError {}

/// Get package manager command for install.
fn get_package_manager(distro: &str) -> Result<(&'static str, Vec<&'static str>), HelperError> {
    let distro_lower = distro.to_lowercase();
    if distro_lower.contains("arch") {
        Ok(("pacman", vec!["-S", "--noconfirm"]))
    } else if distro_lower.contains("debian")
        || distro_lower.contains("ubuntu")
        || distro_lower.contains("mint")
    {
        Ok(("apt-get", vec!["install", "-y"]))
    } else if distro_lower.contains("fedora") {
        Ok(("dnf", vec!["install", "-y"]))
    } else if distro_lower.contains("rhel") || distro_lower.contains("centos") {
        Ok(("yum", vec!["install", "-y"]))
    } else {
        Err(HelperError::UnknownPackageManager(distro.to_string()))
    }
}

/// Get package manager command for remove.
fn get_package_manager_remove(distro: &str) -> Result<(&'static str, Vec<&'static str>), HelperError> {
    let distro_lower = distro.to_lowercase();
    if distro_lower.contains("arch") {
        Ok(("pacman", vec!["-R", "--noconfirm"]))
    } else if distro_lower.contains("debian")
        || distro_lower.contains("ubuntu")
        || distro_lower.contains("mint")
    {
        Ok(("apt-get", vec!["remove", "-y"]))
    } else if distro_lower.contains("fedora") {
        Ok(("dnf", vec!["remove", "-y"]))
    } else if distro_lower.contains("rhel") || distro_lower.contains("centos") {
        Ok(("yum", vec!["remove", "-y"]))
    } else {
        Err(HelperError::UnknownPackageManager(distro.to_string()))
    }
}

/// Get current timestamp.
fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
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

    #[test]
    fn test_helper_manager() {
        let mut manager = HelperManager::new();

        manager.record_anna_install("lm_sensors");
        assert!(manager.is_tracked("lm_sensors"));
        assert!(manager.anna_installed().contains(&"lm_sensors"));

        manager.record_detected("nvme_cli");
        assert!(manager.user_installed().contains(&"nvme_cli"));
    }

    #[test]
    fn test_helper_state_usage() {
        let mut state = HelperState::installed_by_anna("test");
        assert_eq!(state.use_count, 0);

        state.record_use();
        assert_eq!(state.use_count, 1);
        assert!(state.last_used.is_some());
    }

    #[test]
    fn test_package_manager_detection() {
        assert!(get_package_manager("Arch Linux").is_ok());
        assert!(get_package_manager("Ubuntu 22.04").is_ok());
        assert!(get_package_manager("Fedora 39").is_ok());
    }
}
