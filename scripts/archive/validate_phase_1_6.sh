#!/bin/bash
# Phase 1.6 Mirror Audit - Production Validation Harness
# Anna v1.6.0-rc.1 - Portable CI/CD validation script
# Generated: 2025-11-12
#
# Usage: ./scripts/validate_phase_1_6.sh
# Exit codes: 0 = all tests pass, 1 = one or more tests fail

set -e  # Exit on any error
set -u  # Exit on undefined variable

# Detect script directory and repo root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to repo root for consistent paths
cd "$REPO_ROOT"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT="${REPO_ROOT}/validation_report_${TIMESTAMP}.txt"

echo "=== Phase 1.6 Mirror Audit Validation Harness ===" | tee "$REPORT"
echo "Timestamp: $(date -u)" | tee -a "$REPORT"
echo "Version: 1.6.0-rc.1" | tee -a "$REPORT"
echo "Repo root: $REPO_ROOT" | tee -a "$REPORT"
echo | tee -a "$REPORT"

# Test counters
PASS=0
FAIL=0

# Helper function
test_result() {
    local test_name="$1"
    local result="$2"
    if [ "$result" = "PASS" ]; then
        echo "✓ $test_name: PASS" | tee -a "$REPORT"
        ((PASS++))
    else
        echo "✗ $test_name: FAIL - $3" | tee -a "$REPORT"
        ((FAIL++))
    fi
}

# ============================================================================
# A) CLI INTEGRATION TESTS
# ============================================================================
echo "A) CLI Integration Tests" | tee -a "$REPORT"
echo "─────────────────────────" | tee -a "$REPORT"

# Determine binary location (debug or release)
if [ -f "target/release/annactl" ] && [ -f "target/release/annad" ]; then
    ANNACTL="target/release/annactl"
    ANNAD="target/release/annad"
    BUILD_TYPE="release"
elif [ -f "target/debug/annactl" ] && [ -f "target/debug/annad" ]; then
    ANNACTL="target/debug/annactl"
    ANNAD="target/debug/annad"
    BUILD_TYPE="debug"
else
    echo "✗ No binaries found. Run 'cargo build' first." | tee -a "$REPORT"
    exit 1
fi

echo "Using $BUILD_TYPE binaries" | tee -a "$REPORT"
test_result "A1: Binaries exist" "PASS"

# Verify version (accept 1.6.0-rc.1 or later, including 1.7.0-alpha.1)
VERSION=$(./"$ANNACTL" --version 2>&1 | grep -oE "[0-9]+\.[0-9]+\.[0-9]+-(rc|alpha)\.[0-9]+" || true)
if [ "$VERSION" = "1.6.0-rc.1" ] || [ "$VERSION" = "1.7.0-alpha.1" ]; then
    test_result "A2: Version correct" "PASS"
else
    test_result "A2: Version correct" "FAIL" "Got $VERSION, expected 1.6.0-rc.1 or 1.7.0-alpha.1"
fi

# Test command parsing (daemon not required)
./"$ANNACTL" mirror audit-forecast --help > /dev/null 2>&1
test_result "A3: audit-forecast --help" "PASS"

./"$ANNACTL" mirror reflect-temporal --help > /dev/null 2>&1
test_result "A4: reflect-temporal --help" "PASS"

# Verify --json flag exists
if ./"$ANNACTL" mirror audit-forecast --help 2>&1 | grep -q "\-\-json"; then
    test_result "A5: --json flag present" "PASS"
else
    test_result "A5: --json flag present" "FAIL" "--json flag not found in help"
fi

# ============================================================================
# B) CODE INSPECTION: STATE PERSISTENCE
# ============================================================================
echo | tee -a "$REPORT"
echo "B) State Persistence (Code Inspection)" | tee -a "$REPORT"
echo "───────────────────────────────────────" | tee -a "$REPORT"

# Verify append-only implementation
if grep -q "\.append(true)" crates/annad/src/mirror_audit/mod.rs; then
    test_result "B1: Append-only mode" "PASS"
else
    test_result "B1: Append-only mode" "FAIL" ".append(true) not found"
fi

# Verify sync_all for durability
if grep -q "sync_all()" crates/annad/src/mirror_audit/mod.rs; then
    test_result "B2: Durability (sync_all)" "PASS"
else
    test_result "B2: Durability (sync_all)" "FAIL" "sync_all() not found"
fi

# Verify state persistence
if grep -q "async fn save_state" crates/annad/src/mirror_audit/mod.rs; then
    test_result "B3: State persistence function" "PASS"
else
    test_result "B3: State persistence function" "FAIL" "save_state function not found"
fi

# ============================================================================
# C) TEMPORAL INTEGRITY SCORE VERIFICATION
# ============================================================================
echo | tee -a "$REPORT"
echo "C) Temporal Integrity Score Formula" | tee -a "$REPORT"
echo "────────────────────────────────────" | tee -a "$REPORT"

# Verify TIS calculation exists
if grep -q "pub fn calculate_temporal_integrity" crates/annad/src/mirror_audit/align.rs; then
    test_result "C1: TIS calculation exists" "PASS"
else
    test_result "C1: TIS calculation exists" "FAIL" "calculate_temporal_integrity not found"
fi

# Verify weights in implementation
if grep -A 5 "TemporalIntegrityScore::calculate" crates/annad/src/mirror_audit/types.rs | grep -q "0.5.*0.3.*0.2"; then
    test_result "C2: TIS weights (0.5/0.3/0.2)" "PASS"
else
    test_result "C2: TIS weights (0.5/0.3/0.2)" "FAIL" "Weights not found or incorrect"
fi

# Verify MAE calculation
if grep -q "mean_absolute_error" crates/annad/src/mirror_audit/align.rs; then
    test_result "C3: MAE calculation" "PASS"
else
    test_result "C3: MAE calculation" "FAIL" "mean_absolute_error not found"
fi

# ============================================================================
# D) BIAS DETECTION IMPLEMENTATION
# ============================================================================
echo | tee -a "$REPORT"
echo "D) Bias Detection" | tee -a "$REPORT"
echo "─────────────────" | tee -a "$REPORT"

# Verify bias types
BIAS_TYPES="ConfirmationBias RecencyBias AvailabilityBias StrainUnderestimation HealthOverestimation EmpathyInconsistency"
for bias in $BIAS_TYPES; do
    if grep -q "$bias" crates/annad/src/mirror_audit/types.rs; then
        test_result "D: Bias type $bias" "PASS"
    else
        test_result "D: Bias type $bias" "FAIL" "$bias not found in types"
    fi
done

# Verify minimum thresholds
if grep -q "MIN_SAMPLE_SIZE.*5" crates/annad/src/mirror_audit/bias.rs; then
    test_result "D: Min sample size (5)" "PASS"
else
    test_result "D: Min sample size (5)" "FAIL" "MIN_SAMPLE_SIZE not 5"
fi

if grep -q "MIN_CONFIDENCE.*0\.6" crates/annad/src/mirror_audit/bias.rs; then
    test_result "D: Min confidence (0.6)" "PASS"
else
    test_result "D: Min confidence (0.6)" "FAIL" "MIN_CONFIDENCE not 0.6"
fi

# ============================================================================
# E) ADVISORY-ONLY VERIFICATION
# ============================================================================
echo | tee -a "$REPORT"
echo "E) Advisory-Only Mode" | tee -a "$REPORT"
echo "─────────────────────" | tee -a "$REPORT"

# Verify no auto-apply in RPC handlers
if ! grep -i "auto.*apply\|automatically.*apply" crates/annad/src/rpc_server.rs | grep -qi mirror; then
    test_result "E1: No auto-apply in RPC" "PASS"
else
    test_result "E1: No auto-apply in RPC" "FAIL" "Found auto-apply references"
fi

# Verify advisory warnings in CLI
if grep -q "Advisory Only" crates/annactl/src/mirror_commands.rs; then
    test_result "E2: Advisory warnings in CLI" "PASS"
else
    test_result "E2: Advisory warnings in CLI" "FAIL" "Advisory warnings not found"
fi

# Verify CHANGELOG security model
if grep -q "Advisory-only adjustments" CHANGELOG.md && grep -q "never auto-executed" CHANGELOG.md; then
    test_result "E3: Security model documented" "PASS"
else
    test_result "E3: Security model documented" "FAIL" "Security model not documented"
fi

# ============================================================================
# F) SECURITY MODEL
# ============================================================================
echo | tee -a "$REPORT"
echo "F) Security Model" | tee -a "$REPORT"
echo "─────────────────" | tee -a "$REPORT"

# Verify audit trail paths
if grep -q "mirror-audit.jsonl" crates/annad/src/mirror_audit/mod.rs; then
    test_result "F1: Audit log path configured" "PASS"
else
    test_result "F1: Audit log path configured" "FAIL" "Audit log path not found"
fi

# Verify state file path
if grep -q "mirror_audit/state.json" crates/annad/src/mirror_audit/mod.rs; then
    test_result "F2: State file path configured" "PASS"
else
    test_result "F2: State file path configured" "FAIL" "State file path not found"
fi

# Verify conscience sovereignty preserved
if grep -q "Conscience sovereignty preserved" CHANGELOG.md; then
    test_result "F3: Conscience sovereignty documented" "PASS"
else
    test_result "F3: Conscience sovereignty documented" "FAIL" "Conscience sovereignty not documented"
fi

# ============================================================================
# SUMMARY
# ============================================================================
echo | tee -a "$REPORT"
echo "═══════════════════════════════════════" | tee -a "$REPORT"
echo "VALIDATION SUMMARY" | tee -a "$REPORT"
echo "═══════════════════════════════════════" | tee -a "$REPORT"
echo "PASS: $PASS" | tee -a "$REPORT"
echo "FAIL: $FAIL" | tee -a "$REPORT"
TOTAL=$((PASS + FAIL))
echo "TOTAL: $TOTAL" | tee -a "$REPORT"
echo | tee -a "$REPORT"
echo "Report saved to: $REPORT" | tee -a "$REPORT"

if [ $FAIL -eq 0 ]; then
    echo | tee -a "$REPORT"
    echo "✓ ALL TESTS PASSED" | tee -a "$REPORT"
    echo "Status: READY FOR DEPLOYMENT" | tee -a "$REPORT"
    exit 0
else
    echo | tee -a "$REPORT"
    echo "✗ SOME TESTS FAILED" | tee -a "$REPORT"
    echo "Status: REQUIRES ATTENTION" | tee -a "$REPORT"
    exit 1
fi
