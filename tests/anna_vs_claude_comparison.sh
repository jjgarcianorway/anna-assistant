#!/bin/bash
# Anna vs Claude Comparison Testing
# Goal: Measure Anna against Claude baseline on identical questions

set -euo pipefail

TEST_DIR="/tmp/anna_vs_claude_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$TEST_DIR"

echo "=== Anna vs Claude Comparison Test ==="
echo "Test Directory: $TEST_DIR"
echo ""

# Test questions (ground truth verifiable)
declare -a QUESTIONS=(
    "What is my disk usage?"
    "How much RAM am I using?"
    "What is my current load average?"
    "Is the annad service running?"
    "What is my hostname?"
    "How many CPU cores do I have?"
    "What kernel version am I running?"
    "What is my current user?"
    "How much swap am I using?"
    "What is my uptime?"
)

# Ground truth verification commands
declare -A GROUND_TRUTH=(
    ["What is my disk usage?"]="df -h / | tail -1 | awk '{print \$5}'"
    ["How much RAM am I using?"]="free -h | grep Mem | awk '{print \$3}'"
    ["What is my current load average?"]="uptime | grep -oP 'load average: \K[\d.]+'"
    ["Is the annad service running?"]="systemctl is-active annad 2>/dev/null || echo 'inactive'"
    ["What is my hostname?"]="hostname"
    ["How many CPU cores do I have?"]="nproc"
    ["What kernel version am I running?"]="uname -r"
    ["What is my current user?"]="whoami"
    ["How much swap am I using?"]="free -h | grep Swap | awk '{print \$3}'"
    ["What is my uptime?"]="uptime -p"
)

# Results tracking
declare -A ANNA_TIMES
declare -A ANNA_CORRECT
declare -A ANNA_ANSWERS

ANNA_TOTAL=0
ANNA_ACCURATE=0
ANNA_TOTAL_TIME=0

echo "Testing Anna..."
echo ""

for question in "${QUESTIONS[@]}"; do
    echo "Q: $question"

    # Get ground truth
    truth_cmd="${GROUND_TRUTH[$question]}"
    ground_truth=$(eval "$truth_cmd" 2>/dev/null || echo "unknown")
    echo "  Ground Truth: $ground_truth"

    # Test Anna
    start=$(date +%s%N)
    anna_answer=$(timeout 60 annactl "$question" 2>&1 || echo "TIMEOUT")
    end=$(date +%s%N)
    anna_time=$(( (end - start) / 1000000 ))

    ANNA_TIMES["$question"]=$anna_time
    ANNA_ANSWERS["$question"]="$anna_answer"
    ANNA_TOTAL_TIME=$((ANNA_TOTAL_TIME + anna_time))
    ANNA_TOTAL=$((ANNA_TOTAL + 1))

    # Check if Anna's answer contains ground truth
    is_correct=false
    if [ "$ground_truth" != "unknown" ]; then
        if echo "$anna_answer" | grep -qF "$ground_truth"; then
            is_correct=true
            ANNA_ACCURATE=$((ANNA_ACCURATE + 1))
        fi
    fi

    ANNA_CORRECT["$question"]=$is_correct

    echo "  Anna: ${anna_answer:0:100}..."
    echo "  Time: ${anna_time}ms"
    echo "  Accurate: $is_correct"
    echo ""
done

# Calculate Anna stats
ANNA_AVG_TIME=$((ANNA_TOTAL_TIME / ANNA_TOTAL))
ANNA_ACCURACY_PCT=$(( (ANNA_ACCURATE * 100) / ANNA_TOTAL ))

echo "========================================="
echo "ANNA RESULTS"
echo "========================================="
echo "Total Questions:   $ANNA_TOTAL"
echo "Accurate Answers:  $ANNA_ACCURATE / $ANNA_TOTAL ($ANNA_ACCURACY_PCT%)"
echo "Average Time:      ${ANNA_AVG_TIME}ms"
echo "Total Time:        ${ANNA_TOTAL_TIME}ms"
echo ""

# Save results
cat > "$TEST_DIR/results.json" <<EOF
{
  "test_run": "$(date -Iseconds)",
  "anna": {
    "version": "$(annactl --version 2>/dev/null || echo 'unknown')",
    "total_questions": $ANNA_TOTAL,
    "accurate_answers": $ANNA_ACCURATE,
    "accuracy_percentage": $ANNA_ACCURACY_PCT,
    "average_time_ms": $ANNA_AVG_TIME,
    "total_time_ms": $ANNA_TOTAL_TIME
  },
  "questions": [
EOF

# Add individual question results
first=true
for question in "${QUESTIONS[@]}"; do
    [ "$first" = false ] && echo "," >> "$TEST_DIR/results.json"
    first=false

    cat >> "$TEST_DIR/results.json" <<EOF
    {
      "question": "$question",
      "ground_truth": "$(eval "${GROUND_TRUTH[$question]}" 2>/dev/null || echo 'unknown')",
      "anna_time_ms": ${ANNA_TIMES["$question"]},
      "anna_correct": ${ANNA_CORRECT["$question"]}
    }
EOF
done

cat >> "$TEST_DIR/results.json" <<EOF

  ]
}
EOF

echo "Results saved to: $TEST_DIR/results.json"
echo ""

# Rating criteria
echo "========================================="
echo "RELIABILITY RATING"
echo "========================================="
echo ""

if [ $ANNA_ACCURACY_PCT -ge 90 ]; then
    echo "Accuracy: ✓ EXCELLENT (≥90%)"
    accuracy_rating="EXCELLENT"
elif [ $ANNA_ACCURACY_PCT -ge 75 ]; then
    echo "Accuracy: ⚠️  GOOD (≥75%)"
    accuracy_rating="GOOD"
elif [ $ANNA_ACCURACY_PCT -ge 60 ]; then
    echo "Accuracy: ⚠️  ACCEPTABLE (≥60%)"
    accuracy_rating="ACCEPTABLE"
else
    echo "Accuracy: ✗ POOR (<60%)"
    accuracy_rating="POOR"
fi

if [ $ANNA_AVG_TIME -lt 5000 ]; then
    echo "Speed:    ✓ FAST (<5s average)"
    speed_rating="FAST"
elif [ $ANNA_AVG_TIME -lt 15000 ]; then
    echo "Speed:    → MODERATE (5-15s average)"
    speed_rating="MODERATE"
else
    echo "Speed:    ⚠️  SLOW (>15s average)"
    speed_rating="SLOW"
fi

echo ""
echo "Overall Rating: $accuracy_rating / $speed_rating"
echo ""

# Recommendations
echo "========================================="
echo "RECOMMENDATIONS"
echo "========================================="
echo ""

if [ $ANNA_ACCURACY_PCT -lt 90 ]; then
    echo "⚠️  Accuracy below 90%:"
    for question in "${QUESTIONS[@]}"; do
        if [ "${ANNA_CORRECT[$question]}" = "false" ]; then
            echo "  - Fix: $question"
        fi
    done
    echo ""
fi

if [ $ANNA_AVG_TIME -gt 5000 ]; then
    echo "⚠️  Average response time >5s:"
    for question in "${QUESTIONS[@]}"; do
        time=${ANNA_TIMES["$question"]}
        if [ $time -gt 5000 ]; then
            echo "  - Optimize: $question (${time}ms)"
        fi
    done
    echo ""
fi

# Exit code based on reliability
if [ $ANNA_ACCURACY_PCT -ge 90 ] && [ $ANNA_AVG_TIME -lt 15000 ]; then
    echo "✓ PASS: Anna meets reliability standards"
    exit 0
elif [ $ANNA_ACCURACY_PCT -ge 75 ]; then
    echo "⚠️  WARNING: Anna needs improvement"
    exit 1
else
    echo "✗ FAIL: Anna reliability below acceptable threshold"
    exit 2
fi
