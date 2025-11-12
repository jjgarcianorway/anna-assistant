# Persistent Context Layer

**Phase 3.5: Session Continuity** - Anna's memory between sessions

## Overview

The Persistent Context Layer enables Anna to maintain continuity across daemon restarts, system reboots, and user sessions. This is **not AI memory** - it's a structured record of what the system did, learned, and preferences the user configured.

## Design Principles

1. **Lightweight**: Use SQLite for zero-dependency persistence
2. **Privacy-First**: No personal data, only system metadata
3. **Human-Readable**: SQL queries should be understandable
4. **Append-Only**: Historical records are never deleted (only archived)
5. **Queryable**: Enable pattern detection and learning
6. **Offline**: No cloud sync, all data stays local

## Database Schema

### Location

```
System Mode:  /var/lib/anna/context.db
User Mode:    $XDG_DATA_HOME/anna/context.db (or ~/.local/share/anna/context.db)
```

### Tables

#### 1. action_history

Tracks all actions Anna performed:

```sql
CREATE TABLE action_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    action_type TEXT NOT NULL,  -- 'update', 'install', 'repair', etc.
    command TEXT NOT NULL,        -- Full command executed
    outcome TEXT NOT NULL,        -- 'success', 'failure', 'cancelled'
    duration_ms INTEGER,          -- Time taken in milliseconds
    error_message TEXT,           -- If failed, why
    affected_items TEXT,          -- JSON array of packages/services affected
    user_id TEXT,                 -- UID or username
    session_id TEXT,              -- Links related actions
    advice_id TEXT,               -- If triggered by advice
    resource_snapshot TEXT        -- JSON: {ram_mb, cpu_cores, disk_gb}
);

CREATE INDEX idx_action_timestamp ON action_history(timestamp);
CREATE INDEX idx_action_type ON action_history(action_type);
CREATE INDEX idx_action_outcome ON action_history(outcome);
CREATE INDEX idx_session ON action_history(session_id);
```

#### 2. system_state_log

Historical system state snapshots:

```sql
CREATE TABLE system_state_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    state TEXT NOT NULL,           -- 'healthy', 'degraded', 'critical'
    total_memory_mb INTEGER,
    available_memory_mb INTEGER,
    cpu_cores INTEGER,
    disk_total_gb INTEGER,
    disk_available_gb INTEGER,
    uptime_seconds INTEGER,
    monitoring_mode TEXT,          -- 'minimal', 'light', 'full'
    is_constrained BOOLEAN,
    virtualization TEXT,           -- 'none', 'kvm', 'docker', etc.
    session_type TEXT,             -- 'desktop:wayland', 'ssh', etc.
    failed_probes TEXT,            -- JSON array of failed probe names
    package_count INTEGER,         -- Total packages installed
    outdated_count INTEGER,        -- Packages needing update
    boot_id TEXT                   -- Unique per boot (from /proc/sys/kernel/random/boot_id)
);

CREATE INDEX idx_state_timestamp ON system_state_log(timestamp);
CREATE INDEX idx_boot ON system_state_log(boot_id);
```

#### 3. user_preferences

User-configured settings and learned preferences:

```sql
CREATE TABLE user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    value_type TEXT NOT NULL,     -- 'string', 'integer', 'boolean', 'json'
    set_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    set_by TEXT,                  -- 'user', 'system', 'learning'
    description TEXT
);

-- Example data:
-- key='experience_level', value='intermediate', value_type='string', set_by='learning'
-- key='monitoring_mode_override', value='light', value_type='string', set_by='user'
-- key='auto_update_enabled', value='true', value_type='boolean', set_by='user'
-- key='preferred_shell', value='fish', value_type='string', set_by='user'
```

#### 4. command_usage

Track command usage for learning and recommendations:

```sql
CREATE TABLE command_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    command TEXT NOT NULL,
    subcommand TEXT,
    flags TEXT,                   -- JSON array of flags used
    exit_code INTEGER,
    was_helpful BOOLEAN,          -- Did user find output useful?
    led_to_action BOOLEAN,        -- Did command result in system action?
    context_state TEXT            -- System state when run
);

CREATE INDEX idx_command ON command_usage(command);
CREATE INDEX idx_command_timestamp ON command_usage(timestamp);
```

#### 5. learning_patterns

Detected patterns and learned behaviors:

```sql
CREATE TABLE learning_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_type TEXT NOT NULL,   -- 'time_of_day', 'resource_usage', 'command_sequence'
    pattern_data TEXT NOT NULL,   -- JSON with pattern details
    confidence REAL,              -- 0.0 to 1.0
    first_detected DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_confirmed DATETIME,
    confirmation_count INTEGER DEFAULT 1,
    actionable BOOLEAN DEFAULT FALSE,
    recommended_action TEXT
);

-- Example patterns:
-- type='time_of_day', data='{"command":"update","preferred_hour":22}'
-- type='resource_usage', data='{"peak_hours":[9,10,14,15],"off_peak":[22,23,0,1]}'
-- type='command_sequence', data='{"sequence":["health","doctor","repair"],"frequency":12}'
```

#### 6. session_metadata

Track user sessions for context:

```sql
CREATE TABLE session_metadata (
    session_id TEXT PRIMARY KEY,
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    user_id TEXT,
    session_type TEXT,            -- 'interactive', 'automated', 'cron'
    commands_count INTEGER DEFAULT 0,
    actions_count INTEGER DEFAULT 0,
    boot_id TEXT
);
```

## API Design

### Rust Module Structure

```
crates/anna_common/src/context/
├── mod.rs              # Public API
├── db.rs               # SQLite connection management
├── actions.rs          # Action history CRUD
├── state.rs            # System state logging
├── preferences.rs      # User preferences
├── usage.rs            # Command usage tracking
├── learning.rs         # Pattern detection
└── query.rs            # Complex queries and analytics
```

### Core Operations

#### Record Action

```rust
pub async fn record_action(
    action_type: &str,
    command: &str,
    outcome: ActionOutcome,
    duration_ms: u64,
    error: Option<String>,
    affected_items: Vec<String>,
) -> Result<i64>;
```

#### Log System State

```rust
pub async fn log_system_state(
    profile: &SystemProfile,
    state: &SystemState,
    failed_probes: Vec<String>,
) -> Result<i64>;
```

#### Get/Set Preference

```rust
pub async fn get_preference<T: FromStr>(key: &str) -> Result<Option<T>>;
pub async fn set_preference<T: ToString>(
    key: &str,
    value: T,
    set_by: PreferenceSource,
) -> Result<()>;
```

#### Track Command Usage

```rust
pub async fn track_command_usage(
    command: &str,
    subcommand: Option<&str>,
    flags: Vec<String>,
    exit_code: i32,
) -> Result<i64>;
```

#### Detect Patterns

```rust
pub async fn detect_patterns() -> Result<Vec<LearnedPattern>>;
pub async fn confirm_pattern(pattern_id: i64) -> Result<()>;
```

## Usage Examples

### 1. Learning Optimal Update Time

```rust
// After each successful update, record the time
context::record_action(
    "update",
    "annactl update",
    ActionOutcome::Success,
    45000, // 45 seconds
    None,
    vec!["linux".to_string(), "systemd".to_string()],
).await?;

// Periodically analyze patterns
let patterns = context::detect_patterns().await?;
for pattern in patterns {
    if pattern.pattern_type == "time_of_day" &&
       pattern.confidence > 0.8 {
        // User tends to update at 10 PM
        // Suggest: "Would you like to enable auto-update at 10 PM?"
    }
}
```

### 2. Resource Usage Prediction

```rust
// Log system state every 60 seconds (already doing this)
context::log_system_state(&profile, &state, failed_probes).await?;

// Query for resource patterns
let peak_times = query::get_peak_resource_usage_hours().await?;
// Returns: [9, 10, 14, 15] (work hours)

// Use this to recommend:
// "System is typically busy 9-11 AM and 2-4 PM.
//  Consider running updates at 10 PM when resources are idle."
```

### 3. Command Recommendations

```rust
// Track what users do after seeing degraded state
if state == SystemState::Degraded {
    let common_next = query::get_common_commands_after_state("degraded").await?;
    // Returns: ["health", "doctor", "repair"] in order of frequency

    // Suggest proactively:
    println!("💡 Recommended: annactl doctor");
}
```

### 4. User Experience Level Detection

```rust
// Count successful advanced commands
let advanced_count = query::count_successful_commands_by_category(
    CommandCategory::Advanced
).await?;

if advanced_count > 20 {
    context::set_preference(
        "experience_level",
        "intermediate",
        PreferenceSource::Learning,
    ).await?;
}
```

## Privacy & Security

### What We Store

✅ Command names and timestamps
✅ System resource metrics
✅ Action outcomes (success/failure)
✅ Package names affected
✅ User preferences (explicitly set)

### What We DON'T Store

❌ Command arguments (may contain sensitive data)
❌ File contents
❌ Passwords or secrets
❌ User personal data
❌ Network traffic
❌ Browsing history

### Data Retention

- **Action History**: Keep last 10,000 entries (~6 months typical)
- **System State Log**: Keep last 50,000 entries (~12 months at 60s intervals)
- **Command Usage**: Keep last 5,000 entries
- **Learning Patterns**: Keep indefinitely (small dataset)
- **User Preferences**: Keep indefinitely

### Cleanup Policy

```sql
-- Auto-cleanup old entries (run weekly)
DELETE FROM action_history
WHERE id NOT IN (
    SELECT id FROM action_history
    ORDER BY timestamp DESC
    LIMIT 10000
);

DELETE FROM system_state_log
WHERE id NOT IN (
    SELECT id FROM system_state_log
    ORDER BY timestamp DESC
    LIMIT 50000
);
```

## Migration Strategy

### Phase 1: Create Tables (v3.0.0-alpha.4)

- Implement SQLite schema
- Add connection pool
- Create basic CRUD operations
- Write migrations system

### Phase 2: Integrate Logging (v3.0.0-alpha.5)

- Hook action_history into executor
- Hook system_state_log into profiler
- Track command_usage in annactl

### Phase 3: Preferences & Learning (v3.0.0-alpha.6)

- Implement user_preferences API
- Build pattern detection algorithms
- Create recommendation engine

### Phase 4: UI Integration (v3.0.0-alpha.7)

- `annactl history` - Show action history
- `annactl patterns` - Show learned patterns
- `annactl stats` - Usage statistics
- `annactl prefs` - Manage preferences

## Query Examples

### Most Common Actions

```sql
SELECT action_type, COUNT(*) as count
FROM action_history
WHERE timestamp > datetime('now', '-30 days')
GROUP BY action_type
ORDER BY count DESC
LIMIT 10;
```

### Success Rate by Action

```sql
SELECT
    action_type,
    COUNT(*) as total,
    SUM(CASE WHEN outcome = 'success' THEN 1 ELSE 0 END) as successes,
    ROUND(100.0 * SUM(CASE WHEN outcome = 'success' THEN 1 ELSE 0 END) / COUNT(*), 1) as success_rate
FROM action_history
GROUP BY action_type
HAVING total > 5
ORDER BY success_rate ASC;
```

### Resource Trends

```sql
SELECT
    DATE(timestamp) as date,
    AVG(available_memory_mb) as avg_memory,
    MIN(available_memory_mb) as min_memory,
    MAX(available_memory_mb) as max_memory
FROM system_state_log
WHERE timestamp > datetime('now', '-7 days')
GROUP BY DATE(timestamp)
ORDER BY date;
```

### Command Popularity

```sql
SELECT
    command,
    COUNT(*) as uses,
    COUNT(DISTINCT DATE(timestamp)) as days_used
FROM command_usage
WHERE timestamp > datetime('now', '-30 days')
GROUP BY command
ORDER BY uses DESC
LIMIT 20;
```

## Testing

```bash
# Unit tests for each module
cargo test --package anna_common context

# Integration test
cargo test --package annad test_context_integration

# Manual testing
annactl context stats
annactl context history --last 10
annactl context patterns
```

## Future Enhancements

- **Export/Import**: Backup context to JSON
- **Analytics Dashboard**: Web UI for insights
- **Multi-User**: Separate contexts per user
- **Sync**: Optional sync between machines (encrypted)
- **ML Integration**: TensorFlow for advanced pattern detection

---

**Status**: Phase 3.5 - Design complete
**Next**: Phase 3.6 - Implement SQLite schema and basic operations
**Author**: Anna Persistent Context Team
**License**: Custom (see LICENSE file)

Citation: [sqlite:best-practices], [data-retention:gdpr-principles]
