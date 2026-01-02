//! Package Tracker Implementation
//!
//! Main tracker for managing package installation records.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{InstalledBy, PackageManager, PackageRecord};

/// Package installation tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageTracker {
    /// All package records
    pub records: Vec<PackageRecord>,
    /// Count by installer
    pub by_installer: HashMap<String, u64>,
    /// Count by manager
    pub by_manager: HashMap<String, u64>,
    /// Total Anna-installed packages
    pub anna_installed_count: u64,
    /// Total user-installed packages
    pub user_installed_count: u64,
}

impl PackageTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a package installation
    pub fn record_install(&mut self, record: PackageRecord) {
        let installer_key = format!("{:?}", record.installed_by);
        let manager_key = record.manager.name().to_string();

        *self.by_installer.entry(installer_key).or_insert(0) += 1;
        *self.by_manager.entry(manager_key).or_insert(0) += 1;

        match record.installed_by {
            InstalledBy::Anna => self.anna_installed_count += 1,
            InstalledBy::User => self.user_installed_count += 1,
            _ => {}
        }

        self.records.push(record);
    }

    /// Mark a package as removed
    pub fn record_removal(&mut self, name: &str) -> bool {
        let found = self.records.iter().position(|r| r.name == name && r.is_installed);
        if let Some(idx) = found {
            self.records[idx].is_installed = false;
            self.records[idx].removed_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            );
            true
        } else {
            false
        }
    }

    /// Get package by name
    pub fn get(&self, name: &str) -> Option<&PackageRecord> {
        self.records.iter().find(|r| r.name == name)
    }

    /// Get currently installed packages
    pub fn installed(&self) -> Vec<&PackageRecord> {
        self.records.iter().filter(|r| r.is_installed).collect()
    }

    /// Get packages installed by Anna
    pub fn anna_installed(&self) -> Vec<&PackageRecord> {
        self.records
            .iter()
            .filter(|r| r.installed_by == InstalledBy::Anna && r.is_installed)
            .collect()
    }

    /// Get packages installed by user
    pub fn user_installed(&self) -> Vec<&PackageRecord> {
        self.records
            .iter()
            .filter(|r| r.installed_by == InstalledBy::User && r.is_installed)
            .collect()
    }

    /// Get recent installations
    pub fn recent(&self, limit: usize) -> Vec<&PackageRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get packages by manager
    pub fn by_package_manager(&self, manager: PackageManager) -> Vec<&PackageRecord> {
        self.records.iter().filter(|r| r.manager == manager).collect()
    }

    /// Total package count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Currently installed count
    pub fn installed_count(&self) -> usize {
        self.installed().len()
    }

    /// Get removed packages
    pub fn removed(&self) -> Vec<&PackageRecord> {
        self.records.iter().filter(|r| !r.is_installed).collect()
    }
}
