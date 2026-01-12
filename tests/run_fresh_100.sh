#!/bin/bash
# Run fresh 100 questions through Anna and capture results
# Usage: ./run_fresh_100.sh

QUESTIONS_FILE="fresh_100_questions.txt"
OUTPUT_DIR="fresh_100_results_$(date +%Y%m%d_%H%M%S)"
TIMEOUT=60

mkdir -p "$OUTPUT_DIR"

# Extract just the questions (skip comments and empty lines)
grep -E "^[0-9]+\." "$QUESTIONS_FILE" | while read -r line; do
    num=$(echo "$line" | cut -d. -f1)
    question=$(echo "$line" | cut -d. -f2- | sed 's/^ *//')

    # Create safe filename
    safe_name=$(echo "$question" | tr ' ' '_' | tr -cd 'a-zA-Z0-9_' | head -c 40)
    outfile="$OUTPUT_DIR/q${num}_${safe_name}.txt"

    echo "[$num/100] $question"

    # Time the request
    start_time=$(date +%s.%N)

    # Run Anna with timeout
    timeout $TIMEOUT annactl "$question" > "$outfile" 2>&1
    exit_code=$?

    end_time=$(date +%s.%N)
    elapsed=$(echo "$end_time - $start_time" | bc)

    # Record timing
    echo "TIME: ${elapsed}s" >> "$outfile"
    echo "EXIT: $exit_code" >> "$outfile"

    if [ $exit_code -eq 124 ]; then
        echo "  [TIMEOUT after ${TIMEOUT}s]"
    else
        echo "  [Done in ${elapsed}s]"
    fi
done

# Generate summary
echo ""
echo "=== SUMMARY ==="
total=$(ls "$OUTPUT_DIR"/*.txt 2>/dev/null | wc -l)
timeouts=$(grep -l "exit_code=124\|TIMEOUT" "$OUTPUT_DIR"/*.txt 2>/dev/null | wc -l)
echo "Total: $total"
echo "Timeouts: $timeouts"
echo "Success: $((total - timeouts))"
echo "Results in: $OUTPUT_DIR"
