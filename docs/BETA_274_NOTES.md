# Beta.274: NL Routing QA v3 – 700-Question Suite & Coverage Reports

**Status**: ✅ Complete
**Dependencies**: Beta.248-273 (NL routing foundation and proactive engine)
**Date**: 2025-11-23

## Overview

Beta.274 is a pure QA and measurement release that expands Anna's natural language routing test suite from 250 to 700 tests and adds comprehensive coverage metrics. This release makes NO changes to routing logic—it only observes, measures, and documents current behavior.

## Key Deliverables

### 1. Expanded Test Suite (250 → 700 tests)

**File**: `crates/annactl/tests/data/regression_nl_big.toml`

Added 450 new tests covering:

- **Proactive Status Queries** (30 tests): "show proactive status", "summarize top issues", "correlation summary"
- **Typo and Noise Patterns** (40 tests): typos, punctuation noise, polite fluff, emojis
- **Network Queries** (35 tests): network status, connectivity, DNS, routing, firewall
- **Temporal Patterns** (30 tests): "today", "yesterday", "since reboot", time-qualified queries
- **Short/Terse Queries** (30 tests): "ok?", "problems", "disk ok", abbreviated forms
- **Resource-Specific** (145 tests):
  - Disk I/O, SMART, filesystems, RAID, LVM (40 tests)
  - CPU, memory, load, processes (35 tests)
  - Package management, pacman, AUR (30 tests)
  - Boot, hardware, drivers (35 tests)
  - Security, permissions, auth (25 tests)
- **Diagnostic Variants** (50 tests): analyze, inspect, validate, audit, monitor patterns
- **Conversational/Ambiguous** (40 tests): meta questions, help requests, vague queries
- **Mixed Patterns** (65 tests): cron, services, containers, VMs, displays, power

All tests maintain the same metadata structure:
```toml
[[test]]
id = "big-XXX"
query = "user question"
expect_route = "diagnostic|status|conversational"
notes = "description"
priority = "high|medium|low"
classification = "correct|router_bug|test_unrealistic|ambiguous"
target_route = "ideal route"
current_route = "actual route"
```

### 2. Coverage Metrics

**File**: `crates/annactl/tests/regression_nl_big.rs`

Added comprehensive coverage reporting:

**Overall Metrics:**
```
Total tests:  700
Passed:       XXX (XX.X%)
Failed:       XXX (XX.X%)
```

**Per-Route Coverage:**
```
diagnostic:      XXX/YYY (ZZ.Z%)
status:          XXX/YYY (ZZ.Z%)
conversational:  XXX/YYY (ZZ.Z%)
```

**Per-Classification Coverage:**
```
correct:            XXX/YYY (ZZ.Z%)
router_bug:         XXX/YYY (ZZ.Z%)
test_unrealistic:   XXX/YYY (ZZ.Z%)
ambiguous:          XXX/YYY (ZZ.Z%)
```

**Per-Priority Coverage:**
```
high:    XXX/YYY (ZZ.Z%)
medium:  XXX/YYY (ZZ.Z%)
low:     XXX/YYY (ZZ.Z%)
```

### 3. End-to-End Integration Tests

**File**: `crates/annactl/tests/regression_nl_end_to_end.rs`

Created 20 new end-to-end tests that validate BOTH routing AND content:

- **8 Diagnostic Tests**: Verify [SUMMARY], [DETAILS], [COMMANDS] markers
- **8 Status Tests**: Verify Daemon, LLM, log structure
- **4 Conversational Tests**: Verify reasonable response structure

Unlike the big suite (routing only), these tests ensure responses contain expected formatting and content markers.

### 4. Documentation Updates

**Files Modified:**
- `docs/NL_ROUTING_TAXONOMY.md`: Updated test counts and Beta.274 changes
- `docs/REGRESSION_TESTING_NL_V1.md`: Updated version and test suite status
- `docs/BETA_274_NOTES.md`: This file

## Test Distribution

### By Route (Expected):
- Diagnostic: ~350 tests (50%)
- Conversational: ~280 tests (40%)
- Status: ~70 tests (10%)

### By Classification:
- Correct: ~500 tests (71%)
- Router bug: ~100 tests (14%)
- Test unrealistic: ~60 tests (9%)
- Ambiguous: ~40 tests (6%)

### By Priority:
- Low: ~450 tests (64%)
- Medium: ~200 tests (29%)
- High: ~50 tests (7%)

## Global Constraints Compliance

✅ **NO routing changes**: Zero modifications to unified_query_handler.rs or system_report.rs
✅ **NO CLI/TUI changes**: Pure test infrastructure
✅ **NO LLM involvement**: All tests are deterministic
✅ **Backward compatible**: Existing tests preserved
✅ **Additive only**: New tests added, none removed
✅ **Measurement focus**: Observing current behavior, not "fixing"

## Design Decisions

### Why 700 tests (not 500 or 1000)?

700 provides comprehensive coverage across all categories while remaining manageable:
- Large enough to catch edge cases and pattern variations
- Small enough to run quickly (<10 seconds)
- Aligns with natural category boundaries (30-50 tests per category)

### Why separate big suite from smoke suite?

- **Smoke suite (178 tests)**: Must pass 100%, guards against regressions
- **Big suite (700 tests)**: Measurement mode, documents current behavior
- **End-to-end (20 tests)**: Content validation, ensures proper formatting

### Why not enforce accuracy on big suite?

The big suite intentionally includes:
- Known routing bugs (documented for future fixes)
- Unrealistic test expectations (may need test corrections)
- Ambiguous queries (could go either way)

Enforcing 100% would require either:
1. Fixing all router bugs (out of scope for QA release)
2. Lowering test expectations (defeats measurement purpose)

Instead, we measure and classify failures for informed future work.

### Why add coverage metrics now?

Coverage metrics enable:
- **Targeted improvements**: Focus on high-priority failures
- **Classification tracking**: Monitor router_bug vs test_unrealistic ratios
- **Route balance**: Ensure all routes have adequate test coverage
- **Progress measurement**: Track accuracy improvements across releases

## Verification

```bash
# Compile all tests
cargo build --release -p annactl

# Run big suite with coverage metrics
cargo test -p annactl --test regression_nl_big

# Run end-to-end content validation
cargo test -p annactl --test regression_nl_end_to_end

# Run all NL tests
cargo test -p annactl --lib --test regression_nl_smoke --test regression_nl_big --test regression_nl_end_to_end
```

Expected results:
- regression_nl_smoke: 178/178 passing (100%)
- regression_nl_big: Reports coverage metrics, always passes
- regression_nl_end_to_end: 20/20 passing (100%)

## Files Modified

### Test Data
1. `crates/annactl/tests/data/regression_nl_big.toml` - Expanded 250→700 tests

### Test Harness
2. `crates/annactl/tests/regression_nl_big.rs` - Added coverage metrics
3. `crates/annactl/tests/regression_nl_end_to_end.rs` - NEW (20 tests)

### Documentation
4. `docs/NL_ROUTING_TAXONOMY.md` - Updated test counts
5. `docs/REGRESSION_TESTING_NL_V1.md` - Updated version info
6. `docs/BETA_274_NOTES.md` - This file
7. `CHANGELOG.md` - Beta.274 entry
8. `Cargo.toml` - Version bump to 5.7.0-beta.274
9. `README.md` - Badge update

## Future Work

Beta.275+ may include:
- Routing improvements targeting high-priority router_bugs
- Test expectation corrections for test_unrealistic cases
- Additional test categories (error handling, recipe queries)
- Temporal test suite for time-based patterns
- Proactive-specific routing patterns

## Success Criteria

✅ Test suite expanded from 250 to 700 tests
✅ Coverage metrics implemented and reporting
✅ 20 end-to-end tests created and passing
✅ All documentation updated
✅ Zero routing logic changes
✅ All tests compile and run successfully

Beta.274 provides the measurement foundation for future routing improvements without changing any production behavior.

## Citation

- Beta.248: Initial large NL suite (250 tests)
- Beta.249-257: Incremental routing improvements
- Beta.270-273: Proactive correlation engine
- Beta.274: QA expansion and measurement (this release)
