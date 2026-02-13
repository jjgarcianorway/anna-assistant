#!/bin/bash
# Test Anna against ChatGPT's 100 sysadmin questions
# Comprehensive evaluation of Anna's capabilities

set -euo pipefail

QUESTIONS_FILE="tests/chatgpt_100_questions.txt"
RESULTS_DIR="/tmp/anna_chatgpt100_$(date +%Y%m%d_%H%M%S)"
RESULTS_FILE="$RESULTS_DIR/results.json"
TIMEOUT=60

mkdir -p "$RESULTS_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=== Anna vs ChatGPT 100 Questions Test ==="
echo "Test Directory: $RESULTS_DIR"
echo ""

# Initialize results
cat > "$RESULTS_FILE" <<EOF
{
  "test_date": "$(date -Iseconds)",
  "anna_version": "$(annactl --version | head -1)",
  "total_questions": 0,
  "completed": 0,
  "failed": 0,
  "timed_out": 0,
  "instant_answers": 0,
  "avg_response_time_ms": 0,
  "by_level": {},
  "questions": []
}
EOF

TOTAL=0
COMPLETED=0
FAILED=0
TIMED_OUT=0
INSTANT=0
TOTAL_TIME=0

CURRENT_LEVEL=""

# Read questions
while IFS= read -r line; do
    # Skip empty lines
    [[ -z "$line" ]] && continue

    # Track level headers
    if [[ "$line" =~ ^#\ Level ]]; then
        CURRENT_LEVEL=$(echo "$line" | sed 's/^# //' | sed 's/[–-].*//' | xargs)
        echo ""
        echo "=== $CURRENT_LEVEL ==="
        continue
    fi

    # Skip other comments
    [[ "$line" =~ ^# ]] && continue

    TOTAL=$((TOTAL + 1))
    QUESTION="$line"

    echo -n "Q$TOTAL: ${QUESTION:0:60}... "

    # Run question with timeout
    start=$(date +%s%N)
    if output=$(timeout $TIMEOUT annactl "$QUESTION" 2>&1); then
        end=$(date +%s%N)
        duration=$(( (end - start) / 1000000 ))
        TOTAL_TIME=$((TOTAL_TIME + duration))

        # Check if instant answer (0 iterations)
        if echo "$output" | grep -q "(0 iterations)"; then
            INSTANT=$((INSTANT + 1))
            echo -e "${GREEN}✓ INSTANT${NC} (${duration}ms)"
        elif echo "$output" | grep -qE "CRITICAL|ERROR|failed"; then
            # Contains error but didn't fail
            COMPLETED=$((COMPLETED + 1))
            iterations=$(echo "$output" | grep -oP '\(\K[0-9]+(?= iterations\))' | tail -1 || echo "?")
            echo -e "${YELLOW}⚠ ANSWER${NC} (${duration}ms, $iterations iter)"
        else
            COMPLETED=$((COMPLETED + 1))
            iterations=$(echo "$output" | grep -oP '\(\K[0-9]+(?= iterations\))' | tail -1 || echo "?")
            echo -e "${GREEN}✓ PASS${NC} (${duration}ms, $iterations iter)"
        fi

        # Save answer
        echo "$output" > "$RESULTS_DIR/q${TOTAL}_answer.txt"
    else
        exit_code=$?
        if [ $exit_code -eq 124 ]; then
            TIMED_OUT=$((TIMED_OUT + 1))
            echo -e "${RED}✗ TIMEOUT${NC} (>$TIMEOUT"s")"
        else
            FAILED=$((FAILED + 1))
            echo -e "${RED}✗ FAIL${NC} (exit $exit_code)"
        fi
    fi

    # Brief pause to avoid overwhelming the daemon
    sleep 0.5

done < "$QUESTIONS_FILE"

# Calculate stats
if [ $COMPLETED -gt 0 ]; then
    AVG_TIME=$((TOTAL_TIME / COMPLETED))
else
    AVG_TIME=0
fi

INSTANT_PCT=$(( (INSTANT * 100) / TOTAL ))
SUCCESS_RATE=$(( ((COMPLETED + INSTANT) * 100) / TOTAL ))

# Print summary
echo ""
echo "========================================="
echo "TEST SUMMARY"
echo "========================================="
echo ""
echo "Total Questions:    $TOTAL"
echo "Completed:          $((COMPLETED + INSTANT))"
echo "  - Instant:        $INSTANT ($INSTANT_PCT%)"
echo "  - With LLM:       $COMPLETED"
echo "Failed:             $FAILED"
echo "Timed Out:          $TIMED_OUT"
echo ""
echo "Success Rate:       $SUCCESS_RATE%"
echo "Avg Response Time:  ${AVG_TIME}ms"
echo ""
echo "========================================="
echo "Results saved to: $RESULTS_DIR"
echo "========================================="

# Update JSON results
jq --arg total "$TOTAL" \
   --arg completed "$((COMPLETED + INSTANT))" \
   --arg failed "$FAILED" \
   --arg timed_out "$TIMED_OUT" \
   --arg instant "$INSTANT" \
   --arg avg_time "$AVG_TIME" \
   --arg success "$SUCCESS_RATE" \
   '. + {
     total_questions: ($total | tonumber),
     completed: ($completed | tonumber),
     failed: ($failed | tonumber),
     timed_out: ($timed_out | tonumber),
     instant_answers: ($instant | tonumber),
     avg_response_time_ms: ($avg_time | tonumber),
     success_rate_pct: ($success | tonumber)
   }' "$RESULTS_FILE" > "$RESULTS_FILE.tmp" && mv "$RESULTS_FILE.tmp" "$RESULTS_FILE"

echo ""
echo "Detailed results: cat $RESULTS_FILE"
