# Beta.272: Proactive Surfacing, Priorities, and Remediation Integration

**Status**: ✅ Complete
**Dependencies**: Beta.270 (Proactive Correlation Engine), Beta.271 (Plumbing v1)
**Date**: 2025-11-23

## Overview

Beta.272 surfaces proactive correlation insights to users through CLI health output, natural language queries, and TUI display. It connects the proactive engine's correlation logic (Beta.270) and plumbing (Beta.271) to user-facing features with actionable remediation guidance.

## Key Features

### 1. CLI [PROACTIVE] Section

**File**: `crates/annactl/src/diagnostic_formatter.rs`

Added a new `[PROACTIVE]` section to health output that appears after `[SUMMARY]` and before `[COMMANDS]`:

```
[PROACTIVE]
Top correlated issues:
1. ✗ Network priority issue: slow Ethernet is outranking fast WiFi
2. ⚠ Disk pressure on /
3. ⚠ Service flapping: sshd restarted 5 times in 15 minutes
```

**Behavior**:
- Only shown when `proactive_issues` is non-empty
- Issues sorted by severity (critical > warning > info > trend)
- Capped at 10 issues maximum
- Numbered list with severity markers (✗ critical, ⚠ warning, ℹ info, ~ trend)
- Each issue shows the human-readable summary from the proactive engine

**Implementation Details**:
- Added `severity_priority_proactive()` helper function (lines 652-663)
- Modified both `format_diagnostic_report()` and `format_diagnostic_report_with_query()`
- Sorting uses descending severity priority to ensure critical issues appear first

### 2. Remediation Composer

**File**: `crates/annactl/src/sysadmin_answers.rs`

New function: `compose_top_proactive_remediation()` (lines 1696-1850)

Maps proactive engine root causes to appropriate remediation composers:

| Root Cause | Remediation Domain | Details |
|------------|-------------------|---------|
| `network_routing_conflict` | Network routing fix | Shows `ip route`, interface speed checks, metric configuration |
| `network_priority_mismatch` | Network priority fix | Same as routing conflict |
| `network_quality_degradation` | Network quality fix | Packet loss diagnostics, error checks |
| `disk_pressure` | Disk fix | Calls `compose_disk_fix_answer("/", 85, true)` |
| `disk_log_growth` | Disk fix | Same as disk pressure |
| `service_flapping` | Service diagnostics | Journalctl commands, service restart analysis |
| `service_under_load` | Service diagnostics | Same as service flapping |
| `service_config_error` | Service diagnostics | Same as service flapping |
| `memory_pressure` | Memory fix | Calls `compose_memory_fix_answer(85.0, Some(50.0), true)` |
| `cpu_overload` | CPU fix | Calls `compose_cpu_fix_answer(90.0, true, Some("unknown"))` |
| `kernel_regression` | Device diagnostics | Kernel rollback guidance |
| `device_hotplug` | Device diagnostics | Same as kernel regression |
| Unknown causes | Generic guidance | Shows root cause, confidence, timestamps, basic diagnostics |

**Output Format**: All remediation answers follow the canonical `[SUMMARY]`/`[DETAILS]`/`[COMMANDS]` format.

### 3. Natural Language Routing

**File**: `crates/annactl/src/unified_query_handler.rs`

Added deterministic pattern detection and routing for proactive remediation queries.

**New Functions**:
- `is_proactive_remediation_query()` (lines 1536-1598): Detects 40+ patterns
- `handle_proactive_remediation_query()` (lines 1600-1668): Fetches brain data and generates remediation

**Patterns Recognized** (40+ total):
- "what should i fix first"
- "what should i fix"
- "what are my top issues"
- "show me my top issues"
- "top issues", "top problems"
- "most important issue/problem"
- "main problem"
- "any critical problems/issues"
- "highest priority issue"
- "priority issues/problems"
- And many more variations

**Routing Tier**: TIER 0.6 (after diagnostic routing, before recipe routing)

**Behavior**:
- If no proactive issues: Returns "No correlated issues detected" message with basic health check guidance
- If issues exist: Sorts by severity, selects top issue, calls `compose_top_proactive_remediation()`
- Returns `UnifiedQueryResult::ConversationalAnswer` with high confidence
- Sources: `["proactive engine + deterministic remediation"]`

### 4. TUI Brain Panel Display

**File**: `crates/annactl/src/tui/brain.rs`

Enhanced the brain diagnostics panel to display proactive correlation insights.

**Changes**:
- Added `proactive_severity_priority()` helper function (lines 67-78)
- Added proactive section rendering after brain insights (lines 148-203)
- Shows "Proactive Correlations:" header with styled text
- Displays up to 5 proactive issues (space-constrained for TUI)
- Uses same severity markers and colors as CLI (✗, ⚠, ℹ, ~)
- Shows "... and N more" if more than 5 issues exist
- Adds dim separator line between insights and proactive section

**Visual Example**:
```
─────────────────────────────────────────
Proactive Correlations:
1. ✗ Network priority issue
2. ⚠ Disk pressure on /
3. ⚠ Service flapping: sshd
   ... and 2 more
```

**State Wiring**: Already completed in Beta.271 (`AnnaTuiState.proactive_issues` field)

### 5. Module Visibility Fix

**File**: `crates/annactl/src/main.rs`

Added module declarations to make `sysadmin_answers` and `net_diagnostics` available in binary context:

```rust
mod sysadmin_answers; // Beta.263
mod net_diagnostics; // Beta.265
```

**Why Needed**: These modules were declared in `lib.rs` but not `main.rs`, causing "unresolved import" errors when the binary tried to use them. The fix ensures both library and binary contexts can access these modules.

## Testing

**File**: `crates/annactl/tests/regression_proactive_surfacing.rs` (NEW)

**28 regression tests** covering:

### CLI [PROACTIVE] Section (8 tests)
- Section appears when issues exist
- Section hidden when no issues
- Severity markers displayed (✗, ⚠, ℹ)
- Numbered list formatting
- Issue capping at 10
- Severity-based sorting (critical first)
- Positioning (after SUMMARY, before COMMANDS)

### Remediation Composer (15 tests)
- Network routing conflict mapping
- Network priority mismatch mapping
- Network quality degradation mapping
- Disk pressure mapping (calls compose_disk_fix_answer)
- Disk log growth mapping
- Service flapping mapping
- Service under load mapping
- Service config error mapping
- Memory pressure mapping (calls compose_memory_fix_answer)
- CPU overload mapping (calls compose_cpu_fix_answer)
- Kernel regression mapping
- Device hotplug mapping
- Unknown root cause fallback
- Confidence display in generic remediation
- Timestamp display in generic remediation

### Severity Priority (5 tests)
- Critical = highest priority (4)
- Warning = second priority (3)
- Info = third priority (2)
- Trend = fourth priority (1)
- Unknown = lowest priority (0)

**Test Results**: ✅ All 28 tests pass

## Files Modified

### Core Implementation
- `crates/annactl/src/diagnostic_formatter.rs` - Added [PROACTIVE] section, severity_priority_proactive()
- `crates/annactl/src/sysadmin_answers.rs` - Added compose_top_proactive_remediation()
- `crates/annactl/src/unified_query_handler.rs` - Added NL routing for proactive queries
- `crates/annactl/src/tui/brain.rs` - Added proactive issues rendering, proactive_severity_priority()
- `crates/annactl/src/main.rs` - Added module declarations for binary context

### Testing
- `crates/annactl/tests/regression_proactive_surfacing.rs` - **NEW**: 28 comprehensive tests

## Global Constraints Compliance

✅ **Determinism only**: No LLM usage - all routing and remediation is rule-based
✅ **No new commands**: Pure enhancement to existing features
✅ **No new TUI keybindings**: Display-only enhancement
✅ **Backward compatibility**: Optional sections, no breaking changes
✅ **No destructive auto-execution**: Remediation provides guidance only
✅ **Standard output format**: All remediation uses [SUMMARY]/[DETAILS]/[COMMANDS]
✅ **No Rust types in output**: All enums mapped to user-safe strings
✅ **Proactive filters**: Confidence >= 0.7 (server-side), cap at 10 (CLI) or 5 (TUI), severity sorting

## Design Decisions

### Why TIER 0.6 for NL routing?
Placed after diagnostic routing (TIER 0.5) but before recipe routing (TIER 1.0) to ensure specific diagnostic queries are handled first, but proactive queries take precedence over generic recipes.

### Why 10 issues for CLI, 5 for TUI?
CLI has more vertical space and can scroll. TUI is space-constrained and needs to fit within a panel alongside other information.

### Why generic fallback for unknown root causes?
Future-proofs the system - as new correlation rules are added to the proactive engine (Beta.270), they'll automatically surface with basic guidance until specific remediation mappings are added.

### Why not use LLM for remediation?
Maintains determinism guarantee. Proactive engine is deterministic (Beta.270), so remediation should be too. This ensures predictable, testable, and reliable behavior.

## Future Work

**Beta.273+** may include:
- Interactive drill-down into correlated signals (click to expand)
- Trend visualization (flapping patterns, escalating issues)
- State persistence for assessment history comparison
- Historian integration for temporal context
- Remediation rule mapping via `rule_id` field
- Auto-suggest based on user context (skill level, system state)
- TUI keybinding to jump directly to top proactive issue

## Verification

```bash
# Compile daemon (no changes in Beta.272, but verify dependencies)
cargo build --release -p annad

# Compile client
cargo build --release -p annactl

# Run Beta.272 regression tests
cargo test -p annactl --test regression_proactive_surfacing

# Run Beta.271 tests (ensure no regression)
cargo test -p annactl --test regression_proactive_plumbing

# Full test suite
cargo test -p annactl
```

**Expected**: All tests pass, no breaking changes to existing functionality.

## Usage Examples

### CLI: Check system health with proactive issues
```bash
$ annactl "check my system health"

[SUMMARY]
System health: Degraded (2 warnings detected)

ℹ Proactive engine detected 2 correlated issue pattern(s).

[PROACTIVE]
Top correlated issues:
1. ⚠ Network priority mismatch: slow interface preferred over fast
2. ⚠ Service flapping: sshd restarted 4 times in 12 minutes

[DETAILS]
...

[COMMANDS]
...
```

### CLI: Ask "what should I fix first"
```bash
$ annactl "what should I fix first"

[SUMMARY]
Network routing issue detected: slow interface has default route

[DETAILS]
The proactive engine detected a network routing or priority problem.
This typically indicates:
- Slow interface has default route while faster interface doesn't
- Multiple default routes causing routing conflicts
- Interface priority misconfiguration

[COMMANDS]
# Check current routing table
$ ip route show

# Check interface speeds and status
$ ip link show
$ ethtool <interface> | grep Speed
...
```

### TUI: Brain panel shows proactive issues
```
┌─ Brain Diagnostics (2 insights) ────────┐
│ ⚠ Network interface mismatch            │
│   Evidence: eth0 preferred over wlan0   │
│   Fix: ip route show                    │
│                                         │
│ ℹ High disk usage on /var               │
│   Evidence: 78% used                    │
│   Fix: journalctl --vacuum-size=100M    │
│                                         │
│ ─────────────────────────────────────── │
│                                         │
│ Proactive Correlations:                 │
│ 1. ⚠ Network priority mismatch          │
│ 2. ℹ Disk log growth trend              │
└─────────────────────────────────────────┘
```

## Citation

- [archwiki:System_maintenance] - Proactive monitoring principles
- Beta.217: Sysadmin Brain foundation
- Beta.270: Proactive Correlation Engine
- Beta.271: Proactive Engine Plumbing v1
