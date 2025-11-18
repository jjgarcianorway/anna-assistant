//! SQLite Persistence for Change Logs
//!
//! Stores change units and their actions for historical tracking and rollback capability

use crate::change_log::*;
use crate::context::db::DbLocation;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Change log database
pub struct ChangeLogDb {
    conn: Arc<Mutex<Connection>>,
    _location: DbLocation,
}

impl ChangeLogDb {
    /// Open or create change log database
    pub async fn open(location: DbLocation) -> Result<Self> {
        let db_path = Self::get_db_path(&location)?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create database directory")?;
        }

        info!("Opening change log database at: {}", db_path.display());

        // Open connection in blocking context
        let conn = tokio::task::spawn_blocking(move || -> Result<Connection> {
            let conn = Connection::open(&db_path).context("Failed to open change log database")?;

            // Enable WAL mode for better concurrency
            conn.pragma_update(None, "journal_mode", "WAL")
                .context("Failed to enable WAL mode")?;

            // Enable foreign keys
            conn.pragma_update(None, "foreign_keys", "ON")
                .context("Failed to enable foreign keys")?;

            Ok(conn)
        })
        .await??;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            _location: location.clone(),
        };

        // Initialize schema
        db.initialize_schema().await?;

        Ok(db)
    }

    /// Get database path based on location
    fn get_db_path(location: &DbLocation) -> Result<PathBuf> {
        match location {
            DbLocation::System => Ok(PathBuf::from("/var/lib/anna/changes.db")),
            DbLocation::User => {
                let base_dir = if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
                    PathBuf::from(xdg_data)
                } else if let Ok(home) = std::env::var("HOME") {
                    PathBuf::from(home).join(".local/share")
                } else {
                    anyhow::bail!("Could not determine user data directory");
                };
                Ok(base_dir.join("anna").join("changes.db"))
            }
            DbLocation::Custom(path) => Ok(path.clone()),
        }
    }

    /// Initialize database schema
    async fn initialize_schema(&self) -> Result<()> {
        let conn = Arc::clone(&self.conn);

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = conn.blocking_lock();

            // Create change_units table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS change_units (
                    id TEXT PRIMARY KEY,
                    label TEXT NOT NULL,
                    user_request TEXT NOT NULL,
                    start_time DATETIME NOT NULL,
                    end_time DATETIME,
                    status TEXT NOT NULL,
                    notes TEXT,
                    metrics_before TEXT,
                    metrics_after TEXT
                )",
                [],
            )?;

            // Create change_actions table - store as JSON for simplicity
            conn.execute(
                "CREATE TABLE IF NOT EXISTS change_actions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    change_unit_id TEXT NOT NULL,
                    action_json TEXT NOT NULL,
                    timestamp DATETIME NOT NULL,
                    success BOOLEAN NOT NULL,
                    description TEXT NOT NULL,
                    FOREIGN KEY (change_unit_id) REFERENCES change_units(id) ON DELETE CASCADE
                )",
                [],
            )?;

            // Create indexes
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_change_units_start
                 ON change_units(start_time)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_change_units_status
                 ON change_units(status)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_change_actions_unit
                 ON change_actions(change_unit_id)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_change_actions_timestamp
                 ON change_actions(timestamp)",
                [],
            )?;

            debug!("Change log schema initialized");
            Ok(())
        })
        .await?
    }

    /// Save a change unit to the database
    pub async fn save_change_unit(&self, unit: &ChangeUnit) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let unit = unit.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = conn.blocking_lock();

            let status_str = match unit.status {
                ChangeStatus::InProgress => "in_progress",
                ChangeStatus::Success => "success",
                ChangeStatus::Partial => "partial",
                ChangeStatus::Failed => "failed",
                ChangeStatus::RolledBack => "rolled_back",
            };

            let notes = if unit.notes.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&unit.notes)?)
            };

            let metrics_before = unit.metrics_before
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;

            let metrics_after = unit.metrics_after
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;

            // Insert or replace change unit
            conn.execute(
                "INSERT OR REPLACE INTO change_units
                 (id, label, user_request, start_time, end_time, status, notes, metrics_before, metrics_after)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    unit.id,
                    unit.label,
                    unit.user_request,
                    unit.start_time.to_rfc3339(),
                    unit.end_time.as_ref().map(|t| t.to_rfc3339()),
                    status_str,
                    notes,
                    metrics_before,
                    metrics_after,
                ],
            )?;

            // Delete existing actions for this unit (in case of update)
            conn.execute(
                "DELETE FROM change_actions WHERE change_unit_id = ?1",
                params![unit.id],
            )?;

            // Insert actions as JSON
            for action in &unit.actions {
                let action_json = serde_json::to_string(&action.action_type)?;

                conn.execute(
                    "INSERT INTO change_actions
                     (change_unit_id, action_json, timestamp, success, description)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        unit.id,
                        action_json,
                        action.timestamp.to_rfc3339(),
                        action.success,
                        action.description,
                    ],
                )?;
            }

            debug!("Saved change unit: {}", unit.id);
            Ok(())
        })
        .await?
    }

    /// Load a change unit by ID
    pub async fn load_change_unit(&self, id: &str) -> Result<Option<ChangeUnit>> {
        let conn = Arc::clone(&self.conn);
        let id = id.to_string();

        tokio::task::spawn_blocking(move || -> Result<Option<ChangeUnit>> {
            let conn = conn.blocking_lock();

            // Load change unit
            let mut stmt = conn.prepare(
                "SELECT id, label, user_request, start_time, end_time, status, notes,
                        metrics_before, metrics_after
                 FROM change_units WHERE id = ?1",
            )?;

            let unit_result = stmt.query_row(params![id], |row| {
                let id: String = row.get(0)?;
                let label: String = row.get(1)?;
                let user_request: String = row.get(2)?;
                let start_time_str: String = row.get(3)?;
                let end_time_str: Option<String> = row.get(4)?;
                let status_str: String = row.get(5)?;
                let notes_str: Option<String> = row.get(6)?;
                let metrics_before_str: Option<String> = row.get(7)?;
                let metrics_after_str: Option<String> = row.get(8)?;

                let start_time = DateTime::parse_from_rfc3339(&start_time_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                let end_time = end_time_str
                    .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
                    .transpose()
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                let status = match status_str.as_str() {
                    "in_progress" => ChangeStatus::InProgress,
                    "success" => ChangeStatus::Success,
                    "partial" => ChangeStatus::Partial,
                    "failed" => ChangeStatus::Failed,
                    "rolled_back" => ChangeStatus::RolledBack,
                    _ => ChangeStatus::InProgress,
                };

                let notes: Vec<String> = notes_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default();

                let metrics_before = metrics_before_str.and_then(|s| serde_json::from_str(&s).ok());

                let metrics_after = metrics_after_str.and_then(|s| serde_json::from_str(&s).ok());

                Ok((
                    id,
                    label,
                    user_request,
                    start_time,
                    end_time,
                    status,
                    notes,
                    metrics_before,
                    metrics_after,
                ))
            });

            let (
                id,
                label,
                user_request,
                start_time,
                end_time,
                status,
                notes,
                metrics_before,
                metrics_after,
            ) = match unit_result {
                Ok(data) => data,
                Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
                Err(e) => return Err(e.into()),
            };

            // Load actions
            let mut stmt = conn.prepare(
                "SELECT action_json, timestamp, success, description
                 FROM change_actions WHERE change_unit_id = ?1 ORDER BY id",
            )?;

            let actions_result: Result<Vec<ChangeAction>, _> = stmt
                .query_map(params![id], |row| {
                    let action_json: String = row.get(0)?;
                    let timestamp_str: String = row.get(1)?;
                    let success: bool = row.get(2)?;
                    let description: String = row.get(3)?;

                    let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                    // Parse action type from JSON (simplified - not storing rollback info)
                    let action_type: ActionType = serde_json::from_str(&action_json)
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                    Ok(ChangeAction {
                        action_type,
                        timestamp,
                        description,
                        success,
                        rollback_info: None, // TODO: Store rollback info separately
                    })
                })?
                .collect();

            let actions = actions_result?;

            let unit = ChangeUnit {
                id,
                label,
                user_request,
                start_time,
                end_time,
                actions,
                status,
                notes,
                metrics_before,
                metrics_after,
            };

            Ok(Some(unit))
        })
        .await?
    }

    /// Get recent change units
    pub async fn get_recent_changes(&self, limit: usize) -> Result<Vec<ChangeUnit>> {
        let conn = Arc::clone(&self.conn);

        let ids = tokio::task::spawn_blocking(move || -> Result<Vec<String>> {
            let conn = conn.blocking_lock();

            let mut stmt = conn.prepare(
                "SELECT id FROM change_units
                 ORDER BY start_time DESC LIMIT ?1",
            )?;

            let ids: Vec<String> = stmt
                .query_map(params![limit as i64], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(ids)
        })
        .await??;

        // Load each change unit
        let mut units = Vec::new();
        for id in ids {
            if let Some(unit) = self.load_change_unit(&id).await? {
                units.push(unit);
            }
        }

        Ok(units)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_save_and_load_change_unit() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_changes.db");
        let location = DbLocation::Custom(db_path);

        let db = ChangeLogDb::open(location).await.unwrap();

        // Create a test change unit
        let mut unit = ChangeUnit::new("test-change", "Test change description");
        unit.add_action(ChangeAction::command(
            "echo",
            vec!["hello".to_string()],
            "Print hello",
        ));
        unit.complete(ChangeStatus::Success);

        // Save it
        db.save_change_unit(&unit).await.unwrap();

        // Load it back
        let loaded = db.load_change_unit(&unit.id).await.unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, unit.id);
        assert_eq!(loaded.label, "test-change");
        assert_eq!(loaded.user_request, "Test change description");
        assert_eq!(loaded.status, ChangeStatus::Success);
        assert_eq!(loaded.actions.len(), 1);
    }

    #[tokio::test]
    async fn test_get_recent_changes() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_recent.db");
        let location = DbLocation::Custom(db_path);

        let db = ChangeLogDb::open(location).await.unwrap();

        // Create multiple change units
        for i in 0..5 {
            let mut unit = ChangeUnit::new(&format!("change-{}", i), &format!("Change {}", i));
            unit.complete(ChangeStatus::Success);
            db.save_change_unit(&unit).await.unwrap();
        }

        // Get recent changes
        let recent = db.get_recent_changes(3).await.unwrap();
        assert_eq!(recent.len(), 3);
    }
}
