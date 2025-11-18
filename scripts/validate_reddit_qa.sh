#!/bin/bash
# Reddit QA Validation Runner - Beta.80
# Tests Anna's responses against real r/archlinux questions

set -e

QUESTIONS_FILE="${1:-data/reddit_questions.json}"
SAMPLE_SIZE="${2:-10}"
OUTPUT_FILE="reddit_validation_results.md"

echo "=== Reddit QA Validation ==="
echo "Questions file: $QUESTIONS_FILE"
echo "Sample size: $SAMPLE_SIZE"
echo "Output: $OUTPUT_FILE"
echo ""

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed"
    exit 1
fi

# Check if Ollama is available
if ! command -v ollama &> /dev/null; then
    echo "Error: ollama is required but not installed"
    exit 1
fi

# Get the current LLM model from Anna's config
MODEL=$(ollama list | tail -n +2 | head -n 1 | awk '{print $1}')
if [ -z "$MODEL" ]; then
    echo "Error: No Ollama models found"
    exit 1
fi

echo "Using model: $MODEL"
echo ""

# Initialize results file
cat > "$OUTPUT_FILE" <<EOF
# Reddit QA Validation Report
**Date:** $(date)
**Model:** $MODEL
**Sample Size:** $SAMPLE_SIZE

## Results

EOF

# Query Anna's LLM for each question
question_num=0
helpful_count=0
total_time=0

while [ $question_num -lt $SAMPLE_SIZE ]; do
    # Extract question
    question_data=$(jq -r ".[$question_num]" "$QUESTIONS_FILE")

    if [ "$question_data" == "null" ]; then
        echo "Warning: Not enough questions in file"
        break
    fi

    title=$(echo "$question_data" | jq -r '.title')
    body=$(echo "$question_data" | jq -r '.body')
    score=$(echo "$question_data" | jq -r '.score')
    comments=$(echo "$question_data" | jq -r '.num_comments')
    url=$(echo "$question_data" | jq -r '.url')

    echo "[$((question_num + 1))/$SAMPLE_SIZE] Testing: $title"

    # Build prompt for Anna
    prompt="You are Anna, an expert Arch Linux assistant. A user asks:

Title: $title

$body

Provide a helpful, accurate, and actionable answer. Focus on solving their problem."

    # Query Ollama with timing
    start_time=$(date +%s%N)

    response=$(curl -s http://localhost:11434/api/generate -d "{
        \"model\": \"$MODEL\",
        \"prompt\": $(echo "$prompt" | jq -Rs .),
        \"stream\": false
    }" | jq -r '.response')

    end_time=$(date +%s%N)
    elapsed_ms=$(( (end_time - start_time) / 1000000 ))
    total_time=$((total_time + elapsed_ms))

    # Simple heuristic: helpful if response is substantial and contains actionable info
    word_count=$(echo "$response" | wc -w)
    has_commands=$(echo "$response" | grep -E '(sudo|pacman|systemctl|journalctl)' && echo "yes" || echo "no")

    if [ "$word_count" -ge 50 ] && [ "$has_commands" == "yes" ]; then
        helpful_count=$((helpful_count + 1))
        status="✅ HELPFUL"
    elif [ "$word_count" -ge 50 ]; then
        status="⚠️  PARTIAL"
    else
        status="❌ UNHELPFUL"
    fi

    echo "   Response: ${word_count} words, ${elapsed_ms}ms - $status"

    # Write to results file
    cat >> "$OUTPUT_FILE" <<EOF
---

### Question #$((question_num + 1)): $title

**Reddit Score:** $score upvotes, $comments comments
**URL:** $url

**Question:**
\`\`\`
$body
\`\`\`

**Anna's Response ($status):**
\`\`\`
$response
\`\`\`

**Metrics:**
- Word count: $word_count
- Response time: ${elapsed_ms}ms
- Contains commands: $has_commands

EOF

    question_num=$((question_num + 1))
done

# Calculate metrics
avg_time=$((total_time / question_num))
helpful_pct=$(echo "scale=1; ($helpful_count * 100) / $question_num" | bc)

# Write summary
cat >> "$OUTPUT_FILE" <<EOF

---

## Summary

**Total Questions Tested:** $question_num
**Helpful Responses:** $helpful_count ($helpful_pct%)
**Average Response Time:** ${avg_time}ms
**Model:** $MODEL

### Criteria for "Helpful"
- ✅ Response has 50+ words AND contains actionable commands
- ⚠️  Response has 50+ words but lacks specific commands
- ❌ Response is too short (< 50 words)

### Interpretation

EOF

if (( $(echo "$helpful_pct >= 70" | bc -l) )); then
    cat >> "$OUTPUT_FILE" <<EOF
**PASS** - Anna provides helpful responses to most questions ($helpful_pct% helpful rate)
EOF
elif (( $(echo "$helpful_pct >= 50" | bc -l) )); then
    cat >> "$OUTPUT_FILE" <<EOF
**MODERATE** - Anna provides decent responses but has room for improvement ($helpful_pct% helpful rate)
EOF
else
    cat >> "$OUTPUT_FILE" <<EOF
**NEEDS IMPROVEMENT** - Anna struggles to provide consistently helpful responses ($helpful_pct% helpful rate)
EOF
fi

echo ""
echo "✅ Validation complete!"
echo "📊 Results: $helpful_count/$question_num helpful ($helpful_pct%)"
echo "⏱️  Avg response time: ${avg_time}ms"
echo "💾 Full report: $OUTPUT_FILE"
echo ""
