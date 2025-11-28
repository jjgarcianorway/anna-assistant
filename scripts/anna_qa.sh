#!/bin/bash
# Anna QA Test Harness v0.81.0
#
# Runs QA scenarios against Anna and outputs structured JSON results.
# Usage: ANNA_QA_MODE=1 ./scripts/anna_qa.sh [scenario]
#
# Scenarios:
#   cpu     - Test CPU-related questions
#   mem     - Test memory-related questions
#   all     - Run all scenarios (default)
#   single  - Run single question from $ANNA_QA_QUESTION
#
# Environment:
#   ANNA_QA_MODE=1      - Required. Enables JSON output from annactl
#   ANNA_QA_QUESTION    - Custom question for 'single' scenario
#   ANNA_QA_OUTPUT_DIR  - Directory for output files (default: /tmp/anna_qa)
#   ANNA_QA_VERBOSE     - Set to 1 for verbose output

set -e

# Configuration
OUTPUT_DIR="${ANNA_QA_OUTPUT_DIR:-/tmp/anna_qa}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="${OUTPUT_DIR}/qa_results_${TIMESTAMP}.jsonl"

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Ensure QA mode is enabled
if [[ -z "$ANNA_QA_MODE" ]]; then
    echo -e "${RED}[ERROR]${NC} ANNA_QA_MODE must be set to 1"
    echo "Usage: ANNA_QA_MODE=1 ./scripts/anna_qa.sh [scenario]"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Test questions by scenario
CPU_QUESTIONS=(
    "How many CPU threads do I have?"
    "What CPU model do I have?"
    "How many physical cores?"
)

MEM_QUESTIONS=(
    "How much RAM do I have?"
    "How much memory is available?"
)

ALL_QUESTIONS=("${CPU_QUESTIONS[@]}" "${MEM_QUESTIONS[@]}")

# Function to run a single QA test
run_qa_test() {
    local question="$1"
    local scenario="$2"

    echo -e "${CYAN}[TEST]${NC} $question"

    # Run annactl with QA mode and capture output
    local start_time=$(date +%s%N)
    local result
    result=$(annactl "$question" 2>/dev/null) || true
    local end_time=$(date +%s%N)
    local wall_ms=$(( (end_time - start_time) / 1000000 ))

    # Parse JSON output
    if echo "$result" | jq -e . >/dev/null 2>&1; then
        # Valid JSON - extract fields
        local reliability=$(echo "$result" | jq -r '.reliability_label // "unknown"')
        local headline=$(echo "$result" | jq -r '.headline // ""')
        local junior_ms=$(echo "$result" | jq -r '.junior_ms // 0')
        local senior_ms=$(echo "$result" | jq -r '.senior_ms // 0')

        # Color code by reliability
        case "$reliability" in
            Green)  echo -e "  ${GREEN}[OK]${NC} $reliability - ${headline:0:60}..." ;;
            Yellow) echo -e "  ${YELLOW}[PARTIAL]${NC} $reliability - ${headline:0:60}..." ;;
            Red)    echo -e "  ${RED}[FAIL]${NC} $reliability - ${headline:0:60}..." ;;
            *)      echo -e "  ${RED}[ERROR]${NC} Unknown reliability: $reliability" ;;
        esac

        # Append to JSONL results with metadata
        echo "$result" | jq -c ". + {
            \"_qa_metadata\": {
                \"scenario\": \"$scenario\",
                \"question\": \"$question\",
                \"wall_ms\": $wall_ms,
                \"timestamp\": \"$(date -Iseconds)\"
            }
        }" >> "$RESULTS_FILE"

        if [[ -n "$ANNA_QA_VERBOSE" ]]; then
            echo -e "  Timing: wall=${wall_ms}ms junior=${junior_ms}ms senior=${senior_ms}ms"
        fi
    else
        # Invalid JSON - log error
        echo -e "  ${RED}[ERROR]${NC} Invalid JSON output"
        echo "{\"_qa_error\": \"invalid_json\", \"_qa_metadata\": {\"scenario\": \"$scenario\", \"question\": \"$question\", \"timestamp\": \"$(date -Iseconds)\"}}" >> "$RESULTS_FILE"
    fi

    # Small delay between tests to avoid overloading LLM
    sleep 0.5
}

# Function to run a scenario
run_scenario() {
    local scenario="$1"
    shift
    local questions=("$@")

    echo -e "\n${CYAN}===== Scenario: $scenario =====${NC}\n"

    for q in "${questions[@]}"; do
        run_qa_test "$q" "$scenario"
    done
}

# Function to summarize results
summarize_results() {
    echo -e "\n${CYAN}===== QA Summary =====${NC}\n"

    if [[ -f "$RESULTS_FILE" ]]; then
        local total=$(wc -l < "$RESULTS_FILE")
        local green=$(grep -c '"reliability_label":"Green"' "$RESULTS_FILE" || echo 0)
        local yellow=$(grep -c '"reliability_label":"Yellow"' "$RESULTS_FILE" || echo 0)
        local red=$(grep -c '"reliability_label":"Red"' "$RESULTS_FILE" || echo 0)
        local errors=$(grep -c '"_qa_error"' "$RESULTS_FILE" || echo 0)

        echo -e "Total tests: $total"
        echo -e "${GREEN}Green (pass):${NC} $green"
        echo -e "${YELLOW}Yellow (partial):${NC} $yellow"
        echo -e "${RED}Red (fail):${NC} $red"
        echo -e "${RED}Errors:${NC} $errors"
        echo -e "\nResults saved to: $RESULTS_FILE"
    else
        echo -e "${RED}No results file found${NC}"
    fi
}

# Main
SCENARIO="${1:-all}"

echo -e "${CYAN}Anna QA Harness v0.81.0${NC}"
echo -e "Output: $RESULTS_FILE\n"

case "$SCENARIO" in
    cpu)
        run_scenario "cpu" "${CPU_QUESTIONS[@]}"
        ;;
    mem)
        run_scenario "mem" "${MEM_QUESTIONS[@]}"
        ;;
    all)
        run_scenario "cpu" "${CPU_QUESTIONS[@]}"
        run_scenario "mem" "${MEM_QUESTIONS[@]}"
        ;;
    single)
        if [[ -z "$ANNA_QA_QUESTION" ]]; then
            echo -e "${RED}[ERROR]${NC} ANNA_QA_QUESTION must be set for single scenario"
            exit 1
        fi
        run_qa_test "$ANNA_QA_QUESTION" "single"
        ;;
    *)
        echo -e "${RED}[ERROR]${NC} Unknown scenario: $SCENARIO"
        echo "Available: cpu, mem, all, single"
        exit 1
        ;;
esac

summarize_results
