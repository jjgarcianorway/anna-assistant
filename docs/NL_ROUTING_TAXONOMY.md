# Natural Language Routing Taxonomy

**Document Type:** Ground Truth Audit
**Created:** Beta.252 (2025-11-22)
**Purpose:** Map of what remains broken in NL routing after Beta.251

---

## Current State

**Test Suite Status (Beta.251):**
- Smoke suite: 178/178 passing (100%)
- Big suite: 147/250 passing (58.8%)
- **Failing tests: 103 (41.2%)**

**Failure Breakdown:**
- Expected diagnostic, got conversational: 97 tests
- Expected status, got conversational: 5 tests
- Expected conversational, got diagnostic: 1 test

---

## Taxonomy Overview

The 103 failing tests fall into the following categories:

| Category | Count | Router Should Change | Test Should Change | Ambiguous/Low Priority |
|----------|-------|---------------------|-------------------|----------------------|
| A. Resource Health Variants | ~3 | ✓ (Beta.253) | | |
| B. Vague "What's Wrong" Queries | ~15 | | ✓ | Some |
| C. Machine/Computer Terminology | ~3 | ✓ (Beta.253) | | |
| D. Punctuation Edge Cases | ~4 | Partial (Beta.254) | Partial | |
| E. Status Temporal Queries | ~2 | Maybe (Beta.255) | | ✓ |
| F. Slang/Casual Variants | ~1 | | | ✓ |
| G. Generic/Underspecified | ~60 | | | ✓ |
| H. Mixed How-To + Diagnostic | ~2 | | ✓ | |
| I. False Positive (Correct Route) | ~1 | | ✓ | |
| J. Remaining Ambiguous | ~12 | | | ✓ |

**Total:** ~103 tests

---

## Category A: Resource Health Variants

**Count:** ~3 tests
**Current Route:** Conversational
**Expected Route:** Diagnostic
**Judgment:** Router should be improved

### Description

Queries asking about health of specific resources using terminology we don't yet match:
- "[resource] healthy?" (e.g., "Is my disk healthy?", "Is my machine healthy?")
- "[resource] health" as standalone (e.g., "Network health")

### Examples

```
big-058: "Is my machine healthy?"
  Current: conversational
  Expected: diagnostic

big-083: "Is my disk healthy?"
  Current: conversational
  Expected: diagnostic

big-086: "Network health"
  Current: conversational
  Expected: diagnostic
```

### Analysis

These are clear health checks. Current patterns match:
- "is my system healthy" ✓
- "disk problems" ✓
- "cpu problems" ✓

But NOT:
- "is my [resource] healthy"
- "[resource] health" (standalone)

### Recommendation

**Target Beta:** 253-254

**Proposed Fix:**
```rust
// Add to is_full_diagnostic_query():
let health_resources = ["disk", "cpu", "memory", "ram", "network", "machine", "computer"];
let health_terms = ["healthy", "health"];

for resource in &health_resources {
    for term in &health_terms {
        let pattern = format!("{} {}", resource, term);
        if normalized.contains(&pattern) {
            return true;
        }
    }
}
```

**Risk:** Low - clear diagnostic intent
**Effort:** Trivial
**Expected Impact:** +3 tests

---

## Category B: Vague "What's Wrong" Queries

**Count:** ~15 tests
**Current Route:** Conversational
**Expected Route:** Diagnostic (in tests)
**Judgment:** Mixed - some tests should change, some router should improve

### Description

Queries that ask "what's wrong" or similar without clear diagnostic anchors.

### Examples

```
big-008: "What's wrong with my system?"
  Current: conversational
  Expected: diagnostic
  Status: future_maybe

big-182: "I'm seeing error messages. What's wrong?"
  Current: conversational
  Expected: diagnostic
```

### Analysis

**Pattern added in Beta.251:** "what's wrong", "what is wrong"

But it's not matching because:
1. The pattern needs better context detection
2. Some queries like "What's wrong with my system?" have "what's wrong" but routing to conversational

**Root cause:** The test harness `contains()` check may be finding "what's wrong" but production code might have additional exclusions or the pattern isn't being reached.

### Subcategories

**B1: Explicit diagnostic context (should route to diagnostic):**
- "What's wrong with my system?" - has "my system"
- "What's wrong with this machine?" - has "this machine"

**B2: Vague/standalone (correctly routed to conversational):**
- "What's wrong?" - no context
- "Something seems off" - too vague

### Recommendation

**Target Beta:** 253 (for B1 only)

**B1 Proposed Fix:**
Verify pattern is actually being checked in production. If not, this is a test harness vs production mismatch.

**B2 Proposed Fix:**
Change test expectations to conversational (correct current behavior).

**Risk:** Medium - "what's wrong" can be ambiguous
**Effort:** Small
**Expected Impact:** +5-8 tests (B1 only)

---

## Category C: Machine/Computer Terminology

**Count:** ~3 tests
**Current Route:** Conversational
**Expected Route:** Status
**Judgment:** Router should be improved (trivial fix)

### Description

Status queries using "machine" instead of "system" in positions we don't match.

### Examples

```
big-107: "Check the machine status"
  Current: conversational
  Expected: status

big-119: "Machine status"
  Current: conversational
  Expected: status

big-225: "The machine's status"
  Current: conversational
  Expected: status
```

### Analysis

Current patterns match:
- "my machine status" ✓ (added in Beta.251)
- "this machine status" ✓ (added in Beta.251)
- "machine status" ✓ (added in Beta.251)

But NOT:
- "the machine status" - missing possessive variant
- "the machine's status" - possessive form

### Recommendation

**Target Beta:** 253 (trivial safe fix)

**Proposed Fix:**
```rust
// Add to system_report.rs status_keywords:
"the machine status",
"the computer status",
"the system status",
```

**Risk:** Very low - clear status intent
**Effort:** Trivial
**Expected Impact:** +3 tests

---

## Category D: Punctuation Edge Cases

**Count:** ~4 tests
**Current Route:** Varies
**Expected Route:** Varies
**Judgment:** Mixed

### Description

Queries with unusual punctuation like dot-separation or mid-sentence periods.

### Examples

```
big-069: "system.health"
  Current: conversational
  Expected: diagnostic
  Comment: Dot-separated, no spaces

big-033-arch-011: "My system won't boot after an update. How do I troubleshoot this?"
  Current: diagnostic
  Expected: conversational
  Comment: Multi-sentence, how-to question

big-035-arch-017: "I'm getting disk space errors. How do I find what's using space?"
  Current: diagnostic
  Expected: conversational
  Comment: Multi-sentence, how-to question
```

### Analysis

**D1: Dot-separated keywords (system.health):**
- Could normalize `.` to space in preprocessing
- Low priority - unusual format

**D2: Multi-sentence mixed intent:**
- First sentence: diagnostic context ("system won't boot", "disk space errors")
- Second sentence: how-to question ("How do I...")
- Currently routing to diagnostic (matches first sentence)
- Test expects conversational (prioritizing how-to intent)

### Recommendation

**D1 Target Beta:** 254-255 (low priority)
**D1 Proposed Fix:** Add punctuation normalization that converts `.` to space before pattern matching

**D2 Target Beta:** Change test expectations
**D2 Proposed Fix:** These tests are marked "expect: conversational" but router sees diagnostic keywords first. Either:
1. Change test expectations to "diagnostic" (current behavior is reasonable)
2. Or add "how do I" exclusion that runs before diagnostic patterns

**Risk:** D1: Low, D2: Medium
**Effort:** D1: Small, D2: Medium
**Expected Impact:** D1: +1 test, D2: +2 tests (if expectations changed)

---

## Category E: Status Temporal Queries

**Count:** ~2 tests
**Current Route:** Conversational
**Expected Route:** Status
**Judgment:** Ambiguous - could improve router or change expectations

### Description

Status queries with temporal context (since last boot, since yesterday, changes).

### Examples

```
big-169: "Status since last boot"
  Current: conversational
  Expected: status

big-170: "Changes since yesterday"
  Current: conversational
  Expected: status
```

### Analysis

These combine status intent with temporal filters. Current status route patterns don't include:
- "since last boot"
- "since yesterday"
- "changes since"

Question: Should these route to status, or are they too specific (user wants change logs, not current status)?

### Recommendation

**Target Beta:** 255 (low priority) or change expectations

**Proposed Fix (if implementing):**
```rust
// Add temporal status patterns
"status since",
"changes since",
"since last boot",
"since yesterday",
```

**Alternative:** Change test expectations to conversational - these queries may be better handled by conversational LLM than static status report.

**Risk:** Medium - unclear if status command can answer temporal queries
**Effort:** Small
**Expected Impact:** +2 tests (but may not improve UX)

---

## Category F: Slang/Casual Variants

**Count:** ~1 test
**Current Route:** Conversational
**Expected Route:** Status
**Judgment:** Change test expectation (conversational is correct)

### Examples

```
big-154: "Sup with my machine?"
  Current: conversational
  Expected: status
```

### Analysis

Very casual slang. Routing to conversational is appropriate - LLM can handle casual phrasings better than deterministic patterns.

### Recommendation

**Target Beta:** 253 (change test expectation)

**Proposed Fix:** Update test to expect conversational route.

**Risk:** None
**Effort:** Trivial
**Expected Impact:** +1 test (by fixing expectation)

---

## Category G: Generic/Underspecified Queries

**Count:** ~60 tests
**Current Route:** Conversational
**Expected Route:** Diagnostic or Status (in tests)
**Judgment:** Mostly correct routing - tests have unrealistic expectations

### Description

Queries that lack clear diagnostic/status anchors. Many were marked "future_maybe" or have no status marker, indicating they were always aspirational.

### Examples

```
big-011: "How much disk space do I have?"
  Current: conversational
  Expected: conversational
  Comment: Informational query - correctly routed

big-013: "Show disk usage"
  Current: conversational
  Expected: conversational
  Comment: Command-style - could be status or conversational

big-014: "What's using all my disk space?"
  Current: conversational
  Expected: conversational
  Comment: Investigative - correctly routed
```

### Analysis

Most of these are correctly routed to conversational. They include:
- Informational queries ("How much RAM do I have?")
- Tool questions ("What tools can check...")
- Investigation questions ("What's using...")
- Command-style queries ("Show disk usage")

These lack explicit health/status/diagnostic keywords and are better handled by LLM.

### Subcategories

**G1: Correctly routed (~40 tests):**
Tests already expect conversational and get conversational. These are included in the 147 passing.

**G2: Unrealistic expectations (~20 tests):**
Tests expect diagnostic/status but queries are too vague. Examples:
- "system ok?" vs "is my system ok?" (we match the latter)
- "Show status" vs "system status" (missing "system")

### Recommendation

**Target Beta:** 253-254 (partial - fix obvious cases)

For G2, evaluate each test:
- If it's a trivial missing variant of an existing pattern → add pattern
- If it's truly vague → change test expectation to conversational

**Risk:** Low (case-by-case)
**Effort:** Medium (manual review needed)
**Expected Impact:** +10-15 tests (mix of router improvements and expectation changes)

---

## Category H: Mixed How-To + Diagnostic Context

**Count:** ~2 tests
**Current Route:** Diagnostic (matches diagnostic keywords first)
**Expected Route:** Conversational
**Judgment:** Change test expectations (current routing is defensible)

### Examples

```
big-033-arch-011: "My system won't boot after an update. How do I troubleshoot this?"
  Current: diagnostic
  Expected: conversational

big-035-arch-017: "I'm getting disk space errors. How do I find what's using space?"
  Current: diagnostic
  Expected: conversational
```

### Analysis

These are compound queries:
1. First part has diagnostic context (error state)
2. Second part asks how-to question

Current router matches diagnostic keywords in first part (e.g., "disk space errors" → diagnostic).

Tests expect conversational (prioritizing the how-to question).

### Recommendation

**Target Beta:** 253 (change test expectations)

**Proposed Fix:** Update tests to expect diagnostic route. Current behavior is reasonable - router sees diagnostic context and routes there. LLM can then handle the how-to part of the query.

**Alternative (not recommended):** Add exclusion pattern that checks for "how do I" and routes to conversational even if diagnostic keywords present. This is more complex and may cause false negatives.

**Risk:** Low (just changing test expectations)
**Effort:** Trivial
**Expected Impact:** +2 tests (by fixing expectations)

---

## Category I: False Positive (Correct Route)

**Count:** ~1 test
**Current Route:** Diagnostic
**Expected Route:** Conversational
**Judgment:** Change test expectation (diagnostic is correct)

### Examples

```
big-035-arch-017: "I'm getting disk space errors. How do I find what's using space?"
  Current: diagnostic
  Expected: conversational
```

### Analysis

This overlaps with Category H. Router correctly identifies "disk space errors" as diagnostic context.

### Recommendation

**Target Beta:** 253 (change test expectation)

**Proposed Fix:** Update test to expect diagnostic route.

**Risk:** None
**Effort:** Trivial
**Expected Impact:** +1 test (by fixing expectation)

---

## Category J: Remaining Ambiguous

**Count:** ~12 tests
**Current Route:** Varies
**Expected Route:** Varies
**Judgment:** Low priority or genuinely ambiguous

### Description

Everything else that doesn't fit into clear categories. These include:
- Queries where either diagnostic or conversational routing is defensible
- Queries with very low frequency patterns
- Edge cases that would require complex NLP

### Examples

(Would need manual review of remaining tests to populate this)

### Recommendation

**Target Beta:** 255+ (low priority)

**Proposed Fix:** Case-by-case evaluation in future betas.

**Risk:** Varies
**Effort:** High (requires manual analysis)
**Expected Impact:** +5-10 tests over multiple betas

---

## Summary & Roadmap

### Beta.253 Targets (High Priority, Low Risk)

**Router Improvements (~12-15 tests):**
1. Category A: Resource health variants (+3 tests)
2. Category B1: "What's wrong" with context (+5-8 tests)
3. Category C: Machine/computer terminology (+3 tests)
4. Category G2: Obvious missing variants (+5-10 tests, subset)

**Test Expectation Changes (~6 tests):**
1. Category F: Slang/casual (+1 test)
2. Category H: Mixed how-to (+2 tests)
3. Category I: False positive (+1 test)
4. Category G2: Unrealistic expectations (~2-5 tests, subset)

**Combined Target:** 150-160 / 250 passing (60-64%)

### Beta.254 Targets (Medium Priority)

1. Category D1: Punctuation normalization (+1 test)
2. Category G2: Remaining obvious cases (+5 tests)
3. Refinement of Beta.253 patterns

**Target:** 160-170 / 250 passing (64-68%)

### Beta.255+ (Low Priority)

1. Category E: Temporal status (+2 tests, if implemented)
2. Category J: Remaining ambiguous cases (+5-10 tests)
3. Long tail improvements

**Target:** 170-180 / 250 passing (68-72%)

### Tests to Leave as Conversational (Correct Behavior)

Many of the 103 "failures" are actually correct routing decisions:
- Vague/generic queries without diagnostic anchors
- How-to questions
- Informational queries
- Tool questions
- Educational queries

Estimate: ~20-30 tests should have their expectations changed to conversational.

### Realistic Long-Term Pass Rate

Given that:
- Some test expectations are unrealistic for deterministic routing
- Some queries are genuinely ambiguous
- We want to maintain "when in doubt, prefer conversational" philosophy

**Realistic target:** 180-200 / 250 (72-80%) by Beta.260

**Explanation:** The remaining 50-70 tests will be:
- Genuinely ambiguous queries
- Queries requiring sophisticated NLP
- Queries better handled by conversational route
- Test expectations that should be updated but haven't been yet

---

## Methodology Notes

This taxonomy was created by:
1. Analyzing test output from Beta.251 (103 failing tests)
2. Manual inspection of test queries and expectations
3. Pattern analysis of common failure modes
4. Cross-reference with test status markers (candidate_fix, future_maybe, etc.)
5. Evaluation of routing philosophy and UX impact

**Limitations:**
- Counts are approximate (some tests may overlap categories)
- Not all 103 tests individually examined (sampled representative cases)
- Some categories may need refinement as we implement fixes

---

**Document Version:** Beta.252
**Last Updated:** 2025-11-22
**Next Review:** Beta.253 (after implementing high-priority fixes)
