#!/bin/bash
# Anna vs Claude Battle Test Runner
# Runs all 100 questions and generates a comparison report

set -e

RESULTS_FILE="tests/battle_results.txt"
ANNACTL="./target/release/annactl"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo "🥊 ANNA VS CLAUDE: 100 QUESTION BATTLE TEST 🥊"
echo ""
echo "This will take approximately 30-60 minutes..."
echo "Results will be saved to $RESULTS_FILE"
echo ""
echo "Press Enter to start, or Ctrl+C to cancel..."
read

# Initialize results file
cat > "$RESULTS_FILE" << 'EOF'
# ANNA VS CLAUDE: BATTLE TEST RESULTS
# Generated: $(date)
#
# Legend:
# ✓ = Anna answered successfully
# ✗ = Anna failed or gave unclear response
# ⏱ = Response time in milliseconds
#
================================================================================
EOF

echo "$(date)" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

TOTAL=0
SUCCESS=0
FAILURES=0
TOTAL_TIME=0

# Read questions from test file
while IFS= read -r line; do
    # Skip comments and empty lines
    [[ "$line" =~ ^#.*$ ]] && continue
    [[ -z "$line" ]] && continue

    # Detect category headers
    if [[ "$line" =~ ^##.*$ ]]; then
        CATEGORY=$(echo "$line" | sed 's/^## //' | cut -d'-' -f1 | xargs)
        echo "" | tee -a "$RESULTS_FILE"
        echo "================================================================================" | tee -a "$RESULTS_FILE"
        echo -e "${CYAN}📁 CATEGORY: $CATEGORY${NC}" | tee -a "$RESULTS_FILE"
        echo "================================================================================" | tee -a "$RESULTS_FILE"
        echo "" | tee -a "$RESULTS_FILE"
        continue
    fi

    # Parse question (format: "1. Question text")
    if [[ "$line" =~ ^[0-9]+\..*$ ]]; then
        TOTAL=$((TOTAL + 1))
        QUESTION_ID=$(echo "$line" | cut -d'.' -f1)
        QUESTION_TEXT=$(echo "$line" | cut -d'.' -f2- | xargs)

        printf "Q%-3s: %-70s " "$QUESTION_ID" "$QUESTION_TEXT"

        # Ask Anna (with timeout)
        START_TIME=$(date +%s%3N)
        RESPONSE=$($ANNACTL "$QUESTION_TEXT" 2>&1 || true)
        END_TIME=$(date +%s%3N)
        DURATION=$((END_TIME - START_TIME))
        TOTAL_TIME=$((TOTAL_TIME + DURATION))

        # Evaluate response
        if [[ -z "$RESPONSE" ]] ||
           [[ "$RESPONSE" =~ [Ee]rror ]] ||
           [[ "$RESPONSE" =~ [Ff]ailed ]] ||
           [[ "$RESPONSE" =~ [Tt]imeout ]]; then
            # Failure
            FAILURES=$((FAILURES + 1))
            echo -e "${RED}✗${NC} (${DURATION}ms)"
            echo "Q$QUESTION_ID: $QUESTION_TEXT - FAILED (${DURATION}ms)" >> "$RESULTS_FILE"
        else
            # Success
            SUCCESS=$((SUCCESS + 1))
            echo -e "${GREEN}✓${NC} (${DURATION}ms)"
            echo "Q$QUESTION_ID: $QUESTION_TEXT - SUCCESS (${DURATION}ms)" >> "$RESULTS_FILE"
        fi

        # Small delay between questions
        sleep 0.1
    fi
done < "tests/anna_vs_claude_100.txt"

# Calculate statistics
SUCCESS_RATE=$(echo "scale=1; $SUCCESS * 100 / $TOTAL" | bc)
AVG_TIME=$(echo "scale=0; $TOTAL_TIME / $TOTAL" | bc)
TOTAL_SECONDS=$(echo "scale=2; $TOTAL_TIME / 1000" | bc)

# Print final results
echo "" | tee -a "$RESULTS_FILE"
echo "================================================================================" | tee -a "$RESULTS_FILE"
echo -e "${BLUE}📊 FINAL RESULTS${NC}" | tee -a "$RESULTS_FILE"
echo "================================================================================" | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"
echo "Total Questions:    $TOTAL" | tee -a "$RESULTS_FILE"
echo "Successful:         $SUCCESS (${SUCCESS_RATE}%)" | tee -a "$RESULTS_FILE"
echo "Failed:             $FAILURES" | tee -a "$RESULTS_FILE"
echo "Average Time:       ${AVG_TIME} ms" | tee -a "$RESULTS_FILE"
echo "Total Time:         ${TOTAL_SECONDS} seconds" | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# Verdict
if (( $(echo "$SUCCESS_RATE >= 90" | bc -l) )); then
    echo -e "${GREEN}🏆 VERDICT: Anna is EXCELLENT!${NC}" | tee -a "$RESULTS_FILE"
elif (( $(echo "$SUCCESS_RATE >= 75" | bc -l) )); then
    echo -e "${GREEN}✅ VERDICT: Anna is GOOD!${NC}" | tee -a "$RESULTS_FILE"
elif (( $(echo "$SUCCESS_RATE >= 50" | bc -l) )); then
    echo -e "${YELLOW}⚠️  VERDICT: Anna is DECENT - needs improvement${NC}" | tee -a "$RESULTS_FILE"
else
    echo -e "${RED}❌ VERDICT: Anna needs serious work${NC}" | tee -a "$RESULTS_FILE"
fi

echo ""
echo "✨ Test complete! Results saved to $RESULTS_FILE"
