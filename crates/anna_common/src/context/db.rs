// Database Connection Management for Persistent Context
// Phase 3.6: Session Continuity

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
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
            use libc::geteuid;
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
            let conn = Connection::open(&db_path).context("Failed to open SQLite database")?;

            // Enable WAL mode for better concurrency
            conn.pragma_update(None, "journal_mode", "WAL")
                .context("Failed to enable WAL mode")?;

            // Set WAL synchronous mode for better cross-user compatibility
            conn.pragma_update(None, "synchronous", "NORMAL")
                .context("Failed to set synchronous mode")?;

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

            // Historian: timeline of upgrades/config changes
            conn.execute(
                "CREATE TABLE IF NOT EXISTS timeline_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    kind TEXT NOT NULL,
                    from_version TEXT,
                    to_version TEXT,
                    details TEXT,
                    outcome TEXT
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_timeline_ts
                 ON timeline_events(ts DESC)",
                [],
            )?;

            // Historian: boot/shutdown tracking
            conn.execute(
                "CREATE TABLE IF NOT EXISTS boot_sessions (
                    boot_id TEXT PRIMARY KEY,
                    ts_start DATETIME NOT NULL,
                    ts_end DATETIME,
                    goal TEXT,
                    time_to_goal_ms INTEGER,
                    degraded INTEGER,
                    fsck_ran INTEGER,
                    fsck_duration_ms INTEGER,
                    shutdown_duration_ms INTEGER,
                    early_kernel_errors_count INTEGER,
                    boot_health_score INTEGER
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS boot_unit_slowlog (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    boot_id TEXT NOT NULL,
                    unit TEXT NOT NULL,
                    duration_ms INTEGER,
                    state TEXT,
                    FOREIGN KEY(boot_id) REFERENCES boot_sessions(boot_id)
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_boot_sessions_ts
                 ON boot_sessions(ts_start DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_boot_units_boot
                 ON boot_unit_slowlog(boot_id)",
                [],
            )?;

            // Historian: CPU usage windows
            conn.execute(
                "CREATE TABLE IF NOT EXISTS cpu_windows (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_start DATETIME NOT NULL,
                    window_end DATETIME NOT NULL,
                    avg_util_per_core TEXT,
                    peak_util_per_core TEXT,
                    idle_background_load REAL,
                    throttling_events INTEGER,
                    spikes_over_100pct INTEGER
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS cpu_top_processes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_start DATETIME NOT NULL,
                    process_name TEXT NOT NULL,
                    cpu_time_seconds REAL
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_cpu_windows_start
                 ON cpu_windows(window_start DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_cpu_top_window
                 ON cpu_top_processes(window_start DESC)",
                [],
            )?;

            // Historian: Memory/Swap
            conn.execute(
                "CREATE TABLE IF NOT EXISTS mem_windows (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_start DATETIME NOT NULL,
                    avg_ram_mb REAL,
                    peak_ram_mb REAL,
                    swap_used_mb_avg REAL,
                    swap_used_mb_peak REAL
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS oom_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts DATETIME NOT NULL,
                    process_name TEXT,
                    victim INTEGER,
                    rss_mb REAL
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_mem_windows_start
                 ON mem_windows(window_start DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_oom_ts
                 ON oom_events(ts DESC)",
                [],
            )?;

            // Historian: Disk capacity, growth, I/O
            conn.execute(
                "CREATE TABLE IF NOT EXISTS fs_capacity_daily (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts DATETIME NOT NULL,
                    mountpoint TEXT NOT NULL,
                    total_gb REAL,
                    free_gb REAL
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS fs_growth (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_start DATETIME NOT NULL,
                    mountpoint TEXT NOT NULL,
                    path_prefix TEXT,
                    delta_gb REAL,
                    contributors TEXT
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS fs_io_windows (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_start DATETIME NOT NULL,
                    mountpoint TEXT NOT NULL,
                    read_mb_s_avg REAL,
                    write_mb_s_avg REAL,
                    latency_ms_p50 REAL,
                    latency_ms_p95 REAL,
                    queue_depth_avg REAL,
                    io_errors INTEGER
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_fs_capacity_ts
                 ON fs_capacity_daily(ts DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_fs_growth_window
                 ON fs_growth(window_start DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_fs_io_window
                 ON fs_io_windows(window_start DESC)",
                [],
            )?;

            // Historian: Network quality
            conn.execute(
                "CREATE TABLE IF NOT EXISTS net_windows (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_start DATETIME NOT NULL,
                    iface TEXT,
                    target TEXT,
                    latency_ms_avg REAL,
                    latency_ms_p95 REAL,
                    packet_loss_pct REAL,
                    dns_failures INTEGER,
                    dhcp_failures INTEGER
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS net_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts DATETIME NOT NULL,
                    iface TEXT,
                    event TEXT
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_net_windows_start
                 ON net_windows(window_start DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_net_events_ts
                 ON net_events(ts DESC)",
                [],
            )?;

            // Historian: Service reliability
            conn.execute(
                "CREATE TABLE IF NOT EXISTS service_health (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts DATETIME NOT NULL,
                    service TEXT NOT NULL,
                    state TEXT,
                    time_in_failed_ms INTEGER,
                    avg_start_time_ms INTEGER,
                    config_change_ts DATETIME
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS service_restarts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts DATETIME NOT NULL,
                    service TEXT NOT NULL,
                    reason TEXT
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_service_health_ts
                 ON service_health(ts DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_service_restart_ts
                 ON service_restarts(ts DESC)",
                [],
            )?;

            // Historian: Error/Warning signatures
            conn.execute(
                "CREATE TABLE IF NOT EXISTS log_window_counts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_start DATETIME NOT NULL,
                    errors INTEGER,
                    warnings INTEGER,
                    criticals INTEGER,
                    source TEXT
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS log_signatures (
                    signature_hash TEXT PRIMARY KEY,
                    first_seen DATETIME NOT NULL,
                    last_seen DATETIME NOT NULL,
                    count INTEGER NOT NULL,
                    source TEXT,
                    sample_message TEXT,
                    status TEXT
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_log_window_start
                 ON log_window_counts(window_start DESC)",
                [],
            )?;

            // Historian: Baselines and deltas
            conn.execute(
                "CREATE TABLE IF NOT EXISTS baselines (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    label TEXT NOT NULL,
                    created_at DATETIME NOT NULL,
                    metrics TEXT
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS baseline_deltas (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts DATETIME NOT NULL,
                    baseline_id INTEGER NOT NULL,
                    metric TEXT NOT NULL,
                    delta_pct REAL,
                    context TEXT,
                    impact_score REAL,
                    FOREIGN KEY(baseline_id) REFERENCES baselines(id)
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_baselines_created
                 ON baselines(created_at DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_baseline_deltas_ts
                 ON baseline_deltas(ts DESC)",
                [],
            )?;

            // Historian: User behavior (opt-in)
            conn.execute(
                "CREATE TABLE IF NOT EXISTS usage_patterns (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_start DATETIME NOT NULL,
                    active_hours_detected INTEGER,
                    heavy_load_minutes INTEGER,
                    low_load_minutes INTEGER,
                    package_updates_count INTEGER,
                    anna_runs INTEGER
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS app_usage (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_start DATETIME NOT NULL,
                    app TEXT NOT NULL,
                    minutes_active INTEGER,
                    category TEXT
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_usage_patterns_window
                 ON usage_patterns(window_start DESC)",
                [],
            )?;

            // Historian: LLM stats
            conn.execute(
                "CREATE TABLE IF NOT EXISTS llm_usage_windows (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_start DATETIME NOT NULL,
                    model_name TEXT,
                    total_calls INTEGER,
                    success_calls INTEGER,
                    latency_ms_avg REAL,
                    latency_ms_p95 REAL,
                    backend_rss_mb REAL,
                    gpu_util_pct_avg REAL,
                    cpu_util_pct_avg REAL,
                    failed_calls INTEGER,
                    cost_estimate REAL,
                    delta_latency_ms_avg REAL,
                    delta_latency_ms_p95 REAL
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS llm_model_changes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts DATETIME NOT NULL,
                    model_name TEXT NOT NULL,
                    reason TEXT,
                    hw_requirements TEXT,
                    notes TEXT
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_llm_usage_window
                 ON llm_usage_windows(window_start DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_llm_model_changes
                 ON llm_model_changes(ts DESC)",
                [],
            )?;

            // Historian: Repair metrics
            conn.execute(
                "CREATE TABLE IF NOT EXISTS repair_metrics (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    repair_id INTEGER NOT NULL,
                    metric TEXT NOT NULL,
                    before_value REAL,
                    after_value REAL,
                    units TEXT,
                    FOREIGN KEY(repair_id) REFERENCES repair_history(id)
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_repair_metrics_repair
                 ON repair_metrics(repair_id)",
                [],
            )?;

            // Historian: Synthesized scores
            conn.execute(
                "CREATE TABLE IF NOT EXISTS health_scores (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts DATETIME NOT NULL,
                    stability_score INTEGER,
                    performance_score INTEGER,
                    noise_score INTEGER,
                    trend_stability TEXT,
                    trend_performance TEXT,
                    trend_noise TEXT,
                    last_regression TEXT,
                    last_regression_cause TEXT,
                    last_improvement TEXT,
                    last_improvement_cause TEXT
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_health_scores_ts
                 ON health_scores(ts DESC)",
                [],
            )?;

            // Beta.84: File-level indexing - track every file on the system
            conn.execute(
                "CREATE TABLE IF NOT EXISTS file_index (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT NOT NULL,
                    size_bytes INTEGER NOT NULL,
                    mtime DATETIME NOT NULL,
                    owner_uid INTEGER NOT NULL,
                    owner_gid INTEGER NOT NULL,
                    permissions INTEGER NOT NULL,
                    file_type TEXT NOT NULL,
                    indexed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;
            conn.execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_file_index_path
                 ON file_index(path)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_file_index_mtime
                 ON file_index(mtime DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_file_index_size
                 ON file_index(size_bytes DESC)",
                [],
            )?;

            // Beta.84: File changes tracking
            conn.execute(
                "CREATE TABLE IF NOT EXISTS file_changes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT NOT NULL,
                    change_type TEXT NOT NULL,
                    old_size INTEGER,
                    new_size INTEGER,
                    old_mtime DATETIME,
                    new_mtime DATETIME,
                    old_permissions INTEGER,
                    new_permissions INTEGER,
                    detected_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_file_changes_detected
                 ON file_changes(detected_at DESC)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_file_changes_path
                 ON file_changes(path)",
                [],
            )?;

            // Beta.86: Personality system (16 traits stored in database)
            conn.execute(
                "CREATE TABLE IF NOT EXISTS personality (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    trait_key TEXT NOT NULL UNIQUE,
                    trait_name TEXT NOT NULL,
                    value INTEGER NOT NULL CHECK (value >= 0 AND value <= 10),
                    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_personality_key
                 ON personality(trait_key)",
                [],
            )?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_personality_updated
                 ON personality(updated_at DESC)",
                [],
            )?;

            debug!("Database schema initialized successfully");
            Ok(())
        })
        .await??;

        // Run migrations for schema changes
        self.run_migrations().await?;

        info!("Context database schema ready");
        Ok(())
    }

    /// Run database migrations for backwards compatibility
    async fn run_migrations(&self) -> Result<()> {
        let conn = Arc::clone(&self.conn);

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = conn.blocking_lock();

            // Migration 1: Add updated_at column to user_preferences (v5.5.0)
            info!("Checking database migrations...");

            // Check if user_preferences table exists first
            let table_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='user_preferences'",
                    [],
                    |row| {
                        let count: i64 = row.get(0)?;
                        Ok(count > 0)
                    },
                )
                .context("Failed to check if user_preferences table exists")?;

            if !table_exists {
                debug!("user_preferences table doesn't exist yet, skipping migration");
                return Ok(());
            }

            // Check if column exists
            let column_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('user_preferences') WHERE name='updated_at'",
                    [],
                    |row| {
                        let count: i64 = row.get(0)?;
                        Ok(count > 0)
                    },
                )
                .context("Failed to check if updated_at column exists")?;

            if !column_exists {
                info!("Running migration: Adding updated_at column to user_preferences");

                // SQLite doesn't allow non-constant defaults, so use NULL
                conn.execute(
                    "ALTER TABLE user_preferences ADD COLUMN updated_at DATETIME",
                    [],
                )
                .context("Failed to add updated_at column")?;

                // Copy set_at values to updated_at for existing rows
                conn.execute(
                    "UPDATE user_preferences SET updated_at = set_at WHERE updated_at IS NULL",
                    [],
                )
                .context("Failed to backfill updated_at values")?;

                info!("✓ Migration complete: Added updated_at column to user_preferences");
            } else {
                debug!("Migration not needed: updated_at column already exists");
            }

            // Migration 2: Add LLM usage window columns (model_name, totals, cost, deltas)
            let llm_table_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='llm_usage_windows'",
                    [],
                    |row| {
                        let count: i64 = row.get(0)?;
                        Ok(count > 0)
                    },
                )
                .unwrap_or(false);

            if llm_table_exists {
                let add_column_if_missing = |col: &str, sql: &str, conn: &rusqlite::Connection| -> Result<()> {
                    let exists: bool = conn
                        .query_row(
                            "SELECT COUNT(*) FROM pragma_table_info('llm_usage_windows') WHERE name=?1",
                            [col],
                            |row| {
                                let count: i64 = row.get(0)?;
                                Ok(count > 0)
                            },
                        )
                        .unwrap_or(false);
                    if !exists {
                        conn.execute(sql, []).context(format!("Failed to add column {}", col))?;
                        info!("✓ Migration complete: Added {} to llm_usage_windows", col);
                    }
                    Ok(())
                };

                let _ = add_column_if_missing("model_name", "ALTER TABLE llm_usage_windows ADD COLUMN model_name TEXT", &conn);
                let _ = add_column_if_missing("total_calls", "ALTER TABLE llm_usage_windows ADD COLUMN total_calls INTEGER", &conn);
                let _ = add_column_if_missing("success_calls", "ALTER TABLE llm_usage_windows ADD COLUMN success_calls INTEGER", &conn);
                let _ = add_column_if_missing("cost_estimate", "ALTER TABLE llm_usage_windows ADD COLUMN cost_estimate REAL", &conn);
                let _ = add_column_if_missing("delta_latency_ms_avg", "ALTER TABLE llm_usage_windows ADD COLUMN delta_latency_ms_avg REAL", &conn);
                let _ = add_column_if_missing("delta_latency_ms_p95", "ALTER TABLE llm_usage_windows ADD COLUMN delta_latency_ms_p95 REAL", &conn);
            }

            Ok(())
        })
        .await?
        .context("Database migration task failed")?;

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
    pub async fn save_language_config(
        &self,
        config: &crate::language::LanguageConfig,
    ) -> Result<()> {
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
                conn.execute("DELETE FROM user_preferences WHERE key = ?1", ["language"])?;
            }

            // Store detected system language for reference
            if let Some(lang) = system_lang {
                conn.execute(
                    "INSERT OR REPLACE INTO user_preferences (key, value, value_type, description)
                     VALUES (?1, ?2, ?3, ?4)",
                    [
                        "system_language",
                        &lang,
                        "string",
                        "Auto-detected system language",
                    ],
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

            // Check if updated_at column exists, use appropriate SQL
            let has_updated_at: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('user_preferences') WHERE name='updated_at'",
                    [],
                    |row| {
                        let count: i64 = row.get(0)?;
                        Ok(count > 0)
                    },
                )?;

            if has_updated_at {
                conn.execute(
                    "INSERT OR REPLACE INTO user_preferences (key, value, value_type, updated_at)
                     VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
                    params!["llm_config", config_json, "json"],
                )?;
            } else {
                // Fallback for databases without updated_at column
                conn.execute(
                    "INSERT OR REPLACE INTO user_preferences (key, value, value_type)
                     VALUES (?1, ?2, ?3)",
                    params!["llm_config", config_json, "json"],
                )?;
            }

            // Force FULL checkpoint to ensure data is immediately visible
            // PASSIVE wasn't enough - annactl reads before checkpoint completes
            // FULL ensures all WAL data is written to main DB file synchronously
            conn.pragma_update(None, "wal_checkpoint", "FULL")?;

            debug!("Saved LLM configuration and checkpointed WAL (FULL)");
            Ok(())
        })
        .await??;

        Ok(())
    }

    /// Save a string preference
    pub async fn save_preference(&self, key: &str, value: &str) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let key = key.to_string();
        let value = value.to_string();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = conn.blocking_lock();

            // Check if updated_at column exists, use appropriate SQL
            let has_updated_at: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('user_preferences') WHERE name='updated_at'",
                    [],
                    |row| {
                        let count: i64 = row.get(0)?;
                        Ok(count > 0)
                    },
                )?;

            if has_updated_at {
                conn.execute(
                    "INSERT OR REPLACE INTO user_preferences (key, value, value_type, updated_at)
                     VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
                    params![key, value, "string"],
                )?;
            } else {
                // Fallback for databases without updated_at column
                conn.execute(
                    "INSERT OR REPLACE INTO user_preferences (key, value, value_type)
                     VALUES (?1, ?2, ?3)",
                    params![key, value, "string"],
                )?;
            }

            debug!("Saved preference: {}", key);
            Ok(())
        })
        .await??;

        Ok(())
    }

    /// Load a string preference
    pub async fn load_preference(&self, key: &str) -> Result<Option<String>> {
        let conn = Arc::clone(&self.conn);
        let key = key.to_string();

        tokio::task::spawn_blocking(move || -> Result<Option<String>> {
            let conn = conn.blocking_lock();

            let result: Result<String, _> = conn.query_row(
                "SELECT value FROM user_preferences WHERE key = ?1",
                params![key],
                |row| row.get(0),
            );

            match result {
                Ok(value) => Ok(Some(value)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        })
        .await?
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

        let (user_lang_str, system_lang_str) =
            tokio::task::spawn_blocking(move || -> Result<(Option<String>, Option<String>)> {
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

    #[tokio::test]
    async fn test_migration_adds_updated_at_column() {
        use rusqlite::Connection;

        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_migration.db");

        // Create a database with old schema (no updated_at column)
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute(
                "CREATE TABLE user_preferences (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    value_type TEXT NOT NULL,
                    set_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    set_by TEXT,
                    description TEXT
                )",
                [],
            )
            .unwrap();

            // Insert some test data
            conn.execute(
                "INSERT INTO user_preferences (key, value, value_type, set_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params!["test_key", "test_value", "string", "2025-11-01 10:00:00"],
            )
            .unwrap();
        }

        // Open with ContextDb - should run migration
        let location = DbLocation::Custom(db_path.clone());
        let db = ContextDb::open(location).await.unwrap();

        // Verify updated_at column exists and was populated
        let has_updated_at = db
            .execute(|conn| {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('user_preferences') WHERE name='updated_at'",
                    [],
                    |row| row.get(0),
                )?;
                Ok(count > 0)
            })
            .await
            .unwrap();

        assert!(
            has_updated_at,
            "updated_at column should exist after migration"
        );

        // Verify existing data was migrated (updated_at should equal set_at)
        let migrated_correctly = db
            .execute(|conn| {
                let updated_at: String = conn.query_row(
                    "SELECT updated_at FROM user_preferences WHERE key = ?1",
                    params!["test_key"],
                    |row| row.get(0),
                )?;
                Ok(updated_at == "2025-11-01 10:00:00")
            })
            .await
            .unwrap();

        assert!(
            migrated_correctly,
            "Existing rows should have updated_at = set_at after migration"
        );
    }

    #[tokio::test]
    async fn test_save_preference_works_after_migration() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_save_pref.db");
        let location = DbLocation::Custom(db_path);

        let db = ContextDb::open(location).await.unwrap();

        // This should work without errors (uses updated_at column)
        db.save_preference("test_key", "test_value").await.unwrap();

        // Verify it was saved
        let loaded = db.load_preference("test_key").await.unwrap();
        assert_eq!(loaded, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_save_llm_config_works_after_migration() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_llm_config.db");
        let location = DbLocation::Custom(db_path);

        let db = ContextDb::open(location).await.unwrap();

        // Create a test LLM config
        let config = crate::llm::LlmConfig::disabled();

        // This should work without errors (uses updated_at column)
        db.save_llm_config(&config).await.unwrap();

        // Verify it was saved
        let loaded = db.load_llm_config().await.unwrap();
        assert_eq!(loaded.mode, crate::llm::LlmMode::Disabled);
    }
}
