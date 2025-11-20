//! File Backup System with SHA256 Verification
//!
//! Handles safe file backups before modifications, with cryptographic verification
//! and rollback capability.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// Default backup directory (system-level)
pub const SYSTEM_BACKUP_DIR: &str = "/var/lib/anna/backups";

/// Default backup directory (user-level)
pub fn user_backup_dir() -> Result<PathBuf> {
    let base = if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/share")
    } else {
        anyhow::bail!("Could not determine user data directory");
    };
    Ok(base.join("anna").join("backups"))
}

/// File backup record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBackup {
    /// Original file path
    pub original_path: PathBuf,

    /// Backup file path
    pub backup_path: PathBuf,

    /// SHA256 hash of original content
    pub sha256: String,

    /// File size in bytes
    pub size_bytes: u64,

    /// When the backup was created
    pub created_at: SystemTime,

    /// Change set ID this backup belongs to
    pub change_set_id: String,

    /// Operation type
    pub operation: FileOperation,
}

/// Type of file operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOperation {
    /// File was modified
    Modified,

    /// File was created (backup is empty marker)
    Created,

    /// File was deleted
    Deleted,
}

impl FileBackup {
    /// Create a backup of a file before modification
    ///
    /// Returns the FileBackup record on success.
    pub fn create_backup(
        original_path: impl AsRef<Path>,
        change_set_id: impl Into<String>,
        operation: FileOperation,
    ) -> Result<Self> {
        let original_path = original_path.as_ref();
        let change_set_id = change_set_id.into();

        // Determine backup directory based on whether we're root
        let backup_dir = if nix::unistd::getuid().is_root() {
            PathBuf::from(SYSTEM_BACKUP_DIR)
        } else {
            user_backup_dir()?
        };

        // Ensure backup directory exists
        fs::create_dir_all(&backup_dir).context("Failed to create backup directory")?;

        // Generate backup filename
        let backup_filename = Self::generate_backup_filename(original_path, &change_set_id);
        let backup_path = backup_dir.join(&backup_filename);

        debug!(
            "Creating backup: {} -> {}",
            original_path.display(),
            backup_path.display()
        );

        match operation {
            FileOperation::Modified | FileOperation::Deleted => {
                // File exists - copy it and compute hash
                if !original_path.exists() {
                    anyhow::bail!("Original file does not exist: {}", original_path.display());
                }

                let content = fs::read(original_path)
                    .with_context(|| format!("Failed to read file: {}", original_path.display()))?;

                let size_bytes = content.len() as u64;
                let sha256 = Self::compute_sha256(&content);

                fs::write(&backup_path, &content).with_context(|| {
                    format!("Failed to write backup: {}", backup_path.display())
                })?;

                info!(
                    "Backed up {} ({} bytes, sha256: {})",
                    original_path.display(),
                    size_bytes,
                    &sha256[..16]
                );

                Ok(Self {
                    original_path: original_path.to_path_buf(),
                    backup_path,
                    sha256,
                    size_bytes,
                    created_at: SystemTime::now(),
                    change_set_id,
                    operation,
                })
            }
            FileOperation::Created => {
                // File is about to be created - create empty marker
                fs::write(&backup_path, b"").with_context(|| {
                    format!("Failed to write backup marker: {}", backup_path.display())
                })?;

                debug!(
                    "Created backup marker for new file: {}",
                    original_path.display()
                );

                Ok(Self {
                    original_path: original_path.to_path_buf(),
                    backup_path,
                    sha256: String::new(), // No hash for new files
                    size_bytes: 0,
                    created_at: SystemTime::now(),
                    change_set_id,
                    operation: FileOperation::Created,
                })
            }
        }
    }

    /// Verify backup integrity against current backup file
    pub fn verify_integrity(&self) -> Result<bool> {
        if self.operation == FileOperation::Created {
            // Created files don't have content to verify
            return Ok(self.backup_path.exists());
        }

        if !self.backup_path.exists() {
            warn!("Backup file missing: {}", self.backup_path.display());
            return Ok(false);
        }

        let content = fs::read(&self.backup_path)
            .with_context(|| format!("Failed to read backup: {}", self.backup_path.display()))?;

        let current_hash = Self::compute_sha256(&content);

        if current_hash != self.sha256 {
            warn!(
                "Backup integrity check failed for {}: expected {}, got {}",
                self.backup_path.display(),
                &self.sha256[..16],
                &current_hash[..16]
            );
            return Ok(false);
        }

        debug!("Backup integrity verified: {}", self.backup_path.display());
        Ok(true)
    }

    /// Restore this backup to its original location
    ///
    /// Returns true if restoration was successful.
    pub fn restore(&self) -> Result<bool> {
        debug!(
            "Restoring backup: {} -> {}",
            self.backup_path.display(),
            self.original_path.display()
        );

        // Verify integrity first
        if !self.verify_integrity()? {
            anyhow::bail!(
                "Backup integrity check failed, refusing to restore: {}",
                self.backup_path.display()
            );
        }

        match self.operation {
            FileOperation::Modified | FileOperation::Deleted => {
                // Restore the original content
                let content = fs::read(&self.backup_path).with_context(|| {
                    format!("Failed to read backup: {}", self.backup_path.display())
                })?;

                // Ensure parent directory exists
                if let Some(parent) = self.original_path.parent() {
                    fs::create_dir_all(parent).context("Failed to create parent directory")?;
                }

                fs::write(&self.original_path, &content).with_context(|| {
                    format!("Failed to restore file: {}", self.original_path.display())
                })?;

                info!(
                    "Restored {} from backup ({} bytes)",
                    self.original_path.display(),
                    content.len()
                );

                Ok(true)
            }
            FileOperation::Created => {
                // File was created by Anna - delete it
                if self.original_path.exists() {
                    fs::remove_file(&self.original_path).with_context(|| {
                        format!(
                            "Failed to delete created file: {}",
                            self.original_path.display()
                        )
                    })?;

                    info!("Deleted created file: {}", self.original_path.display());
                }

                Ok(true)
            }
        }
    }

    /// Delete this backup file
    pub fn delete_backup(&self) -> Result<()> {
        if self.backup_path.exists() {
            fs::remove_file(&self.backup_path).with_context(|| {
                format!("Failed to delete backup: {}", self.backup_path.display())
            })?;

            debug!("Deleted backup: {}", self.backup_path.display());
        }

        Ok(())
    }

    /// Compute SHA256 hash of content
    fn compute_sha256(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Generate a unique backup filename
    fn generate_backup_filename(original_path: &Path, change_set_id: &str) -> String {
        // Sanitize the original path for use in filename
        let path_str = original_path.to_string_lossy();
        let sanitized = path_str
            .replace("/", "_")
            .replace("\\", "_")
            .replace("..", "_");

        // Truncate if too long
        let sanitized = if sanitized.len() > 100 {
            &sanitized[..100]
        } else {
            &sanitized
        };

        format!("{}_{}", change_set_id, sanitized)
    }
}

/// Backup storage manager
pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new() -> Result<Self> {
        let backup_dir = if nix::unistd::getuid().is_root() {
            PathBuf::from(SYSTEM_BACKUP_DIR)
        } else {
            user_backup_dir()?
        };

        Ok(Self { backup_dir })
    }

    /// Get total size of all backups in bytes
    pub fn total_backup_size(&self) -> Result<u64> {
        let mut total = 0u64;

        if !self.backup_dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            if entry.path().is_file() {
                total += entry.metadata()?.len();
            }
        }

        Ok(total)
    }

    /// Get total size in a human-readable format
    pub fn total_backup_size_human(&self) -> Result<String> {
        let bytes = self.total_backup_size()?;

        let size = if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        };

        Ok(size)
    }

    /// Get number of backup files
    pub fn backup_count(&self) -> Result<usize> {
        if !self.backup_dir.exists() {
            return Ok(0);
        }

        let count = fs::read_dir(&self.backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .count();

        Ok(count)
    }

    /// Delete backups older than given age (in days)
    pub fn delete_old_backups(&self, age_days: u64) -> Result<Vec<PathBuf>> {
        let mut deleted = Vec::new();

        if !self.backup_dir.exists() {
            return Ok(deleted);
        }

        let cutoff = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            - (age_days * 24 * 60 * 60);

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let metadata = entry.metadata()?;
                let modified = metadata.modified()?;
                let modified_secs = modified.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();

                if modified_secs < cutoff {
                    fs::remove_file(&path)?;
                    deleted.push(path);
                }
            }
        }

        if !deleted.is_empty() {
            info!("Deleted {} old backup files", deleted.len());
        }

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_file_backup_and_restore() {
        let temp_dir = tempdir().unwrap();

        // Create a test file
        let original_path = temp_dir.path().join("test.txt");
        fs::write(&original_path, b"Hello, World!").unwrap();

        // Create backup
        let backup =
            FileBackup::create_backup(&original_path, "test-change-123", FileOperation::Modified)
                .unwrap();

        // Verify integrity
        assert!(backup.verify_integrity().unwrap());

        // Modify original file
        fs::write(&original_path, b"Modified content").unwrap();

        // Restore backup
        assert!(backup.restore().unwrap());

        // Verify restoration
        let restored_content = fs::read(&original_path).unwrap();
        assert_eq!(restored_content, b"Hello, World!");
    }

    #[test]
    fn test_sha256_verification() {
        let content = b"Test content for hashing";
        let hash = FileBackup::compute_sha256(content);

        // Verify hash is hex string
        assert_eq!(hash.len(), 64); // SHA256 is 256 bits = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_created_file_backup() {
        let temp_dir = tempdir().unwrap();
        let new_file_path = temp_dir.path().join("new_file.txt");

        // Create backup marker for a file about to be created
        let backup =
            FileBackup::create_backup(&new_file_path, "test-change-456", FileOperation::Created)
                .unwrap();

        // Simulate file creation
        fs::write(&new_file_path, b"New file content").unwrap();
        assert!(new_file_path.exists());

        // Restore (should delete the created file)
        assert!(backup.restore().unwrap());
        assert!(!new_file_path.exists());
    }

    #[test]
    fn test_backup_manager() {
        let manager = BackupManager::new().unwrap();

        // These should not error
        let _ = manager.backup_count();
        let _ = manager.total_backup_size();
        let size_human = manager.total_backup_size_human().unwrap();

        // Should return a human-readable size
        assert!(
            size_human.contains("B")
                || size_human.contains("KB")
                || size_human.contains("MB")
                || size_human.contains("GB")
        );
    }
}
