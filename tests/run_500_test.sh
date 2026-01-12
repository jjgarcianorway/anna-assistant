#!/bin/bash
# 500 Questions Test: Anna Pattern Matching Analysis
# Tests Anna's pattern coverage and response time

ANNACTL="./target/release/annactl"
QUESTIONS_FILE="./tests/500_questions.txt"
RESULTS_FILE="./tests/500_results.txt"
SUMMARY_FILE="./tests/500_summary.md"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Counters
total=0
pattern_matches=0
needs_llm=0
timeouts=0
errors=0

# Time tracking
total_time_ms=0
pattern_times=()
llm_times=()

echo "=========================================="
echo "  Anna 500 Questions Test"
echo "=========================================="
echo ""

# Check if annactl exists
if [ ! -f "$ANNACTL" ]; then
    echo -e "${RED}Error: annactl not found at $ANNACTL${NC}"
    echo "Run: cargo build --release --workspace"
    exit 1
fi

# Check if questions file exists
if [ ! -f "$QUESTIONS_FILE" ]; then
    echo -e "${RED}Error: Questions file not found at $QUESTIONS_FILE${NC}"
    exit 1
fi

# Clear results file
> "$RESULTS_FILE"

echo "Testing questions from: $QUESTIONS_FILE"
echo "Results will be saved to: $RESULTS_FILE"
echo ""
echo "Progress:"

# Read questions, skip comments and empty lines
while IFS= read -r line || [ -n "$line" ]; do
    # Skip comments and empty lines
    [[ "$line" =~ ^#.*$ ]] && continue
    [[ -z "$line" ]] && continue

    ((total++))

    # Show progress every 10 questions
    if ((total % 10 == 0)); then
        echo -ne "\r  Tested: $total questions..."
    fi

    # Measure time and test the question
    start_time=$(date +%s%3N)

    # Run annactl with timeout and capture output
    # Using --pattern-only flag if available, otherwise just run normally
    result=$($ANNACTL --dry-run "$line" 2>&1 | head -20)
    exit_code=$?

    end_time=$(date +%s%3N)
    elapsed=$((end_time - start_time))
    total_time_ms=$((total_time_ms + elapsed))

    # Analyze result
    if [ $exit_code -ne 0 ]; then
        ((errors++))
        status="ERROR"
    elif echo "$result" | grep -qi "pattern match\|instant answer\|suggested commands"; then
        ((pattern_matches++))
        pattern_times+=($elapsed)
        status="PATTERN"
    elif echo "$result" | grep -qi "needs clarification\|could you\|please specify"; then
        ((needs_llm++))
        llm_times+=($elapsed)
        status="CLARIFY"
    elif [ $elapsed -gt 5000 ]; then
        ((timeouts++))
        status="SLOW"
    else
        ((needs_llm++))
        llm_times+=($elapsed)
        status="LLM"
    fi

    # Log result
    echo "[$status] ${elapsed}ms: $line" >> "$RESULTS_FILE"

done < "$QUESTIONS_FILE"

echo -e "\r  Tested: $total questions... Done!     "
echo ""

# Calculate statistics
pattern_pct=$((pattern_matches * 100 / total))
llm_pct=$((needs_llm * 100 / total))
timeout_pct=$((timeouts * 100 / total))
error_pct=$((errors * 100 / total))
avg_time=$((total_time_ms / total))

# Calculate pattern average time
if [ ${#pattern_times[@]} -gt 0 ]; then
    pattern_total=0
    for t in "${pattern_times[@]}"; do
        pattern_total=$((pattern_total + t))
    done
    pattern_avg=$((pattern_total / ${#pattern_times[@]}))
else
    pattern_avg=0
fi

# Calculate LLM average time
if [ ${#llm_times[@]} -gt 0 ]; then
    llm_total=0
    for t in "${llm_times[@]}"; do
        llm_total=$((llm_total + t))
    done
    llm_avg=$((llm_total / ${#llm_times[@]}))
else
    llm_avg=0
fi

# Generate summary
cat > "$SUMMARY_FILE" << EOF
# Anna 500 Questions Test Results
Date: $(date)
Version: $(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

## Summary

| Metric | Value |
|--------|-------|
| Total Questions | $total |
| Pattern Matches | $pattern_matches ($pattern_pct%) |
| Needs LLM/Clarification | $needs_llm ($llm_pct%) |
| Timeouts (>5s) | $timeouts ($timeout_pct%) |
| Errors | $errors ($error_pct%) |

## Response Times

| Category | Average Time |
|----------|--------------|
| Pattern Match | ${pattern_avg}ms |
| LLM/Clarification | ${llm_avg}ms |
| Overall Average | ${avg_time}ms |

## Analysis

### Pattern Coverage Rate: ${pattern_pct}%

EOF

if [ $pattern_pct -ge 80 ]; then
    echo "**Excellent!** Anna provides instant answers for most questions." >> "$SUMMARY_FILE"
elif [ $pattern_pct -ge 60 ]; then
    echo "**Good.** Anna handles majority of questions with patterns." >> "$SUMMARY_FILE"
elif [ $pattern_pct -ge 40 ]; then
    echo "**Moderate.** Room for improvement in pattern coverage." >> "$SUMMARY_FILE"
else
    echo "**Needs Work.** Many questions require LLM processing." >> "$SUMMARY_FILE"
fi

cat >> "$SUMMARY_FILE" << EOF

### Speed Comparison

- **Pattern responses**: ~${pattern_avg}ms (instant)
- **LLM responses**: ~${llm_avg}ms (requires API call)
- **Speed advantage**: Pattern matching is ~$((llm_avg / (pattern_avg + 1)))x faster

## Category Breakdown

EOF

# Count by category from results
echo "### Questions by Response Type" >> "$SUMMARY_FILE"
echo "" >> "$SUMMARY_FILE"
echo '```' >> "$SUMMARY_FILE"
echo "PATTERN matches: $(grep -c '^\[PATTERN\]' "$RESULTS_FILE")" >> "$SUMMARY_FILE"
echo "CLARIFY needed:  $(grep -c '^\[CLARIFY\]' "$RESULTS_FILE")" >> "$SUMMARY_FILE"
echo "LLM needed:      $(grep -c '^\[LLM\]' "$RESULTS_FILE")" >> "$SUMMARY_FILE"
echo "SLOW (>5s):      $(grep -c '^\[SLOW\]' "$RESULTS_FILE")" >> "$SUMMARY_FILE"
echo "ERRORS:          $(grep -c '^\[ERROR\]' "$RESULTS_FILE")" >> "$SUMMARY_FILE"
echo '```' >> "$SUMMARY_FILE"

# Print summary to console
echo "=========================================="
echo "  RESULTS SUMMARY"
echo "=========================================="
echo ""
echo -e "Total Questions:     ${GREEN}$total${NC}"
echo -e "Pattern Matches:     ${GREEN}$pattern_matches${NC} ($pattern_pct%)"
echo -e "Needs LLM:           ${YELLOW}$needs_llm${NC} ($llm_pct%)"
echo -e "Timeouts:            ${RED}$timeouts${NC} ($timeout_pct%)"
echo -e "Errors:              ${RED}$errors${NC} ($error_pct%)"
echo ""
echo "Average Response Times:"
echo -e "  Pattern:  ${GREEN}${pattern_avg}ms${NC}"
echo -e "  LLM:      ${YELLOW}${llm_avg}ms${NC}"
echo -e "  Overall:  ${avg_time}ms"
echo ""
echo "Results saved to: $RESULTS_FILE"
echo "Summary saved to: $SUMMARY_FILE"
