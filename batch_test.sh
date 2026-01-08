#!/bin/bash
# Batch test anna with questions

ANNACTL="./target/release/annactl"
RESULTS_DIR="test_results"
mkdir -p "$RESULTS_DIR"

total=0
success=0
empty=0
failed=0
wiki_found=0

echo "Testing anna with 100 questions..."
echo "==================================="

while IFS= read -r question; do
    [[ "$question" =~ ^#.*$ ]] && continue
    [[ -z "$question" ]] && continue

    ((total++))
    printf "[%3d] %-60s " "$total" "${question:0:57}..."

    # Run with timeout
    output=$(timeout 60 $ANNACTL "$question" 2>&1)
    exit_code=$?

    # Save output
    echo "$output" > "$RESULTS_DIR/q${total}.txt"

    # Check for wiki results
    if echo "$output" | grep -q "WIKI → ANNA: found articles"; then
        ((wiki_found++))
    fi

    # Extract answer - strip ANSI codes and check for content
    clean_output=$(echo "$output" | sed 's/\x1b\[[0-9;]*m//g')
    answer=$(echo "$clean_output" | grep -A 1000 "ANSWER:" | tail -n +2 | grep -v "iterations)" | head -10)

    if [[ $exit_code -eq 124 ]]; then
        ((failed++))
        echo "TIMEOUT"
    elif [[ $exit_code -ne 0 ]]; then
        ((failed++))
        echo "FAILED"
    elif [[ -z "$(echo "$answer" | tr -d '[:space:]')" ]]; then
        ((empty++))
        echo "EMPTY"
    else
        ((success++))
        echo "OK"
    fi

done < test_questions.txt

echo ""
echo "==================================="
echo "RESULTS SUMMARY"
echo "==================================="
echo "Total questions:   $total"
echo "Successful:        $success ($(( success * 100 / total ))%)"
echo "Empty answers:     $empty ($(( empty * 100 / total ))%)"
echo "Failed/Timeout:    $failed ($(( failed * 100 / total ))%)"
echo "Wiki hits:         $wiki_found ($(( wiki_found * 100 / total ))%)"
echo ""

# Save summary
cat > "$RESULTS_DIR/summary.txt" << EOF
Test Run: $(date)
=================================
Total questions:   $total
Successful:        $success ($(( success * 100 / total ))%)
Empty answers:     $empty ($(( empty * 100 / total ))%)
Failed/Timeout:    $failed ($(( failed * 100 / total ))%)
Wiki hits:         $wiki_found ($(( wiki_found * 100 / total ))%)
EOF
