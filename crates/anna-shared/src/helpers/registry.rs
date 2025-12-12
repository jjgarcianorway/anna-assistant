//! Helpers registry (v0.0.221).
//! v0.0.466: Enhanced with smart management per Phase 32.

use serde::{Deserialize, Serialize};

use super::types::{HelperPackage, InstallSource};

/// Registry of helper packages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelpersRegistry {
    /// Tracked packages
    pub packages: Vec<HelperPackage>,
}

impl HelpersRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
        }
    }

    /// Add or update a package
    pub fn register(&mut self, package: HelperPackage) {
        if let Some(existing) = self.packages.iter_mut().find(|p| p.id == package.id) {
            *existing = package;
        } else {
            self.packages.push(package);
        }
    }

    /// Get a package by ID
    pub fn get(&self, id: &str) -> Option<&HelperPackage> {
        self.packages.iter().find(|p| p.id == id)
    }

    /// Get a mutable package by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut HelperPackage> {
        self.packages.iter_mut().find(|p| p.id == id)
    }

    /// Remove a package
    pub fn remove(&mut self, id: &str) -> Option<HelperPackage> {
        if let Some(pos) = self.packages.iter().position(|p| p.id == id) {
            Some(self.packages.remove(pos))
        } else {
            None
        }
    }

    /// Get all packages installed by Anna
    pub fn anna_installed(&self) -> Vec<&HelperPackage> {
        self.packages
            .iter()
            .filter(|p| p.installed_by_anna())
            .collect()
    }

    /// Get all required packages
    pub fn required_packages(&self) -> Vec<&HelperPackage> {
        self.packages.iter().filter(|p| p.required).collect()
    }

    /// Get all available packages
    pub fn available_packages(&self) -> Vec<&HelperPackage> {
        self.packages.iter().filter(|p| p.available).collect()
    }

    /// Get all unavailable required packages
    pub fn missing_required(&self) -> Vec<&HelperPackage> {
        self.packages
            .iter()
            .filter(|p| p.required && !p.available)
            .collect()
    }

    /// Check if all required packages are available
    pub fn all_required_available(&self) -> bool {
        self.packages
            .iter()
            .filter(|p| p.required)
            .all(|p| p.available)
    }

    /// Get count of packages
    pub fn len(&self) -> usize {
        self.packages.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Clear all packages (for reset)
    pub fn clear(&mut self) {
        self.packages.clear();
    }

    /// v0.0.466: Record usage of a helper
    pub fn record_usage(&mut self, id: &str) {
        if let Some(pkg) = self.get_mut(id) {
            pkg.record_usage();
        }
    }

    /// v0.0.466: Register a helper as installed by Anna
    pub fn register_anna_installed(&mut self, id: &str, name: &str) {
        let mut pkg = HelperPackage::new(id, name);
        pkg.install_source = InstallSource::Anna;
        pkg.available = true;
        self.register(pkg);
    }

    /// v0.0.466: Get only Anna-installed helpers (for removal on uninstall)
    pub fn get_anna_removable(&self) -> Vec<&HelperPackage> {
        self.packages
            .iter()
            .filter(|p| p.install_source == InstallSource::Anna && !p.required)
            .collect()
    }

    /// v0.0.466: Remove all Anna-installed helpers (for uninstall)
    /// Returns list of removed package IDs
    pub fn remove_anna_installed(&mut self) -> Vec<String> {
        let to_remove: Vec<String> = self
            .packages
            .iter()
            .filter(|p| p.install_source == InstallSource::Anna && !p.required)
            .map(|p| p.id.clone())
            .collect();

        self.packages.retain(|p| {
            !(p.install_source == InstallSource::Anna && !p.required)
        });

        to_remove
    }

    /// v0.0.466: Check if a helper is useful given current hardware
    /// Returns false if the helper requires hardware that isn't present
    pub fn is_useful(&self, id: &str, available_hardware: &[&str]) -> bool {
        if let Some(pkg) = self.get(id) {
            if let Some(ref req) = pkg.hardware_requirement {
                return available_hardware.iter().any(|h| h.eq_ignore_ascii_case(req));
            }
        }
        true // No requirement = useful
    }

    /// v0.0.466: Get helpers that should be skipped (useless without hardware)
    pub fn get_useless_helpers(&self, available_hardware: &[&str]) -> Vec<&HelperPackage> {
        self.packages
            .iter()
            .filter(|p| {
                if let Some(ref req) = p.hardware_requirement {
                    !available_hardware.iter().any(|h| h.eq_ignore_ascii_case(req))
                } else {
                    false
                }
            })
            .collect()
    }
}
