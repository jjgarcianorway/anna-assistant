#!/bin/bash
# Anna vs Claude Comparison Test v2
# Tests 100 NEW tricky questions with pattern matching improvements

QUESTIONS_FILE="${1:-tests/tricky_100_v2.txt}"
RESULTS_DIR="tests/results_v2_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

ANNACTL="./target/release/annactl"

# Check if annactl exists
if [[ ! -x "$ANNACTL" ]]; then
    echo "ERROR: annactl not found at $ANNACTL"
    exit 1
fi

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "========================================"
echo "  Anna Performance Test v2"
echo "  Testing pattern matching improvements"
echo "========================================"
echo ""

# Read questions (skip comments and empty lines)
mapfile -t QUESTIONS < <(grep -v '^#' "$QUESTIONS_FILE" | grep -v '^$')
TOTAL=${#QUESTIONS[@]}

echo "Total questions: $TOTAL"
echo "Results directory: $RESULTS_DIR"
echo ""

# Counters
SUCCESS=0
TIMEOUT=0
ERROR=0
PATTERN_MATCH=0
CLARIFICATION=0

# Track response times
declare -a RESPONSE_TIMES

# Test each question
for i in "${!QUESTIONS[@]}"; do
    Q="${QUESTIONS[$i]}"
    NUM=$((i + 1))

    printf "[%3d/%3d] %s\n" "$NUM" "$TOTAL" "${Q:0:60}..."

    # Time the response
    START=$(date +%s%N)

    # Run with 30s timeout
    RESPONSE=$( timeout 30 "$ANNACTL" "$Q" 2>&1 )
    EXIT_CODE=$?

    END=$(date +%s%N)
    ELAPSED_MS=$(( (END - START) / 1000000 ))
    RESPONSE_TIMES+=($ELAPSED_MS)

    # Save full response
    echo "QUESTION: $Q" > "$RESULTS_DIR/q${NUM}.txt"
    echo "TIME_MS: $ELAPSED_MS" >> "$RESULTS_DIR/q${NUM}.txt"
    echo "EXIT_CODE: $EXIT_CODE" >> "$RESULTS_DIR/q${NUM}.txt"
    echo "---" >> "$RESULTS_DIR/q${NUM}.txt"
    echo "$RESPONSE" >> "$RESULTS_DIR/q${NUM}.txt"

    # Analyze response
    if [[ $EXIT_CODE -eq 124 ]]; then
        printf "  ${RED}TIMEOUT${NC} (30s)\n"
        ((TIMEOUT++))
    elif [[ $EXIT_CODE -ne 0 ]]; then
        printf "  ${RED}ERROR${NC} (exit $EXIT_CODE)\n"
        ((ERROR++))
    else
        # Check for pattern match (instant response < 100ms usually means pattern)
        if [[ $ELAPSED_MS -lt 100 ]]; then
            printf "  ${GREEN}PATTERN${NC} (%dms)\n" "$ELAPSED_MS"
            ((PATTERN_MATCH++))
            ((SUCCESS++))
        # Check for clarification requests
        elif echo "$RESPONSE" | grep -qiE "(more specific|could you clarify|what exactly|please provide more|which.*specifically)"; then
            printf "  ${YELLOW}CLARIFY${NC} (%dms)\n" "$ELAPSED_MS"
            ((CLARIFICATION++))
            ((SUCCESS++))
        else
            printf "  ${GREEN}OK${NC} (%dms)\n" "$ELAPSED_MS"
            ((SUCCESS++))
        fi
    fi
done

echo ""
echo "========================================"
echo "  RESULTS SUMMARY"
echo "========================================"
echo ""
echo "Total questions: $TOTAL"
echo "Successful: $SUCCESS ($(( SUCCESS * 100 / TOTAL ))%)"
echo "  - Pattern matched: $PATTERN_MATCH"
echo "  - Asked for clarification: $CLARIFICATION"
echo "  - Direct answers: $((SUCCESS - PATTERN_MATCH - CLARIFICATION))"
echo "Timeouts: $TIMEOUT"
echo "Errors: $ERROR"
echo ""

# Calculate timing stats
if [[ ${#RESPONSE_TIMES[@]} -gt 0 ]]; then
    # Sort for median
    IFS=$'\n' SORTED=($(sort -n <<<"${RESPONSE_TIMES[*]}"))
    unset IFS

    MID=$(( ${#SORTED[@]} / 2 ))
    MEDIAN=${SORTED[$MID]}

    SUM=0
    for t in "${RESPONSE_TIMES[@]}"; do
        SUM=$((SUM + t))
    done
    AVG=$((SUM / ${#RESPONSE_TIMES[@]}))

    MIN=${SORTED[0]}
    MAX=${SORTED[-1]}

    echo "Response times:"
    echo "  Minimum: ${MIN}ms"
    echo "  Maximum: ${MAX}ms"
    echo "  Median:  ${MEDIAN}ms"
    echo "  Average: ${AVG}ms"
fi

echo ""
echo "Results saved to: $RESULTS_DIR"

# Save summary
cat > "$RESULTS_DIR/SUMMARY.txt" << EOF
Anna Performance Test v2 - $(date)
=====================================

Questions: $TOTAL
Successful: $SUCCESS ($(( SUCCESS * 100 / TOTAL ))%)
  - Pattern matched: $PATTERN_MATCH ($(( PATTERN_MATCH * 100 / TOTAL ))%)
  - Clarification: $CLARIFICATION ($(( CLARIFICATION * 100 / TOTAL ))%)
  - Direct answers: $((SUCCESS - PATTERN_MATCH - CLARIFICATION))
Timeouts: $TIMEOUT
Errors: $ERROR

Response times:
  Min: ${MIN:-N/A}ms
  Max: ${MAX:-N/A}ms
  Median: ${MEDIAN:-N/A}ms
  Average: ${AVG:-N/A}ms
EOF

echo "Summary saved to: $RESULTS_DIR/SUMMARY.txt"
