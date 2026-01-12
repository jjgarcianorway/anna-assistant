#!/bin/bash
# Test Anna with 100 questions and record timing

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QUESTIONS_FILE="$SCRIPT_DIR/questions_100.txt"
RESULTS_FILE="$SCRIPT_DIR/anna_results.txt"
ANNACTL="$SCRIPT_DIR/../../target/release/annactl"

# Build release if needed
if [ ! -f "$ANNACTL" ]; then
    echo "Building release binary..."
    cd "$SCRIPT_DIR/../.." && cargo build --release --quiet
fi

# Clear previous results
> "$RESULTS_FILE"

echo "Running Anna test with 100 questions..."
echo "Results will be saved to: $RESULTS_FILE"
echo ""

question_num=0
total_time=0
success_count=0
timeout_count=0

while IFS= read -r line; do
    # Skip comments and empty lines
    [[ "$line" =~ ^#.*$ ]] && continue
    [[ -z "$line" ]] && continue

    question_num=$((question_num + 1))

    echo "[$question_num/100] $line"

    # Time the question
    start_time=$(date +%s.%N)

    # Run with 60 second timeout
    response=$(timeout 60 "$ANNACTL" "$line" 2>&1)
    exit_code=$?

    end_time=$(date +%s.%N)
    elapsed=$(echo "$end_time - $start_time" | bc)

    # Check for timeout
    if [ $exit_code -eq 124 ]; then
        status="TIMEOUT"
        timeout_count=$((timeout_count + 1))
        response="[TIMEOUT after 60s]"
    else
        status="OK"
        success_count=$((success_count + 1))
        total_time=$(echo "$total_time + $elapsed" | bc)
    fi

    # Save result
    echo "========================================" >> "$RESULTS_FILE"
    echo "Q$question_num: $line" >> "$RESULTS_FILE"
    echo "Time: ${elapsed}s | Status: $status" >> "$RESULTS_FILE"
    echo "----------------------------------------" >> "$RESULTS_FILE"
    echo "$response" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"

    echo "  -> ${elapsed}s ($status)"

done < "$QUESTIONS_FILE"

# Calculate stats
if [ $success_count -gt 0 ]; then
    avg_time=$(echo "scale=2; $total_time / $success_count" | bc)
else
    avg_time="N/A"
fi

echo ""
echo "========================================"
echo "ANNA TEST COMPLETE"
echo "========================================"
echo "Questions: $question_num"
echo "Success: $success_count"
echo "Timeouts: $timeout_count"
echo "Total time: ${total_time}s"
echo "Average time: ${avg_time}s"
echo ""
echo "Results saved to: $RESULTS_FILE"

# Save summary
echo "" >> "$RESULTS_FILE"
echo "========================================"  >> "$RESULTS_FILE"
echo "SUMMARY" >> "$RESULTS_FILE"
echo "Questions: $question_num" >> "$RESULTS_FILE"
echo "Success: $success_count" >> "$RESULTS_FILE"
echo "Timeouts: $timeout_count" >> "$RESULTS_FILE"
echo "Total time: ${total_time}s" >> "$RESULTS_FILE"
echo "Average time: ${avg_time}s" >> "$RESULTS_FILE"
