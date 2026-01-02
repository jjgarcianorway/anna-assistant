//! Backup history storage and retrieval

use super::types::{BackupRecord, BackupStatus, BackupType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Backup history tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackupHistory {
    /// All backup records
    pub records: Vec<BackupRecord>,
    /// Count by type
    pub by_type: HashMap<String, u64>,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// Number of restores performed
    pub restore_count: u64,
}

impl BackupHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a backup record
    pub fn add(&mut self, record: BackupRecord) {
        let type_key = format!("{:?}", record.backup_type);
        *self.by_type.entry(type_key).or_insert(0) += 1;
        self.total_size_bytes += record.size_bytes;
        self.records.push(record);
    }

    /// Get backup by ID
    pub fn get(&self, id: &str) -> Option<&BackupRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Get mutable backup by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut BackupRecord> {
        self.records.iter_mut().find(|r| r.id == id)
    }

    /// Mark a backup as restored
    pub fn mark_restored(&mut self, id: &str) -> bool {
        if let Some(record) = self.get_mut(id) {
            record.status = BackupStatus::Restored;
            record.restored_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            );
            self.restore_count += 1;
            true
        } else {
            false
        }
    }

    /// Mark a backup as deleted
    pub fn mark_deleted(&mut self, id: &str) -> bool {
        // Find the index and size first to avoid borrow issues
        let found = self.records.iter().position(|r| r.id == id);
        if let Some(idx) = found {
            let size = self.records[idx].size_bytes;
            self.total_size_bytes = self.total_size_bytes.saturating_sub(size);
            self.records[idx].status = BackupStatus::Deleted;
            true
        } else {
            false
        }
    }

    /// Get active (restorable) backups
    pub fn active(&self) -> Vec<&BackupRecord> {
        self.records.iter().filter(|r| r.is_restorable()).collect()
    }

    /// Get recent backups
    pub fn recent(&self, limit: usize) -> Vec<&BackupRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get backups by type
    pub fn by_backup_type(&self, backup_type: BackupType) -> Vec<&BackupRecord> {
        self.records
            .iter()
            .filter(|r| r.backup_type == backup_type)
            .collect()
    }

    /// Get backups for a specific file
    pub fn for_file(&self, path: &str) -> Vec<&BackupRecord> {
        self.records
            .iter()
            .filter(|r| r.original_path == path)
            .collect()
    }

    /// Get backups for a change ID
    pub fn for_change(&self, change_id: &str) -> Vec<&BackupRecord> {
        self.records
            .iter()
            .filter(|r| r.change_id.as_deref() == Some(change_id))
            .collect()
    }

    /// Total backup count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Active backup count
    pub fn active_count(&self) -> usize {
        self.active().len()
    }

    /// Expire old backups
    pub fn expire_old(&mut self, now: u64) -> usize {
        let mut count = 0;
        for record in &mut self.records {
            if record.status == BackupStatus::Active && record.is_expired(now) {
                record.status = BackupStatus::Expired;
                count += 1;
            }
        }
        count
    }

    /// Calculate total active size
    pub fn active_size_bytes(&self) -> u64 {
        self.active().iter().map(|r| r.size_bytes).sum()
    }
}
