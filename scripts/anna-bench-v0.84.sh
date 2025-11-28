#!/usr/bin/env bash
# Anna Benchmark Script v0.84.0
# Runs a fixed set of questions and captures structured metrics

set -euo pipefail

# Configuration
ANNACTL="${ANNACTL:-annactl}"
LOG_DIR="${ANNA_BENCH_LOG_DIR:-/var/log/anna/bench_v0.84}"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
OUTPUT_FILE="${LOG_DIR}/benchmark-${TIMESTAMP}.json"
VERBOSE="${ANNA_BENCH_VERBOSE:-0}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Ensure log directory exists
mkdir -p "$LOG_DIR" 2>/dev/null || {
    echo -e "${YELLOW}Warning: Cannot create $LOG_DIR, using /tmp${NC}"
    LOG_DIR="/tmp/anna-bench"
    mkdir -p "$LOG_DIR"
    OUTPUT_FILE="${LOG_DIR}/benchmark-${TIMESTAMP}.json"
}

# Test questions with expected answer patterns
# Format: "TEST_ID|QUESTION|EXPECTED_PATTERN|CATEGORY"
QUESTIONS=(
    # Hardware basics
    "HW-01|how many cores has my computer?|core|hardware"
    "HW-02|what CPU model do I have?|AMD\|Intel\|CPU|hardware"
    "HW-03|how much RAM is installed?|GB\|MB\|memory|hardware"
    "HW-04|how much free RAM do I have?|GB\|MB\|free\|available|hardware"
    "HW-05|how much disk space is free on root?|GB\|TB\|free\|available|hardware"

    # Self health
    "HEALTH-01|diagnose your own health|daemon\|model\|status\|health|self_health"

    # Unsupported domains (should clearly refuse)
    "UNS-01|is my wifi stable?|probe\|cannot\|unable\|no way|unsupported"
    "UNS-02|is Steam installed?|probe\|cannot\|unable\|package|unsupported"

    # Learning loop - repeat questions
    "LEARN-01a|how many CPU threads do I have?|thread\|core|learning"
    "LEARN-01b|how many CPU threads do I have?|thread\|core|learning"
    "LEARN-01c|how many CPU threads do I have?|thread\|core|learning"
)

# Results array
declare -a RESULTS=()

# Function to run a single benchmark
run_benchmark() {
    local test_id="$1"
    local question="$2"
    local expected_pattern="$3"
    local category="$4"

    echo -e "${BLUE}[$test_id]${NC} $question"

    local start_ts=$(date +%s%3N)
    local start_iso=$(date -Iseconds)

    # Run annactl with benchmark mode and QA mode for structured output
    local output
    local exit_code=0
    output=$(ANNA_BENCH_MODE=1 ANNA_QA_MODE=1 timeout 120 "$ANNACTL" "$question" 2>&1) || exit_code=$?

    local end_ts=$(date +%s%3N)
    local end_iso=$(date -Iseconds)
    local duration_ms=$((end_ts - start_ts))

    # Check if answer matches expected pattern
    local success="false"
    if echo "$output" | grep -qiE "$expected_pattern"; then
        success="true"
        echo -e "  ${GREEN}PASS${NC} (${duration_ms}ms)"
    else
        echo -e "  ${RED}FAIL${NC} (${duration_ms}ms) - pattern not found"
    fi

    # Extract confidence from QA output if available
    local confidence="0.0"
    if echo "$output" | grep -q '"score_overall"'; then
        confidence=$(echo "$output" | grep -o '"score_overall":[0-9.]*' | cut -d: -f2 || echo "0.0")
    fi

    # Extract junior/senior timing if available
    local junior_ms="0"
    local senior_ms="0"
    if echo "$output" | grep -q '"junior_ms"'; then
        junior_ms=$(echo "$output" | grep -o '"junior_ms":[0-9]*' | cut -d: -f2 || echo "0")
    fi
    if echo "$output" | grep -q '"senior_ms"'; then
        senior_ms=$(echo "$output" | grep -o '"senior_ms":[0-9]*' | cut -d: -f2 || echo "0")
    fi

    # Extract probes used
    local probes_used="[]"
    if echo "$output" | grep -q '"probes_used"'; then
        probes_used=$(echo "$output" | grep -o '"probes_used":\[[^]]*\]' | cut -d: -f2- || echo "[]")
    fi

    # Build JSON result
    local result=$(cat <<EOF
{
    "test_id": "$test_id",
    "category": "$category",
    "question": "$question",
    "timestamp_start": "$start_iso",
    "timestamp_end": "$end_iso",
    "duration_ms": $duration_ms,
    "junior_ms": $junior_ms,
    "senior_ms": $senior_ms,
    "confidence": $confidence,
    "success": $success,
    "exit_code": $exit_code,
    "probes_used": $probes_used
}
EOF
)

    RESULTS+=("$result")

    # Verbose output
    if [[ "$VERBOSE" == "1" ]]; then
        echo -e "  ${YELLOW}Output:${NC}"
        echo "$output" | head -20 | sed 's/^/    /'
    fi

    # Brief pause between questions
    sleep 1
}

# Main execution
echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}  Anna Benchmark Suite v0.84.0${NC}"
echo -e "${GREEN}======================================${NC}"
echo ""
echo -e "Timestamp: ${TIMESTAMP}"
echo -e "Output: ${OUTPUT_FILE}"
echo -e "Questions: ${#QUESTIONS[@]}"
echo ""

# Check annactl exists
if ! command -v "$ANNACTL" &> /dev/null; then
    echo -e "${RED}Error: annactl not found at: $ANNACTL${NC}"
    echo "Set ANNACTL env var to the correct path"
    exit 1
fi

# Check annad is running
if ! pgrep -f "annad" > /dev/null; then
    echo -e "${YELLOW}Warning: annad does not appear to be running${NC}"
    echo "Start with: systemctl start annad"
fi

echo -e "${BLUE}Starting benchmark run...${NC}"
echo ""

# Run all benchmarks
for entry in "${QUESTIONS[@]}"; do
    IFS='|' read -r test_id question expected category <<< "$entry"
    run_benchmark "$test_id" "$question" "$expected" "$category"
done

echo ""
echo -e "${BLUE}Generating report...${NC}"

# Calculate summary statistics
total_tests=${#RESULTS[@]}
passed=0
total_duration=0
total_confidence=0

for result in "${RESULTS[@]}"; do
    if echo "$result" | grep -q '"success": true'; then
        ((passed++))
    fi
    duration=$(echo "$result" | grep -o '"duration_ms": [0-9]*' | cut -d: -f2 | tr -d ' ')
    confidence=$(echo "$result" | grep -o '"confidence": [0-9.]*' | cut -d: -f2 | tr -d ' ')
    total_duration=$((total_duration + duration))
    # Handle floating point for confidence
    total_confidence=$(echo "$total_confidence + $confidence" | bc -l 2>/dev/null || echo "$total_confidence")
done

avg_duration=$((total_duration / total_tests))
pass_rate=$((passed * 100 / total_tests))

# Build final JSON output
results_json=$(printf '%s\n' "${RESULTS[@]}" | paste -sd, -)

cat > "$OUTPUT_FILE" <<EOF
{
    "benchmark_version": "0.84.0",
    "timestamp": "$TIMESTAMP",
    "summary": {
        "total_tests": $total_tests,
        "passed": $passed,
        "failed": $((total_tests - passed)),
        "pass_rate_percent": $pass_rate,
        "total_duration_ms": $total_duration,
        "avg_duration_ms": $avg_duration
    },
    "results": [
        $results_json
    ]
}
EOF

echo ""
echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}  Benchmark Complete${NC}"
echo -e "${GREEN}======================================${NC}"
echo ""
echo -e "Total tests:     $total_tests"
echo -e "Passed:          ${GREEN}$passed${NC}"
echo -e "Failed:          ${RED}$((total_tests - passed))${NC}"
echo -e "Pass rate:       ${pass_rate}%"
echo -e "Total duration:  ${total_duration}ms"
echo -e "Avg duration:    ${avg_duration}ms"
echo ""
echo -e "Results saved to: ${OUTPUT_FILE}"

# Exit with failure if pass rate < 50%
if [[ $pass_rate -lt 50 ]]; then
    echo -e "${RED}BENCHMARK FAILED: Pass rate below 50%${NC}"
    exit 1
fi

echo -e "${GREEN}BENCHMARK PASSED${NC}"
exit 0
