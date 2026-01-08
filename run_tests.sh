#!/bin/bash
# Test anna with 100 questions and evaluate results

ANNACTL="./target/release/annactl"
QUESTIONS_FILE="test_questions.txt"
RESULTS_DIR="test_results"
SUMMARY_FILE="$RESULTS_DIR/summary.txt"

mkdir -p "$RESULTS_DIR"

# Initialize counters
total=0
success=0
empty=0
failed=0

echo "Running anna test suite..."
echo "=========================="
echo ""

# Read questions, skip comments and empty lines
while IFS= read -r line; do
    # Skip comments and empty lines
    [[ "$line" =~ ^#.*$ ]] && continue
    [[ -z "$line" ]] && continue

    ((total++))

    echo "[$total] $line"

    # Run annactl and capture output
    output=$($ANNACTL "$line" 2>&1)
    exit_code=$?

    # Save full output
    echo "$output" > "$RESULTS_DIR/q${total}.txt"

    # Extract just the answer
    answer=$(echo "$output" | sed -n '/^ANSWER:/,/^$/p' | tail -n +2)

    # Evaluate result
    if [[ $exit_code -ne 0 ]]; then
        ((failed++))
        echo "  -> FAILED (exit code $exit_code)"
    elif [[ -z "$answer" ]] || [[ "$answer" =~ ^[[:space:]]*$ ]]; then
        ((empty++))
        echo "  -> EMPTY ANSWER"
    else
        ((success++))
        # Show first 100 chars of answer
        preview=$(echo "$answer" | head -c 100 | tr '\n' ' ')
        echo "  -> OK: ${preview}..."
    fi

    echo ""

done < "$QUESTIONS_FILE"

# Calculate percentages
success_pct=$((success * 100 / total))
empty_pct=$((empty * 100 / total))
failed_pct=$((failed * 100 / total))

# Print summary
echo "=========================="
echo "SUMMARY"
echo "=========================="
echo "Total questions: $total"
echo "Successful:      $success ($success_pct%)"
echo "Empty answers:   $empty ($empty_pct%)"
echo "Failed:          $failed ($failed_pct%)"
echo ""

# Save summary
cat > "$SUMMARY_FILE" << EOF
Test Run: $(date)
========================
Total questions: $total
Successful:      $success ($success_pct%)
Empty answers:   $empty ($empty_pct%)
Failed:          $failed ($failed_pct%)
EOF

echo "Results saved to $RESULTS_DIR/"
