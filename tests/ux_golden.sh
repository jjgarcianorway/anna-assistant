#!/bin/bash
# UX Golden Transcript Test Harness
# Validates UX contract per docs/UX_SPEC.md
# Does NOT require Ollama or root access

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS_COUNT=0
FAIL_COUNT=0

pass() { echo -e "${GREEN}[PASS]${NC} $1"; PASS_COUNT=$((PASS_COUNT + 1)); }
fail() { echo -e "${RED}[FAIL]${NC} $1"; FAIL_COUNT=$((FAIL_COUNT + 1)); }
info() { echo -e "${YELLOW}[INFO]${NC} $1"; }

echo "======================================"
echo "  UX GOLDEN TEST HARNESS"
echo "  $(date -u)"
echo "======================================"
echo

cd "$REPO_ROOT"

# ================================================
# TEST 1: Snapshot Tests Pass
# ================================================
echo "=== T1: Snapshot Tests ==="
if cargo test --package annactl snapshot_tests 2>&1 | grep -q "test result: ok"; then
    pass "All snapshot tests pass"
else
    fail "Snapshot tests failed"
fi
echo

# ================================================
# TEST 2: UX Contract Patterns
# ================================================
echo "=== T2: UX Contract Patterns ==="

# Check: No "ANSWER:" in step.rs or streaming.rs
if grep -q '"ANSWER:' crates/annactl/src/streaming.rs crates/annactl/src/display/step.rs 2>/dev/null; then
    fail "Found 'ANSWER:' prefix (should use 'Anna:')"
else
    pass "No 'ANSWER:' prefix found"
fi

# Check: No "Please confirm:" wrapper
if grep -q '"Please confirm:' crates/annactl/src/display/step.rs 2>/dev/null; then
    fail "Found 'Please confirm:' wrapper"
else
    pass "No 'Please confirm:' wrapper"
fi

# Check: No "Missing information:" wrapper
if grep -q '"Missing information:' crates/annactl/src/display/step.rs 2>/dev/null; then
    fail "Found 'Missing information:' wrapper"
else
    pass "No 'Missing information:' wrapper"
fi

# Check: No "SYSTEM ALERT" header
if grep -q '"SYSTEM ALERT' crates/annactl/src/display/step.rs 2>/dev/null; then
    fail "Found 'SYSTEM ALERT' header"
else
    pass "No 'SYSTEM ALERT' header"
fi

# Check: No verbose timeout explanation
TIMEOUT_LINES=$(grep -A20 "fn print_timeout_error" crates/annactl/src/display/step.rs | wc -l)
if [ "$TIMEOUT_LINES" -lt 10 ]; then
    pass "Timeout error is concise ($TIMEOUT_LINES lines)"
else
    fail "Timeout error too verbose ($TIMEOUT_LINES lines)"
fi

# Check: Consistent indicators [OK]/[!]/[X]
if grep -q '\[OK\]' crates/annactl/src/ui.rs && \
   grep -q '\[!\]' crates/annactl/src/ui.rs && \
   grep -q '\[X\]' crates/annactl/src/ui.rs; then
    pass "Consistent status indicators [OK]/[!]/[X]"
else
    fail "Inconsistent status indicators"
fi

# Check: Error message uses "Unable to complete" not "[FAILED]"
if grep -q '\[FAILED\]' crates/annactl/src/streaming.rs 2>/dev/null; then
    fail "Found '[FAILED]' marker (should use 'Unable to complete')"
else
    pass "No '[FAILED]' marker found"
fi
echo

# ================================================
# TEST 3: Exposure Level Boundaries
# ================================================
echo "=== T3: Exposure Boundaries ==="

# Check: Debug-only output uses debug variable from is_debug_mode()
DEBUG_VAR=$(grep -c "if debug" crates/annactl/src/display/step.rs || echo "0")
DEBUG_INIT=$(grep -c "is_debug_mode()" crates/annactl/src/display/step.rs || echo "0")
if [ "$DEBUG_VAR" -gt 5 ] && [ "$DEBUG_INIT" -gt 0 ]; then
    pass "Debug mode boundary checks found (init:$DEBUG_INIT, uses:$DEBUG_VAR)"
else
    fail "Insufficient debug mode boundary checks"
fi

# Check: No forbidden patterns in user-facing output
FORBIDDEN_PATTERNS="sudo systemctl|Run: sudo|Try: sudo"
if grep -E "$FORBIDDEN_PATTERNS" crates/annactl/src/display/step.rs 2>/dev/null | grep -v test | grep -v "#"; then
    fail "Found forbidden patterns in step.rs"
else
    pass "No forbidden command patterns in step.rs"
fi
echo

# ================================================
# TEST 4: Golden Fixtures Exist
# ================================================
echo "=== T4: Golden Fixtures ==="

if [ -f "$SCRIPT_DIR/golden/t1_simple_query.fixture" ]; then
    pass "T1 fixture exists"
else
    fail "T1 fixture missing"
fi

if [ -f "$SCRIPT_DIR/golden/t2_confirmation.fixture" ]; then
    pass "T2 fixture exists"
else
    fail "T2 fixture missing"
fi

if [ -f "$SCRIPT_DIR/golden/t3_failure.fixture" ]; then
    pass "T3 fixture exists"
else
    fail "T3 fixture missing"
fi
echo

# ================================================
# TEST 4B: Exposure Level Fixtures
# ================================================
echo "=== T4B: Exposure Level Fixtures ==="

# T4: Silent exposure - only Done packet
if [ -f "$SCRIPT_DIR/golden/t4_silent_exposure.fixture" ]; then
    pass "T4 silent fixture exists"
    # Validate: No Step packets in silent mode
    if grep -q '"Step"' "$SCRIPT_DIR/golden/t4_silent_exposure.fixture"; then
        fail "T4 silent fixture should not contain Step packets"
    else
        pass "T4 silent fixture contains no Step packets"
    fi
    # Validate: Must have Done packet
    if grep -q '"Done"' "$SCRIPT_DIR/golden/t4_silent_exposure.fixture"; then
        pass "T4 silent fixture contains Done packet"
    else
        fail "T4 silent fixture missing Done packet"
    fi
else
    fail "T4 silent fixture missing"
fi

# T5: Summary exposure - minimal progress
if [ -f "$SCRIPT_DIR/golden/t5_summary_exposure.fixture" ]; then
    pass "T5 summary fixture exists"
    # Validate: Should have FinalPrompt step
    if grep -q '"FinalPrompt"' "$SCRIPT_DIR/golden/t5_summary_exposure.fixture"; then
        pass "T5 summary fixture contains FinalPrompt"
    else
        fail "T5 summary fixture missing FinalPrompt"
    fi
    # Validate: Should NOT have debug-only steps (AnnaToLlm, CommandExec)
    if grep -qE '"AnnaToLlm"|"CommandExec"' "$SCRIPT_DIR/golden/t5_summary_exposure.fixture"; then
        fail "T5 summary fixture should not contain debug-only steps"
    else
        pass "T5 summary fixture excludes debug-only steps"
    fi
else
    fail "T5 summary fixture missing"
fi

# T6: Debug exposure - all steps visible
if [ -f "$SCRIPT_DIR/golden/t6_debug_exposure.fixture" ]; then
    pass "T6 debug fixture exists"
    # Validate: Should have debug-only steps
    if grep -q '"AnnaToLlm"' "$SCRIPT_DIR/golden/t6_debug_exposure.fixture"; then
        pass "T6 debug fixture contains AnnaToLlm"
    else
        fail "T6 debug fixture missing AnnaToLlm"
    fi
    if grep -q '"CommandExec"' "$SCRIPT_DIR/golden/t6_debug_exposure.fixture"; then
        pass "T6 debug fixture contains CommandExec"
    else
        fail "T6 debug fixture missing CommandExec"
    fi
    if grep -q '"CommandOutput"' "$SCRIPT_DIR/golden/t6_debug_exposure.fixture"; then
        pass "T6 debug fixture contains CommandOutput"
    else
        fail "T6 debug fixture missing CommandOutput"
    fi
else
    fail "T6 debug fixture missing"
fi
echo

# ================================================
# TEST 4C: Status Fixtures (Phase 21)
# ================================================
echo "=== T4C: Status Fixtures ==="

# Status healthy fixture
if [ -f "$SCRIPT_DIR/golden/status_healthy.fixture" ]; then
    pass "status_healthy fixture exists"
    # Validate: Must have all 7 sections
    if grep -q "VERSION" "$SCRIPT_DIR/golden/status_healthy.fixture" && \
       grep -q "UPDATES" "$SCRIPT_DIR/golden/status_healthy.fixture" && \
       grep -q "SERVICE" "$SCRIPT_DIR/golden/status_healthy.fixture" && \
       grep -q "PERMISSIONS" "$SCRIPT_DIR/golden/status_healthy.fixture" && \
       grep -q "CONFIG" "$SCRIPT_DIR/golden/status_healthy.fixture" && \
       grep -q "HELPERS" "$SCRIPT_DIR/golden/status_healthy.fixture" && \
       grep -q "MODELS" "$SCRIPT_DIR/golden/status_healthy.fixture"; then
        pass "status_healthy fixture contains all 7 sections"
    else
        fail "status_healthy fixture missing sections"
    fi
else
    fail "status_healthy fixture missing"
fi

# Status daemon down fixture
if [ -f "$SCRIPT_DIR/golden/status_daemon_down.fixture" ]; then
    pass "status_daemon_down fixture exists"
    if grep -q '\[X\].*not running' "$SCRIPT_DIR/golden/status_daemon_down.fixture"; then
        pass "status_daemon_down shows [X] not running"
    else
        fail "status_daemon_down missing [X] not running"
    fi
else
    fail "status_daemon_down fixture missing"
fi

# Status no group fixture
if [ -f "$SCRIPT_DIR/golden/status_no_group.fixture" ]; then
    pass "status_no_group fixture exists"
    if grep -q '\[X\].*not in anna group' "$SCRIPT_DIR/golden/status_no_group.fixture"; then
        pass "status_no_group shows [X] not in anna group"
    else
        fail "status_no_group missing [X] not in anna group"
    fi
else
    fail "status_no_group fixture missing"
fi

# Status no updates fixture
if [ -f "$SCRIPT_DIR/golden/status_no_updates.fixture" ]; then
    pass "status_no_updates fixture exists"
    if grep -q "unknown" "$SCRIPT_DIR/golden/status_no_updates.fixture" && \
       grep -q "never" "$SCRIPT_DIR/golden/status_no_updates.fixture"; then
        pass "status_no_updates shows unknown/never values"
    else
        fail "status_no_updates missing unknown/never values"
    fi
else
    fail "status_no_updates fixture missing"
fi
echo

# ================================================
# TEST 5: UX Spec Exists and Valid
# ================================================
echo "=== T5: UX Spec Document ==="

if [ -f "$REPO_ROOT/docs/UX_SPEC.md" ]; then
    pass "UX_SPEC.md exists"

    # Check required sections
    if grep -q "Prefix Rules" "$REPO_ROOT/docs/UX_SPEC.md"; then
        pass "Contains Prefix Rules section"
    else
        fail "Missing Prefix Rules section"
    fi

    if grep -q "Do Not Regress" "$REPO_ROOT/docs/UX_SPEC.md"; then
        pass "Contains regression checklist"
    else
        fail "Missing regression checklist"
    fi

    # Check line count
    SPEC_LINES=$(wc -l < "$REPO_ROOT/docs/UX_SPEC.md")
    if [ "$SPEC_LINES" -le 250 ]; then
        pass "UX_SPEC.md is within limit ($SPEC_LINES/250 lines)"
    else
        fail "UX_SPEC.md exceeds 250 lines ($SPEC_LINES)"
    fi
else
    fail "UX_SPEC.md not found"
fi

# STATUS_SPEC.md (Phase 21)
if [ -f "$REPO_ROOT/docs/STATUS_SPEC.md" ]; then
    pass "STATUS_SPEC.md exists"

    # Check required sections
    if grep -q "Section Order" "$REPO_ROOT/docs/STATUS_SPEC.md"; then
        pass "STATUS_SPEC contains Section Order"
    else
        fail "STATUS_SPEC missing Section Order"
    fi

    if grep -q "Do Not Regress" "$REPO_ROOT/docs/STATUS_SPEC.md"; then
        pass "STATUS_SPEC contains regression checklist"
    else
        fail "STATUS_SPEC missing regression checklist"
    fi

    # Check line count
    STATUS_LINES=$(wc -l < "$REPO_ROOT/docs/STATUS_SPEC.md")
    if [ "$STATUS_LINES" -le 250 ]; then
        pass "STATUS_SPEC.md is within limit ($STATUS_LINES/250 lines)"
    else
        fail "STATUS_SPEC.md exceeds 250 lines ($STATUS_LINES)"
    fi
else
    fail "STATUS_SPEC.md not found"
fi
echo

# ================================================
# Summary
# ================================================
echo "======================================"
echo "  SUMMARY"
echo "======================================"
echo -e "Passed: ${GREEN}$PASS_COUNT${NC}"
echo -e "Failed: ${RED}$FAIL_COUNT${NC}"
echo

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo -e "${RED}UX GOLDEN TESTS FAILED${NC}"
    exit 1
fi

echo -e "${GREEN}UX GOLDEN TESTS PASSED${NC}"
exit 0
