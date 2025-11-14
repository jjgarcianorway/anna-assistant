// Database Connection Management for Persistent Context
// Phase 3.6: Session Continuity

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Database location based on execution mode
#[derive(Debug, Clone)]
pub enum DbLocation {
    /// System mode: /var/lib/anna/context.db
    System,
    /// User mode: $XDG_DATA_HOME/anna/context.db or ~/.local/share/anna/context.db
    User,
    /// Custom path for testing
    Custom(PathBuf),
}

impl DbLocation {
    pub fn path(&self) -> Result<PathBuf> {
        match self {
            DbLocation::System => Ok(PathBuf::from("/var/lib/anna/context.db")),
            DbLocation::User => {
                // Try XDG_DATA_HOME first, fall back to ~/.local/share
                let base_dir = if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
                    PathBuf::from(xdg_data)
                } else if let Ok(home) = std::env::var("HOME") {
                    PathBuf::from(home).join(".local/share")
                } else {
                    anyhow::bail!("Could not determine user data directory");
                };
                Ok(base_dir.join("anna").join("context.db"))
            }
            DbLocation::Custom(path) => Ok(path.clone()),
        }
    }

    /// Determine location based on current user privileges
    pub fn auto_detect() -> Self {
        // Check if running as root
        #[cfg(unix)]
        {
            use libc::{geteuid};
            if unsafe { geteuid() } == 0 {
                return DbLocation::System;
            }
        }
        DbLocation::User
    }
}

/// SQLite connection pool (simplified - single connection with mutex for now)
pub struct ContextDb {
    conn: Arc<Mutex<Connection>>,
    location: DbLocation,
}

impl ContextDb {
    /// Open or create database at the specified location
    pub async fn open(location: DbLocation) -> Result<Self> {
        let db_path = location.path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create database directory")?;
        }

        info!("Opening context database at: {}", db_path.display());

        // Open connection in blocking context
        let conn = tokio::task::spawn_blocking(move || -> Result<Connection> {
            let conn = Connection::open(&db_path)
                .context("Failed to open SQLite database")?;

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
            location: location.clone(),
        };

        // Initialize schema
        db.initialize_schema().await?;

        Ok(db)
    }

    /// Initialize database schema
    async fn initialize_schema(&self) -> Result<()> {
        let conn = Arc::clone(&self.conn);

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = conn.blocking_lock();
            // Create action_history table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS action_history (
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
            )?;

            // Create indexes for action_history
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_action_timestamp
                 ON action_history(timestamp)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_action_type
                 ON action_history(action_type)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_action_outcome
                 ON action_history(outcome)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_session
                 ON action_history(session_id)",
                [],
            )?;

            // Create system_state_log table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS system_state_log (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    state TEXT NOT NULL,
                    total_memory_mb INTEGER,
                    available_memory_mb INTEGER,
                    cpu_cores INTEGER,
                    disk_total_gb INTEGER,
                    disk_available_gb INTEGER,
                    uptime_seconds INTEGER,
                    monitoring_mode TEXT,
                    is_constrained BOOLEAN,
                    virtualization TEXT,
                    session_type TEXT,
                    failed_probes TEXT,
                    package_count INTEGER,
                    outdated_count INTEGER,
                    boot_id TEXT
                )",
                [],
            )?;

            // Create indexes for system_state_log
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_state_timestamp
                 ON system_state_log(timestamp)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_boot
                 ON system_state_log(boot_id)",
                [],
            )?;

            // Create user_preferences table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS user_preferences (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    value_type TEXT NOT NULL,
                    set_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    set_by TEXT,
                    description TEXT
                )",
                [],
            )?;

            // Create command_usage table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS command_usage (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    command TEXT NOT NULL,
                    subcommand TEXT,
                    flags TEXT,
                    exit_code INTEGER,
                    was_helpful BOOLEAN,
                    led_to_action BOOLEAN,
                    context_state TEXT
                )",
                [],
            )?;

            // Create indexes for command_usage
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_command
                 ON command_usage(command)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_command_timestamp
                 ON command_usage(timestamp)",
                [],
            )?;

            // Create learning_patterns table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS learning_patterns (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    pattern_type TEXT NOT NULL,
                    pattern_data TEXT NOT NULL,
                    confidence REAL,
                    first_detected DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    last_confirmed DATETIME,
                    confirmation_count INTEGER DEFAULT 1,
                    actionable BOOLEAN DEFAULT FALSE,
                    recommended_action TEXT
                )",
                [],
            )?;

            // Create session_metadata table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS session_metadata (
                    session_id TEXT PRIMARY KEY,
                    start_time DATETIME NOT NULL,
                    end_time DATETIME,
                    user_id TEXT,
                    session_type TEXT,
                    commands_count INTEGER DEFAULT 0,
                    actions_count INTEGER DEFAULT 0,
                    boot_id TEXT
                )",
                [],
            )?;

            // Create issue_tracking table (Phase 4.6: Noise Control)
            conn.execute(
                "CREATE TABLE IF NOT EXISTS issue_tracking (
                    issue_key TEXT PRIMARY KEY,
                    first_seen DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    last_seen DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    last_shown DATETIME,
                    times_shown INTEGER DEFAULT 0,
                    times_ignored INTEGER DEFAULT 0,
                    last_repair_attempt DATETIME,
                    repair_success BOOLEAN,
                    severity TEXT NOT NULL,
                    last_details TEXT
                )",
                [],
            )?;

            // Create indexes for issue_tracking
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_issue_last_seen
                 ON issue_tracking(last_seen)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_issue_severity
                 ON issue_tracking(severity)",
                [],
            )?;

            // Create issue_decisions table (Phase 4.9: User Control)
            conn.execute(
                "CREATE TABLE IF NOT EXISTS issue_decisions (
                    issue_key TEXT PRIMARY KEY,
                    decision_type TEXT NOT NULL,
                    snooze_until DATETIME,
                    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;

            // Create index for issue_decisions snooze_until
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_decision_snooze
                 ON issue_decisions(snooze_until)",
                [],
            )?;

            // Create repair_history table (Phase 5.1: Safety Rails)
            conn.execute(
                "CREATE TABLE IF NOT EXISTS repair_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    issue_key TEXT NOT NULL,
                    repair_action_id TEXT NOT NULL,
                    result TEXT NOT NULL,
                    summary TEXT NOT NULL
                )",
                [],
            )?;

            // Create index for repair_history timestamp (for recent queries)
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_repair_timestamp
                 ON repair_history(timestamp DESC)",
                [],
            )?;

            // Create observations table (Phase 5.2: Observer Layer)
            conn.execute(
                "CREATE TABLE IF NOT EXISTS observations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    issue_key TEXT NOT NULL,
                    severity INTEGER NOT NULL,
                    profile TEXT NOT NULL,
                    visible INTEGER NOT NULL,
                    decision TEXT
                )",
                [],
            )?;

            // Create indexes for observations queries
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_observations_timestamp
                 ON observations(timestamp DESC)",
                [],
            )?;

            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_observations_issue
                 ON observations(issue_key, timestamp DESC)",
                [],
            )?;

            debug!("Database schema initialized successfully");
            Ok(())
        })
        .await??;

        info!("Context database schema ready");
        Ok(())
    }

    /// Get a reference to the connection (for use in spawn_blocking)
    pub fn conn(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    /// Execute a query in a blocking context
    pub async fn execute<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Connection) -> Result<R> + Send + 'static,
        R: Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            f(&conn)
        })
        .await?
    }

    /// Get database location
    pub fn location(&self) -> &DbLocation {
        &self.location
    }

    /// Perform database maintenance (cleanup old entries)
    pub async fn maintenance(&self) -> Result<()> {
        info!("Running database maintenance...");

        let conn = Arc::clone(&self.conn);

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = conn.blocking_lock();
            // Clean up old action_history entries (keep last 10,000)
            let deleted_actions = conn.execute(
                "DELETE FROM action_history
                 WHERE id NOT IN (
                     SELECT id FROM action_history
                     ORDER BY timestamp DESC
                     LIMIT 10000
                 )",
                [],
            )?;

            if deleted_actions > 0 {
                info!("Deleted {} old action_history entries", deleted_actions);
            }

            // Clean up old system_state_log entries (keep last 50,000)
            let deleted_states = conn.execute(
                "DELETE FROM system_state_log
                 WHERE id NOT IN (
                     SELECT id FROM system_state_log
                     ORDER BY timestamp DESC
                     LIMIT 50000
                 )",
                [],
            )?;

            if deleted_states > 0 {
                info!("Deleted {} old system_state_log entries", deleted_states);
            }

            // Clean up old command_usage entries (keep last 5,000)
            let deleted_commands = conn.execute(
                "DELETE FROM command_usage
                 WHERE id NOT IN (
                     SELECT id FROM command_usage
                     ORDER BY timestamp DESC
                     LIMIT 5000
                 )",
                [],
            )?;

            if deleted_commands > 0 {
                info!("Deleted {} old command_usage entries", deleted_commands);
            }

            // Vacuum database to reclaim space
            conn.execute("VACUUM", [])?;

            info!("Database maintenance complete");
            Ok(())
        })
        .await??;

        Ok(())
    }

    /// Save language configuration
    pub async fn save_language_config(&self, config: &crate::language::LanguageConfig) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let user_lang = config.user_language.map(|l| l.code().to_string());
        let system_lang = config.system_language.map(|l| l.code().to_string());

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = conn.blocking_lock();

            // Store user language preference
            if let Some(lang) = user_lang {
                conn.execute(
                    "INSERT OR REPLACE INTO user_preferences (key, value, value_type, description)
                     VALUES (?1, ?2, ?3, ?4)",
                    ["language", &lang, "string", "User's preferred language"],
                )?;
            } else {
                // Remove user preference to fall back to system
                conn.execute(
                    "DELETE FROM user_preferences WHERE key = ?1",
                    ["language"],
                )?;
            }

            // Store detected system language for reference
            if let Some(lang) = system_lang {
                conn.execute(
                    "INSERT OR REPLACE INTO user_preferences (key, value, value_type, description)
                     VALUES (?1, ?2, ?3, ?4)",
                    ["system_language", &lang, "string", "Auto-detected system language"],
                )?;
            }

            Ok(())
        })
        .await??;

        Ok(())
    }

    /// Save LLM configuration
    pub async fn save_llm_config(&self, config: &crate::llm::LlmConfig) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let config_json = serde_json::to_string(config)?;

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = conn.blocking_lock();

            conn.execute(
                "INSERT OR REPLACE INTO user_preferences (key, value, value_type, updated_at)
                 VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
                params!["llm_config", config_json, "json"],
            )?;

            debug!("Saved LLM configuration");
            Ok(())
        })
        .await??;

        Ok(())
    }

    /// Load LLM configuration
    pub async fn load_llm_config(&self) -> Result<crate::llm::LlmConfig> {
        let conn = Arc::clone(&self.conn);

        tokio::task::spawn_blocking(move || -> Result<crate::llm::LlmConfig> {
            let conn = conn.blocking_lock();

            let result: Result<String, _> = conn.query_row(
                "SELECT value FROM user_preferences WHERE key = ?1",
                params!["llm_config"],
                |row| row.get(0),
            );

            match result {
                Ok(json) => {
                    let config: crate::llm::LlmConfig = serde_json::from_str(&json)?;
                    Ok(config)
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    // No config saved yet, return default
                    Ok(crate::llm::LlmConfig::default())
                }
                Err(e) => Err(e.into()),
            }
        })
        .await?
    }

    /// Load language configuration
    pub async fn load_language_config(&self) -> Result<crate::language::LanguageConfig> {
        let conn = Arc::clone(&self.conn);

        let (user_lang_str, system_lang_str) = tokio::task::spawn_blocking(move || -> Result<(Option<String>, Option<String>)> {
            let conn = conn.blocking_lock();

            // Load user language preference
            let user_lang = conn
                .query_row(
                    "SELECT value FROM user_preferences WHERE key = ?1",
                    ["language"],
                    |row| row.get::<_, String>(0),
                )
                .ok();

            // Load system language
            let system_lang = conn
                .query_row(
                    "SELECT value FROM user_preferences WHERE key = ?1",
                    ["system_language"],
                    |row| row.get::<_, String>(0),
                )
                .ok();

            Ok((user_lang, system_lang))
        })
        .await??;

        let mut config = crate::language::LanguageConfig::new();

        // Parse user language
        if let Some(lang_str) = user_lang_str {
            config.user_language = crate::language::Language::from_str(&lang_str);
        }

        // Use stored system language if available, otherwise detect fresh
        if let Some(lang_str) = system_lang_str {
            config.system_language = crate::language::Language::from_str(&lang_str);
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_db_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let location = DbLocation::Custom(db_path.clone());

        let db = ContextDb::open(location).await.unwrap();

        // Verify database file was created
        assert!(db_path.exists());

        // Verify we can execute queries
        let result = db
            .execute(|conn| {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                    [],
                    |row| row.get(0),
                )?;
                Ok(count)
            })
            .await
            .unwrap();

        // Should have created 7 tables (Phase 4.9 added issue_decisions)
        assert!(result >= 7);
    }

    #[tokio::test]
    async fn test_location_auto_detect() {
        let location = DbLocation::auto_detect();
        // Should detect based on privileges
        match location {
            DbLocation::System | DbLocation::User => {
                // Either is valid
            }
            DbLocation::Custom(_) => panic!("Should not be custom"),
        }
    }
}
