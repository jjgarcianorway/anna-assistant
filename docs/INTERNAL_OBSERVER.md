# Internal Observer Layer

**Phase 5.2 - Behavioral Analysis Infrastructure**

This document describes Anna's internal observation and behavioral analysis system introduced in Phase 5.2. This is **infrastructure-only** with zero user-facing changes.

---

## Overview

The Observer Layer transforms Anna from a snapshot-based analyzer into a time-series observer with memory. Every time a user runs `annactl daily` or `annactl status`, Anna silently records observations about system state, building a historical database for pattern detection.

## Architecture

### 1. Observations Table (Time-Series Memory)

**Schema:**
```sql
CREATE TABLE observations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    issue_key TEXT NOT NULL,
    severity INTEGER NOT NULL,        -- 0=Info, 1=Warning, 2=Critical
    profile TEXT NOT NULL,             -- Laptop/Desktop/Server-Like
    visible INTEGER NOT NULL,          -- boolean (1=visible, 0=deemphasized)
    decision TEXT                      -- nullable (ack/snooze/none)
);

CREATE INDEX idx_observations_timestamp ON observations(timestamp DESC);
CREATE INDEX idx_observations_issue ON observations(issue_key, timestamp DESC);
```

**Purpose:** Capture snapshots of system state after all transformations (visibility hints, user decisions) are applied.

**Recording Trigger:** Observations are recorded at the end of:
- `annactl daily` (after visibility filtering)
- `annactl status` (all issues including deemphasized)

**Issue Key Strategy:**
- Primary: Use `repair_action_id` if present (stable identifier)
- Fallback: Use issue `title` if repair_action_id is None
- This ensures stable tracking across observations

### 2. Context API (`anna_common::context`)

**Functions:**

```rust
/// Record an observation for behavioral analysis
pub async fn record_observation(
    issue_key: impl Into<String>,
    severity: i32,              // 0=Info, 1=Warning, 2=Critical
    profile: impl Into<String>,
    visible: bool,
    decision: Option<String>,   // "acknowledged", "snoozed", or None
) -> Result<i64>

/// Get observations for a specific issue within time window
pub async fn get_observations(
    issue_key: &str,
    days_back: i64,
) -> Result<Vec<Observation>>

/// Get all observations within time window (for pattern analysis)
pub async fn get_all_observations(days_back: i64) -> Result<Vec<Observation>>
```

**Observation Struct:**
```rust
pub struct Observation {
    pub id: i64,
    pub timestamp: String,      // RFC3339 format
    pub issue_key: String,
    pub severity: i32,
    pub profile: String,
    pub visible: bool,
    pub decision: Option<String>,
}
```

### 3. Insights Engine (`anna_common::insights`)

**Main API:**
```rust
/// Generate behavioral insights from observation history
pub async fn generate_insights(days_back: i64) -> Result<InsightReport>
```

**Pattern Detectors (Internal Only):**

#### a. Flapping Detector

- **What it detects:** Issues that appear and disappear frequently
- **Threshold:** >5 visibility state changes in 14 days
- **Min observations:** 10+ required to detect pattern
- **Confidence:** Based on change frequency (0.0-1.0)

**Example:**
```
Issue "bluetooth-service" appeared 8 times in 12 days
→ Flapping pattern detected (confidence: 0.8)
```

#### b. Escalation Detector

- **What it detects:** Issues increasing in severity over time
- **Transitions tracked:** Info → Warning, Warning → Critical, Info → Critical
- **Min observations:** 2+ required
- **Confidence:** Based on severity delta (0.0-1.0)

**Example:**
```
Issue "disk-space" observed:
  Day 1: severity=0 (Info)
  Day 10: severity=1 (Warning)
  Day 15: severity=2 (Critical)
→ Escalation detected (confidence: 1.0)
```

#### c. Long-term Trend Detector

- **What it detects:** Issues visible for extended periods without user action
- **Threshold:** >14 days continuously visible with no decision (ack/snooze)
- **Confidence:** Based on duration beyond threshold

**Example:**
```
Issue "orphaned-packages" visible for 21 days
User has not acknowledged or snoozed
→ Long-term unaddressed issue (confidence: 0.7)
```

#### d. Profile Transition Detector

- **What it detects:** Machine profile changes during observation period
- **Use case:** VM scenarios (Laptop → Desktop when battery removed)
- **Confidence:** Always 1.0 (explicit state change)

**Example:**
```
Issue "tlp-config" observed:
  Day 1: profile=Laptop
  Day 5: profile=Desktop
→ Profile transition detected
```

### 4. InsightReport Structure

```rust
pub struct InsightReport {
    pub analysis_window_days: i64,
    pub total_observations: usize,
    pub top_recurring_issues: Vec<RecurringIssue>,
    pub patterns: Vec<BehaviorInsight>,
}

pub struct RecurringIssue {
    pub issue_key: String,
    pub appearance_count: usize,
    pub first_seen: String,
    pub last_seen: String,
}

pub struct BehaviorInsight {
    pub pattern_type: PatternType,    // Flapping/Escalation/LongTerm/Profile
    pub issue_key: String,
    pub description: String,
    pub confidence: f64,              // 0.0 - 1.0
    pub metadata: InsightMetadata,
}
```

---

## Integration Points

### Daily Command Hook

**Location:** `crates/annactl/src/daily_command.rs:92-112`

```rust
// Phase 5.2: Record observations for behavioral analysis
// This happens AFTER all visibility hints and decisions are applied
let profile_str = profile_to_string(profile);
for issue in &caretaker_analysis.issues {
    // Use repair_action_id as stable key, fallback to title if not present
    let issue_key = issue.repair_action_id.clone()
        .unwrap_or_else(|| issue.title.clone());

    let severity_int = severity_to_int(&issue.severity);
    let visible = issue.visibility != IssueVisibility::Deemphasized;
    let decision = issue.decision_info.as_ref().map(|(d, _)| d.clone());

    // Silently record observation - no error handling needed (fire and forget)
    let _ = context::record_observation(
        issue_key,
        severity_int,
        profile_str.clone(),
        visible,
        decision,
    ).await;
}
```

### Status Command Hook

**Location:** `crates/annactl/src/steward_commands.rs:89-109`

Same integration pattern as daily command. Status records observations for ALL issues (including deemphasized) since status shows complete system state.

---

## Design Principles

### 1. Silent Operation

- Observations are recorded via fire-and-forget pattern
- No errors shown to users
- Database failures don't affect command execution
- Completely transparent to users

### 2. Recording After Transformations

Observations are recorded AFTER:
1. Caretaker brain analysis (initial detection)
2. Visibility hints applied (noise control)
3. User decisions applied (acknowledge/snooze)

This ensures observations reflect the **final state** users actually see.

### 3. Stable Issue Keys

- Primary: `repair_action_id` (e.g., "disk-space", "failed-service")
- Fallback: `title` if repair_action_id is None
- This allows consistent tracking even if title wording changes

### 4. Profile Awareness

Every observation records the machine profile at that moment. This enables:
- Profile-specific pattern detection
- VM scenario understanding (profile transitions)
- Desktop vs. laptop behavioral differences

---

## Future Phases

### Phase 5.3 (Planned)

- Make insights user-visible
- Add `annactl insights` command
- Show patterns in `daily` output (controlled, non-noisy)
- "You've snoozed 'orphaned-packages' 5 times - consider addressing it"

### Phase 5.4 (Planned)

- Predictive analysis based on historical patterns
- "Disk space has escalated in the past - likely to reach critical in 3 days"
- Integration with repair system for proactive fixes

### Phase 6.0 (Future)

- Cross-machine insights (if collective mode enabled)
- "This issue affects 80% of similar systems - common configuration problem"

---

## Testing Strategy

### Unit Tests

- Pattern detector logic (insights.rs)
- Time span calculations
- Severity conversions

### Integration Tests

- End-to-end observation recording
- Database persistence
- Insight generation from real observation data

### Manual Testing

```bash
# Generate observations by running commands multiple times
annactl daily
# Wait some time, modify system state, run again
annactl daily

# Query observations directly (requires DB access)
sqlite3 /var/lib/anna/context.db "SELECT * FROM observations ORDER BY timestamp DESC LIMIT 10;"

# Test insight generation (internal API, not exposed to users yet)
# This will be testable via Rust integration tests
```

---

## Performance Considerations

### Write Performance

- Single INSERT per issue per command
- Async fire-and-forget (doesn't block command)
- Indexed writes (~5ms per observation)

### Query Performance

- Indexed on timestamp DESC (recent queries fast)
- Indexed on (issue_key, timestamp) for issue-specific queries
- Pattern detectors scan 30 days max (~2-3K observations)
- Expected query time: <50ms for full insight generation

### Storage Growth

- ~150 bytes per observation
- Daily command with 5 issues = 750 bytes/day
- 30 days = ~22KB
- 1 year = ~270KB

Storage is negligible. No cleanup needed for years.

---

## Database Migration

Observations table is added via idempotent schema initialization:

```rust
conn.execute(
    "CREATE TABLE IF NOT EXISTS observations (...)",
    [],
)?;
```

Existing databases automatically upgraded on next command execution.

---

## Code Locations

**Database Schema:**
- `crates/anna_common/src/context/db.rs:326-351`

**Context API:**
- `crates/anna_common/src/context/mod.rs:231-354`

**Insights Engine:**
- `crates/anna_common/src/insights.rs` (480 lines)

**Integration Hooks:**
- `crates/annactl/src/daily_command.rs:92-112`
- `crates/annactl/src/steward_commands.rs:89-109`

---

## Summary

Phase 5.2 is pure foundational infrastructure with no user-facing changes. Anna now silently observes and remembers system behavior over time, building the memory layer required for future behavioral analysis and predictive capabilities.

**Before Phase 5.2:** Every command was independent, no memory.
**After Phase 5.2:** Anna builds a time-series history of system behavior.

This is the moment Anna becomes an observer.
