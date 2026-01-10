#!/bin/bash
# Quick Anna Test - 20 questions, faster timeout
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANNACTL="${SCRIPT_DIR}/../target/release/annactl"
QUESTIONS_FILE="$SCRIPT_DIR/quick_20_test.txt"
RESULTS_DIR="$SCRIPT_DIR/quick_results_$(date +%Y%m%d_%H%M%S)"
TIMEOUT_SECS=45

mkdir -p "$RESULTS_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║     Quick Anna Test - 20 Questions (45s timeout)   ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════╝${NC}"
echo "Results: $RESULTS_DIR"
echo "Started: $(date)"
echo ""

# Check annactl
if [ ! -f "$ANNACTL" ]; then
    echo "Building..."
    cargo build --release --workspace --quiet
fi

# Ensure daemon is running
if ! "$ANNACTL" status &>/dev/null; then
    echo "Starting daemon..."
    "$ANNACTL" --daemon &
    sleep 3
fi

# Stats
total=0
completed=0
timeouts=0
clarifications=0
answers=0
declare -a times

echo -e "${BLUE}Running tests...${NC}"
echo ""

while IFS='|' read -r num category question || [ -n "$num" ]; do
    [[ "$num" =~ ^#.*$ ]] && continue
    [[ -z "$num" ]] && continue

    total=$((total + 1))

    printf "${YELLOW}[%2d/20]${NC} %-12s %.45s" "$num" "[$category]" "$question"

    start_ns=$(date +%s%N)

    if timeout $TIMEOUT_SECS "$ANNACTL" "$question" > "$RESULTS_DIR/q${num}.txt" 2>&1; then
        end_ns=$(date +%s%N)
        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        times+=($elapsed_ms)
        completed=$((completed + 1))

        response=$(cat "$RESULTS_DIR/q${num}.txt")

        # Check if clarification
        if echo "$response" | grep -qiE "Could you.*specific|Could you.*clarify|need more"; then
            clarifications=$((clarifications + 1))
            printf " ${YELLOW}?${NC} %5dms CLARIFY\n" "$elapsed_ms"
        else
            answers=$((answers + 1))
            printf " ${GREEN}✓${NC} %5dms\n" "$elapsed_ms"
        fi
    else
        timeouts=$((timeouts + 1))
        printf " ${RED}TIMEOUT${NC}\n"
    fi

    sleep 0.2
done < "$QUESTIONS_FILE"

# Calculate stats
if [ ${#times[@]} -gt 0 ]; then
    total_time=0
    for t in "${times[@]}"; do
        total_time=$((total_time + t))
    done
    avg=$((total_time / ${#times[@]}))

    IFS=$'\n' sorted=($(sort -n <<<"${times[*]}")); unset IFS
    median=${sorted[$(( ${#sorted[@]} / 2 ))]}
    min=${sorted[0]}
    max=${sorted[-1]}
else
    avg=0; median=0; min=0; max=0
fi

completion_pct=$((completed * 100 / total))
answer_pct=$((answers * 100 / total))

echo ""
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}RESULTS${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Completed:     ${GREEN}$completed${NC}/$total ($completion_pct%)"
echo -e "Real answers:  ${GREEN}$answers${NC}/$total ($answer_pct%)"
echo -e "Clarifications: ${YELLOW}$clarifications${NC}"
echo -e "Timeouts:      ${RED}$timeouts${NC}"
echo ""
echo "Response times:"
echo "  Avg:    ${avg}ms"
echo "  Median: ${median}ms"
echo "  Min:    ${min}ms"
echo "  Max:    ${max}ms"
echo ""

# Grade
if [ $answer_pct -ge 80 ]; then grade="A"
elif [ $answer_pct -ge 60 ]; then grade="B"
elif [ $answer_pct -ge 40 ]; then grade="C"
elif [ $answer_pct -ge 20 ]; then grade="D"
else grade="F"
fi

echo -e "${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "  GRADE: ${GREEN}$grade${NC}  ($answer_pct% answered, ${median}ms median)"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

# Save summary
cat > "$RESULTS_DIR/summary.json" << EOF
{
  "date": "$(date -Iseconds)",
  "total": $total,
  "completed": $completed,
  "answers": $answers,
  "clarifications": $clarifications,
  "timeouts": $timeouts,
  "completion_pct": $completion_pct,
  "answer_pct": $answer_pct,
  "avg_ms": $avg,
  "median_ms": $median,
  "min_ms": $min,
  "max_ms": $max,
  "grade": "$grade"
}
EOF

echo ""
echo "Saved to: $RESULTS_DIR/summary.json"
