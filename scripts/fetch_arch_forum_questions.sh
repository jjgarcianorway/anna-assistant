#!/bin/bash
# Fetch questions from Arch Linux official forums (BBS)
# URL: https://bbs.archlinux.org/

OUTPUT_FILE="${1:-data/arch_forum_questions.json}"

echo "Fetching questions from Arch Linux forums..." >&2

# The Arch BBS doesn't have a public API, so we'll fetch via RSS feeds
# Categories of interest:
# - Installation: https://bbs.archlinux.org/extern.php?action=feed&fid=11&type=rss
# - Kernel & Hardware: https://bbs.archlinux.org/extern.php?action=feed&fid=7&type=rss
# - Networking: https://bbs.archlinux.org/extern.php?action=feed&fid=13&type=rss
# - Laptop Issues: https://bbs.archlinux.org/extern.php?action=feed&fid=14&type=rss

CATEGORIES=(
    "11:Installation"
    "7:Kernel_Hardware"
    "13:Networking"
    "14:Laptop_Issues"
    "15:Desktop_Environments"
)

mkdir -p "$(dirname "$OUTPUT_FILE")"

echo "[" > "$OUTPUT_FILE"
first=true

for cat in "${CATEGORIES[@]}"; do
    fid="${cat%%:*}"
    name="${cat##*:}"

    echo "Fetching from category: $name (fid=$fid)..." >&2

    # Fetch RSS feed
    rss_url="https://bbs.archlinux.org/extern.php?action=feed&fid=${fid}&type=rss"

    curl -s -H "User-Agent: Anna Assistant Forum Validator/1.0" "$rss_url" | \
        xmllint --xpath '//item' - 2>/dev/null | \
        sed 's/<item>/\n<item>/g' | \
        grep '<item>' | \
        while IFS= read -r item; do
            title=$(echo "$item" | xmllint --xpath 'string(//title)' - 2>/dev/null | head -1)
            link=$(echo "$item" | xmllint --xpath 'string(//link)' - 2>/dev/null | head -1)
            desc=$(echo "$item" | xmllint --xpath 'string(//description)' - 2>/dev/null | head -1)
            pubdate=$(echo "$item" | xmllint --xpath 'string(//pubDate)' - 2>/dev/null | head -1)

            # Extract post ID from link
            post_id=$(echo "$link" | grep -oP 'id=\K[0-9]+' || echo "unknown")

            # Clean description (remove HTML tags)
            desc_clean=$(echo "$desc" | sed 's/<[^>]*>//g' | sed 's/&quot;/"/g' | sed 's/&amp;/\&/g' | head -c 500)

            if [ -n "$title" ] && [ -n "$link" ]; then
                if [ "$first" = false ]; then
                    echo "," >> "$OUTPUT_FILE"
                fi
                first=false

                # Output JSON object
                jq -n \
                    --arg id "$post_id" \
                    --arg title "$title" \
                    --arg body "$desc_clean" \
                    --arg url "$link" \
                    --arg category "$name" \
                    --arg pubdate "$pubdate" \
                    '{id: $id, title: $title, body: $body, url: $url, category: $category, pubdate: $pubdate}' \
                    >> "$OUTPUT_FILE"
            fi
        done

    # Rate limit
    sleep 2
done

echo "]" >> "$OUTPUT_FILE"

# Validate JSON
if jq empty "$OUTPUT_FILE" 2>/dev/null; then
    count=$(jq 'length' "$OUTPUT_FILE")
    echo "Successfully fetched $count forum questions" >&2
    echo "Saved to: $OUTPUT_FILE" >&2
else
    echo "Error: Invalid JSON generated" >&2
    exit 1
fi
