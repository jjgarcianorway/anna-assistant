// Persistent Context Layer
// Phase 3.6: Session Continuity
//
// Anna's memory system - tracks actions, system state, and learns over time.
// All data stays local, no cloud sync, privacy-first design.

pub mod actions;
pub mod db;
pub mod historian;
pub mod noise_control;

// Re-export commonly used types
pub use actions::{ActionHistory, ActionOutcome, ResourceSnapshot};
pub use db::{ContextDb, DbLocation};
pub use noise_control::{
    apply_issue_decisions, apply_visibility_hints, clear_issue_decision, filter_issues_by_noise_control,
    get_issue_decision, get_issue_state, mark_issue_ignored, mark_issue_repaired, mark_issue_shown,
    set_issue_acknowledged, set_issue_snoozed, update_issue_state, DecisionType, IssueDecision,
    IssueState, NoiseControlConfig,
};
pub use historian::*;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::{debug, info};

/// Global context database instance
static CONTEXT_DB: OnceCell<Arc<ContextDb>> = OnceCell::const_new();

/// Initialize the global context database
pub async fn initialize() -> Result<()> {
    let location = DbLocation::auto_detect();
    let db = ContextDb::open(location).await?;
    CONTEXT_DB
        .set(Arc::new(db))
        .map_err(|_| anyhow::anyhow!("Context database already initialized"))?;
    info!("Context database initialized");
    Ok(())
}

/// Initialize with a custom location (for testing)
pub async fn initialize_with_location(location: DbLocation) -> Result<()> {
    let db = ContextDb::open(location).await?;
    CONTEXT_DB
        .set(Arc::new(db))
        .map_err(|_| anyhow::anyhow!("Context database already initialized"))?;
    info!("Context database initialized (custom location)");
    Ok(())
}

/// Get the global context database
pub fn db() -> Option<Arc<ContextDb>> {
    CONTEXT_DB.get().cloned()
}

/// Record an action in the history
pub async fn record_action(
    action_type: impl Into<String>,
    command: impl Into<String>,
    outcome: ActionOutcome,
    duration_ms: Option<u64>,
    error: Option<String>,
    affected_items: Vec<String>,
) -> Result<i64> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;

    let mut action = ActionHistory::new(action_type, command, outcome);

    if let Some(duration) = duration_ms {
        action = action.with_duration(duration);
    }

    if let Some(err) = error {
        action = action.with_error(err);
    }

    if !affected_items.is_empty() {
        action = action.with_affected_items(affected_items);
    }

    let id = db
        .execute(move |conn| action.insert(conn))
        .await?;

    debug!("Recorded action with ID: {}", id);
    Ok(id)
}

/// Get recent actions
pub async fn get_recent_actions(limit: usize) -> Result<Vec<ActionHistory>> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;

    let actions = db
        .execute(move |conn| actions::get_recent_actions(conn, limit))
        .await?;

    Ok(actions)
}

/// Get actions by type
pub async fn get_actions_by_type(action_type: String, limit: usize) -> Result<Vec<ActionHistory>> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;

    let actions = db
        .execute(move |conn| actions::get_actions_by_type(conn, &action_type, limit))
        .await?;

    Ok(actions)
}

/// Get success rate for an action type
pub async fn get_success_rate(action_type: String) -> Result<f64> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;

    let rate = db
        .execute(move |conn| actions::get_success_rate(conn, &action_type))
        .await?;

    Ok(rate)
}

/// Get total action count
pub async fn get_action_count() -> Result<i64> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;

    let count = db
        .execute(actions::get_action_count)
        .await?;

    Ok(count)
}

/// Run database maintenance (cleanup old entries)
pub async fn maintenance() -> Result<()> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;
    db.maintenance().await
}

/// Ensure context database is initialized and ready to use (Phase 4.7)
///
/// This is idempotent and safe to call on every run. It will:
/// - Initialize the database if not already initialized
/// - Create tables if they don't exist
/// - Return quickly if already initialized
pub async fn ensure_initialized() -> Result<()> {
    if db().is_some() {
        // Already initialized
        return Ok(());
    }

    // Initialize with auto-detected location
    initialize().await
}

// Phase 5.1: Repair History Functions

/// Record a repair action in history
pub async fn record_repair(
    issue_key: impl Into<String>,
    repair_action_id: impl Into<String>,
    result: impl Into<String>,
    summary: impl Into<String>,
) -> Result<i64> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;

    let issue_key = issue_key.into();
    let repair_action_id = repair_action_id.into();
    let result = result.into();
    let summary = summary.into();

    let id = db
        .execute(move |conn| {
            conn.execute(
                "INSERT INTO repair_history (issue_key, repair_action_id, result, summary)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![&issue_key, &repair_action_id, &result, &summary],
            )?;
            Ok(conn.last_insert_rowid())
        })
        .await?;

    debug!("Recorded repair with ID: {}", id);
    Ok(id)
}

/// Repair history entry
#[derive(Debug, Clone, serde::Serialize)]
pub struct RepairHistoryEntry {
    pub id: i64,
    pub timestamp: String,
    pub issue_key: String,
    pub repair_action_id: String,
    pub result: String,
    pub summary: String,
}

/// Get recent repairs from history
pub async fn get_recent_repairs(limit: usize) -> Result<Vec<RepairHistoryEntry>> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;

    let entries = db
        .execute(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, timestamp, issue_key, repair_action_id, result, summary
                 FROM repair_history
                 ORDER BY timestamp DESC
                 LIMIT ?1",
            )?;

            let rows = stmt.query_map([limit], |row| {
                Ok(RepairHistoryEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    issue_key: row.get(2)?,
                    repair_action_id: row.get(3)?,
                    result: row.get(4)?,
                    summary: row.get(5)?,
                })
            })?;

            let mut entries = Vec::new();
            for row in rows {
                entries.push(row?);
            }
            Ok(entries)
        })
        .await?;

    Ok(entries)
}

// Phase 5.2: Observation Recording Functions

/// Observation entry for time-series analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct Observation {
    pub id: i64,
    pub timestamp: String,
    pub issue_key: String,
    pub severity: i32,
    pub profile: String,
    pub visible: bool,
    pub decision: Option<String>,
}

/// Record an observation for behavioral analysis
pub async fn record_observation(
    issue_key: impl Into<String>,
    severity: i32,
    profile: impl Into<String>,
    visible: bool,
    decision: Option<String>,
) -> Result<i64> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;

    let issue_key = issue_key.into();
    let profile = profile.into();

    let id = db
        .execute(move |conn| {
            conn.execute(
                "INSERT INTO observations (issue_key, severity, profile, visible, decision)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![&issue_key, severity, &profile, visible as i32, &decision],
            )?;
            Ok(conn.last_insert_rowid())
        })
        .await?;

    debug!("Recorded observation with ID: {}", id);
    Ok(id)
}

/// Get observations for an issue within a time window
pub async fn get_observations(
    issue_key: &str,
    days_back: i64,
) -> Result<Vec<Observation>> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;

    let issue_key = issue_key.to_string();

    let observations = db
        .execute(move |conn| {
            let cutoff = chrono::Utc::now() - chrono::Duration::days(days_back);
            let cutoff_str = cutoff.to_rfc3339();

            let mut stmt = conn.prepare(
                "SELECT id, timestamp, issue_key, severity, profile, visible, decision
                 FROM observations
                 WHERE issue_key = ?1 AND timestamp >= ?2
                 ORDER BY timestamp DESC",
            )?;

            let rows = stmt.query_map([&issue_key, &cutoff_str], |row| {
                Ok(Observation {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    issue_key: row.get(2)?,
                    severity: row.get(3)?,
                    profile: row.get(4)?,
                    visible: row.get::<_, i32>(5)? != 0,
                    decision: row.get(6)?,
                })
            })?;

            let mut observations = Vec::new();
            for row in rows {
                observations.push(row?);
            }
            Ok(observations)
        })
        .await?;

    Ok(observations)
}

/// Get all observations within a time window (for pattern analysis)
pub async fn get_all_observations(days_back: i64) -> Result<Vec<Observation>> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;

    let observations = db
        .execute(move |conn| {
            let cutoff = chrono::Utc::now() - chrono::Duration::days(days_back);
            let cutoff_str = cutoff.to_rfc3339();

            let mut stmt = conn.prepare(
                "SELECT id, timestamp, issue_key, severity, profile, visible, decision
                 FROM observations
                 WHERE timestamp >= ?1
                 ORDER BY timestamp DESC",
            )?;

            let rows = stmt.query_map([&cutoff_str], |row| {
                Ok(Observation {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    issue_key: row.get(2)?,
                    severity: row.get(3)?,
                    profile: row.get(4)?,
                    visible: row.get::<_, i32>(5)? != 0,
                    decision: row.get(6)?,
                })
            })?;

            let mut observations = Vec::new();
            for row in rows {
                observations.push(row?);
            }
            Ok(observations)
        })
        .await?;

    Ok(observations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_direct_db_usage() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let location = DbLocation::Custom(db_path);

        // Use database directly without global singleton
        let db = ContextDb::open(location).await.unwrap();

        // Test recording an action
        let action = ActionHistory::new("update", "annactl update", ActionOutcome::Success)
            .with_duration(45000)
            .with_affected_items(vec!["linux".to_string(), "systemd".to_string()]);

        let id = db
            .execute(move |conn| action.insert(conn))
            .await
            .unwrap();

        assert!(id > 0);

        // Test retrieving actions
        let actions = db
            .execute(|conn| actions::get_recent_actions(conn, 10))
            .await
            .unwrap();

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_type, "update");
        assert_eq!(actions[0].command, "annactl update");
    }

    #[tokio::test]
    async fn test_direct_success_rate() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let location = DbLocation::Custom(db_path);

        let db = ContextDb::open(location).await.unwrap();

        // Record 3 successful actions and 1 failure
        for _ in 0..3 {
            let action =
                ActionHistory::new("update", "annactl update", ActionOutcome::Success);
            db.execute(move |conn| action.insert(conn)).await.unwrap();
        }

        let action = ActionHistory::new("update", "annactl update", ActionOutcome::Failure)
            .with_error("Network error");
        db.execute(move |conn| action.insert(conn)).await.unwrap();

        // Check success rate
        let rate = db
            .execute(|conn| actions::get_success_rate(conn, "update"))
            .await
            .unwrap();

        assert_eq!(rate, 75.0);
    }
}
