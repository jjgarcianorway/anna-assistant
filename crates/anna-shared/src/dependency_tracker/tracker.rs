//! Dependency tracker implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{DependencyRecord, DependencyStatus, DependencyType};

/// Dependency tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyTracker {
    /// All dependency records
    pub records: Vec<DependencyRecord>,
    /// Count by type
    pub by_type: HashMap<String, u64>,
    /// Count by status
    pub by_status: HashMap<String, u64>,
    /// Packages with missing dependencies
    pub broken_packages: Vec<String>,
    /// Last full scan
    pub last_scan: Option<u64>,
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dependency record
    pub fn add(&mut self, record: DependencyRecord) {
        *self.by_type.entry(record.dep_type.name().to_string()).or_insert(0) += 1;
        *self.by_status.entry(record.status.name().to_string()).or_insert(0) += 1;

        if record.status == DependencyStatus::Missing {
            if !self.broken_packages.contains(&record.package) {
                self.broken_packages.push(record.package.clone());
            }
        }
        self.records.push(record);
    }

    /// Get dependencies for a package
    pub fn deps_for(&self, package: &str) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.package == package).collect()
    }

    /// Get reverse dependencies (what depends on this)
    pub fn reverse_deps(&self, dependency: &str) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.dependency == dependency).collect()
    }

    /// Check if package has missing deps
    pub fn has_missing(&self, package: &str) -> bool {
        self.records
            .iter()
            .any(|r| r.package == package && r.status == DependencyStatus::Missing)
    }

    /// Check if package is safe to remove (nothing depends on it)
    pub fn safe_to_remove(&self, package: &str) -> bool {
        !self.records.iter().any(|r| {
            r.dependency == package
                && r.status == DependencyStatus::Installed
                && r.dep_type == DependencyType::Runtime
        })
    }

    /// Get orphaned packages (installed but nothing depends on them)
    pub fn orphaned(&self) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.status == DependencyStatus::Orphaned).collect()
    }

    /// Get missing dependencies
    pub fn missing(&self) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.status == DependencyStatus::Missing).collect()
    }

    /// Get outdated dependencies
    pub fn outdated(&self) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.status == DependencyStatus::Outdated).collect()
    }

    /// Get by type
    pub fn by_dep_type(&self, dep_type: DependencyType) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.dep_type == dep_type).collect()
    }

    /// Update dependency status
    pub fn update_status(&mut self, package: &str, dep: &str, status: DependencyStatus) -> bool {
        let found = self.records.iter().position(|r| r.package == package && r.dependency == dep);
        if let Some(idx) = found {
            let old_status = self.records[idx].status;
            if let Some(count) = self.by_status.get_mut(old_status.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_status.entry(status.name().to_string()).or_insert(0) += 1;
            self.records[idx].status = status;
            true
        } else {
            false
        }
    }

    /// Total record count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Missing count
    pub fn missing_count(&self) -> usize {
        self.records.iter().filter(|r| r.status == DependencyStatus::Missing).count()
    }

    /// Record a full scan
    pub fn record_scan(&mut self, timestamp: u64) {
        self.last_scan = Some(timestamp);
    }
}
