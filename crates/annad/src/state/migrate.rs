//! State migration from v1 to v2 with backup and rollback (Phase 1.10)
//!
//! Provides forward-only migration with SHA256 checksum verification and automatic rollback on failure.

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{error, info, warn};

use super::v2::{StateV1, StateV2};

/// Migration result
#[derive(Debug)]
pub enum MigrationResult {
    Success,
    AlreadyV2,
    RolledBack(String),
}

/// State migrator
pub struct StateMigrator {
    state_dir: PathBuf,
}

impl StateMigrator {
    pub fn new(state_dir: PathBuf) -> Self {
        Self { state_dir }
    }

    /// Get state file path
    fn state_path(&self) -> PathBuf {
        self.state_dir.join("state.json")
    }

    /// Get v1 backup path
    fn backup_path(&self) -> PathBuf {
        self.state_dir.join("state.backup.v1")
    }

    /// Get checksum path
    fn checksum_path(&self) -> PathBuf {
        self.state_dir.join("state.backup.v1.sha256")
    }

    /// Get v2 state path
    fn v2_path(&self) -> PathBuf {
        self.state_dir.join("state.v2.json")
    }

    /// Compute SHA256 checksum of file
    async fn compute_checksum(&self, path: &Path) -> Result<String> {
        let content = fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    /// Verify checksum
    async fn verify_checksum(&self, path: &Path, expected: &str) -> Result<bool> {
        let actual = self.compute_checksum(path).await?;
        Ok(actual == expected)
    }

    /// Check if state is already v2
    pub async fn is_v2(&self) -> bool {
        let state_path = self.state_path();
        if !state_path.exists() {
            return false;
        }

        // Try to parse as v2
        if let Ok(content) = fs::read_to_string(&state_path).await {
            if let Ok(state) = serde_json::from_str::<StateV2>(&content) {
                return state.schema_version == 2;
            }
        }

        false
    }

    /// Migrate state from v1 to v2
    pub async fn migrate(&self, node_id: String) -> Result<MigrationResult> {
        info!("Starting state v1 → v2 migration");

        // Check if already v2
        if self.is_v2().await {
            info!("State is already v2, skipping migration");
            return Ok(MigrationResult::AlreadyV2);
        }

        let state_path = self.state_path();
        if !state_path.exists() {
            info!("No existing state found, creating new v2 state");
            let state = StateV2::new(node_id);
            state.save(&state_path).await?;
            return Ok(MigrationResult::Success);
        }

        // Step 1: Create backup of v1 state
        info!("Creating backup of v1 state");
        let backup_path = self.backup_path();
        fs::copy(&state_path, &backup_path).await
            .map_err(|e| anyhow!("Failed to create backup: {}", e))?;

        // Step 2: Compute and save checksum
        info!("Computing checksum of backup");
        let checksum = self.compute_checksum(&backup_path).await?;
        let checksum_path = self.checksum_path();
        fs::write(&checksum_path, &checksum).await
            .map_err(|e| anyhow!("Failed to write checksum: {}", e))?;

        info!("Backup checksum: {}", checksum);

        // Step 3: Load v1 state
        info!("Loading v1 state");
        let v1_state = StateV1::load(&state_path).await
            .map_err(|e| {
                error!("Failed to load v1 state: {}", e);
                e
            })?;

        // Step 4: Convert to v2
        info!("Converting v1 state to v2");
        let v2_state = v1_state.to_v2(node_id);

        // Step 5: Save v2 state (temp file first)
        info!("Saving v2 state");
        let v2_path = self.v2_path();
        v2_state.save(&v2_path).await
            .map_err(|e| {
                error!("Failed to save v2 state: {}", e);
                e
            })?;

        // Step 6: Verify backup checksum
        info!("Verifying backup checksum");
        let checksum_valid = self.verify_checksum(&backup_path, &checksum).await?;

        if !checksum_valid {
            error!("Backup checksum verification failed!");
            self.rollback().await?;
            return Ok(MigrationResult::RolledBack(
                "Checksum verification failed".to_string()
            ));
        }

        // Step 7: Replace state.json with v2 state
        info!("Replacing state.json with v2 state");
        fs::rename(&v2_path, &state_path).await
            .map_err(|e| {
                error!("Failed to rename v2 state: {}", e);
                e
            })?;

        // Step 8: Append migration entry to audit log
        self.log_migration_event(&checksum).await?;

        info!("✓ State migration v1 → v2 completed successfully");
        Ok(MigrationResult::Success)
    }

    /// Rollback to v1 state
    pub async fn rollback(&self) -> Result<()> {
        error!("CRITICAL: Rolling back to v1 state");

        let backup_path = self.backup_path();
        let state_path = self.state_path();
        let checksum_path = self.checksum_path();

        if !backup_path.exists() {
            return Err(anyhow!("No backup found for rollback"));
        }

        // Verify backup checksum before restore
        if checksum_path.exists() {
            let expected_checksum = fs::read_to_string(&checksum_path).await?;
            let checksum_valid = self.verify_checksum(&backup_path, &expected_checksum).await?;

            if !checksum_valid {
                error!("CRITICAL: Backup checksum invalid, cannot safely rollback!");
                return Err(anyhow!("Backup corrupted, rollback aborted"));
            }
        }

        // Restore backup
        fs::copy(&backup_path, &state_path).await
            .map_err(|e| anyhow!("Failed to restore backup: {}", e))?;

        warn!("State rolled back to v1 from backup");

        // Log rollback event
        self.log_rollback_event().await?;

        Ok(())
    }

    /// Log migration event to audit log
    async fn log_migration_event(&self, checksum: &str) -> Result<()> {
        let log_entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event": "state_migration",
            "from_version": 1,
            "to_version": 2,
            "backup_checksum": checksum,
            "result": "success",
        });

        self.append_audit_log(&log_entry).await
    }

    /// Log rollback event to audit log
    async fn log_rollback_event(&self) -> Result<()> {
        let log_entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event": "state_rollback",
            "from_version": 2,
            "to_version": 1,
            "reason": "migration_verification_failed",
            "result": "rolled_back",
        });

        self.append_audit_log(&log_entry).await
    }

    /// Append entry to audit log
    async fn append_audit_log(&self, entry: &serde_json::Value) -> Result<()> {
        let log_path = PathBuf::from("/var/log/anna/audit.jsonl");

        // Create log directory if needed
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent).await.ok(); // Ignore error if exists
        }

        let mut log_line = serde_json::to_string(entry)?;
        log_line.push('\n');

        // Append to log (create if doesn't exist)
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await?;

        use tokio::io::AsyncWriteExt;
        file.write_all(log_line.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_migration_with_new_state() {
        let temp_dir = tempdir().unwrap();
        let migrator = StateMigrator::new(temp_dir.path().to_path_buf());

        let result = migrator.migrate("node_test".to_string()).await.unwrap();
        assert!(matches!(result, MigrationResult::Success));

        // Verify state.json exists and is v2
        assert!(migrator.is_v2().await);
    }

    #[tokio::test]
    async fn test_checksum_verification() {
        let temp_dir = tempdir().unwrap();
        let migrator = StateMigrator::new(temp_dir.path().to_path_buf());

        // Create test file
        let test_path = temp_dir.path().join("test.txt");
        fs::write(&test_path, "test content").await.unwrap();

        // Compute checksum
        let checksum = migrator.compute_checksum(&test_path).await.unwrap();

        // Verify same content
        let valid = migrator.verify_checksum(&test_path, &checksum).await.unwrap();
        assert!(valid);

        // Verify different content
        fs::write(&test_path, "different content").await.unwrap();
        let valid = migrator.verify_checksum(&test_path, &checksum).await.unwrap();
        assert!(!valid);
    }
}
