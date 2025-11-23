# Beta.271: Proactive Engine Plumbing & Surfacing v1

**Status**: ✅ Complete
**Dependencies**: Beta.270 (Proactive Correlation Engine)
**Date**: 2025-11-23

## Overview

Beta.271 connects the Beta.270 proactive correlation engine to the health pipeline and exposes correlated issues through the RPC protocol with minimal user-facing changes. This is pure plumbing - no new commands, no UI redesign, just data flow infrastructure.

## Implementation Summary

### 1. Type System Extensions

#### Server-Side Types (`crates/annad/src/steward/types.rs`)
- **`ProactiveIssueSummary`**: User-safe representation of correlated issues
  - `root_cause`: String label (e.g., "network_routing_conflict", not Rust enum)
  - `severity`: Lowercase string ("critical", "warning", "info", "trend")
  - `summary`: Human-readable one-liner
  - `rule_id`: Optional, for future remediation mapping
  - `confidence`: Float (0.7-1.0, server filters < 0.7)
  - `first_seen`, `last_seen`: ISO 8601 timestamps (RFC 3339)

#### IPC Protocol (`crates/anna_common/src/ipc.rs`)
- **`ProactiveIssueSummaryData`**: Wire format matching server-side structure
- **Extended `BrainAnalysisData`**:
  ```rust
  pub struct BrainAnalysisData {
      // ... existing fields ...
      #[serde(default)]  // Backward compatibility
      pub proactive_issues: Vec<ProactiveIssueSummaryData>,
  }
  ```

### 2. RPC Integration (`crates/annad/src/rpc_server.rs`)

**Modified**: `BrainAnalysis` method handler (lines 2126-2173)

After brain sysadmin analysis:
1. Construct `ProactiveInput` from health report and brain insights
2. Call `compute_proactive_assessment(&proactive_input)`
3. Convert assessment to summaries using `assessment_to_summaries()`
4. Convert summaries to IPC format (`ProactiveIssueSummaryData`)
5. Include in `BrainAnalysisData` response

**TODOs left for future iterations**:
- `previous_assessment`: Load from persistent state
- `historian_context`: Populate from historian service

### 3. Conversion Functions (`crates/annad/src/intel/proactive_engine.rs`)

Three new public functions (lines 1462-1513):

- **`root_cause_to_string(root_cause: &RootCause) -> String`**
  Maps internal `RootCause` enum to user-safe snake_case labels
  - `RootCause::NetworkRoutingConflict` → `"network_routing_conflict"`
  - `RootCause::DiskPressure` → `"disk_pressure"`
  - etc.

- **`severity_to_string(severity: IssueSeverity) -> String`**
  Maps `IssueSeverity` enum to lowercase strings
  - `IssueSeverity::Critical` → `"critical"`
  - `IssueSeverity::Warning` → `"warning"`
  - etc.

- **`assessment_to_summaries(assessment: &ProactiveAssessment) -> Vec<steward::ProactiveIssueSummary>`**
  Converts `ProactiveAssessment` to user-safe summary format
  - Respects `MAX_DISPLAYED_ISSUES` = 10 cap
  - Confidence filtering happens upstream in `compute_proactive_assessment()`
  - Converts timestamps to RFC 3339 format

### 4. Client-Side Integration (`crates/annactl`)

#### Diagnostic Formatter (`src/diagnostic_formatter.rs`)
**Modified**: Both `format_diagnostic_report()` and `format_diagnostic_report_with_query()`

Added Beta.271 section after `[SUMMARY]`, before `[DETAILS]`:
```
ℹ Proactive engine detected N correlated issue pattern(s).
```

Only shown when `proactive_issues` is non-empty.

#### TUI State (`src/tui_state.rs`, `src/tui/brain.rs`, `src/tui/input.rs`)
- **Added field**: `proactive_issues: Vec<ProactiveIssueSummaryData>` to `AnnaTuiState`
- **Updated**: `update_brain_analysis()` to populate `state.proactive_issues`
- **Updated**: State initialization in `tui/input.rs` to include empty vec

**Note**: TUI rendering of proactive issues deferred to later Beta iteration.

### 5. Backward Compatibility

**Guaranteed** via:
- `#[serde(default)]` on `proactive_issues` field
- Old clients ignore new field (serde skips unknown fields)
- New clients handle missing field as empty vec
- No breaking changes to existing RPC methods

**Regression test**: `test_rpc_backward_compatibility_empty_issues` validates old JSON deserializes correctly.

### 6. Testing (`crates/annactl/tests/regression_proactive_plumbing.rs`)

**15 regression tests** covering:

#### RPC Protocol (3 tests)
- Field existence
- Backward compatibility (missing field defaults to empty)
- Serialization round-trip

#### Formatter Display (6 tests)
- Proactive count shown when present
- Hidden when empty
- Singular vs plural handling
- Positioning (after SUMMARY, before COMMANDS)
- Summary mode display
- Full mode display

#### Issue Capping (2 tests)
- MAX_DISPLAYED_ISSUES = 10 enforcement
- Zero issues edge case

#### Data Mapping (3 tests)
- Confidence filtering (informational - server-side)
- Root cause user-safe labels (no Rust enum leakage)
- Severity lowercase strings

#### Timestamp Format (1 test)
- ISO 8601 / RFC 3339 compliance

**All 15 tests pass**: ✅

## Global Constraints Compliance

✅ **No Rust types in user output**: All enums mapped to strings
✅ **Issue cap**: MAX_DISPLAYED_ISSUES = 10 enforced in `assessment_to_summaries()`
✅ **Confidence threshold**: >= 0.7 filtering in `compute_proactive_assessment()` (Beta.270)
✅ **Temporal windows**: 15min/1h/24h windows used by proactive engine (Beta.270)
✅ **Deterministic behavior**: No LLM, no randomness, pure correlation logic
✅ **Backward compatibility**: `#[serde(default)]` ensures old clients work

## Files Modified

### Core Implementation
- `crates/annad/src/steward/types.rs` - Added `ProactiveIssueSummary`
- `crates/annad/src/steward/mod.rs` - Exported new type
- `crates/anna_common/src/ipc.rs` - Added `ProactiveIssueSummaryData`, extended `BrainAnalysisData`
- `crates/annad/src/intel/proactive_engine.rs` - Added conversion functions
- `crates/annad/src/intel/mod.rs` - Exported conversion functions
- `crates/annad/src/rpc_server.rs` - Integrated proactive engine into BrainAnalysis handler

### Client-Side
- `crates/annactl/src/diagnostic_formatter.rs` - Added proactive issue count display
- `crates/annactl/src/tui_state.rs` - Added `proactive_issues` field
- `crates/annactl/src/tui/brain.rs` - Wire proactive issues to state
- `crates/annactl/src/tui/input.rs` - Initialize proactive_issues in state clones

### Testing
- `crates/annactl/tests/regression_proactive_plumbing.rs` - **NEW**: 15 regression tests
- Fixed all existing tests to include `proactive_issues: vec![]` field

## Not Included (By Design)

❌ **No new annactl commands** - Pure plumbing iteration
❌ **No TUI rendering** - State wired, rendering deferred
❌ **No remediation mapping** - `rule_id` placeholder for future
❌ **No state persistence** - `previous_assessment` TODO for later
❌ **No historian integration** - `historian_context` TODO for later
❌ **No LLM involvement** - Deterministic only

## Future Work

**Beta.272+** will likely include:
- TUI panel for proactive issues display
- State persistence for assessment history
- Historian context integration
- Remediation rule mapping (`rule_id` → action plans)
- Trend visualization (flapping, escalating, long-term issues)
- Interactive drill-down into correlated signals

## Verification

```bash
# Compile daemon
cargo build --release -p annad

# Compile client
cargo build --release -p annactl

# Run Beta.271 regression tests
cargo test -p annactl --test regression_proactive_plumbing

# Verify backward compatibility
cargo test -p annactl regression_health_content
cargo test -p annactl regression_sysadmin
```

**Expected**: All tests pass, no breaking changes.

## Citation

[archwiki:System_maintenance] - System monitoring principles
Beta.217: Sysadmin Brain foundation
Beta.270: Proactive Engine implementation
