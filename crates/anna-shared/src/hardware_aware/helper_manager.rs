//! Helper manager for lifecycle operations (v0.0.434).
//!
//! Manages installation, tracking, and usage of helper tools.

use super::helper_entry::{HelperCatalog, HelperEntry};
use super::helper_error::HelperError;
use super::helper_state::{timestamp_now, HelperInstalledBy, HelperState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
fn get_package_manager_remove(
    distro: &str,
) -> Result<(&'static str, Vec<&'static str>), HelperError> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_package_manager_detection() {
        assert!(get_package_manager("Arch Linux").is_ok());
        assert!(get_package_manager("Ubuntu 22.04").is_ok());
        assert!(get_package_manager("Fedora 39").is_ok());
    }
}
