#!/usr/bin/env bash
# Validate Anna's responses against post-install question suite
# Tests response quality, expected commands, warnings, and diagnostic methodology

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_QUESTIONS=0
PASSED=0
FAILED=0
WARNINGS=0

# Configuration
QUESTIONS_FILE="${1:-$PROJECT_ROOT/data/post_install_questions.json}"
MAX_QUESTIONS="${2:-10}"  # Default to 10 questions for quick validation
RESULTS_FILE="${3:-$PROJECT_ROOT/post_install_validation_results.md}"

# Check if annactl is available
if ! command -v annactl &> /dev/null; then
    echo -e "${RED}Error: annactl not found in PATH${NC}"
    echo "Please ensure Anna is installed and annactl is accessible"
    exit 1
fi

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq not found${NC}"
    echo "Please install jq: sudo pacman -S jq"
    exit 1
fi

# Create results file header
cat > "$RESULTS_FILE" << EOF
# Post-Install Question Validation Results

**Date:** $(date '+%Y-%m-%d %H:%M:%S')
**Anna Version:** $(annactl --version 2>&1 || echo "Unknown")
**Questions File:** $QUESTIONS_FILE
**Questions Tested:** $MAX_QUESTIONS

---

## Test Results

EOF

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Anna Post-Install Question Validation Suite             ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Questions file:${NC} $QUESTIONS_FILE"
echo -e "${BLUE}Testing:${NC} Up to $MAX_QUESTIONS questions"
echo -e "${BLUE}Results file:${NC} $RESULTS_FILE"
echo ""

# Extract questions from JSON
QUESTIONS=$(jq -c ".questions[] | select(.id <= $MAX_QUESTIONS)" "$QUESTIONS_FILE")

# Count total questions
TOTAL_QUESTIONS=$(echo "$QUESTIONS" | wc -l)

echo -e "${GREEN}Found $TOTAL_QUESTIONS questions to test${NC}"
echo ""

# Process each question
QUESTION_NUM=0
while IFS= read -r question_json; do
    QUESTION_NUM=$((QUESTION_NUM + 1))

    # Extract question details
    ID=$(echo "$question_json" | jq -r '.id')
    CATEGORY=$(echo "$question_json" | jq -r '.category')
    DIFFICULTY=$(echo "$question_json" | jq -r '.difficulty')
    QUESTION=$(echo "$question_json" | jq -r '.question')
    EXPECTED_COMMANDS=$(echo "$question_json" | jq -r '.expected_commands // [] | join(", ")')
    EXPECTED_TOPICS=$(echo "$question_json" | jq -r '.expected_topics // [] | join(", ")')
    WARNING_REQUIRED=$(echo "$question_json" | jq -r '.warning_required // empty')

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Question $QUESTION_NUM/$TOTAL_QUESTIONS (ID: $ID)${NC}"
    echo -e "${BLUE}Category:${NC} $CATEGORY | ${BLUE}Difficulty:${NC} $DIFFICULTY"
    echo -e "${YELLOW}Q:${NC} $QUESTION"
    echo ""

    # Query Anna (with timeout)
    echo -e "${BLUE}Querying Anna...${NC}"
    RESPONSE=$(timeout 60s annactl "$QUESTION" 2>&1 || echo "ERROR: Query timeout or failed")

    # Write to results file
    cat >> "$RESULTS_FILE" << EOF
### Question $QUESTION_NUM: $QUESTION

**Category:** $CATEGORY | **Difficulty:** $DIFFICULTY

**Anna's Response:**
\`\`\`
$RESPONSE
\`\`\`

EOF

    # Validation checks
    QUESTION_PASSED=true
    VALIDATION_NOTES=""

    # Check 1: Response not empty or error
    if [[ "$RESPONSE" == *"ERROR"* ]] || [[ -z "$RESPONSE" ]]; then
        QUESTION_PASSED=false
        VALIDATION_NOTES+="❌ Response failed or empty\n"
        echo -e "${RED}  ✗ Response failed or empty${NC}"
    else
        echo -e "${GREEN}  ✓ Response received${NC}"
    fi

    # Check 2: Expected commands mentioned
    if [[ -n "$EXPECTED_COMMANDS" ]]; then
        COMMANDS_FOUND=0
        IFS=',' read -ra CMD_ARRAY <<< "$EXPECTED_COMMANDS"
        for cmd in "${CMD_ARRAY[@]}"; do
            cmd=$(echo "$cmd" | xargs)  # Trim whitespace
            if [[ "$RESPONSE" == *"$cmd"* ]]; then
                COMMANDS_FOUND=$((COMMANDS_FOUND + 1))
            fi
        done

        if [[ $COMMANDS_FOUND -gt 0 ]]; then
            echo -e "${GREEN}  ✓ Expected commands found ($COMMANDS_FOUND/${#CMD_ARRAY[@]})${NC}"
            VALIDATION_NOTES+="✓ Expected commands found: $COMMANDS_FOUND/${#CMD_ARRAY[@]}\n"
        else
            echo -e "${YELLOW}  ⚠ No expected commands found${NC}"
            VALIDATION_NOTES+="⚠ Expected commands not found: $EXPECTED_COMMANDS\n"
            WARNINGS=$((WARNINGS + 1))
        fi
    fi

    # Check 3: Expected topics mentioned
    if [[ -n "$EXPECTED_TOPICS" ]]; then
        TOPICS_FOUND=0
        IFS=',' read -ra TOPIC_ARRAY <<< "$EXPECTED_TOPICS"
        for topic in "${TOPIC_ARRAY[@]}"; do
            topic=$(echo "$topic" | xargs)
            topic_lower=$(echo "$topic" | tr '[:upper:]' '[:lower:]')
            response_lower=$(echo "$RESPONSE" | tr '[:upper:]' '[:lower:]')
            if [[ "$response_lower" == *"$topic_lower"* ]]; then
                TOPICS_FOUND=$((TOPICS_FOUND + 1))
            fi
        done

        if [[ $TOPICS_FOUND -gt 0 ]]; then
            echo -e "${GREEN}  ✓ Expected topics found ($TOPICS_FOUND/${#TOPIC_ARRAY[@]})${NC}"
            VALIDATION_NOTES+="✓ Expected topics found: $TOPICS_FOUND/${#TOPIC_ARRAY[@]}\n"
        else
            echo -e "${YELLOW}  ⚠ No expected topics found${NC}"
            VALIDATION_NOTES+="⚠ Expected topics not found: $EXPECTED_TOPICS\n"
            WARNINGS=$((WARNINGS + 1))
        fi
    fi

    # Check 4: Warning present if required
    if [[ -n "$WARNING_REQUIRED" ]]; then
        warning_lower=$(echo "$WARNING_REQUIRED" | tr '[:upper:]' '[:lower:]')
        response_lower=$(echo "$RESPONSE" | tr '[:upper:]' '[:lower:]')

        if [[ "$response_lower" == *"$warning_lower"* ]] || \
           [[ "$RESPONSE" == *"⚠"* ]] || \
           [[ "$RESPONSE" == *"warning"* ]] || \
           [[ "$RESPONSE" == *"Warning"* ]] || \
           [[ "$RESPONSE" == *"NEVER"* ]] || \
           [[ "$RESPONSE" == *"avoid"* ]]; then
            echo -e "${GREEN}  ✓ Warning present${NC}"
            VALIDATION_NOTES+="✓ Warning present: $WARNING_REQUIRED\n"
        else
            QUESTION_PASSED=false
            echo -e "${RED}  ✗ Required warning missing: $WARNING_REQUIRED${NC}"
            VALIDATION_NOTES+="❌ Required warning missing: $WARNING_REQUIRED\n"
        fi
    fi

    # Update counters
    if $QUESTION_PASSED; then
        PASSED=$((PASSED + 1))
        echo -e "${GREEN}  ═══ PASSED ═══${NC}"
        cat >> "$RESULTS_FILE" << EOF
**Validation:** ✅ PASSED

$VALIDATION_NOTES

EOF
    else
        FAILED=$((FAILED + 1))
        echo -e "${RED}  ═══ FAILED ═══${NC}"
        cat >> "$RESULTS_FILE" << EOF
**Validation:** ❌ FAILED

$VALIDATION_NOTES

EOF
    fi

    echo ""

    # Brief pause between questions
    sleep 0.5

done <<< "$QUESTIONS"

# Calculate success rate
SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", ($PASSED / $TOTAL_QUESTIONS) * 100}")

# Final summary
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    FINAL RESULTS                          ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Passed:${NC}   $PASSED / $TOTAL_QUESTIONS questions"
echo -e "${RED}Failed:${NC}   $FAILED / $TOTAL_QUESTIONS questions"
echo -e "${YELLOW}Warnings:${NC} $WARNINGS issues"
echo -e "${BLUE}Success Rate:${NC} $SUCCESS_RATE%"
echo ""

# Append summary to results file
cat >> "$RESULTS_FILE" << EOF

---

## Summary

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Questions** | $TOTAL_QUESTIONS | 100% |
| **Passed** | $PASSED | $SUCCESS_RATE% |
| **Failed** | $FAILED | $(awk "BEGIN {printf \"%.1f\", ($FAILED / $TOTAL_QUESTIONS) * 100}")% |
| **Warnings** | $WARNINGS | - |

### Assessment

EOF

if (( $(echo "$SUCCESS_RATE >= 90" | bc -l) )); then
    echo -e "${GREEN}✅ EXCELLENT: Success rate ≥90%${NC}"
    echo "✅ **EXCELLENT:** Anna is performing at a professional level (≥90% success rate)." >> "$RESULTS_FILE"
elif (( $(echo "$SUCCESS_RATE >= 75" | bc -l) )); then
    echo -e "${GREEN}✅ GOOD: Success rate ≥75%${NC}"
    echo "✅ **GOOD:** Anna is performing well (≥75% success rate). Some improvements may be beneficial." >> "$RESULTS_FILE"
elif (( $(echo "$SUCCESS_RATE >= 60" | bc -l) )); then
    echo -e "${YELLOW}⚠ ACCEPTABLE: Success rate ≥60%${NC}"
    echo "⚠ **ACCEPTABLE:** Anna is functional but needs improvement (≥60% success rate)." >> "$RESULTS_FILE"
else
    echo -e "${RED}❌ NEEDS IMPROVEMENT: Success rate <60%${NC}"
    echo "❌ **NEEDS IMPROVEMENT:** Anna requires significant prompt or training improvements (<60% success rate)." >> "$RESULTS_FILE"
fi

echo ""
echo -e "${BLUE}Results saved to:${NC} $RESULTS_FILE"
echo ""

# Exit with appropriate code
if [[ $FAILED -eq 0 ]]; then
    exit 0
else
    exit 1
fi
