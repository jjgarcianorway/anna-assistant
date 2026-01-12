#!/bin/bash
# Run 100 questions through Anna and record results for comparison
# v0.0.999

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QUESTIONS_FILE="$SCRIPT_DIR/tricky_100_questions.txt"
OUTPUT_DIR="$SCRIPT_DIR/comparison_results_$(date +%Y%m%d_%H%M%S)"
ANNACTL="${ANNACTL:-./target/release/annactl}"
TIMEOUT_SEC=45

mkdir -p "$OUTPUT_DIR"

echo "========================================"
echo "Anna vs Claude Comparison Test"
echo "========================================"
echo "Questions: $QUESTIONS_FILE"
echo "Output: $OUTPUT_DIR"
echo "Timeout: ${TIMEOUT_SEC}s per question"
echo ""

# Stats tracking
total=0
answered=0
clarification=0
timeout=0
error=0

# Parse questions (format: number|category|question)
while IFS='|' read -r num category question || [[ -n "$question" ]]; do
    # Skip comments and empty lines
    [[ "$num" =~ ^# ]] && continue
    [[ -z "$question" ]] && continue

    total=$((total + 1))

    echo -n "[$num] $category: "

    # Run through Anna with timeout
    start_time=$(date +%s.%N)
    result=$(timeout "${TIMEOUT_SEC}s" $ANNACTL "$question" 2>&1)
    exit_code=$?
    end_time=$(date +%s.%N)
    elapsed=$(echo "$end_time - $start_time" | bc)

    # Save full output
    {
        echo "Question: $question"
        echo "Category: $category"
        echo "Exit Code: $exit_code"
        echo "Time: ${elapsed}s"
        echo "---"
        echo "$result"
    } > "$OUTPUT_DIR/q${num}_${category}.txt"

    # Analyze result
    if [[ $exit_code -eq 124 ]]; then
        echo "TIMEOUT (${elapsed}s)"
        timeout=$((timeout + 1))
    elif [[ $exit_code -ne 0 ]]; then
        echo "ERROR ($exit_code)"
        error=$((error + 1))
    elif echo "$result" | grep -qi "could you\|clarif\|more specific\|what.*mean"; then
        echo "CLARIFICATION (${elapsed}s)"
        clarification=$((clarification + 1))
    else
        echo "ANSWERED (${elapsed}s)"
        answered=$((answered + 1))
    fi

done < "$QUESTIONS_FILE"

# Summary
echo ""
echo "========================================"
echo "RESULTS SUMMARY"
echo "========================================"
echo "Total questions: $total"
echo "Answered directly: $answered ($(echo "scale=1; $answered * 100 / $total" | bc)%)"
echo "Asked clarification: $clarification ($(echo "scale=1; $clarification * 100 / $total" | bc)%)"
echo "Timed out: $timeout ($(echo "scale=1; $timeout * 100 / $total" | bc)%)"
echo "Errors: $error ($(echo "scale=1; $error * 100 / $total" | bc)%)"
echo ""
echo "Detailed results in: $OUTPUT_DIR"

# Save summary
{
    echo "Anna Comparison Test Results"
    echo "Date: $(date)"
    echo "Version: $(./target/release/annactl --version 2>/dev/null || echo 'unknown')"
    echo ""
    echo "Total: $total"
    echo "Answered: $answered"
    echo "Clarification: $clarification"
    echo "Timeout: $timeout"
    echo "Error: $error"
} > "$OUTPUT_DIR/SUMMARY.txt"
