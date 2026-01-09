#!/bin/bash
# Anna vs Claude evaluation script
# Runs questions through annactl and saves responses

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ANNACTL="$SCRIPT_DIR/../target/release/annactl"
QUESTIONS_FILE="$SCRIPT_DIR/questions.txt"
OUTPUT_DIR="$SCRIPT_DIR/results"

mkdir -p "$OUTPUT_DIR"

# Extract just the questions (skip comments and empty lines)
grep -v '^#' "$QUESTIONS_FILE" | grep -v '^[[:space:]]*$' | head -n "${1:-20}" | while IFS= read -r question; do
    # Create safe filename from question
    safe_name=$(echo "$question" | tr ' ' '_' | tr -cd '[:alnum:]_' | head -c 50)
    output_file="$OUTPUT_DIR/${safe_name}.txt"

    echo ">>> $question"

    # Run through annactl with timeout
    timeout 60 "$ANNACTL" "$question" > "$output_file" 2>&1

    # Show brief result
    head -5 "$output_file"
    echo "---"
done

echo "Results saved to $OUTPUT_DIR/"
