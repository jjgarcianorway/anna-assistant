#!/bin/bash
# Quick test with 20 questions

ANNACTL="./target/release/annactl"
RESULTS_DIR="test_results_20"
mkdir -p "$RESULTS_DIR"

total=0
success=0
empty=0
failed=0
wiki_found=0

echo "Testing anna with 20 questions..."
echo "==================================="

while IFS= read -r question; do
    [[ -z "$question" ]] && continue

    ((total++))
    printf "[%2d] %-55s " "$total" "${question:0:52}..."

    # Run with 90 second timeout
    output=$(timeout 90 $ANNACTL "$question" 2>&1)
    exit_code=$?

    # Save output
    echo "$output" > "$RESULTS_DIR/q${total}.txt"

    # Strip ANSI codes for analysis
    clean=$(echo "$output" | sed 's/\x1b\[[0-9;]*m//g')

    # Check for wiki results
    if echo "$clean" | grep -q "found articles"; then
        ((wiki_found++))
    fi

    # Extract answer
    answer=$(echo "$clean" | sed -n '/^ANSWER:/,/iterations)/p' | grep -v "^ANSWER:" | grep -v "iterations)")

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

done < test_20.txt

echo ""
echo "==================================="
echo "RESULTS"
echo "==================================="
echo "Total:       $total"
echo "Success:     $success ($(( success * 100 / total ))%)"
echo "Empty:       $empty ($(( empty * 100 / total ))%)"
echo "Failed:      $failed ($(( failed * 100 / total ))%)"
echo "Wiki hits:   $wiki_found ($(( wiki_found * 100 / total ))%)"
