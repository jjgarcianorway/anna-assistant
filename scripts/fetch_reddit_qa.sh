#!/bin/bash
# Fetch r/archlinux questions for QA validation
# Beta.78: Reddit-based validation system

set -e

OUTPUT_FILE="${1:-reddit_questions.json}"
LIMIT="${2:-1000}"

echo "=== Fetching r/archlinux Questions ==="
echo "Output: $OUTPUT_FILE"
echo "Target: $LIMIT questions"
echo ""

# Reddit JSON API endpoints (no auth required for public data)
fetch_batch() {
    local timeperiod="$1"
    local after="$2"
    local url="https://www.reddit.com/r/archlinux/top.json?t=$timeperiod&limit=100"

    if [ -n "$after" ]; then
        url="${url}&after=${after}"
    fi

    echo "Fetching $timeperiod (after=$after)..." >&2
    curl -s -A "Anna Assistant QA Validator 1.0" "$url"
}

# Initialize output array
echo "[]" > "$OUTPUT_FILE"

collected=0
total_target=$LIMIT

# Fetch from different time periods
for period in month week; do
    after=""

    while [ $collected -lt $total_target ]; do
        response=$(fetch_batch "$period" "$after")

        # Check if we got valid response
        if ! echo "$response" | jq -e '.data.children' > /dev/null 2>&1; then
            echo "Invalid response from Reddit API"
            break
        fi

        # Extract and append posts
        echo "$response" | jq '[.data.children[] | {
            id: .data.id,
            title: .data.title,
            body: .data.selftext,
            score: .data.score,
            num_comments: .data.num_comments,
            url: .data.url
        }]' | jq --argjson existing "$(cat "$OUTPUT_FILE")" '$existing + .' > "${OUTPUT_FILE}.tmp"

        mv "${OUTPUT_FILE}.tmp" "$OUTPUT_FILE"

        # Update collected count
        collected=$(jq 'length' "$OUTPUT_FILE")
        echo "Collected: $collected/$total_target"

        # Check if we got any posts
        post_count=$(echo "$response" | jq '.data.children | length')
        if [ "$post_count" -eq 0 ]; then
            echo "No more posts for $period"
            break
        fi

        # Get next page token
        after=$(echo "$response" | jq -r '.data.after // empty')

        if [ -z "$after" ] || [ "$after" == "null" ]; then
            echo "No more pages for $period"
            break
        fi

        # Rate limiting
        sleep 2
    done

    if [ $collected -ge $total_target ]; then
        break
    fi
done

echo ""
echo "✅ Collected $collected questions"
echo "💾 Saved to: $OUTPUT_FILE"
echo ""
echo "Next steps:"
echo "1. Review questions: jq '.[0:5]' $OUTPUT_FILE"
echo "2. Run validation: cargo test --test reddit_qa_integration"
echo "3. Generate report: cargo run --bin reddit_qa_validator"
