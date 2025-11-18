#!/bin/bash
# Fetch actual Reddit comments for comparison with Anna's responses
# Usage: ./fetch_reddit_comments.sh <questions_json> <output_comparison>

QUESTIONS_FILE="${1:-data/reddit_questions.json}"
OUTPUT_FILE="${2:-reddit_answer_comparison.md}"

if [[ ! -f "$QUESTIONS_FILE" ]]; then
    echo "Error: Questions file not found: $QUESTIONS_FILE" >&2
    exit 1
fi

echo "# Reddit Answer Quality Comparison" > "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "Comparing Anna's LLM responses with actual top-voted Reddit community answers." >> "$OUTPUT_FILE"
echo "Generated: $(date)" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Get total number of questions
TOTAL=$(jq 'length' "$QUESTIONS_FILE")
echo "Analyzing $TOTAL questions..." >&2

count=0
for row in $(jq -r '.[] | @base64' "$QUESTIONS_FILE"); do
    count=$((count + 1))
    if [ $count -gt 10 ]; then
        break  # Limit to 10 for initial comparison
    fi

    _jq() {
        echo "${row}" | base64 --decode | jq -r "${1}"
    }

    post_id=$(_jq '.id')
    title=$(_jq '.title')
    score=$(_jq '.score')
    num_comments=$(_jq '.num_comments')

    echo "[$count/$TOTAL] Fetching comments for: $title" >&2

    # Fetch post comments from Reddit API
    comments_url="https://www.reddit.com/r/archlinux/comments/${post_id}.json"

    # Fetch and parse top comment
    top_comment=$(curl -s -H "User-Agent: Anna Assistant Validator/1.0" "$comments_url" 2>/dev/null | \
        jq -r '.[1].data.children[0].data |
        if .body then
            "**Author:** u/\(.author) | **Score:** \(.score) upvotes\n\n\(.body)"
        else
            "No comments available"
        end' 2>/dev/null)

    if [ $? -ne 0 ] || [ -z "$top_comment" ]; then
        top_comment="[Failed to fetch comments]"
    fi

    # Get Anna's response from validation results (if exists)
    anna_response="[Run validation first to get Anna's response]"

    # Write comparison
    cat << EOF >> "$OUTPUT_FILE"
---

## Question #$count: $title

**Reddit Post:**
- Score: $score upvotes
- Comments: $num_comments
- ID: $post_id

### Top Community Answer (Most Upvoted)

$top_comment

### Anna's LLM Response

$anna_response

### Comparison Analysis

- **Community answer length:** $(echo "$top_comment" | wc -w) words
- **Anna's response length:** $(echo "$anna_response" | wc -w) words
- **Quality assessment:** [Manual review required]

EOF

    # Rate limit - be respectful to Reddit API
    sleep 2
done

echo "" >> "$OUTPUT_FILE"
echo "Comparison complete. Results saved to: $OUTPUT_FILE" >&2
echo "Manual review required to assess answer quality." >&2
