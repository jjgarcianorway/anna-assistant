#!/bin/bash
# Anna vs Claude Comparison Test
# Tests 100 tricky real-world questions for reliability, response time, and accuracy

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANNACTL="/home/lhoqvso/anna-assistant/target/release/annactl"
QUESTIONS_FILE="$SCRIPT_DIR/tricky_100_questions.txt"
RESULTS_DIR="$SCRIPT_DIR/comparison_results_$(date +%Y%m%d_%H%M%S)"
SUMMARY_FILE="$RESULTS_DIR/summary.json"

mkdir -p "$RESULTS_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  Anna Performance Test${NC}"
echo -e "${BLUE}  100 Tricky Real-World Questions${NC}"
echo -e "${BLUE}================================${NC}"
echo ""
echo "Results directory: $RESULTS_DIR"
echo "Started at: $(date)"
echo ""

# Check if annactl exists
if [ ! -f "$ANNACTL" ]; then
    echo -e "${RED}Error: annactl not found at $ANNACTL${NC}"
    echo "Run: cargo build --release --workspace"
    exit 1
fi

# Check if questions file exists
if [ ! -f "$QUESTIONS_FILE" ]; then
    echo -e "${RED}Error: Questions file not found at $QUESTIONS_FILE${NC}"
    exit 1
fi

# Initialize counters
total=0
success=0
timeout_count=0
error_count=0
total_time=0

# Read questions and test
while IFS='|' read -r num category question || [ -n "$num" ]; do
    # Skip comments and empty lines
    [[ "$num" =~ ^#.*$ ]] && continue
    [[ -z "$num" ]] && continue

    total=$((total + 1))

    echo -e "${YELLOW}[$num/100]${NC} ${BLUE}[$category]${NC} $question"

    # Prepare output file
    result_file="$RESULTS_DIR/q${num}_anna.json"

    # Record start time (nanoseconds)
    start_time=$(date +%s%N)

    # Run anna with timeout (60 seconds max)
    if timeout 60 "$ANNACTL" "$question" > "$RESULTS_DIR/q${num}_anna.txt" 2>&1; then
        end_time=$(date +%s%N)
        elapsed_ms=$(( (end_time - start_time) / 1000000 ))
        total_time=$((total_time + elapsed_ms))
        success=$((success + 1))

        # Get response length
        response_len=$(wc -c < "$RESULTS_DIR/q${num}_anna.txt")

        # Save metadata
        cat > "$result_file" << EOF
{
  "question_num": $num,
  "category": "$category",
  "question": "$question",
  "status": "success",
  "response_time_ms": $elapsed_ms,
  "response_length": $response_len
}
EOF

        if [ $elapsed_ms -lt 3000 ]; then
            echo -e "  ${GREEN}OK${NC} (${elapsed_ms}ms, ${response_len} bytes)"
        elif [ $elapsed_ms -lt 10000 ]; then
            echo -e "  ${YELLOW}OK${NC} (${elapsed_ms}ms, ${response_len} bytes)"
        else
            echo -e "  ${YELLOW}SLOW${NC} (${elapsed_ms}ms, ${response_len} bytes)"
        fi
    else
        exit_code=$?
        end_time=$(date +%s%N)
        elapsed_ms=$(( (end_time - start_time) / 1000000 ))

        if [ $exit_code -eq 124 ]; then
            timeout_count=$((timeout_count + 1))
            status="timeout"
            echo -e "  ${RED}TIMEOUT${NC} (60s limit)"
        else
            error_count=$((error_count + 1))
            status="error"
            echo -e "  ${RED}ERROR${NC} (exit code: $exit_code)"
        fi

        cat > "$result_file" << EOF
{
  "question_num": $num,
  "category": "$category",
  "question": "$question",
  "status": "$status",
  "response_time_ms": $elapsed_ms,
  "exit_code": $exit_code
}
EOF
    fi

    # Brief pause between questions
    sleep 0.5

done < "$QUESTIONS_FILE"

# Calculate statistics
avg_time=0
if [ $success -gt 0 ]; then
    avg_time=$((total_time / success))
fi

success_rate=$(echo "scale=1; $success * 100 / $total" | bc)

echo ""
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  Test Results Summary${NC}"
echo -e "${BLUE}================================${NC}"
echo ""
echo "Total questions:  $total"
echo -e "Successful:       ${GREEN}$success${NC}"
echo -e "Timeouts:         ${RED}$timeout_count${NC}"
echo -e "Errors:           ${RED}$error_count${NC}"
echo ""
echo "Success rate:     ${success_rate}%"
echo "Avg response:     ${avg_time}ms"
echo "Total test time:  $((total_time / 1000))s"
echo ""
echo "Completed at: $(date)"

# Save summary
cat > "$SUMMARY_FILE" << EOF
{
  "test_date": "$(date -Iseconds)",
  "total_questions": $total,
  "successful": $success,
  "timeouts": $timeout_count,
  "errors": $error_count,
  "success_rate_percent": $success_rate,
  "avg_response_time_ms": $avg_time,
  "total_test_time_ms": $total_time,
  "results_directory": "$RESULTS_DIR"
}
EOF

echo "Summary saved to: $SUMMARY_FILE"

# Generate category breakdown
echo ""
echo -e "${BLUE}Category Breakdown:${NC}"
for cat in Ambiguous EdgeCase MultiStep Obscure Security Performance Recovery Context; do
    cat_total=$(grep -c "|$cat|" "$QUESTIONS_FILE" 2>/dev/null || echo 0)
    cat_success=$(ls "$RESULTS_DIR"/q*_anna.json 2>/dev/null | xargs grep -l "\"status\": \"success\"" 2>/dev/null | xargs grep -l "\"category\": \"$cat\"" 2>/dev/null | wc -l || echo 0)
    if [ "$cat_total" -gt 0 ]; then
        cat_rate=$(echo "scale=0; $cat_success * 100 / $cat_total" | bc)
        echo "  $cat: $cat_success/$cat_total (${cat_rate}%)"
    fi
done
