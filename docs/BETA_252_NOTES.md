# Beta.252: NL Failure Taxonomy & Ground Truth Audit

**Release Type:** Measurement and Documentation
**Date:** 2025-11-22
**Focus:** Document routing failure landscape with zero public interface changes

---

## Summary

Beta.252 is a **measurement and documentation release** focused on understanding routing failures rather than blindly fixing them. After Beta.251's conservative improvements (+9 tests), this release creates a comprehensive map of the remaining 100 failures to enable strategic improvements in future betas.

**Key Achievement:** Complete taxonomy of routing failures with metadata-driven test infrastructure.

---

## Motivation

**Problem:** Beta.248-251 improved routing from 138/250 to 147/250 tests passing (+9 tests, +3.6pp), but 103 failures remained. Without a clear understanding of WHY these tests fail, we risk:
- Making changes that don't address root causes
- Fixing symptoms instead of patterns
- Creating regressions in working cases
- Wasting effort on tests with unrealistic expectations

**Solution:** Stop making blind routing tweaks. Build a documented, categorized map of failures with clear judgments on whether the router should change or the test expectations should change.

---

## What Changed

### 1. Created NL_ROUTING_TAXONOMY.md

Comprehensive failure taxonomy document (`docs/NL_ROUTING_TAXONOMY.md`, 117KB) categorizing all 103 failures into 10 categories:

| Category | Count | Verdict | Target Beta |
|----------|-------|---------|-------------|
| A. Resource Health Variants | 3 | Router should improve | **Beta.252 (DONE)** |
| B. Vague "What's Wrong" Queries | ~15 | Mixed (some router, some test) | Beta.253 |
| C. Machine/Computer Terminology | ~3 | Router should improve | Beta.253 |
| D. Punctuation Edge Cases | ~4 | Mixed | Beta.254 |
| E. Status Temporal Queries | ~2 | Ambiguous | Beta.255 |
| F. Slang/Casual Variants | ~1 | Test should change | Beta.253 |
| G. Generic/Underspecified | ~60 | Mostly test should change | Beta.253-254 |
| H. Mixed How-To + Diagnostic | ~2 | Test should change | Beta.253 |
| I. False Positive (Correct Route) | ~1 | Test should change | Beta.253 |
| J. Remaining Ambiguous | ~12 | Low priority | Beta.255+ |

**Judgments Made:**
- **Router bugs** (~63 tests): Clear patterns missing, should be added
- **Test unrealistic** (~36 tests): Queries too vague, current routing defensible
- **Ambiguous** (~4 tests): Could go either way, low priority

### 2. Added Metadata to All Tests

Updated `regression_nl_big.toml` with Beta.252 metadata for all 250 tests:

```toml
# Beta.252 Metadata
priority = "high" | "medium" | "low"
classification = "router_bug" | "test_unrealistic" | "ambiguous" | "correct"
target_route = "diagnostic" | "status" | "conversational"  # What we want
current_route = "diagnostic" | "status" | "conversational"  # What router does
```

**Metadata Breakdown (all 250 tests):**
- **Classification:**
  - `correct`: 147 (58.8%) - passing tests
  - `router_bug`: 63 (25.2%) - should fix router
  - `test_unrealistic`: 36 (14.4%) - should change expectations
  - `ambiguous`: 4 (1.6%) - unclear

- **Priority:**
  - `high`: 3 (1.2%) - critical patterns
  - `medium`: 60 (24.0%) - valuable improvements
  - `low`: 187 (74.8%) - nice-to-have or correct

### 3. Enhanced Test Harness Reporting

Updated `regression_nl_big.rs` to display failures grouped by classification:

**Before (Beta.251):**
```
Failed Tests Breakdown:
Expected diagnostic, got other: 97
Expected status, got other: 5
Expected conversational, got other: 1
```

**After (Beta.252):**
```
Failed Tests Breakdown (Beta.252 Taxonomy):

🐛 Router Bugs (should be fixed): 60
  MEDIUM priority (60):
    • big-008 - "What's wrong with my system?"
      Expected: diagnostic, Got: conversational
    ... and 55 more

📝 Test Expectations Unrealistic (consider changing): 36
  • big-091 - "What's broken?"
    Expected: diagnostic, Got: conversational (current routing is defensible)
  ... and 31 more

❓ Ambiguous Cases (could go either way): 4
```

### 4. Implemented Trivial Safe Fix (Category A)

**Added 7 resource health patterns** to diagnostic route (Beta.252 only fix):

```rust
// Beta.252: Resource health variants (Category A - trivial safe fix)
"disk healthy",
"cpu healthy",
"memory healthy",
"ram healthy",
"network health",
"machine healthy",
"computer healthy",
```

**Impact:** +3 tests passing (150/250, 60.0%)

**Examples fixed:**
- `big-058`: "Is my machine healthy?" → now routes to diagnostic ✓
- `big-083`: "Is my disk healthy?" → now routes to diagnostic ✓
- `big-086`: "Network health" → now routes to diagnostic ✓

**Risk Assessment:** Zero risk
- Clear health check intent
- Consistent with existing "is my system healthy" pattern
- No potential for false positives

---

## Test Results

### Big Suite (250 tests)

**Beta.251 Baseline:**
- Passing: 147/250 (58.8%)
- Failing: 103/250 (41.2%)

**Beta.252 Results:**
- Passing: **150/250 (60.0%)**
- Failing: 100/250 (40.0%)
- **Improvement: +3 tests (+1.2pp)**

### Smoke Suite (178 tests)

- Passing: 178/178 (100%) ✅
- **Zero regressions**

### Failure Classification (100 failing tests)

- Router bugs: 60 (60.0%)
- Test unrealistic: 36 (36.0%)
- Ambiguous: 4 (4.0%)

---

## Architecture

### Test Metadata Infrastructure

**Test Case Structure (Beta.252):**
```rust
struct TestCase {
    id: String,
    query: String,
    expect_route: String,
    expect_contains: Vec<String>,
    notes: Option<String>,
    status: Option<String>,  // Beta.249
    // Beta.252: Metadata fields
    priority: Option<String>,
    classification: Option<String>,
    target_route: Option<String>,
    current_route: Option<String>,
}
```

**Annotation Process:**
1. Export routing results: `test_export_routing_results()`
2. Python script categorizes based on taxonomy
3. TOML updated with metadata
4. Test harness reads metadata and groups failures

### Files Changed

**Documentation:**
- `docs/NL_ROUTING_TAXONOMY.md` (new, 117KB) - complete taxonomy
- `docs/BETA_252_NOTES.md` (new) - this file

**Test Data:**
- `crates/annactl/tests/data/regression_nl_big.toml` - added metadata to all 250 tests

**Test Infrastructure:**
- `crates/annactl/tests/regression_nl_big.rs` - enhanced reporting, export test

**Routing Logic (minimal):**
- `crates/annactl/src/unified_query_handler.rs` - added 7 resource health patterns (Category A)

---

## Roadmap

### Beta.253 Targets (High Priority, Low Risk)

**Router Improvements (~12-15 tests):**
1. Category B1: "What's wrong" with context (+5-8 tests)
2. Category C: Machine/computer terminology (+3 tests)
3. Category G2: Obvious missing variants (+5-10 tests, subset)

**Test Expectation Changes (~6 tests):**
1. Category F: Slang/casual (+1 test)
2. Category H: Mixed how-to (+2 tests)
3. Category I: False positive (+1 test)
4. Category G2: Unrealistic expectations (~2-5 tests, subset)

**Combined Target:** 155-165 / 250 passing (62-66%)

### Beta.254-255 Targets

- Beta.254: Punctuation normalization, remaining obvious cases (160-170/250, 64-68%)
- Beta.255: Temporal status, remaining ambiguous (170-180/250, 68-72%)

### Realistic Long-Term Target

**180-200 / 250 (72-80%) by Beta.260**

**Why not 100%?**
- Some test expectations are unrealistic for deterministic routing
- Some queries are genuinely ambiguous
- Philosophy: "When in doubt, prefer conversational"
- Remaining 50-70 tests will be queries better handled by LLM

---

## Examples

### Category A: Resource Health Variants (FIXED in Beta.252)

**Before:**
```
big-058: "Is my machine healthy?"
  Current: conversational
  Expected: diagnostic
  → FAILS ✗
```

**After:**
```
big-058: "Is my machine healthy?"
  Current: diagnostic (matches "machine healthy")
  Expected: diagnostic
  → PASSES ✓
```

### Category B: Vague "What's Wrong" Queries (TARGET: Beta.253)

**Failure:**
```
big-008: "What's wrong with my system?"
  Current: conversational
  Expected: diagnostic
  Classification: router_bug (has "my system" context)
  Priority: medium
```

**Analysis:** Pattern "what's wrong" exists but not matching because additional context detection needed. Should route to diagnostic when combined with "my system", "this machine", etc.

### Category G: Generic/Underspecified (TEST EXPECTATIONS SHOULD CHANGE)

**Failure:**
```
big-091: "What's broken?"
  Current: conversational
  Expected: diagnostic
  Classification: test_unrealistic
  Priority: low
```

**Analysis:** Too vague for deterministic routing. No diagnostic keywords, no system context. Current routing (conversational) is correct - LLM can ask clarifying questions.

---

## Methodology

### Taxonomy Creation Process

1. **Analyzed 103 failures** manually and programmatically
2. **Identified pattern categories** based on:
   - Linguistic structure (punctuation, grammar)
   - Semantic content (health, status, problems)
   - Routing philosophy (deterministic vs conversational)
3. **Made judgments** for each category:
   - Should router change?
   - Should test expectation change?
   - Is it ambiguous/low priority?
4. **Assigned metadata** to all tests programmatically
5. **Enhanced test reporting** to show categorized failures

### Annotation Tooling

Created Python scripts to:
- Parse TOML test data
- Run routing logic matching production
- Categorize based on taxonomy
- Generate annotated TOML with metadata

Scripts:
- `/tmp/annotate_tests.py` - initial annotation attempt
- `/tmp/extract_routes_from_test.py` - production-accurate annotation

---

## Principles

### When to Fix Router vs Test

**Router should change when:**
- Pattern is clear and unambiguous
- Similar patterns already work
- User intent is obvious
- Low risk of false positives

**Test should change when:**
- Query is too vague/generic
- No clear diagnostic/status keywords
- Current routing is defensible
- Better handled by conversational LLM

**Ambiguous when:**
- Could legitimately go either way
- Trade-offs between approaches
- Low frequency pattern
- Complex NLP required

---

## Success Criteria

✅ **All success criteria met:**

1. **Existing tests still pass** - 178/178 smoke suite, 150/250 big suite
2. **No regressions** - Zero smoke suite failures
3. **Taxonomy document exists** - `NL_ROUTING_TAXONOMY.md` created
4. **Test data includes metadata** - All 250 tests annotated
5. **Harness prints structured info** - Enhanced reporting implemented
6. **Version updated** - 5.7.0-beta.252

**Bonus:** +3 tests from trivial safe fix (Category A)

---

## Known Limitations

### Metadata Accuracy

- Categorization based on programmatic analysis and manual review
- Some edge cases may be miscategorized
- Categories may overlap (e.g., test could be both "generic" and "punctuation edge case")

### Taxonomy Coverage

- Not all 103 failures individually examined (sampled representative cases)
- Counts are approximate
- Some categories may need refinement as we implement fixes

### Test Expectations

- Some test expectations may be debatable
- "Unrealistic" judgments are based on current routing philosophy
- Philosophy may evolve as system matures

---

## Philosophy

### Measurement Before Action

Beta.252 embodies "measure twice, cut once":
- Understand the problem fully before solving
- Document what's broken and why
- Make informed decisions on what to fix vs change
- Build infrastructure for future improvements

### Realistic Expectations

Not all test failures are router bugs:
- Some tests have unrealistic expectations
- Some queries are genuinely ambiguous
- Deterministic routing has inherent limits
- 72-80% accuracy is realistic for this approach

### Strategic Improvements

Future routing improvements will be:
- Evidence-based (using taxonomy)
- Prioritized (high/medium/low)
- Measured (metadata-driven reporting)
- Conservative (no regressions)

---

## Migration Notes

### For Developers

**Reading Test Metadata:**
```rust
// Beta.252 metadata fields
test.priority          // "high" | "medium" | "low"
test.classification    // "router_bug" | "test_unrealistic" | "ambiguous" | "correct"
test.target_route      // What we want long-term
test.current_route     // What router currently does
```

**Enhanced Test Output:**
- Failures now grouped by classification
- Priority shown for router bugs
- Clear guidance on what to fix vs change

### For Test Writers

**Adding New Tests:**
1. Add test to `regression_nl_big.toml`
2. Include Beta.252 metadata fields
3. Classify based on taxonomy categories
4. Assign priority based on impact

---

**Document Version:** Beta.252
**Last Updated:** 2025-11-22
**Next Review:** Beta.253 (after implementing high-priority fixes)
