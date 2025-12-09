//! Helpers registry (v0.0.221).

use serde::{Deserialize, Serialize};

use super::types::HelperPackage;

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
}
