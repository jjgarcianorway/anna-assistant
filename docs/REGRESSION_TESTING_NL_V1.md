# Natural Language Regression Testing Guide - V1

**Version:** Beta.274
**Test Suites:**
- regression_nl_smoke (178 tests, 100% passing)
- regression_nl_big (700 tests, measurement mode)
- regression_nl_end_to_end (20 tests, content validation)
**Status:** Production-ready with comprehensive coverage

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Running Tests](#running-tests)
4. [Test Data Format](#test-data-format)
5. [Adding New Tests](#adding-new-tests)
6. [Test Harness Utilities](#test-harness-utilities)
7. [Interpreting Results](#interpreting-results)
8. [Best Practices](#best-practices)
9. [Troubleshooting](#troubleshooting)

---

## Overview

### Purpose

The Natural Language Regression Test Suite validates that Anna's query routing behaves consistently and correctly across a wide range of user inputs. It ensures that:

- Diagnostic queries route to TIER 0.5 (health check system)
- Status queries route to TIER 0 (system report)
- Recipe queries route to TIER 1 (deterministic action plans)
- Conversational queries route to TIER 3/4 (LLM or telemetry-based answers)
- False positives are prevented (queries that shouldn't trigger special routes)

### Design Principles

1. **Deterministic** - No real daemon dependency, uses mocking
2. **Fast** - Runs in <1 second for 172 tests
3. **Maintainable** - TOML format for easy test authoring
4. **Comprehensive** - Covers exact matches, patterns, edge cases, false positives, contextual awareness
5. **Documented** - Every test has a descriptive `notes` field

### Beta.244 Additions

Beta.244 expanded the test suite from 115 to 172 tests (50% growth), adding coverage for:
- **Conceptual question exclusions** - "what is a healthy system" queries
- **Contextual awareness** - Positive/negative system reference indicators
- **Temporal status patterns** - "how is my system today" queries
- **Importance-based status patterns** - "anything important on this machine" queries

---

## Architecture

### Components

```
crates/annactl/
├── tests/
│   ├── data/
│   │   └── regression_nl_smoke.toml    # Test data (172 test cases)
│   └── regression_nl_smoke.rs          # Test harness (routing logic + assertions)
```

### How It Works

1. **Test Loading**: Harness loads test cases from TOML file
2. **Route Classification**: Each query is classified using `classify_query_route()`
3. **Assertion**: Actual route compared with expected route
4. **Content Validation**: Optional substring checks on mock responses
5. **Reporting**: Pass/fail with detailed diagnostics

### Routing Logic Reproduction

The test harness contains a **copy of production routing logic** from `unified_query_handler.rs` and `system_report.rs`:

```rust
fn classify_query_route(query: &str) -> RouteType {
    // TIER 0.5: Diagnostic queries (Beta.238/239/243/244)
    if is_full_diagnostic_query(query) {
        return RouteType::Diagnostic;
    }

    // TIER 0: System report queries (Beta.243/244)
    if is_system_report_query(query) {
        return RouteType::Status;
    }

    // Default to conversational
    RouteType::Conversational
}
```

**Beta.244 Routing Enhancements:**
- **Conceptual exclusions** - Early detection of "what is", "explain", "define" patterns
- **Contextual awareness** - Positive ("this system") vs negative ("in general") indicators
- **Temporal patterns** - "today", "now", "currently" + system reference → status
- **Importance patterns** - "anything important", "any issues" + system reference → status

**Important:** When updating routing logic in production, you **must** also update the test harness copy.

---

## Running Tests

### Full Suite

```bash
cargo test --test regression_nl_smoke
```

### Specific Test

```bash
cargo test --test regression_nl_smoke test_regression_nl_smoke_suite -- --nocapture
```

### With Output

```bash
cargo test --test regression_nl_smoke -- --nocapture
```

### Quick Validation (No Output)

```bash
cargo test --test regression_nl_smoke -- --quiet
```

### Expected Output

```
🧪 Running Natural Language Regression Suite (Smoke Tests)
══════════════════════════════════════════════════════════

✅ diag-001-health-check - Beta.239 exact match phrase
✅ diag-002-system-check - Beta.239 exact match phrase
...
✅ fn-003-health-status - Contains 'system' + 'health' - should match pattern

══════════════════════════════════════════════════════════
📊 Test Summary:
   Total: 172
   Passed: 172 (100%)
   Failed: 0
══════════════════════════════════════════════════════════

test test_regression_nl_smoke_suite ... ok
```

---

## Test Data Format

### TOML Structure

Tests are defined in `tests/data/regression_nl_smoke.toml`:

```toml
[[test]]
id = "unique-test-identifier"
query = "the natural language query user would type"
expect_route = "diagnostic" | "status" | "recipe" | "conversational" | "fallback"
expect_contains = ["substring1", "substring2"]  # Optional
notes = "human-readable explanation"  # Recommended
```

### Field Descriptions

| Field             | Required | Type     | Description                                              |
|-------------------|----------|----------|----------------------------------------------------------|
| `id`              | ✅ Yes   | String   | Unique test identifier (use descriptive kebab-case)      |
| `query`           | ✅ Yes   | String   | The exact user query to test                             |
| `expect_route`    | ✅ Yes   | String   | Expected routing tier (see values below)                 |
| `expect_contains` | ❌ No    | Array    | Substrings that must appear in response (use sparingly)  |
| `notes`           | ⚠️ Recommended | String | Explanation of what this test validates            |

### Valid Route Values

- `"diagnostic"` - TIER 0.5 (full diagnostic/health check)
- `"status"` - TIER 0 (system report/status query)
- `"recipe"` - TIER 1 (deterministic action plan)
- `"conversational"` - TIER 3/4 (LLM or telemetry-based answer)
- `"fallback"` - Unrecognized/off-topic query

---

## Adding New Tests

### Step 1: Identify Test Category

Choose the appropriate category for your test:

- **Diagnostic exact match** - Known diagnostic phrase
- **Diagnostic pattern** - Contains diagnostic keywords
- **False positive** - Should NOT trigger diagnostic
- **Status query** - System status/information request
- **Conversational baseline** - General queries
- **Edge case** - Boundary conditions, special characters
- **Realistic phrasing** - Common user expressions

### Step 2: Write Test Case

Add to the appropriate section in `regression_nl_smoke.toml`:

```toml
[[test]]
id = "diag-014-new-phrase"
query = "show me a health check"
expect_route = "diagnostic"
expect_contains = ["System health"]
notes = "Beta.XXX - New diagnostic phrase variant"
```

### Step 3: Naming Convention

Use descriptive IDs with category prefix:

```
diag-XXX-*       # Diagnostic route tests
status-XXX-*     # Status route tests
conv-XXX-*       # Conversational tests
false-XXX-*      # False positive prevention
edge-XXX-*       # Edge cases
real-XXX-*       # Realistic user phrasings
ambig-XXX-*      # Ambiguous queries
bXXX-YYY-*       # Beta version specific tests (e.g., b244-001-what-is-healthy)
```

**Beta.244 Test ID Examples:**
```
b244-001-what-is-healthy          # Conceptual question exclusion
b244-003-this-system-health       # Positive contextual indicator
b244-013-general-context          # Negative contextual indicator
b244-015-how-today                # Temporal status pattern
b244-019-anything-important       # Importance-based status pattern
```

### Step 4: Run Tests

```bash
cargo test --test regression_nl_smoke -- --nocapture
```

### Step 5: Update Count

Update summary stats at end of TOML file:

```toml
# Total test cases: XXX (43 from Beta.241 + YY from Beta.242 + ZZ from Beta.XXX)
```

---

## Test Harness Utilities

### Beta.242 Helper Functions

```rust
/// Assert that a query routes to the expected route type
fn assert_route(query: &str, expected: RouteType) -> bool

/// Assert that a response contains any of the given substrings
fn assert_contains_any(response: &str, expected_substrings: &[String])
    -> (bool, Vec<String>)

/// Assert that a response does NOT contain any of the given substrings
fn assert_not_contains(response: &str, forbidden_substrings: &[String])
    -> (bool, Vec<String>)
```

### Using Utilities in Future Expansions

These utilities are available for custom test logic:

```rust
// Example: Custom test validation
let query = "health check";
assert!(assert_route(query, RouteType::Diagnostic));

let response = mock_diagnostic_response();
let (passes, missing) = assert_contains_any(&response, &vec!["System health".to_string()]);
assert!(passes, "Missing substrings: {:?}", missing);
```

---

## Interpreting Results

### Successful Run

```
✅ diag-001-health-check - Beta.239 exact match phrase
...
📊 Test Summary:
   Total: 101
   Passed: 101 (100%)
   Failed: 0
```

All tests passed. Routing behavior is consistent with expectations.

### Failed Test

```
❌ diag-001-health-check - FAILED
   Route mismatch: expected Diagnostic, got Conversational
   Query: "health check"
```

**Interpretation:**
- Query `"health check"` was expected to route to Diagnostic
- Actually routed to Conversational
- Indicates routing logic changed or test expectation is wrong

**Action:**
1. Check if routing logic was intentionally changed
2. If yes: Update test expectation to match new behavior
3. If no: Fix routing logic regression
4. Add `notes` field explaining observed behavior if documenting

### Substring Mismatch

```
❌ diag-001-health-check - FAILED
   Missing substrings: ["System health"]
   Query: "health check"
```

**Interpretation:**
- Route was correct (Diagnostic)
- But response didn't contain expected substring
- Mock response may have changed

**Action:**
- Review mock response functions
- Update `expect_contains` if response format changed legitimately

---

## Best Practices

### 1. Write Descriptive Notes

**Good:**
```toml
notes = "Beta.239 exact match - 'health check' is a core diagnostic phrase"
```

**Bad:**
```toml
notes = "test health check"
```

### 2. Document Observed Behavior

When documenting unexpected behavior (Beta.242 approach):

```toml
notes = "Observed behavior: routes to conversational due to extra whitespace breaking exact match. Candidate for Beta.243 normalization improvement."
```

### 3. Group Related Tests

Keep tests organized by category with clear section headers:

```toml
# ============================================================================
# DIAGNOSTIC QUERIES - TIER 0.5 (Beta.238/239 Phrases)
# ============================================================================
```

### 4. Avoid Over-Using expect_contains

Only use `expect_contains` when validating critical response content. Prefer route validation alone:

```toml
# Good - route validation only
[[test]]
id = "conv-001-how-are-you"
query = "how are you"
expect_route = "conversational"
notes = "Smalltalk query"

# Use sparingly - when response content is critical
[[test]]
id = "diag-001-health-check"
query = "health check"
expect_route = "diagnostic"
expect_contains = ["System health"]  # Only if critical
notes = "Must return health data"
```

### 5. Test Boundary Conditions

Always include edge cases:

- Empty/whitespace queries
- Very long queries
- Special characters
- Mixed case
- Multiple spaces
- Typos (acceptable non-matches)

### 6. Prevent False Positives

For every positive test, add negative tests:

```toml
# Positive test
[[test]]
id = "diag-001-health-check"
query = "health check"
expect_route = "diagnostic"
notes = "Core diagnostic phrase"

# Negative test
[[test]]
id = "false-001-health-insurance"
query = "health insurance"
expect_route = "conversational"
notes = "Contains 'health' but NOT diagnostic context"
```

---

## Troubleshooting

### Issue: Test Harness Out of Sync with Production

**Symptom:**
```
✅ All tests pass in regression suite
❌ Production routes queries differently
```

**Solution:**
The test harness routing logic in `regression_nl_smoke.rs` must be updated when production routing changes.

1. Compare `is_full_diagnostic_query()` in test harness with `unified_query_handler.rs`
2. Update test harness to match production
3. Re-run tests

### Issue: Many Tests Failing After Routing Change

**Symptom:**
```
Failed: 20
```

**Solution:**
1. Determine if routing change was intentional
2. If intentional (Beta.242 approach):
   - Align test expectations with new behavior
   - Add `notes` documenting observed behavior
   - Mark candidates for future fixes
3. If unintentional:
   - Revert routing logic
   - Investigate regression

### Issue: Test Count Mismatch

**Symptom:**
```
Total: 99
Expected: 101
```

**Solution:**
1. Check for syntax errors in TOML (will cause tests to be skipped)
2. Run TOML validator: `cargo test --test regression_nl_smoke`
3. Look for parsing errors in test output

### Issue: Mock Responses Don't Match Production

**Symptom:**
```
❌ Missing substrings: ["expected content"]
```

**Solution:**
Mock responses in test harness are simplified. If production response format changes:

1. Update `mock_diagnostic_response()`, `mock_status_response()`, etc.
2. Or remove overly specific `expect_contains` checks
3. Focus on route validation, not response content

---

## Beta.244 Test Categories and Examples

### Conceptual Question Exclusions

Tests that validate "what is", "explain", "define" patterns route to conversational:

```toml
[[test]]
id = "b244-001-what-is-healthy"
query = "what is a healthy system"
expect_route = "conversational"
notes = "Beta.244: Conceptual exclusion - asking for definition, not diagnostic"

[[test]]
id = "b244-002-explain-health"
query = "can you explain system health to me"
expect_route = "conversational"
notes = "Beta.244: Conceptual pattern 'explain' + 'system health' - educational query"
```

**Key Insight:** These queries contain diagnostic keywords ("healthy system", "system health") but are asking for explanations/definitions rather than requesting actionable diagnostics.

### Contextual Awareness - Positive Indicators

Tests with positive indicators ("this system", "my machine") confirm diagnostic intent:

```toml
[[test]]
id = "b244-003-this-system-health"
query = "check system health on this machine"
expect_route = "diagnostic"
notes = "Beta.244: Positive indicator 'this machine' confirms diagnostic intent"

[[test]]
id = "b244-004-my-machine-health"
query = "how is the health of my machine"
expect_route = "diagnostic"
notes = "Beta.244: Positive indicator 'my machine' confirms actionable query"
```

**Key Insight:** Adding "this", "my", "here" indicates the user wants to check THIS specific machine, not learn about system health in general.

### Contextual Awareness - Negative Indicators

Tests with negative indicators ("in general", "on linux") route to conversational:

```toml
[[test]]
id = "b244-013-general-context"
query = "system health in general on linux"
expect_route = "conversational"
notes = "Beta.244: Negative indicators 'in general' + 'on linux' suggest educational query"

[[test]]
id = "b244-014-theory-question"
query = "what does system health mean in theory"
expect_route = "conversational"
notes = "Beta.244: Negative indicator 'in theory' - theoretical/educational"
```

**Key Insight:** "In general", "in theory", "on linux" (without "this") suggest the user wants general information, not machine-specific diagnostics.

### Temporal Status Patterns

Tests combining temporal indicators with system references route to status:

```toml
[[test]]
id = "b244-015-how-today"
query = "how is my system today"
expect_route = "status"
notes = "Beta.244: Temporal 'today' + system ref 'my system' → status intent"

[[test]]
id = "b244-016-whats-now"
query = "what's happening on this machine now"
expect_route = "status"
notes = "Beta.244: Temporal 'now' + 'this machine' → status check"
```

**Key Insight:** Temporal indicators imply "give me a quick status check right now" rather than "run full diagnostics".

### Importance-Based Status Patterns

Tests combining importance indicators with system references route to status:

```toml
[[test]]
id = "b244-019-anything-important"
query = "anything important to review on this system"
expect_route = "status"
notes = "Beta.244: Importance 'important to review' + 'this system' → status"

[[test]]
id = "b244-020-any-issues"
query = "are there any issues with my machine"
expect_route = "status"
notes = "Beta.244: Importance 'any issues' + 'my machine' → status check"
```

**Key Insight:** "Anything important", "any issues", "anything wrong" combined with system reference suggests quick status overview, not full diagnostic.

### Ambiguous Context Handling

Tests with both positive and negative indicators prefer conversational:

```toml
[[test]]
id = "b244-043-both-indicators"
query = "system health on this machine in general for linux systems"
expect_route = "conversational"
notes = "Beta.244: Both positive ('this machine') and negative ('in general', 'for linux') - ambiguous, prefer conversational"
```

**Key Insight:** When context is ambiguous (contains both specific and general language), route conservatively to conversational to avoid false positives.

---

## Future Expansion Path

### Beta.243+: Recipe Route Coverage

Currently minimal recipe testing. Expand with:

```toml
[[test]]
id = "recipe-001-firewall-enable"
query = "enable firewall"
expect_route = "recipe"
notes = "Firewall enable recipe"
```

### Beta.244+: Fallback Route Validation

Add explicit fallback tests:

```toml
[[test]]
id = "fallback-001-nonsense"
query = "asdf qwerty zxcv"
expect_route = "fallback"
notes = "Random nonsense - should fallback"
```

### Long-Term: 700-Question Suite

Path to comprehensive coverage:
- Beta.241: 43 tests (foundation)
- Beta.242: 101 tests (smoke suite established)
- Beta.243: 115 tests (first routing improvements)
- Beta.244: 172 tests (contextual awareness + 50% growth)
- Beta.245-250: 250-350 tests (recipe expansion, more realistic phrasings)
- Beta.251+: 700+ tests (comprehensive coverage)

---

## Appendix: Complete Example

```toml
# ============================================================================
# DIAGNOSTIC QUERIES - TIER 0.5 (Beta.238/239 Phrases)
# ============================================================================

[[test]]
id = "diag-001-health-check"
query = "health check"
expect_route = "diagnostic"
expect_contains = ["System health"]
notes = "Beta.239 exact match phrase - core diagnostic trigger"

[[test]]
id = "diag-002-system-check"
query = "system check"
expect_route = "diagnostic"
expect_contains = ["System health"]
notes = "Beta.239 exact match phrase"

# ============================================================================
# FALSE POSITIVE PREVENTION - Should NOT trigger diagnostics
# ============================================================================

[[test]]
id = "false-001-health-insurance"
query = "health insurance"
expect_route = "conversational"
notes = "Contains 'health' but not system-related - must NOT trigger diagnostic"

# ============================================================================
# EDGE CASES - Boundary testing
# ============================================================================

[[test]]
id = "edge-001-empty-diagnostic"
query = "diagnostic"
expect_route = "conversational"
notes = "Single word 'diagnostic' without context - should NOT trigger (no 'full' modifier)"
```

---

**Document Version:** V1 (Beta.241/242/243/244)
**Last Updated:** Beta.244 completion
**Maintained By:** Anna development team
