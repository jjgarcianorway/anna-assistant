#!/bin/bash
# Quick Battle Test - 10 questions to demo Anna's capabilities

ANNACTL="./target/release/annactl"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo "🥊 ANNA VS CLAUDE: QUICK DEMO (10 Questions) 🥊"
echo ""

QUESTIONS=(
    "What's my disk usage?"
    "Show me running services"
    "What kernel am I running?"
    "Install htop if not installed"
    "Find all log files larger than 100MB"
    "Show me my boot time breakdown"
    "Set up automatic system backups"
    "Debug why my Bluetooth isn't working"
    "Create a script that tells me jokes when I'm frustrated"
    "Make my system boot faster by sacrificing a random non-essential package each time"
)

SUCCESS=0
TOTAL=${#QUESTIONS[@]}

for i in "${!QUESTIONS[@]}"; do
    Q_NUM=$((i + 1))
    QUESTION="${QUESTIONS[$i]}"

    printf "\n${CYAN}Q%d: %s${NC}\n" "$Q_NUM" "$QUESTION"
    echo "----------------------------------------"

    START=$(date +%s%3N)
    RESPONSE=$($ANNACTL "$QUESTION" 2>&1)
    END=$(date +%s%3N)
    DURATION=$((END - START))

    # Show first few lines of response
    echo "$RESPONSE" | head -15
    if [ $(echo "$RESPONSE" | wc -l) -gt 15 ]; then
        echo "... (truncated)"
    fi

    # Evaluate
    if [[ ! -z "$RESPONSE" ]] &&
       [[ ! "$RESPONSE" =~ [Ee]rror ]] &&
       [[ ! "$RESPONSE" =~ [Ff]ailed ]]; then
        SUCCESS=$((SUCCESS + 1))
        echo -e "\n${GREEN}✓ SUCCESS${NC} (${DURATION}ms)"
    else
        echo -e "\n${RED}✗ FAILED${NC} (${DURATION}ms)"
    fi

    sleep 0.5
done

echo ""
echo "========================================"
echo "📊 RESULTS: $SUCCESS/$TOTAL succeeded"
echo "========================================"
