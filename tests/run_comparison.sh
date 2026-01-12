#!/bin/bash
# Anna v0.0.999 - 100 Question Comparison Test

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QUESTIONS_FILE="$SCRIPT_DIR/tricky_100_questions.txt"
OUTPUT_DIR="$SCRIPT_DIR/comparison_$(date +%Y%m%d_%H%M%S)"
ANNACTL="${1:-./target/release/annactl}"
TIMEOUT_SEC=60

mkdir -p "$OUTPUT_DIR"

echo "Anna v0.0.999 - 100 Question Comparison Test"
echo "============================================="
echo "Started: $(date)"
echo "Using: $ANNACTL"
echo ""

# Stats
total=0
answered=0
clarification=0
timeout_cnt=0
error_cnt=0
team_shown=0

while IFS='|' read -r num category question || [[ -n "$question" ]]; do
    [[ "$num" =~ ^# ]] && continue
    [[ -z "$question" ]] && continue

    total=$((total + 1))
    printf "[%3d] %-12s " "$num" "$category"

    start=$(date +%s)
    result=$(timeout "${TIMEOUT_SEC}s" "$ANNACTL" "$question" 2>&1)
    rc=$?
    end=$(date +%s)
    elapsed=$((end - start))

    # Save result
    {
        echo "Question: $question"
        echo "Category: $category"
        echo "Exit: $rc"
        echo "Time: ${elapsed}s"
        echo "---"
        echo "$result"
    } > "$OUTPUT_DIR/q${num}_${category}.txt"

    # Check for team dialogue
    team_flag=""
    if echo "$result" | grep -q "TICKET\|Anna →"; then
        team_shown=$((team_shown + 1))
        team_flag="[T]"
    fi

    if [[ $rc -eq 124 ]]; then
        echo "TIMEOUT (${elapsed}s)"
        timeout_cnt=$((timeout_cnt + 1))
    elif [[ $rc -ne 0 ]]; then
        echo "ERROR ($rc)"
        error_cnt=$((error_cnt + 1))
    elif echo "$result" | grep -qiE "could you.*specific|need more|clarif|which.*mean"; then
        echo "CLARIF (${elapsed}s) $team_flag"
        clarification=$((clarification + 1))
    else
        echo "ANSWER (${elapsed}s) $team_flag"
        answered=$((answered + 1))
    fi
done < "$QUESTIONS_FILE"

echo ""
echo "============================================="
echo "SUMMARY"
echo "============================================="
echo "Total:         $total"
pct_ans=$((answered * 100 / total))
pct_clar=$((clarification * 100 / total))
pct_team=$((team_shown * 100 / total))
echo "Answered:      $answered ($pct_ans%)"
echo "Clarification: $clarification ($pct_clar%)"
echo "Timeout:       $timeout_cnt"
echo "Error:         $error_cnt"
echo "Team Dialogue: $team_shown ($pct_team%)"
echo ""
echo "Completed: $(date)"
echo "Results: $OUTPUT_DIR"

# Summary file
cat > "$OUTPUT_DIR/SUMMARY.txt" << EOF
Anna v0.0.999 Comparison Test Results
=====================================
Date: $(date)
Total Questions: $total
Direct Answers: $answered ($pct_ans%)
Asked Clarification: $clarification ($pct_clar%)
Timeouts: $timeout_cnt
Errors: $error_cnt
Team Dialogue Shown: $team_shown ($pct_team%)
EOF

echo ""
echo "Done!"
