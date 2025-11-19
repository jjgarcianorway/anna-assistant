#!/bin/bash
# Fetch Linux questions from multiple subreddits for comprehensive QA validation
# Beta.87: Multi-source large-scale testing

set -e

OUTPUT_FILE="${1:-data/linux_questions_multi.json}"
TARGET="${2:-5000}"

echo "=== Fetching Linux Questions from Multiple Subreddits ==="
echo "Output: $OUTPUT_FILE"
echo "Target: $TARGET questions"
echo ""

# Subreddits to fetch from (in priority order)
SUBREDDITS=(
    "archlinux"
    "linux"
    "linuxquestions"
    "linuxadmin"
    "linux4noobs"
)

# Initialize output array
echo "[]" > "$OUTPUT_FILE"

collected=0

# Fetch from each subreddit
for subreddit in "${SUBREDDITS[@]}"; do
    echo ""
    echo "=== Fetching from r/$subreddit ==="

    # Fetch from different time periods
    for period in year all; do
        after=""

        while [ $collected -lt $TARGET ]; do
            url="https://www.reddit.com/r/$subreddit/top.json?t=$period&limit=100"

            if [ -n "$after" ]; then
                url="${url}&after=${after}"
            fi

            echo "  Fetching r/$subreddit ($period, after=$after)..." >&2
            response=$(curl -s -A "Anna Assistant QA Validator 1.0" "$url")

            # Check if we got valid response
            if ! echo "$response" | jq -e '.data.children' > /dev/null 2>&1; then
                echo "  Invalid response from Reddit API"
                break
            fi

            # Extract and append posts
            echo "$response" | jq '[.data.children[] | {
                id: .data.id,
                subreddit: .data.subreddit,
                title: .data.title,
                body: .data.selftext,
                score: .data.score,
                num_comments: .data.num_comments,
                url: .data.url
            }]' > "${OUTPUT_FILE}.new"

            # Merge using file-based jq slurp (avoids argument list too long)
            jq -s '.[0] + .[1]' "$OUTPUT_FILE" "${OUTPUT_FILE}.new" > "${OUTPUT_FILE}.tmp"
            mv "${OUTPUT_FILE}.tmp" "$OUTPUT_FILE"
            rm -f "${OUTPUT_FILE}.new"

            # Update collected count
            collected=$(jq 'length' "$OUTPUT_FILE")
            echo "  Collected: $collected/$TARGET"

            # Check if we got any posts
            post_count=$(echo "$response" | jq '.data.children | length')
            if [ "$post_count" -eq 0 ]; then
                echo "  No more posts for $period"
                break
            fi

            # Get next page token
            after=$(echo "$response" | jq -r '.data.after // empty')

            if [ -z "$after" ] || [ "$after" == "null" ]; then
                echo "  No more pages for $period"
                break
            fi

            # Rate limiting
            sleep 2
        done

        if [ $collected -ge $TARGET ]; then
            break
        fi
    done

    if [ $collected -ge $TARGET ]; then
        echo ""
        echo "✅ Reached target!"
        break
    fi
done

echo ""
echo "✅ Collected $collected questions"
echo "💾 Saved to: $OUTPUT_FILE"
echo ""
echo "Subreddit breakdown:"
jq -r 'group_by(.subreddit) | .[] | "\(.[] | .subreddit): \(length) questions"' "$OUTPUT_FILE" | sort | uniq -c
echo ""
echo "Next steps:"
echo "1. Review questions: jq '.[0:5]' $OUTPUT_FILE"
echo "2. Run validation: ./scripts/validate_reddit_qa.sh $OUTPUT_FILE 100"
echo "3. Merge with existing: jq -s '.[0] + .[1]' data/reddit_questions.json $OUTPUT_FILE > data/all_questions.json"
