#!/bin/bash
# Anna Performance Test v3
# Tests 100 tricky real-world questions measuring:
# - Response time
# - Completion rate (reliability)
# - Answer type (real answer vs clarification request)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANNACTL="${SCRIPT_DIR}/../target/release/annactl"
QUESTIONS_FILE="$SCRIPT_DIR/tricky_100_v3.txt"
RESULTS_DIR="$SCRIPT_DIR/results_v3_$(date +%Y%m%d_%H%M%S)"
TIMEOUT_SECS=60

mkdir -p "$RESULTS_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║         Anna Performance Test v3 - 100 Questions            ║${NC}"
echo -e "${CYAN}║         Measuring: Speed, Reliability, Accuracy             ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Results: $RESULTS_DIR"
echo "Started: $(date)"
echo ""

# Build if needed
if [ ! -f "$ANNACTL" ]; then
    echo -e "${YELLOW}Building annactl...${NC}"
    cargo build --release --workspace --quiet
fi

# Ensure daemon is running
if ! "$ANNACTL" status &>/dev/null; then
    echo -e "${YELLOW}Starting anna daemon...${NC}"
    "$ANNACTL" --daemon &
    sleep 3
fi

# Initialize arrays for statistics
declare -a response_times
declare -a categories
total=0
completed=0
timeout_count=0
error_count=0
clarification_count=0
real_answer_count=0

# Patterns that indicate Anna asked for clarification instead of answering
CLARIFICATION_PATTERNS=(
    "Could you.*specific"
    "Could you.*clarify"
    "Could you.*tell me"
    "Can you.*more detail"
    "What.*specifically"
    "Which.*exactly"
    "I need more information"
    "please provide"
    "what are you trying"
    "Could you please"
    "help me understand"
)

# Function to check if response is a clarification request
is_clarification() {
    local response="$1"
    for pattern in "${CLARIFICATION_PATTERNS[@]}"; do
        if echo "$response" | grep -qiE "$pattern"; then
            return 0
        fi
    done
    return 1
}

# Function to get answer quality score (0-3)
# 0 = error/timeout, 1 = clarification, 2 = short answer, 3 = detailed answer
get_quality_score() {
    local response="$1"
    local status="$2"

    if [ "$status" != "success" ]; then
        echo 0
        return
    fi

    if is_clarification "$response"; then
        echo 1
        return
    fi

    local len=${#response}
    if [ $len -lt 200 ]; then
        echo 2
    else
        echo 3
    fi
}

echo -e "${BLUE}Running tests...${NC}"
echo ""

# Read questions and test
while IFS='|' read -r num category question || [ -n "$num" ]; do
    # Skip comments and empty lines
    [[ "$num" =~ ^#.*$ ]] && continue
    [[ -z "$num" ]] && continue

    total=$((total + 1))
    categories[$total]="$category"

    # Show progress
    printf "${YELLOW}[%3d/100]${NC} ${BLUE}%-10s${NC} %.50s" "$num" "[$category]" "$question"

    # Files for this question
    response_file="$RESULTS_DIR/q${num}_response.txt"
    meta_file="$RESULTS_DIR/q${num}_meta.json"

    # Record start time
    start_ns=$(date +%s%N)

    # Run anna with timeout
    if timeout $TIMEOUT_SECS "$ANNACTL" "$question" > "$response_file" 2>&1; then
        end_ns=$(date +%s%N)
        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        response_times+=($elapsed_ms)
        completed=$((completed + 1))

        # Read response
        response=$(cat "$response_file")
        response_len=${#response}

        # Check if it's a clarification request
        if is_clarification "$response"; then
            clarification_count=$((clarification_count + 1))
            answer_type="clarification"
            status_icon="${YELLOW}?${NC}"
        else
            real_answer_count=$((real_answer_count + 1))
            answer_type="answer"
            status_icon="${GREEN}✓${NC}"
        fi

        quality=$(get_quality_score "$response" "success")

        # Display result
        if [ $elapsed_ms -lt 5000 ]; then
            time_color="${GREEN}"
        elif [ $elapsed_ms -lt 15000 ]; then
            time_color="${YELLOW}"
        else
            time_color="${RED}"
        fi

        printf " ${status_icon} ${time_color}%5dms${NC} %4db\n" "$elapsed_ms" "$response_len"

        # Save metadata
        cat > "$meta_file" << EOF
{
  "num": $num,
  "category": "$category",
  "question": "$(echo "$question" | sed 's/"/\\"/g')",
  "status": "success",
  "answer_type": "$answer_type",
  "response_time_ms": $elapsed_ms,
  "response_length": $response_len,
  "quality_score": $quality
}
EOF
    else
        exit_code=$?
        end_ns=$(date +%s%N)
        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))

        if [ $exit_code -eq 124 ]; then
            timeout_count=$((timeout_count + 1))
            status="timeout"
            printf " ${RED}TIMEOUT${NC}\n"
        else
            error_count=$((error_count + 1))
            status="error"
            printf " ${RED}ERROR($exit_code)${NC}\n"
        fi

        cat > "$meta_file" << EOF
{
  "num": $num,
  "category": "$category",
  "question": "$(echo "$question" | sed 's/"/\\"/g')",
  "status": "$status",
  "answer_type": "none",
  "response_time_ms": $elapsed_ms,
  "exit_code": $exit_code,
  "quality_score": 0
}
EOF
    fi

    # Brief pause between questions
    sleep 0.3

done < "$QUESTIONS_FILE"

# Calculate statistics
if [ ${#response_times[@]} -gt 0 ]; then
    # Sort response times
    IFS=$'\n' sorted=($(sort -n <<<"${response_times[*]}")); unset IFS

    total_time=0
    for t in "${response_times[@]}"; do
        total_time=$((total_time + t))
    done

    avg_time=$((total_time / ${#response_times[@]}))
    min_time=${sorted[0]}
    max_time=${sorted[-1]}

    # Median
    mid=$((${#sorted[@]} / 2))
    if [ $((${#sorted[@]} % 2)) -eq 0 ]; then
        median=$(( (sorted[mid-1] + sorted[mid]) / 2 ))
    else
        median=${sorted[mid]}
    fi

    # P95
    p95_idx=$(( ${#sorted[@]} * 95 / 100 ))
    p95_time=${sorted[p95_idx]}
else
    avg_time=0
    min_time=0
    max_time=0
    median=0
    p95_time=0
fi

completion_rate=$(echo "scale=1; $completed * 100 / $total" | bc)
answer_rate=$(echo "scale=1; $real_answer_count * 100 / $total" | bc)
clarification_rate=$(echo "scale=1; $clarification_count * 100 / $total" | bc)

# Results Summary
echo ""
echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                    RESULTS SUMMARY                          ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Reliability:${NC}"
echo "  Total questions:     $total"
echo -e "  Completed:           ${GREEN}$completed${NC} (${completion_rate}%)"
echo -e "  Timeouts:            ${RED}$timeout_count${NC}"
echo -e "  Errors:              ${RED}$error_count${NC}"
echo ""
echo -e "${BLUE}Response Quality:${NC}"
echo -e "  Real answers:        ${GREEN}$real_answer_count${NC} (${answer_rate}%)"
echo -e "  Clarifications:      ${YELLOW}$clarification_count${NC} (${clarification_rate}%)"
echo ""
echo -e "${BLUE}Response Time:${NC}"
echo "  Average:             ${avg_time}ms"
echo "  Median:              ${median}ms"
echo "  Min:                 ${min_time}ms"
echo "  Max:                 ${max_time}ms"
echo "  P95:                 ${p95_time}ms"
echo ""

# Category breakdown
echo -e "${BLUE}Category Breakdown:${NC}"
for cat in Error Hardware Package Network Display Boot Perf; do
    cat_total=$(grep -c "|$cat|" "$QUESTIONS_FILE" 2>/dev/null || echo 0)
    cat_answered=$(grep -l "\"category\": \"$cat\"" "$RESULTS_DIR"/*.json 2>/dev/null | xargs grep -l "\"answer_type\": \"answer\"" 2>/dev/null | wc -l || echo 0)
    cat_clarify=$(grep -l "\"category\": \"$cat\"" "$RESULTS_DIR"/*.json 2>/dev/null | xargs grep -l "\"answer_type\": \"clarification\"" 2>/dev/null | wc -l || echo 0)

    if [ "$cat_total" -gt 0 ]; then
        printf "  %-10s answered: %2d/%2d  clarify: %2d\n" "$cat" "$cat_answered" "$cat_total" "$cat_clarify"
    fi
done

# Save summary JSON
cat > "$RESULTS_DIR/summary.json" << EOF
{
  "test_date": "$(date -Iseconds)",
  "test_version": "v3",
  "total_questions": $total,
  "completed": $completed,
  "completion_rate_percent": $completion_rate,
  "timeouts": $timeout_count,
  "errors": $error_count,
  "real_answers": $real_answer_count,
  "clarifications": $clarification_count,
  "answer_rate_percent": $answer_rate,
  "response_time_ms": {
    "average": $avg_time,
    "median": $median,
    "min": $min_time,
    "max": $max_time,
    "p95": $p95_time
  }
}
EOF

echo ""
echo "Completed: $(date)"
echo "Results saved to: $RESULTS_DIR"
echo ""

# Quick analysis of what types of questions got clarification
if [ $clarification_count -gt 0 ]; then
    echo -e "${YELLOW}Questions that got clarification requests (sample):${NC}"
    grep -l "\"answer_type\": \"clarification\"" "$RESULTS_DIR"/*.json 2>/dev/null | head -5 | while read f; do
        q=$(jq -r '.question' "$f" 2>/dev/null || echo "?")
        printf "  - %.60s\n" "$q"
    done
    echo ""
fi

# Performance grade
if [ "$answer_rate" = "0" ]; then
    grade="F"
elif (( $(echo "$answer_rate < 30" | bc -l) )); then
    grade="D"
elif (( $(echo "$answer_rate < 50" | bc -l) )); then
    grade="C"
elif (( $(echo "$answer_rate < 70" | bc -l) )); then
    grade="B"
elif (( $(echo "$answer_rate < 85" | bc -l) )); then
    grade="A"
else
    grade="A+"
fi

echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  OVERALL GRADE: ${GREEN}$grade${NC}  (${answer_rate}% answered, ${median}ms median)"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
