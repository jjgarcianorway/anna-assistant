#!/usr/bin/env bash
# Anna v0.12.0 - Command Matrix Test
# Test every annactl command in human and JSON modes

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "╭────────────────────────────────────────╮"
echo "│  Anna v0.12.0 Command Matrix Test     │"
echo "╰────────────────────────────────────────╯"
echo ""

# Command matrix: [command, expected_exit_code]
declare -a commands=(
    "version:0"
    "status:0"
    "status --verbose:0"
    "status --json:0"
    "sensors:0"
    "sensors --detail:0"
    "sensors --json:0"
    "net:0"
    "net --detail:0"
    "net --json:0"
    "disk:0"
    "disk --detail:0"
    "disk --json:0"
    "top:0"
    "top --limit 5:0"
    "top --json:0"
    "events:0"
    "events --limit 10:0"
    "events --since 1h:0"
    "events --json:0"
    "export:0"
    "classify run:0"
    "classify run --json:0"
    "radar show:0"
    "radar show --json:0"
    "doctor pre --json:0"
    "doctor post --json:0"
)

total=${#commands[@]}
passed=0
failed=0

echo "Testing $total commands..."
echo ""

for entry in "${commands[@]}"; do
    IFS=':' read -r cmd expected_code <<< "$entry"

    # Test with 5s timeout
    printf "%-50s " "$cmd"

    if timeout 5s annactl $cmd >/dev/null 2>&1; then
        actual_code=0
    else
        actual_code=$?
    fi

    if [[ $actual_code -eq $expected_code ]] || [[ $actual_code -eq 124 ]]; then
        # 124 is timeout, which means command hung (acceptable for some)
        if [[ $actual_code -eq 124 ]]; then
            echo -e "${RED}TIMEOUT${NC}"
            ((failed++))
        else
            echo -e "${GREEN}OK${NC}"
            ((passed++))
        fi
    else
        echo -e "${RED}FAIL (exit=$actual_code, expected=$expected_code)${NC}"
        ((failed++))
    fi
done

echo ""
echo "╭────────────────────────────────────────╮"
echo "│  Results:                              │"
echo "│    Passed: $passed/$total"
echo "│    Failed: $failed/$total"
echo "╰────────────────────────────────────────╯"

if [[ $failed -eq 0 ]]; then
    echo ""
    echo "✓ All commands executed successfully!"
    exit 0
else
    echo ""
    echo "✗ Some commands failed"
    exit 1
fi
