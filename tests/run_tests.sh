#!/bin/bash
# Anna Test Suite Runner
# Usage: ./run_tests.sh [start_id] [end_id]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANNA_BIN="${SCRIPT_DIR}/../target/release/annactl"
QUESTIONS_FILE="${SCRIPT_DIR}/test_questions.json"
RESULTS_DIR="${SCRIPT_DIR}/results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="${RESULTS_DIR}/test_results_${TIMESTAMP}.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
START_ID=${1:-1}
END_ID=${2:-100}

# Create results directory
mkdir -p "$RESULTS_DIR"

# Check prerequisites
if [ ! -f "$ANNA_BIN" ]; then
    echo -e "${RED}Error: annactl not found at $ANNA_BIN${NC}"
    echo "Please run: cargo build --release --workspace"
    exit 1
fi

if [ ! -f "$QUESTIONS_FILE" ]; then
    echo -e "${RED}Error: questions file not found at $QUESTIONS_FILE${NC}"
    exit 1
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}Warning: jq not installed. Install with: pacman -S jq${NC}"
    exit 1
fi

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         Anna AI Assistant - Test Suite Runner              ║${NC}"
echo -e "${BLUE}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${BLUE}║ Questions: $START_ID - $END_ID${NC}"
echo -e "${BLUE}║ Results: $RESULTS_FILE${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Initialize results JSON
echo '{"metadata": {"started_at": "'$(date -Iseconds)'", "system": "'$(uname -r)'", "distro": "'$(cat /etc/os-release | grep ^PRETTY_NAME | cut -d'"' -f2)'"}, "results": []}' > "$RESULTS_FILE"

# Track statistics
TOTAL=0
SUCCESS=0
FAILED=0
ERRORS=0

# Read and process each question
while IFS= read -r question_data; do
    id=$(echo "$question_data" | jq -r '.id')
    category=$(echo "$question_data" | jq -r '.category')
    question=$(echo "$question_data" | jq -r '.question')

    # Skip if outside range
    if [ "$id" -lt "$START_ID" ] || [ "$id" -gt "$END_ID" ]; then
        continue
    fi

    TOTAL=$((TOTAL + 1))

    echo -e "${YELLOW}[$id/100] ${category}${NC}"
    echo -e "  Q: ${question}"

    # Run Anna and capture output
    START_TIME=$(date +%s)

    # Capture both stdout and stderr, with timeout
    ANSWER=""
    if OUTPUT=$(timeout 120 "$ANNA_BIN" "$question" 2>&1); then
        # Strip all ANSI codes
        CLEAN_OUTPUT=$(echo "$OUTPUT" | sed 's/\x1b\[[0-9;]*m//g')
        # Extract answer: get line starting with ANSWER: and remove prefix
        ANSWER_LINE=$(echo "$CLEAN_OUTPUT" | grep "^ANSWER:" | sed 's/^ANSWER: *//')
        if [ -n "$ANSWER_LINE" ]; then
            ANSWER="$ANSWER_LINE"
        else
            # Alternative: look for content after last "ANSWER:" until next section
            ANSWER=$(echo "$CLEAN_OUTPUT" | awk '/ANSWER:/{p=1;sub(/.*ANSWER: */,"");print;next} p && /^(═|^\()/{p=0} p{print}' | head -20 | tr '\n' ' ' | sed 's/  */ /g' | xargs)
        fi
        STATUS="success"
        SUCCESS=$((SUCCESS + 1))
        echo -e "  ${GREEN}✓ Got answer${NC}"
        echo -e "  ${BLUE}A: ${ANSWER:0:100}...${NC}"
    else
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 124 ]; then
            ANSWER="TIMEOUT"
            STATUS="timeout"
            ERRORS=$((ERRORS + 1))
            echo -e "  ${RED}✗ Timeout${NC}"
        else
            ANSWER="ERROR: $OUTPUT"
            STATUS="error"
            FAILED=$((FAILED + 1))
            echo -e "  ${RED}✗ Error (exit code $EXIT_CODE)${NC}"
        fi
    fi

    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))

    # Escape special characters for JSON
    ANSWER_ESCAPED=$(echo "$ANSWER" | jq -Rs '.')

    # Append result to JSON file
    jq --argjson id "$id" \
       --arg category "$category" \
       --arg question "$question" \
       --argjson answer "$ANSWER_ESCAPED" \
       --arg status "$STATUS" \
       --arg duration "$DURATION" \
       '.results += [{"id": $id, "category": $category, "question": $question, "answer": $answer, "status": $status, "duration_secs": ($duration | tonumber)}]' \
       "$RESULTS_FILE" > "${RESULTS_FILE}.tmp" && mv "${RESULTS_FILE}.tmp" "$RESULTS_FILE"

    # Brief pause to avoid overwhelming the daemon
    sleep 1

done < <(jq -c '.questions[]' "$QUESTIONS_FILE")

# Add summary to results
jq --argjson total "$TOTAL" \
   --argjson success "$SUCCESS" \
   --argjson failed "$FAILED" \
   --argjson errors "$ERRORS" \
   '.metadata.completed_at = "'$(date -Iseconds)'" | .metadata.total = $total | .metadata.success = $success | .metadata.failed = $failed | .metadata.errors = $errors' \
   "$RESULTS_FILE" > "${RESULTS_FILE}.tmp" && mv "${RESULTS_FILE}.tmp" "$RESULTS_FILE"

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                     Test Results Summary                    ║${NC}"
echo -e "${BLUE}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${GREEN}║ Success: $SUCCESS${NC}"
echo -e "${RED}║ Failed:  $FAILED${NC}"
echo -e "${YELLOW}║ Errors:  $ERRORS${NC}"
echo -e "${BLUE}║ Total:   $TOTAL${NC}"
echo -e "${BLUE}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${BLUE}║ Results saved to: $RESULTS_FILE${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
