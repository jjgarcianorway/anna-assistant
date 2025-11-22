# Beta.258: "How is my system today?" Daily Snapshot v1

**Release Date**: 2025-11-22
**Focus**: Daily sysadmin-style snapshots for "today" queries
**Not about**: New commands or routing - content & structuring pass

---

## Summary

Beta.258 transforms "How is my system today?" from a bare health sentence into a short, deterministic daily snapshot that combines diagnostic health with session deltas (kernel, packages, boots). Feels like a senior sysadmin giving a brief morning briefing.

**Test Results**:
- **Smoke suite**: 178/178 (100%) ✓
- **Big suite**: 186/250 (74.4%) - maintained from Beta.257 ✓
- **Health consistency**: 13/13 (100%) - maintained from Beta.257 ✓
- **Daily snapshot content**: 8/8 (100%) new tests ✓

---

## Goals

1. **Daily Snapshot Layer**: Combine diagnostic engine + session metadata into compact briefing
2. **Temporal Wording**: Use "today" language when appropriate
3. **Session Delta Integration**: Show kernel, package, and boot changes
4. **Deterministic Content**: Re-use existing data, no new sources
5. **Graceful Degradation**: Handle missing session metadata cleanly

---

## Technical Changes

### 1. DailySnapshot Abstraction

**File**: `crates/annactl/src/diagnostic_formatter.rs` (lines 48-121)

Added structures to combine health and session delta:

```rust
/// Beta.258: Daily snapshot data combining health and session delta
#[derive(Debug, Clone)]
pub struct DailySnapshot {
    pub overall_health: OverallHealth,
    pub critical_count: usize,
    pub warning_count: usize,
    pub top_issue_summaries: Vec<String>,  // Top 3 issues
    pub kernel_changed: bool,
    pub old_kernel: Option<String>,
    pub new_kernel: Option<String>,
    pub package_delta: i32,
    pub boots_since_last: u32,
}

/// Beta.258: Session delta information
#[derive(Debug, Clone, Default)]
pub struct SessionDelta {
    pub kernel_changed: bool,
    pub old_kernel: Option<String>,
    pub new_kernel: Option<String>,
    pub package_delta: i32,
    pub boots_since_last: u32,
}

/// Beta.258: Compute daily snapshot from diagnostic and session data
pub fn compute_daily_snapshot(
    analysis: &BrainAnalysisData,
    session_delta: SessionDelta,
) -> DailySnapshot
```

**Purpose**: Single struct that merges diagnostic engine output with session metadata deltas.

### 2. Daily Snapshot Formatter

**File**: `crates/annactl/src/diagnostic_formatter.rs` (lines 410-500)

Added formatter for compact sysadmin briefings:

```rust
/// Beta.258: Format daily snapshot for "today" queries
pub fn format_daily_snapshot(snapshot: &DailySnapshot, temporal: bool) -> String
```

**Output Structure**:
1. **Health Summary Line** (temporal-aware):
   - Healthy: `"System health today: **all clear, no critical issues detected.**"`
   - DegradedWarning: `"System health today: **degraded – warning issues detected.**"`
   - DegradedCritical: `"System health today: **degraded – critical issues require attention.**"`

2. **[SESSION DELTA] Section**:
   - Kernel: `"unchanged since last session"` or `"updated (6.17.7 → 6.17.8)"`
   - Packages: `"no changes"` or `"15 package(s) upgraded"`
   - Boots: `"no reboots"` or `"1 reboot since last session"`
   - Issues: `"0 critical, 0 warnings"` or `"2 critical, 1 warning(s)"`

3. **[TOP ISSUES] Section** (if issues present):
   - Shows top 3 issues with severity markers (✗, ⚠)
   - Compact bullet format

**Does NOT include**:
- [COMMANDS] section (that's for full diagnostic report)
- Full [DETAILS] section
- More than 10-12 lines total

### 3. Unified Query Handler Integration

**File**: `crates/annactl/src/unified_query_handler.rs` (lines 1450-1529)

Modified `handle_diagnostic_query()` to use daily snapshot for temporal queries:

```rust
// Beta.258: Check if this is a temporal query
let normalized_query = query.to_lowercase();
let is_temporal = normalized_query.contains("today") || normalized_query.contains("recently");

// Beta.258: For temporal queries, use daily snapshot format
if is_temporal {
    // Load session metadata and compute delta
    let session_delta = match load_last_session() {
        Ok(Some(last_session)) => {
            // Fetch current telemetry and compute diffs
            match crate::system_query::query_system_telemetry() {
                Ok(current_telemetry) => {
                    // Compute kernel/package/boot deltas
                    SessionDelta { ... }
                }
                Err(_) => SessionDelta::default()
            }
        }
        _ => SessionDelta::default()
    };

    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    let report = format_daily_snapshot(&snapshot, true);

    // Return daily snapshot instead of full diagnostic report
    Ok(UnifiedQueryResult::ConversationalAnswer { ... })
} else {
    // Use full diagnostic report for non-temporal queries
    let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, query);
    Ok(UnifiedQueryResult::ConversationalAnswer { ... })
}
```

**Key Design Decision**: For Beta.258, temporal queries get ONLY the daily snapshot (Option A from prompt), not daily snapshot + full report. This keeps output concise and focused.

---

## Daily Snapshot Content Regression Tests

**File**: `crates/annactl/tests/regression_daily_snapshot_content.rs` (new, 8 tests)

Created comprehensive test suite for daily snapshot behavior:

### Test Categories:

1. **Healthy Today Snapshot** (1 test)
   - Verifies: Temporal wording, "all clear" message, session delta section, zero issues

2. **Degraded Snapshots** (2 tests)
   - Critical: "degraded – critical issues require attention", correct counts, top issues with ✗ marker
   - Warning: "degraded – warning issues detected", correct counts

3. **Session Delta Scenarios** (2 tests)
   - Kernel changed: Shows version transition "6.17.7 → 6.17.8", boots count = 1
   - Package upgrades: Shows "15 package(s) upgraded"

4. **Temporal vs Non-Temporal** (1 test)
   - Temporal=false → "System health:" (no "today")
   - Temporal=true → "System health today:"

5. **Graceful Degradation** (1 test)
   - No session metadata → Uses SessionDelta::default(), shows sensible defaults

6. **Top Issues Extraction** (1 test)
   - Verifies top 3 issues extracted correctly
   - Displayed in report with proper severity markers

All 8 tests pass ✅

---

## Before/After Comparison

### Before Beta.258:

```bash
$ annactl "how is my system today?"
[SUMMARY]
System health today: **all clear, no critical issues detected.**

All diagnostic checks passed. System is operating normally.

[COMMANDS]

No actions required - system is healthy.

$ annactl status        # View current status
```

### After Beta.258 (Healthy):

```bash
$ annactl "how is my system today?"
System health today: **all clear, no critical issues detected.**

[SESSION DELTA]
- Kernel: unchanged since last session
- Packages: no changes since last session
- Boots: no reboots since last session
- Issues: 0 critical, 0 warnings
```

### After Beta.258 (Degraded with Changes):

```bash
$ annactl "how is my system today?"
System health today: **degraded – critical issues require attention.**

[SESSION DELTA]
- Kernel: updated since last session (6.17.7 → 6.17.8)
- Packages: 15 package(s) upgraded
- Boots: 1 reboot since last session
- Issues: 2 critical, 1 warning(s)

[TOP ISSUES]
  ✗ Failed service: docker.service
  ✗ Disk space critical: / partition at 95%
  ⚠ Orphaned packages detected (12 packages)
```

---

## Design Principles

### 1. Re-Use Existing Data
- Diagnostic engine output (already have)
- Session metadata (already tracked in `/var/lib/anna/state/last_session.json`)
- No new data collection or sources

### 2. Deterministic Computation
- Session delta computed from exact telemetry diffs
- No LLM, no heuristics, no ambiguity
- Graceful fallback to defaults when metadata unavailable

### 3. Sysadmin Briefing Style
- Short (5-10 lines typically)
- Factual bullet points
- Critical info first (health, then deltas, then top issues)
- No fluff, no commands section (that's for full diagnostic)

### 4. Temporal Awareness
- "today"/"recently" queries → Daily snapshot format
- Other diagnostic queries → Full diagnostic report (from Beta.257)
- Clean separation of concerns

---

## User-Facing Impact

### What Changed:

**Temporal diagnostic queries now get daily snapshot format:**
- "how is my system today?" → Daily snapshot
- "any errors recently?" → Daily snapshot
- "check my system health" → Full diagnostic report (unchanged)

### User Benefits:

1. **Contextual Session Info**: Kernel/package/boot changes at a glance
2. **Compact Morning Briefing**: Perfect for "good morning, how are things?"
3. **Deterministic Delta**: Exact package counts, kernel versions
4. **Issue Summary**: Top 3 problems right up front if present

### Example User Workflow:

```bash
# Morning check-in
$ annactl "how is my system today?"
System health today: **all clear, no critical issues detected.**

[SESSION DELTA]
- Kernel: updated since last session (6.17.7 → 6.17.8)
- Packages: 5 package(s) upgraded
- Boots: 1 reboot since last session
- Issues: 0 critical, 0 warnings

# User now knows: rebooted for kernel update, 5 packages upgraded, all clear

# If they want full diagnostic details:
$ annactl "check my system health"
[SUMMARY]
System health: **all clear, no critical issues detected.**
...
[DETAILS]
...
[COMMANDS]
...
```

---

## Testing Summary

| Test Suite | Status | Details |
|------------|--------|---------|
| Smoke suite (178) | ✅ 100% | No regressions |
| Big suite (250) | ✅ 186/250 (74.4%) | Maintained from Beta.257 |
| Health consistency (13) | ✅ 100% | Maintained from Beta.257 |
| Daily snapshot content (8) | ✅ 100% | New tests, all passing |

**Combined Total**: 385/449 (85.7%), up from 377/441 (85.5%) due to new test suite

**Key Insight**: No routing changes, no big suite regressions. This was purely content structuring work.

---

## Files Changed

| File | Type | Lines | Purpose |
|------|------|-------|---------|
| `crates/annactl/src/diagnostic_formatter.rs` | Modified | +165 | DailySnapshot struct, SessionDelta, formatters |
| `crates/annactl/src/unified_query_handler.rs` | Modified | +50 | Temporal query detection, daily snapshot integration |
| `crates/annactl/tests/regression_daily_snapshot_content.rs` | New | +225 | 8 daily snapshot content tests |

**Total**: 3 files, ~440 lines added/modified

---

## Lessons Learned

1. **Data Re-Use**: Session metadata already tracked, just needed to expose it at query time
2. **Graceful Degradation**: SessionDelta::default() handles first-run cleanly
3. **Separation of Concerns**: Temporal queries → snapshot, others → full diagnostic
4. **Test Coverage**: 8 tests caught kernel version display logic, package delta formatting
5. **User Experience**: Short briefing > verbose diagnostic for "today" queries

---

## Future Work (Not in Scope for Beta.258)

1. **Status Command Integration**: Wire DailySnapshot into `annactl status` summary
2. **More Temporal Terms**: Extend to "this morning", "lately", "this week"
3. **Session History**: Track multiple sessions, show trend lines
4. **TUI Integration**: Daily snapshot widget in TUI header
5. **Command Suggestions**: Add relevant commands to snapshot (currently omitted)

---

## Conclusion

Beta.258 delivers on the promise: "How is my system today?" now returns a real daily sysadmin snapshot combining health status with session deltas. The DailySnapshot abstraction is deterministic, re-uses existing data, and degrades gracefully.

**Key Achievement**: Natural "today" responses without inventing new data sources or breaking existing tests.

**Pass Rate**: Maintained 186/250 (74.4%) on big suite, 100% on smoke and health consistency, 100% on new daily snapshot tests.

---

**Ready for**: Beta.259 - [Next initiative TBD]
