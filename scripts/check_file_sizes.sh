#!/bin/bash
# v6.38.0: File size analysis tool
# Scans crates/**/*.rs and reports files exceeding size limits
#
# Targets:
#   Soft limit: 400 lines
#   Hard limit: 500 lines
#   Whitelist: Documented exceptions

set -euo pipefail

# Color codes
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

SOFT_LIMIT=400
HARD_LIMIT=500

# Whitelist: Files allowed to exceed hard limit (with justification)
# Format: "path:reason"
declare -a WHITELIST=(
    # Add exceptions here if truly necessary
    # Example: "crates/anna_common/src/legacy_compatibility.rs:Complex compatibility layer"
)

echo "=========================================="
echo "Anna Assistant - File Size Analysis"
echo "=========================================="
echo ""
echo "Limits:"
echo "  Soft target: ${SOFT_LIMIT} lines"
echo "  Hard cap:    ${HARD_LIMIT} lines"
echo ""

# Find all .rs files and count lines
declare -a ALL_FILES
declare -a VIOLATORS
declare -a WARNINGS

while IFS= read -r file; do
    lines=$(wc -l < "$file")
    ALL_FILES+=("$lines:$file")

    # Check if whitelisted
    is_whitelisted=false
    for entry in "${WHITELIST[@]}"; do
        path="${entry%%:*}"
        if [[ "$file" == "$path" ]]; then
            is_whitelisted=true
            break
        fi
    done

    if [[ $lines -gt $HARD_LIMIT ]] && [[ "$is_whitelisted" == "false" ]]; then
        VIOLATORS+=("$lines:$file")
    elif [[ $lines -gt $SOFT_LIMIT ]]; then
        WARNINGS+=("$lines:$file")
    fi
done < <(find crates -name "*.rs" -type f | sort)

# Sort by line count (descending)
IFS=$'\n' SORTED_FILES=($(sort -rn <<<"${ALL_FILES[*]}"))
unset IFS

echo "=========================================="
echo "Top 20 Largest Files:"
echo "=========================================="
count=0
for entry in "${SORTED_FILES[@]}"; do
    if [[ $count -ge 20 ]]; then
        break
    fi
    lines="${entry%%:*}"
    file="${entry#*:}"

    if [[ $lines -gt $HARD_LIMIT ]]; then
        printf "${RED}%5d${NC}  %s\n" "$lines" "$file"
    elif [[ $lines -gt $SOFT_LIMIT ]]; then
        printf "${YELLOW}%5d${NC}  %s\n" "$lines" "$file"
    else
        printf "${GREEN}%5d${NC}  %s\n" "$lines" "$file"
    fi
    ((count++))
done

echo ""
echo "=========================================="
echo "Summary:"
echo "=========================================="
echo "Total .rs files: ${#ALL_FILES[@]}"
echo "Files > ${SOFT_LIMIT} lines: ${#WARNINGS[@]}"
echo "Files > ${HARD_LIMIT} lines (VIOLATIONS): ${#VIOLATORS[@]}"
echo ""

if [[ ${#VIOLATORS[@]} -gt 0 ]]; then
    echo "${RED}FAIL: The following files exceed the hard limit (${HARD_LIMIT} lines):${NC}"
    for entry in "${VIOLATORS[@]}"; do
        lines="${entry%%:*}"
        file="${entry#*:}"
        echo "  ${lines} lines: ${file}"
    done
    echo ""
    echo "Action required: Refactor these files into smaller submodules."
    exit 1
else
    echo "${GREEN}PASS: No files exceed the hard limit.${NC}"
    if [[ ${#WARNINGS[@]} -gt 0 ]]; then
        echo "${YELLOW}Note: ${#WARNINGS[@]} files exceed soft limit but are under hard cap.${NC}"
    fi
    exit 0
fi
