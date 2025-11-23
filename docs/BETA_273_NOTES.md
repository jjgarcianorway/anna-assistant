# Beta.273: Proactive Engine v2 – TUI & CLI First-Class Experience

**Status**: ✅ Complete
**Dependencies**: Beta.270 (Proactive Engine), Beta.271 (Plumbing), Beta.272 (Surfacing)
**Date**: 2025-11-23

## Overview

Beta.273 promotes the proactive correlation engine from secondary status to a first-class, primary feature across all user-facing surfaces. Users now see proactive health scores, top issues, and correlated problems prominently highlighted in CLI, TUI, and natural language queries.

## Key Deliverables

### 1. CLI [PROACTIVE SUMMARY] Section

**File**: `crates/annactl/src/diagnostic_formatter.rs`

Added new `[PROACTIVE SUMMARY]` section that appears after `[SUMMARY]`, before `[DETAILS]`:

```
[PROACTIVE SUMMARY]
- 3 correlated issue(s) detected
- Health score: 75/100
- Top root cause: network_routing_conflict
```

**Features**:
- Only appears when proactive issues exist
- Shows total issue count
- Displays health score (0-100, from proactive engine calculation)
- Shows top root cause (highest severity issue, user-safe snake_case string)
- Deterministic formatting

### 2. TUI Header Bar Proactive Indicator

**File**: `crates/annactl/src/tui/render.rs` (`draw_header` function)

Added second line to TUI header when proactive issues exist:

```
Anna v5.7.0-beta.273 | llama3.2:3b | user@hostname
⚡ Top Issue: network_routing_conflict (score: 95)
```

**Features**:
- Appears as second line in header (multi-line paragraph)
- Shows lightning bolt emoji (⚡) for visual prominence
- Displays top issue's root_cause
- Shows score (confidence * 100)
- Bright yellow/orange color (255, 180, 80), bold
- Truncates gracefully if terminal width is narrow

### 3. TUI Proactive Mini-Panel

**File**: `crates/annactl/src/tui/brain.rs` (`draw_brain_panel` function)

Enhanced proactive section with bordered mini-panel:

```
┌ Proactive Analysis ┐
  ✗ routing conflict (score: 17)
  ⚠ memory pressure (score: 9)
  ℹ log growth (score: 6)
└────────────────────┘
```

**Features**:
- Capped at 3 issues (space-constrained for TUI)
- Shows severity markers (✗, ⚠, ℹ, ~)
- Displays root_cause (not summary)
- Shows score inline
- Sorted by severity then score
- Unicode box-drawing characters for visual clarity

### 4. NL Proactive Status Query Routing

**File**: `crates/annactl/src/unified_query_handler.rs`

Added new query handler for proactive status summaries.

**Patterns Recognized** (15 total):
- "show proactive status"
- "proactive status"
- "summarize top issues"
- "summarize issues"
- "what problems do you see"
- "summarize correlations"
- "summarize findings"
- "top correlated issues"
- "show correlations"
- "list proactive issues"
- "proactive summary"
- "correlation summary"
- And more variations

**Response Format**:
```
[SUMMARY]
Proactive engine detected 3 correlated issue(s).
System health score: 75/100

[DETAILS]
1. ✗ network_routing_conflict (severity: critical, score: 95)
2. ⚠ disk_pressure (severity: warning, score: 85)
3. ℹ service_flapping (severity: info, score: 75)

[COMMANDS]
$ annactl status
$ annactl "check my system health"
```

**No Issues Response**:
```
[SUMMARY]
No correlated issues found.

[DETAILS]
The proactive engine did not detect any high-confidence correlations.
...
```

## Implementation Details

### IPC Protocol Extension

**File**: `crates/anna_common/src/ipc.rs`

Added `proactive_health_score` field to `BrainAnalysisData`:

```rust
#[serde(default = "default_health_score")]
pub proactive_health_score: u8,

fn default_health_score() -> u8 {
    100  // Backward compatibility default
}
```

**File**: `crates/annad/src/rpc_server.rs`

Populated field from proactive assessment:

```rust
proactive_health_score: proactive_assessment.health_score,
```

### Score Calculation

Score is calculated consistently across all surfaces:
```
score = (confidence * 100.0) as u8
```

Where confidence ranges from 0.7 to 1.0 (server filters < 0.7).

### Severity Sorting

All surfaces sort by severity priority:
- Critical = 4
- Warning = 3
- Info = 2
- Trend = 1
- Unknown = 0

Top issue is always the highest severity, regardless of confidence.

## Testing

### Beta.273 Regression Tests (19 tests)
**File**: `crates/annactl/tests/regression_proactive_v2_beta273.rs`

- CLI [PROACTIVE SUMMARY] section (7 tests)
- Severity priority (3 tests)
- Health score validation (1 test)
- NL patterns documentation (1 test)
- Cross-surface consistency (4 tests)
- Edge cases (3 tests)

### Consistency Tests (10 tests)
**File**: `crates/annactl/tests/regression_proactive_consistency.rs`

- Root cause naming (2 tests)
- Severity markers (1 test)
- Score calculation (1 test)
- Severity sorting (2 tests)
- Format consistency (2 tests)
- Zero-case behavior (1 test)
- Determinism (1 test)

**Total**: 29 new tests, all passing ✅

## Files Modified

### Core Implementation
1. `crates/anna_common/src/ipc.rs` - Added `proactive_health_score` field
2. `crates/annad/src/rpc_server.rs` - Populated health score from assessment
3. `crates/annactl/src/diagnostic_formatter.rs` - Added [PROACTIVE SUMMARY] section
4. `crates/annactl/src/tui/render.rs` - Added header bar indicator
5. `crates/annactl/src/tui/brain.rs` - Enhanced proactive mini-panel
6. `crates/annactl/src/unified_query_handler.rs` - Added NL status query routing

### Testing
7. `crates/annactl/tests/regression_proactive_v2_beta273.rs` - NEW (19 tests)
8. `crates/annactl/tests/regression_proactive_consistency.rs` - NEW (10 tests)

### Documentation
9. `docs/BETA_273_NOTES.md` - This file
10. `CHANGELOG.md` - Beta.273 entry
11. `Cargo.toml` - Version bump
12. `README.md` - Badge update

## Global Constraints Compliance

✅ **Determinism only**: No LLM usage, all logic is rule-based
✅ **No new commands**: Pure enhancement to existing features
✅ **No new TUI keybindings**: Display-only changes
✅ **Backward compatible**: `#[serde(default)]` ensures old clients work
✅ **No Rust types in output**: All enum names mapped to snake_case strings
✅ **Issue caps**: CLI shows 10, TUI shows 3
✅ **Standard format**: All NL responses use [SUMMARY]/[DETAILS]/[COMMANDS]

## User Experience Impact

### Before Beta.273
- Proactive issues buried in `[PROACTIVE]` section at bottom
- No health score visibility
- Had to know exact queries to get proactive status
- TUI showed issues but not prominently

### After Beta.273
- Health score visible immediately after [SUMMARY]
- Top issue shown in TUI header bar
- Can ask "summarize top issues" naturally
- Proactive panel has clear visual hierarchy
- Correlation data is first-class, not secondary

## Design Decisions

### Why health score in [PROACTIVE SUMMARY]?
Provides at-a-glance system health metric. Score 0-100 is universally understood, unlike raw issue counts.

### Why show top root cause instead of summary?
Root cause is more actionable and technical users prefer causal diagnosis over symptom description.

### Why cap TUI at 3 vs CLI at 10?
TUI is space-constrained and used for quick monitoring. CLI is for detailed analysis.

### Why use confidence * 100 for score?
Confidence already represents issue certainty (0.7-1.0). Scaling to 0-100 makes it intuitive as a "score".

### Why add header indicator?
Most visible location in TUI. Users see critical issues immediately without scrolling.

## Future Work

Beta.274+ may include:
- Click to expand proactive issues in TUI
- Trend graphs for health score over time
- Per-domain health scores (network: 80/100, disk: 95/100)
- Remediation action shortcuts in TUI
- Historical health score tracking

## Verification

```bash
# Compile daemon and client
cargo build --release -p annad
cargo build --release -p annactl

# Run Beta.273 regression tests
cargo test -p annactl --test regression_proactive_v2_beta273
# Result: 19 passed ✅

# Run consistency tests
cargo test -p annactl --test regression_proactive_consistency
# Result: 10 passed ✅

# Full test suite
cargo test -p annactl
# Expected: All pass with Beta.271/272 tests
```

## Citation

- Beta.217: Sysadmin Brain foundation
- Beta.270: Proactive Correlation Engine
- Beta.271: Proactive Engine Plumbing
- Beta.272: Proactive Surfacing and Remediation
