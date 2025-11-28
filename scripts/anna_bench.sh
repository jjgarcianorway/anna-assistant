#!/bin/bash
# Anna Benchmark Script v0.82.0
#
# Runs canonical QA questions multiple times and produces:
# - Raw per-run results (.bench/anna_qa_raw_<timestamp>.jsonl)
# - Aggregated summary JSON (.bench/anna_bench_summary_<timestamp>.json)
# - Human-readable report (QA/ANNA_BENCH_REPORT.md)
#
# Usage:
#   ANNA_QA_MODE=1 ./scripts/anna_bench.sh [--profile razorback] [--runs N]
#
# Environment:
#   ANNA_QA_MODE=1  - Required. Enables JSON output from annactl

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BENCH_DIR="${PROJECT_DIR}/.bench"
QA_DIR="${PROJECT_DIR}/QA"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Defaults
PROFILE="razorback"
RUNS_PER_QUESTION=3

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        --runs)
            RUNS_PER_QUESTION="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: ANNA_QA_MODE=1 ./scripts/anna_bench.sh [--profile razorback] [--runs N]"
            echo ""
            echo "Options:"
            echo "  --profile NAME   Profile name (default: razorback)"
            echo "  --runs N         Runs per question (default: 3)"
            exit 0
            ;;
        *)
            echo -e "${RED}[ERROR]${NC} Unknown option: $1"
            exit 1
            ;;
    esac
done

# Ensure QA mode is enabled
if [[ -z "$ANNA_QA_MODE" ]]; then
    echo -e "${RED}[ERROR]${NC} ANNA_QA_MODE must be set to 1"
    echo "Usage: ANNA_QA_MODE=1 ./scripts/anna_bench.sh [--profile razorback] [--runs N]"
    exit 1
fi

# Create directories
mkdir -p "$BENCH_DIR" "$QA_DIR"

# Output files
RAW_FILE="${BENCH_DIR}/anna_qa_raw_${TIMESTAMP}.jsonl"
SUMMARY_FILE="${BENCH_DIR}/anna_bench_summary_${TIMESTAMP}.json"
REPORT_FILE="${QA_DIR}/ANNA_BENCH_REPORT.md"

# Get Anna version
ANNA_VERSION=$(annactl --version 2>/dev/null | head -1 || echo "unknown")

# ==============================================================================
# Canonical Questions (with IDs)
# ==============================================================================
# Format: "QXXX|question_text|category"
declare -a CANONICAL_QUESTIONS=(
    "Q001|How many CPU threads do I have?|cpu"
    "Q002|How much RAM do I have?|mem"
    "Q003|What CPU model do I have?|cpu"
    "Q004|How many physical cores do I have?|cpu"
    "Q005|How much memory is available right now?|mem"
    "Q006|Is Anna healthy?|self_health"
    "Q007|What Steam games do I have installed?|unsupported"
    "Q008|What DNS servers am I using?|unsupported"
)

# ==============================================================================
# Run a single question and capture QA JSON
# ==============================================================================
run_question() {
    local question_id="$1"
    local question_text="$2"
    local category="$3"
    local run_num="$4"

    local start_time=$(date +%s%N)
    local result
    result=$(annactl "$question_text" 2>/dev/null) || result=""
    local end_time=$(date +%s%N)
    local wall_ms=$(( (end_time - start_time) / 1000000 ))

    # Check if we got valid JSON
    if echo "$result" | jq -e . >/dev/null 2>&1; then
        # Add metadata and write to raw file
        echo "$result" | jq -c ". + {
            \"question_id\": \"$question_id\",
            \"question_text\": \"$question_text\",
            \"category\": \"$category\",
            \"run\": $run_num,
            \"wall_ms\": $wall_ms,
            \"timestamp_utc\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
        }" >> "$RAW_FILE"

        # Extract reliability for display
        local reliability=$(echo "$result" | jq -r '.reliability_label // "unknown"')
        local score=$(echo "$result" | jq -r '.score_overall // 0')

        case "$reliability" in
            Green)  echo -e "    Run $run_num: ${GREEN}$reliability${NC} (${score})" ;;
            Yellow) echo -e "    Run $run_num: ${YELLOW}$reliability${NC} (${score})" ;;
            Red)    echo -e "    Run $run_num: ${RED}$reliability${NC} (${score})" ;;
            *)      echo -e "    Run $run_num: ${RED}ERROR${NC}" ;;
        esac
    else
        # Write error record
        echo "{
            \"question_id\": \"$question_id\",
            \"question_text\": \"$question_text\",
            \"category\": \"$category\",
            \"run\": $run_num,
            \"wall_ms\": $wall_ms,
            \"timestamp_utc\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
            \"headline\": \"Failed to get response\",
            \"details\": [\"No valid JSON returned from annactl\"],
            \"evidence\": [],
            \"reliability_label\": \"Red\",
            \"score_overall\": 0.0,
            \"junior_ms\": 0,
            \"senior_ms\": 0,
            \"iterations\": 0,
            \"probes_used\": [],
            \"error_kind\": \"no_response\",
            \"dialog_trace\": {\"junior_plan_probes\": [], \"junior_had_draft\": false, \"senior_verdict\": \"unknown\"}
        }" | jq -c >> "$RAW_FILE"
        echo -e "    Run $run_num: ${RED}ERROR${NC} (no response)"
    fi
}

# ==============================================================================
# Aggregate results for a question
# ==============================================================================
aggregate_question() {
    local question_id="$1"
    local question_text="$2"

    # Extract all runs for this question
    local runs_json=$(jq -s "[.[] | select(.question_id == \"$question_id\")]" "$RAW_FILE")
    local total_runs=$(echo "$runs_json" | jq 'length')

    # Calculate statistics
    local passes=$(echo "$runs_json" | jq '[.[] | select(.score_overall >= 0.90 and .reliability_label == "Green" and (.error_kind == null or .error_kind == ""))] | length')
    local failures=$((total_runs - passes))

    local avg_score=$(echo "$runs_json" | jq '[.[].score_overall] | add / length')
    local min_score=$(echo "$runs_json" | jq '[.[].score_overall] | min')
    local max_score=$(echo "$runs_json" | jq '[.[].score_overall] | max')

    local avg_latency=$(echo "$runs_json" | jq '[.[] | (.junior_ms + .senior_ms)] | add / length | floor')
    local min_latency=$(echo "$runs_json" | jq '[.[] | (.junior_ms + .senior_ms)] | min')
    local max_latency=$(echo "$runs_json" | jq '[.[] | (.junior_ms + .senior_ms)] | max')

    local avg_iterations=$(echo "$runs_json" | jq '[.[].iterations] | add / length')

    # Most common senior verdict
    local most_common_verdict=$(echo "$runs_json" | jq -r '[.[].dialog_trace.senior_verdict] | group_by(.) | sort_by(-length) | .[0][0] // "unknown"')

    # Most common probes (comma-joined)
    local most_common_probes=$(echo "$runs_json" | jq -r '[.[].probes_used | sort | join(",")] | group_by(.) | sort_by(-length) | .[0][0] // ""')

    # Output JSON object for this question
    cat << EOF
    "$question_id": {
      "question_text": "$question_text",
      "runs": $total_runs,
      "passes": $passes,
      "failures": $failures,
      "avg_score": $avg_score,
      "min_score": $min_score,
      "max_score": $max_score,
      "avg_latency_ms": $avg_latency,
      "min_latency_ms": $min_latency,
      "max_latency_ms": $max_latency,
      "avg_iterations": $avg_iterations,
      "most_common_senior_verdict": "$most_common_verdict",
      "most_common_probes": $(echo "$most_common_probes" | jq -R 'split(",") | if . == [""] then [] else . end')
    }
EOF
}

# ==============================================================================
# Generate markdown report
# ==============================================================================
generate_report() {
    cat > "$REPORT_FILE" << EOF
# Anna Benchmark Report

**Generated:** $(date -u +%Y-%m-%dT%H:%M:%SZ)
**Anna Version:** $ANNA_VERSION
**Profile:** $PROFILE
**Runs per Question:** $RUNS_PER_QUESTION

## Summary

| ID | Question | Score (avg/min/max) | Reliability | Latency ms (avg/min/max) | Iter | Pass/Fail |
|----|----------|---------------------|-------------|--------------------------|------|-----------|
EOF

    # Add rows for each question
    for q in "${CANONICAL_QUESTIONS[@]}"; do
        IFS='|' read -r qid qtext qcat <<< "$q"

        local runs_json=$(jq -s "[.[] | select(.question_id == \"$qid\")]" "$RAW_FILE")
        local total_runs=$(echo "$runs_json" | jq 'length')

        if [[ "$total_runs" -eq 0 ]]; then
            continue
        fi

        local passes=$(echo "$runs_json" | jq '[.[] | select(.score_overall >= 0.90 and .reliability_label == "Green" and (.error_kind == null or .error_kind == ""))] | length')
        local failures=$((total_runs - passes))

        local avg_score=$(echo "$runs_json" | jq '[.[].score_overall] | add / length | . * 100 | floor / 100')
        local min_score=$(echo "$runs_json" | jq '[.[].score_overall] | min | . * 100 | floor / 100')
        local max_score=$(echo "$runs_json" | jq '[.[].score_overall] | max | . * 100 | floor / 100')

        local avg_latency=$(echo "$runs_json" | jq '[.[] | (.junior_ms + .senior_ms)] | add / length | floor')
        local min_latency=$(echo "$runs_json" | jq '[.[] | (.junior_ms + .senior_ms)] | min')
        local max_latency=$(echo "$runs_json" | jq '[.[] | (.junior_ms + .senior_ms)] | max')

        local avg_iterations=$(echo "$runs_json" | jq '[.[].iterations] | add / length | . * 10 | floor / 10')

        # Determine reliability indicator
        local reliability_ind
        if [[ "$failures" -eq 0 ]] && (( $(echo "$avg_score >= 0.90" | bc -l) )); then
            reliability_ind="Green"
        elif (( $(echo "$avg_score >= 0.70" | bc -l) )); then
            reliability_ind="Yellow"
        else
            reliability_ind="Red"
        fi

        # Truncate question for table
        local qtext_short="${qtext:0:35}"
        [[ "${#qtext}" -gt 35 ]] && qtext_short="${qtext_short}..."

        echo "| $qid | $qtext_short | $avg_score / $min_score / $max_score | $reliability_ind | $avg_latency / $min_latency / $max_latency | $avg_iterations | $passes / $failures |" >> "$REPORT_FILE"
    done

    # Summary stats
    local total_questions=$(echo "${CANONICAL_QUESTIONS[@]}" | tr ' ' '\n' | wc -l)
    local green_questions=$(jq -s 'group_by(.question_id) | map(select(all(.score_overall >= 0.90 and .reliability_label == "Green"))) | length' "$RAW_FILE")
    local slow_questions=$(jq -s 'group_by(.question_id) | map(select(([.[] | .junior_ms + .senior_ms] | add / length) > 30000)) | map(.[0].question_id) | .[]' "$RAW_FILE" 2>/dev/null || echo "")
    local flaky_questions=$(jq -s 'group_by(.question_id) | map(select(any(.score_overall < 0.90 or .reliability_label != "Green"))) | map(.[0].question_id) | .[]' "$RAW_FILE" 2>/dev/null || echo "")

    cat >> "$REPORT_FILE" << EOF

## Analysis

- **Total Questions:** $total_questions
- **Fully Green:** $green_questions
- **Slow Questions (>30s):** $(echo "$slow_questions" | tr '\n' ', ' | sed 's/,$//')
- **Flaky Questions:** $(echo "$flaky_questions" | tr '\n' ', ' | sed 's/,$//')

## Files

- Raw results: \`.bench/anna_qa_raw_${TIMESTAMP}.jsonl\`
- Summary JSON: \`.bench/anna_bench_summary_${TIMESTAMP}.json\`
EOF

    echo -e "\nReport written to: ${CYAN}$REPORT_FILE${NC}"
}

# ==============================================================================
# Main
# ==============================================================================
echo -e "${CYAN}${BOLD}Anna Benchmark v0.82.0${NC}"
echo -e "Profile: $PROFILE"
echo -e "Runs per question: $RUNS_PER_QUESTION"
echo -e "Output: $RAW_FILE"
echo ""

# Run each question multiple times
for q in "${CANONICAL_QUESTIONS[@]}"; do
    IFS='|' read -r qid qtext qcat <<< "$q"
    echo -e "${CYAN}[$qid]${NC} $qtext"

    for ((run=1; run<=RUNS_PER_QUESTION; run++)); do
        run_question "$qid" "$qtext" "$qcat" "$run"
        sleep 1  # Brief pause between runs
    done
    echo ""
done

# Generate summary JSON
echo -e "${CYAN}Generating summary...${NC}"

# Build questions object
questions_json="{"
first=true
for q in "${CANONICAL_QUESTIONS[@]}"; do
    IFS='|' read -r qid qtext qcat <<< "$q"
    if [[ "$first" == "true" ]]; then
        first=false
    else
        questions_json+=","
    fi
    questions_json+=$(aggregate_question "$qid" "$qtext")
done
questions_json+="}"

# Write full summary JSON
cat > "$SUMMARY_FILE" << EOF
{
  "meta": {
    "timestamp_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "profile": "$PROFILE",
    "runs_per_question": $RUNS_PER_QUESTION,
    "anna_version": "$ANNA_VERSION"
  },
  "questions": $questions_json
}
EOF

echo -e "Summary written to: ${CYAN}$SUMMARY_FILE${NC}"

# Generate markdown report
generate_report

# Print console summary
echo ""
echo -e "${CYAN}${BOLD}===== Benchmark Summary =====${NC}"
total_runs=$(wc -l < "$RAW_FILE")
green_runs=$(grep -c '"reliability_label":"Green"' "$RAW_FILE" || echo 0)
red_runs=$(grep -c '"reliability_label":"Red"' "$RAW_FILE" || echo 0)
yellow_runs=$((total_runs - green_runs - red_runs))

echo -e "Total runs: $total_runs"
echo -e "${GREEN}Green:${NC} $green_runs"
echo -e "${YELLOW}Yellow:${NC} $yellow_runs"
echo -e "${RED}Red:${NC} $red_runs"
