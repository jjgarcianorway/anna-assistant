#!/bin/bash
# Invariant 2: Anna must never write to user home directories
# Test: No dirs::home_dir() usage in write paths, ~/ and $HOME paths rejected

set -e

echo "Testing Invariant 2: No user home directory writes"

# Check for dirs::home_dir usage in production code (excluding tests and comments)
VIOLATIONS=$(grep -rn "dirs::home_dir" crates/ --include="*.rs" | grep -v "^[^:]*:[0-9]*:\s*//" | grep -v "#\[test\]" | grep -v "_test" || true)

if [ -n "$VIOLATIONS" ]; then
    echo "FAIL: Found dirs::home_dir usage:"
    echo "$VIOLATIONS"
    exit 1
fi

# Verify changes.rs rejects ~/ and $HOME paths
if ! grep -q 'starts_with("~/")' crates/annad/src/changes.rs; then
    echo "FAIL: changes.rs does not reject ~/ paths"
    exit 1
fi

if ! grep -q 'contains("\$HOME")' crates/annad/src/changes.rs; then
    echo "FAIL: changes.rs does not reject \$HOME paths"
    exit 1
fi

# Verify vim.rs and shell.rs are disabled
if grep -q "dirs::" crates/annad/src/recipes/vim.rs; then
    echo "FAIL: vim.rs still uses dirs::"
    exit 1
fi

if grep -q "dirs::" crates/annad/src/recipes/shell.rs; then
    echo "FAIL: shell.rs still uses dirs::"
    exit 1
fi

echo "PASS: No user home directory write paths found"
echo "Invariant 2: PASS"
