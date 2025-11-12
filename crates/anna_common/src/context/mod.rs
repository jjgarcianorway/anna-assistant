// Persistent Context Layer
// Phase 3.6: Session Continuity
//
// Anna's memory system - tracks actions, system state, and learns over time.
// All data stays local, no cloud sync, privacy-first design.

pub mod actions;
pub mod db;

// Re-export commonly used types
pub use actions::{ActionHistory, ActionOutcome, ResourceSnapshot};
pub use db::{ContextDb, DbLocation};

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
        .execute(move |conn| actions::get_action_count(conn))
        .await?;

    Ok(count)
}

/// Run database maintenance (cleanup old entries)
pub async fn maintenance() -> Result<()> {
    let db = db().ok_or_else(|| anyhow::anyhow!("Context database not initialized"))?;
    db.maintenance().await
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
