-- Anna v0.12.0 Telemetry Schema
-- SQLite database for telemetry collection, user classification, and radar scoring

-- User tracking table
CREATE TABLE IF NOT EXISTS users (
    uid INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    first_seen TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_last_seen ON users(last_seen);

-- Metrics table (generic key-value telemetry)
CREATE TABLE IF NOT EXISTS metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    key TEXT NOT NULL,
    val REAL NOT NULL,
    unit TEXT,
    scope TEXT NOT NULL DEFAULT 'system',
    uid INTEGER NULL,
    FOREIGN KEY (uid) REFERENCES users(uid) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_metrics_ts ON metrics(ts);
CREATE INDEX IF NOT EXISTS idx_metrics_key ON metrics(key);
CREATE INDEX IF NOT EXISTS idx_metrics_scope ON metrics(scope);
CREATE INDEX IF NOT EXISTS idx_metrics_uid ON metrics(uid);
CREATE INDEX IF NOT EXISTS idx_metrics_key_ts ON metrics(key, ts);

-- Events table (system events for domain-driven intelligence)
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    kind TEXT NOT NULL,
    message TEXT NOT NULL,
    meta JSON
);

CREATE INDEX IF NOT EXISTS idx_events_ts ON events(ts);
CREATE INDEX IF NOT EXISTS idx_events_kind ON events(kind);

-- Radar scores table (user classification scores)
CREATE TABLE IF NOT EXISTS radar_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    uid INTEGER NOT NULL,
    radar TEXT NOT NULL,
    category TEXT NOT NULL,
    score REAL,
    max REAL NOT NULL DEFAULT 10.0,
    FOREIGN KEY (uid) REFERENCES users(uid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_radar_scores_uid ON radar_scores(uid);
CREATE INDEX IF NOT EXISTS idx_radar_scores_radar ON radar_scores(radar);
CREATE INDEX IF NOT EXISTS idx_radar_scores_ts ON radar_scores(ts);
CREATE INDEX IF NOT EXISTS idx_radar_scores_uid_radar ON radar_scores(uid, radar, ts);

-- Sample queries for v0.12.0

-- Get latest radar scores for a user
-- SELECT radar, category, score, max, ts
-- FROM radar_scores
-- WHERE uid = ? AND ts = (SELECT MAX(ts) FROM radar_scores WHERE uid = radar_scores.uid)
-- ORDER BY radar, category;

-- Get metric history for the last hour
-- SELECT ts, key, val, unit
-- FROM metrics
-- WHERE key = ? AND ts >= datetime('now', '-1 hour')
-- ORDER BY ts DESC;

-- Get recent events
-- SELECT ts, kind, message, meta
-- FROM events
-- WHERE ts >= datetime('now', '-24 hours')
-- ORDER BY ts DESC
-- LIMIT 100;
