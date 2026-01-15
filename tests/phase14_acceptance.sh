#!/bin/bash
# Phase 14 Acceptance Test Runner
# Runs 10 real-world queries across all exposure levels
# Usage: ./tests/phase14_acceptance.sh [level]
# Levels: silent, summary, dialogue, debug, all (default: all)

set -euo pipefail

ANNACTL="${ANNACTL:-./target/release/annactl}"
CONFIG_FILE="/etc/anna/anna.toml"
RESULTS_DIR="/tmp/anna-phase14-results"
LEVEL="${1:-all}"

# Forbidden patterns that must NEVER appear in output
FORBIDDEN_PATTERNS=(
    "sudo systemctl"
    "Run: sudo"
    "Try: sudo"
    "Execute:"
    "Run this command"
    "I think"
    "I decide"
    "I want"
    "I feel"
    "critical"
    "urgent"
    "immediately"
    "you must"
    "edit this file yourself"
    "WARNING!"
    "DANGER"
    "PANIC"
)

# Test queries
QUERIES=(
    "what is my current kernel version?"
    "why did my boot time increase since yesterday?"
    "can you change my GDM resolution to 1920x1080@120hz or at least scale up if it's 4K?"
    "can you ensure my laptop cannot go to sleep or suspend in any case? From GDM screen to closing the lid or anything else."
    "show me why my Wi-Fi disconnects every few hours"
    "free up disk space but don't remove anything important"
    "make my system quieter, the fan ramps up too often"
    "why did you restart a service earlier today?"
    "pretend something is broken and show me how you would react"
    "explain what you did the last time you fixed an error automatically"
)

# Query purposes (for reporting)
PURPOSES=(
    "Baseline informational query"
    "Diagnostics and historical comparison"
    "Desktop/display specialist, privileged boundary"
    "Power management, multi-layer policy"
    "Network specialist, logs, pattern detection"
    "Storage specialist, risk evaluation"
    "Thermal/performance specialist"
    "Audit trail and internal comms"
    "Exposure gate + sanitization stress test"
    "Replay + exposure enforcement"
)

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

check_forbidden_patterns() {
    local output="$1"
    local query_num="$2"
    local level="$3"
    local violations=()

    for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
        if echo "$output" | grep -qi "$pattern"; then
            violations+=("$pattern")
        fi
    done

    if [[ ${#violations[@]} -gt 0 ]]; then
        log_fail "Query $query_num at $level: Found forbidden patterns:"
        for v in "${violations[@]}"; do
            echo "         - '$v'"
        done
        return 1
    fi
    return 0
}

set_exposure_level() {
    local level="$1"
    if [[ ! -f "$CONFIG_FILE" ]]; then
        log_warn "Config file not found at $CONFIG_FILE, skipping level change"
        return 1
    fi

    # Update exposure level in config
    if grep -q "^exposure_level" "$CONFIG_FILE" 2>/dev/null; then
        sudo sed -i "s/^exposure_level.*/exposure_level = \"$level\"/" "$CONFIG_FILE"
    else
        echo "exposure_level = \"$level\"" | sudo tee -a "$CONFIG_FILE" > /dev/null
    fi
    log_info "Set exposure level to: $level"
}

run_query() {
    local query="$1"
    local timeout_sec="${2:-60}"

    timeout "$timeout_sec" "$ANNACTL" "$query" 2>&1 || true
}

run_test_at_level() {
    local level="$1"
    local level_dir="$RESULTS_DIR/$level"
    mkdir -p "$level_dir"

    echo ""
    echo "========================================"
    echo "Testing at exposure level: $level"
    echo "========================================"

    set_exposure_level "$level" || return 1

    local pass_count=0
    local fail_count=0

    for i in "${!QUERIES[@]}"; do
        local qnum=$((i + 1))
        local query="${QUERIES[$i]}"
        local purpose="${PURPOSES[$i]}"
        local outfile="$level_dir/query_${qnum}.txt"

        echo ""
        log_info "Query $qnum: $query"
        log_info "Purpose: $purpose"

        # Run query and capture output
        local output
        output=$(run_query "$query" 60)
        echo "$output" > "$outfile"

        # Check for forbidden patterns
        if check_forbidden_patterns "$output" "$qnum" "$level"; then
            log_pass "Query $qnum: No forbidden patterns"
            ((pass_count++))
        else
            ((fail_count++))
        fi

        # Show truncated output
        local lines
        lines=$(echo "$output" | wc -l)
        if [[ $lines -gt 10 ]]; then
            echo "$output" | head -5
            echo "    ... ($lines lines total, see $outfile)"
        else
            echo "$output"
        fi
    done

    echo ""
    echo "Level $level: $pass_count passed, $fail_count failed"
    return $fail_count
}

count_dialogue_lines() {
    local file="$1"
    # Count non-empty, non-answer lines (rough heuristic)
    grep -cE '^\[|^  \[|specialist|probe|investigating' "$file" 2>/dev/null || echo 0
}

compare_exposure_levels() {
    echo ""
    echo "========================================"
    echo "Exposure Level Comparison"
    echo "========================================"

    for i in "${!QUERIES[@]}"; do
        local qnum=$((i + 1))
        echo ""
        echo "Query $qnum: ${QUERIES[$i]}"

        for level in silent summary dialogue debug; do
            local file="$RESULTS_DIR/$level/query_${qnum}.txt"
            if [[ -f "$file" ]]; then
                local lines
                lines=$(wc -l < "$file")
                local dialogue
                dialogue=$(count_dialogue_lines "$file")
                echo "  $level: $lines lines ($dialogue dialogue)"
            fi
        done
    done
}

verify_exposure_invariants() {
    echo ""
    echo "========================================"
    echo "Exposure Invariant Verification"
    echo "========================================"

    local violations=0

    for i in "${!QUERIES[@]}"; do
        local qnum=$((i + 1))

        local silent_lines=0 summary_lines=0 dialogue_lines=0 debug_lines=0

        [[ -f "$RESULTS_DIR/silent/query_${qnum}.txt" ]] && \
            silent_lines=$(wc -l < "$RESULTS_DIR/silent/query_${qnum}.txt")
        [[ -f "$RESULTS_DIR/summary/query_${qnum}.txt" ]] && \
            summary_lines=$(wc -l < "$RESULTS_DIR/summary/query_${qnum}.txt")
        [[ -f "$RESULTS_DIR/dialogue/query_${qnum}.txt" ]] && \
            dialogue_lines=$(wc -l < "$RESULTS_DIR/dialogue/query_${qnum}.txt")
        [[ -f "$RESULTS_DIR/debug/query_${qnum}.txt" ]] && \
            debug_lines=$(wc -l < "$RESULTS_DIR/debug/query_${qnum}.txt")

        # Invariant: silent <= summary <= dialogue <= debug
        if [[ $silent_lines -gt $summary_lines ]] || \
           [[ $summary_lines -gt $dialogue_lines ]] || \
           [[ $dialogue_lines -gt $debug_lines ]]; then
            log_fail "Query $qnum: Exposure invariant violated"
            log_fail "  silent=$silent_lines, summary=$summary_lines, dialogue=$dialogue_lines, debug=$debug_lines"
            ((violations++))
        else
            log_pass "Query $qnum: Exposure ordering correct"
        fi
    done

    return $violations
}

main() {
    echo "Phase 14 Acceptance Test Suite"
    echo "=============================="
    echo ""

    # Check prerequisites
    if [[ ! -x "$ANNACTL" ]]; then
        log_fail "annactl not found at $ANNACTL"
        log_info "Build with: cargo build --release --workspace"
        exit 1
    fi

    # Check daemon
    if ! "$ANNACTL" status &>/dev/null; then
        log_warn "Anna daemon may not be running"
    fi

    # Create results directory
    rm -rf "$RESULTS_DIR"
    mkdir -p "$RESULTS_DIR"

    local total_failures=0

    if [[ "$LEVEL" == "all" ]]; then
        for level in silent summary dialogue debug; do
            run_test_at_level "$level" || ((total_failures+=$?))
        done

        compare_exposure_levels
        verify_exposure_invariants || ((total_failures+=$?))
    else
        run_test_at_level "$LEVEL" || ((total_failures+=$?))
    fi

    echo ""
    echo "========================================"
    echo "Final Results"
    echo "========================================"
    echo "Results saved to: $RESULTS_DIR"

    if [[ $total_failures -eq 0 ]]; then
        log_pass "All tests passed"
        exit 0
    else
        log_fail "$total_failures total failures"
        exit 1
    fi
}

main "$@"
