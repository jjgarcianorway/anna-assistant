#!/bin/bash
# 20 Crazy Questions - Testing concise answers and response times

ANNACTL="./target/release/annactl"

questions=(
    # Simple facts
    "how much RAM do I have?"
    "what CPU am I using?"
    "what's my IP address?"

    # System status
    "is bluetooth running?"
    "what's eating my CPU right now?"
    "any failed services?"

    # How-to (should skip wiki for vague, use for specific)
    "how do I restart NetworkManager?"
    "how to clear pacman cache?"
    "how do I check battery health?"

    # Troubleshooting
    "why is my wifi slow?"
    "is my GPU being used?"
    "what's using port 8080?"

    # Configuration
    "what kernel parameters am I using?"
    "is wayland or x11 running?"
    "what shell am I using?"

    # Advanced
    "show me the last kernel panic if any"
    "what systemd timers are active?"
    "how much swap do I have?"
    "what DNS servers am I using?"
    "list my block devices"
)

echo "=============================================="
echo "  20 CRAZY QUESTIONS TEST - v0.0.857"
echo "=============================================="
echo ""

for i in "${!questions[@]}"; do
    q="${questions[$i]}"
    num=$((i + 1))

    echo "[$num/20] $q"
    echo "---"

    # Time the response
    start=$(date +%s.%N)

    # Run and capture just the answer (strip ANSI codes)
    answer=$(timeout 180 $ANNACTL "$q" 2>&1 | sed 's/\x1b\[[0-9;]*m//g' | grep -A100 "^ANSWER:" | tail -n +2 | head -20)

    end=$(date +%s.%N)
    duration=$(echo "$end - $start" | bc)

    if [ -z "$answer" ]; then
        echo "Result: TIMEOUT or EMPTY"
    else
        # Count words in answer
        word_count=$(echo "$answer" | wc -w)
        echo "Answer ($word_count words, ${duration}s):"
        echo "$answer" | head -10
        if [ $(echo "$answer" | wc -l) -gt 10 ]; then
            echo "... (truncated)"
        fi
    fi
    echo ""
    echo "=============================================="
done
