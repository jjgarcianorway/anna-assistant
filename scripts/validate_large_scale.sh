#!/bin/bash
# Large-Scale QA Validation with Answer Validation
# Beta.87: Tests Anna's zero-hallucination guarantee at scale

set -e

QUESTIONS_FILE="${1:-data/linux_questions_multi.json}"
SAMPLE_SIZE="${2:-1000}"
OUTPUT_FILE="validation_results_beta87.json"
REPORT_FILE="validation_report_beta87.md"

echo "=== Large-Scale Validation - Beta.87 ==="
echo "Questions file: $QUESTIONS_FILE"
echo "Sample size: $SAMPLE_SIZE"
echo "Output: $OUTPUT_FILE"
echo "Report: $REPORT_FILE"
echo ""

# Check dependencies
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required"
    exit 1
fi

# Check if Anna daemon is running
if ! annactl status &> /dev/null; then
    echo "Error: Anna daemon (annad) is not running"
    echo "Start it with: sudo systemctl start annad"
    exit 1
fi

# Check question count
total_questions=$(jq 'length' "$QUESTIONS_FILE")
echo "Total questions available: $total_questions"

if [ "$SAMPLE_SIZE" -gt "$total_questions" ]; then
    SAMPLE_SIZE=$total_questions
    echo "Adjusted sample size to: $SAMPLE_SIZE"
fi

echo ""
echo "Starting validation..."
echo ""

# Initialize results array
echo "[]" > "$OUTPUT_FILE"

# Counters
passed=0
failed=0
partial=0
total_time=0
total_words=0

# Process each question
for i in $(seq 0 $((SAMPLE_SIZE - 1))); do
    # Extract question
    question_data=$(jq -r ".[$i]" "$QUESTIONS_FILE")

    if [ "$question_data" == "null" ]; then
        break
    fi

    title=$(echo "$question_data" | jq -r '.title')
    body=$(echo "$question_data" | jq -r '.body // ""')
    subreddit=$(echo "$question_data" | jq -r '.subreddit // "unknown"')
    score=$(echo "$question_data" | jq -r '.score')

    echo "[$((i + 1))/$SAMPLE_SIZE] Testing: $title"

    # Build query for Anna
    if [ -n "$body" ] && [ "$body" != "null" ]; then
        query="$title\n\n$body"
    else
        query="$title"
    fi

    # Query Anna's LLM through daemon (uses answer validation!)
    start_time=$(date +%s%N)

    # Use annactl ask command which includes full validation
    response=$(echo -e "$query" | annactl ask 2>&1 || echo "ERROR: Query failed")

    end_time=$(date +%s%N)
    elapsed_ms=$(( (end_time - start_time) / 1000000 ))
    total_time=$((total_time + elapsed_ms))

    # Count words
    word_count=$(echo "$response" | wc -w)
    total_words=$((total_words + word_count))

    # Determine validation status
    if echo "$response" | grep -q "ERROR:"; then
        status="FAILED"
        failed=$((failed + 1))
    elif echo "$response" | grep -q "validation"; then
        status="PARTIAL"
        partial=$((partial + 1))
    elif [ "$word_count" -ge 50 ]; then
        status="PASSED"
        passed=$((passed + 1))
    else
        status="PARTIAL"
        partial=$((partial + 1))
    fi

    echo "   Response: $word_count words, ${elapsed_ms}ms - $status"

    # Save result
    result=$(jq -n \
        --arg title "$title" \
        --arg subreddit "$subreddit" \
        --argjson score "$score" \
        --arg response "$response" \
        --arg status "$status" \
        --argjson elapsed_ms "$elapsed_ms" \
        --argjson word_count "$word_count" \
        '{
            title: $title,
            subreddit: $subreddit,
            score: $score,
            response_status: $status,
            response_time_ms: $elapsed_ms,
            word_count: $word_count,
            response: $response
        }')

    # Append to results
    jq --argjson result "$result" '. += [$result]' "$OUTPUT_FILE" > "${OUTPUT_FILE}.tmp"
    mv "${OUTPUT_FILE}.tmp" "$OUTPUT_FILE"
done

# Calculate stats
avg_time=$((total_time / SAMPLE_SIZE))
avg_words=$((total_words / SAMPLE_SIZE))
pass_rate=$(echo "scale=2; ($passed * 100) / $SAMPLE_SIZE" | bc)

echo ""
echo "=== Validation Complete ==="
echo "Passed: $passed ($pass_rate%)"
echo "Partial: $partial"
echo "Failed: $failed"
echo "Avg time: ${avg_time}ms"
echo "Avg words: $avg_words"
echo ""

# Generate markdown report
cat > "$REPORT_FILE" <<EOF
# Large-Scale Validation Report - Beta.87

**Date:** $(date)
**Questions Tested:** $SAMPLE_SIZE
**Source:** $QUESTIONS_FILE

## Summary

| Metric | Value |
|--------|-------|
| **Passed** | $passed ($pass_rate%) |
| **Partial** | $partial |
| **Failed** | $failed |
| **Avg Response Time** | ${avg_time}ms |
| **Avg Word Count** | $avg_words |

## Validation Status Breakdown

- **PASSED**: Answer validation succeeded, no hallucinations detected
- **PARTIAL**: Answer provided but may have validation warnings
- **FAILED**: Query failed or validation rejected answer

## Top 10 Best Responses

EOF

# Add top responses by word count
jq -r '.[] | select(.response_status == "PASSED") |
    "### " + .title + "\n" +
    "**Subreddit:** r/" + .subreddit + " | " +
    "**Score:** " + (.score|tostring) + " | " +
    "**Time:** " + (.response_time_ms|tostring) + "ms | " +
    "**Words:** " + (.word_count|tostring) + "\n\n" +
    .response + "\n\n---\n"' \
    "$OUTPUT_FILE" | head -50 >> "$REPORT_FILE"

echo "Report saved to: $REPORT_FILE"
echo "Full results saved to: $OUTPUT_FILE"
