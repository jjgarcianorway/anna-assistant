// Action History CRUD Operations
// Phase 3.6: Session Continuity

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Action outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionOutcome {
    Success,
    Failure,
    Cancelled,
}

impl ToString for ActionOutcome {
    fn to_string(&self) -> String {
        match self {
            ActionOutcome::Success => "success".to_string(),
            ActionOutcome::Failure => "failure".to_string(),
            ActionOutcome::Cancelled => "cancelled".to_string(),
        }
    }
}

impl std::str::FromStr for ActionOutcome {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "success" => Ok(ActionOutcome::Success),
            "failure" => Ok(ActionOutcome::Failure),
            "cancelled" => Ok(ActionOutcome::Cancelled),
            _ => anyhow::bail!("Invalid action outcome: {}", s),
        }
    }
}

/// Resource snapshot at time of action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub ram_mb: u64,
    pub cpu_cores: u32,
    pub disk_gb: u64,
}

/// Action history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionHistory {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub action_type: String,
    pub command: String,
    pub outcome: ActionOutcome,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
    pub affected_items: Vec<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub advice_id: Option<String>,
    pub resource_snapshot: Option<ResourceSnapshot>,
}

impl ActionHistory {
    /// Create a new action history entry
    pub fn new(
        action_type: impl Into<String>,
        command: impl Into<String>,
        outcome: ActionOutcome,
    ) -> Self {
        Self {
            id: None,
            timestamp: Utc::now(),
            action_type: action_type.into(),
            command: command.into(),
            outcome,
            duration_ms: None,
            error_message: None,
            affected_items: Vec::new(),
            user_id: None,
            session_id: None,
            advice_id: None,
            resource_snapshot: None,
        }
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Set error message (for failures)
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error_message = Some(error.into());
        self
    }

    /// Set affected items (packages, services, etc.)
    pub fn with_affected_items(mut self, items: Vec<String>) -> Self {
        self.affected_items = items;
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set resource snapshot
    pub fn with_resources(mut self, snapshot: ResourceSnapshot) -> Self {
        self.resource_snapshot = Some(snapshot);
        self
    }

    /// Insert this action into the database
    pub fn insert(&self, conn: &Connection) -> Result<i64> {
        let affected_items_json = serde_json::to_string(&self.affected_items)?;
        let resource_json = self
            .resource_snapshot
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        conn.execute(
            "INSERT INTO action_history (
                timestamp, action_type, command, outcome, duration_ms,
                error_message, affected_items, user_id, session_id,
                advice_id, resource_snapshot
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                self.timestamp.to_rfc3339(),
                self.action_type,
                self.command,
                self.outcome.to_string(),
                self.duration_ms,
                self.error_message,
                affected_items_json,
                self.user_id,
                self.session_id,
                self.advice_id,
                resource_json,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }
}

/// Query for recent actions
pub fn get_recent_actions(conn: &Connection, limit: usize) -> Result<Vec<ActionHistory>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, action_type, command, outcome, duration_ms,
                error_message, affected_items, user_id, session_id,
                advice_id, resource_snapshot
         FROM action_history
         ORDER BY timestamp DESC
         LIMIT ?1",
    )?;

    let actions = stmt
        .query_map(params![limit], |row| {
            let affected_items_json: String = row.get(7)?;
            let affected_items: Vec<String> =
                serde_json::from_str(&affected_items_json).unwrap_or_default();

            let resource_json: Option<String> = row.get(11)?;
            let resource_snapshot = resource_json
                .and_then(|json| serde_json::from_str(&json).ok());

            let timestamp_str: String = row.get(1)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            let outcome_str: String = row.get(4)?;
            let outcome = outcome_str.parse().unwrap_or(ActionOutcome::Failure);

            Ok(ActionHistory {
                id: Some(row.get(0)?),
                timestamp,
                action_type: row.get(2)?,
                command: row.get(3)?,
                outcome,
                duration_ms: row.get(5)?,
                error_message: row.get(6)?,
                affected_items,
                user_id: row.get(8)?,
                session_id: row.get(9)?,
                advice_id: row.get(10)?,
                resource_snapshot,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(actions)
}

/// Query for actions by type
pub fn get_actions_by_type(
    conn: &Connection,
    action_type: &str,
    limit: usize,
) -> Result<Vec<ActionHistory>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, action_type, command, outcome, duration_ms,
                error_message, affected_items, user_id, session_id,
                advice_id, resource_snapshot
         FROM action_history
         WHERE action_type = ?1
         ORDER BY timestamp DESC
         LIMIT ?2",
    )?;

    let actions = stmt
        .query_map(params![action_type, limit], |row| {
            let affected_items_json: String = row.get(7)?;
            let affected_items: Vec<String> =
                serde_json::from_str(&affected_items_json).unwrap_or_default();

            let resource_json: Option<String> = row.get(11)?;
            let resource_snapshot = resource_json
                .and_then(|json| serde_json::from_str(&json).ok());

            let timestamp_str: String = row.get(1)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            let outcome_str: String = row.get(4)?;
            let outcome = outcome_str.parse().unwrap_or(ActionOutcome::Failure);

            Ok(ActionHistory {
                id: Some(row.get(0)?),
                timestamp,
                action_type: row.get(2)?,
                command: row.get(3)?,
                outcome,
                duration_ms: row.get(5)?,
                error_message: row.get(6)?,
                affected_items,
                user_id: row.get(8)?,
                session_id: row.get(9)?,
                advice_id: row.get(10)?,
                resource_snapshot,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(actions)
}

/// Get success rate for a specific action type
pub fn get_success_rate(conn: &Connection, action_type: &str) -> Result<f64> {
    let (total, successful): (i64, i64) = conn.query_row(
        "SELECT
            COUNT(*) as total,
            SUM(CASE WHEN outcome = 'success' THEN 1 ELSE 0 END) as successful
         FROM action_history
         WHERE action_type = ?1",
        params![action_type],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    if total == 0 {
        return Ok(0.0);
    }

    Ok((successful as f64 / total as f64) * 100.0)
}

/// Get total action count
pub fn get_action_count(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM action_history",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Get action count by outcome
pub fn get_action_count_by_outcome(conn: &Connection, outcome: ActionOutcome) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM action_history WHERE outcome = ?1",
        params![outcome.to_string()],
        |row| row.get(0),
    )?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // Create table
        conn.execute(
            "CREATE TABLE action_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                action_type TEXT NOT NULL,
                command TEXT NOT NULL,
                outcome TEXT NOT NULL,
                duration_ms INTEGER,
                error_message TEXT,
                affected_items TEXT,
                user_id TEXT,
                session_id TEXT,
                advice_id TEXT,
                resource_snapshot TEXT
            )",
            [],
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_insert_action() {
        let conn = setup_test_db();

        let action = ActionHistory::new("update", "annactl update", ActionOutcome::Success)
            .with_duration(45000)
            .with_affected_items(vec!["linux".to_string(), "systemd".to_string()]);

        let id = action.insert(&conn).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_recent_actions() {
        let conn = setup_test_db();

        // Insert test data
        let action1 = ActionHistory::new("update", "annactl update", ActionOutcome::Success);
        action1.insert(&conn).unwrap();

        let action2 = ActionHistory::new("health", "annactl health", ActionOutcome::Success);
        action2.insert(&conn).unwrap();

        let actions = get_recent_actions(&conn, 10).unwrap();
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn test_success_rate() {
        let conn = setup_test_db();

        // Insert test data: 3 successes, 1 failure
        ActionHistory::new("update", "annactl update", ActionOutcome::Success)
            .insert(&conn)
            .unwrap();
        ActionHistory::new("update", "annactl update", ActionOutcome::Success)
            .insert(&conn)
            .unwrap();
        ActionHistory::new("update", "annactl update", ActionOutcome::Success)
            .insert(&conn)
            .unwrap();
        ActionHistory::new("update", "annactl update", ActionOutcome::Failure)
            .insert(&conn)
            .unwrap();

        let rate = get_success_rate(&conn, "update").unwrap();
        assert_eq!(rate, 75.0);
    }
}
