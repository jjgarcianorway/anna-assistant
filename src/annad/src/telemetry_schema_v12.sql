-- Anna v0.12.2 Telemetry Schema
-- SQLite database for telemetry collection, user classification, and radar scoring

-- Telemetry snapshots table (v0.12.2 focused collectors)
CREATE TABLE IF NOT EXISTS snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts INTEGER NOT NULL,           -- Unix timestamp
    data TEXT NOT NULL,             -- JSON snapshot data
    UNIQUE(ts)
);

CREATE INDEX IF NOT EXISTS idx_snapshots_ts ON snapshots(ts DESC);

-- System classifications table (v0.12.2 persona detection)
CREATE TABLE IF NOT EXISTS classifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts INTEGER NOT NULL,
    persona TEXT NOT NULL,          -- 'laptop', 'workstation', 'server', 'vm', 'unknown'
    confidence REAL NOT NULL,       -- 0.0 - 1.0
    evidence TEXT NOT NULL,         -- JSON array of evidence strings
    UNIQUE(ts)
);

CREATE INDEX IF NOT EXISTS idx_classifications_ts ON classifications(ts DESC);

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

-- Radar scores table (v0.12.2 system-level radar scores)
CREATE TABLE IF NOT EXISTS radar_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts INTEGER NOT NULL,
    radar_type TEXT NOT NULL,      -- 'health' or 'network'
    category TEXT NOT NULL,         -- e.g., 'cpu_load', 'net_latency'
    score REAL,                     -- 0.0 - 10.0 or NULL if missing
    max REAL NOT NULL DEFAULT 10.0,
    description TEXT,
    UNIQUE(ts, radar_type, category)
);

CREATE INDEX IF NOT EXISTS idx_radar_scores_ts ON radar_scores(ts DESC);
CREATE INDEX IF NOT EXISTS idx_radar_scores_type ON radar_scores(radar_type);

-- Cleanup old data
DELETE FROM snapshots WHERE id NOT IN (
    SELECT id FROM snapshots ORDER BY ts DESC LIMIT 1000
);

DELETE FROM radar_scores WHERE ts < strftime('%s', 'now', '-7 days');

DELETE FROM classifications WHERE ts < strftime('%s', 'now', '-7 days');

-- Sample queries for v0.12.2

-- Get latest snapshot
-- SELECT data FROM snapshots ORDER BY ts DESC LIMIT 1;

-- Get latest classification
-- SELECT persona, confidence, evidence FROM classifications ORDER BY ts DESC LIMIT 1;

-- Get latest radar scores
-- SELECT radar_type, category, score, max, description
-- FROM radar_scores
-- WHERE ts = (SELECT MAX(ts) FROM radar_scores)
-- ORDER BY radar_type, category;
