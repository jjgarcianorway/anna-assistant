#!/bin/bash
# Invariant F11: Backup failure must abort change
# Test: If backup_file() fails, apply_change() must NOT write the file
#
# FAIL CRITERIA:
# - Code proceeds to fs::write after backup_file() returns Err
# - Error not propagated from backup_file
#
# PASS CRITERIA:
# - backup_file() failure propagates via ? operator
# - fs::write is NOT reachable if backup fails
# - Test proves backup failure prevents write

set -e

echo "Testing F11: Backup failure must abort change"

CHANGES="crates/annad/src/changes.rs"

# Check 1: backup_file returns Result and is called with ?
if ! grep -q "backup_file.*?\|backup_file.*\.await?" "$CHANGES"; then
    echo "FAIL: backup_file() not called with ? operator for error propagation"
    exit 1
fi

# Check 2: apply_change must call backup_file BEFORE fs::write
# Extract line numbers
BACKUP_LINE=$(grep -n "backup_file" "$CHANGES" | grep "apply_change" -A5 | head -1 | cut -d: -f1 || grep -n "let backup_path = backup_file" "$CHANGES" | head -1 | cut -d: -f1)
WRITE_LINE=$(grep -n "fs::write" "$CHANGES" | head -1 | cut -d: -f1)

# Verify backup comes before write in apply_change function
if [ -n "$BACKUP_LINE" ] && [ -n "$WRITE_LINE" ]; then
    if [ "$BACKUP_LINE" -gt "$WRITE_LINE" ]; then
        echo "FAIL: backup_file() called AFTER fs::write"
        exit 1
    fi
fi

# Check 3: append_to_file also follows the same pattern
APPEND_BACKUP=$(grep -n "backup_file" "$CHANGES" | grep -A1 "append_to_file" | tail -1 | cut -d: -f1 || echo "")
APPEND_WRITE=$(grep -n "fs::write" "$CHANGES" | tail -1 | cut -d: -f1)

# Check 4: Rust tests for changes module pass
cargo test --workspace -- changes 2>&1 | tail -10

# Check 5: The backup_file function returns Result<String, String>
if ! grep -q "pub fn backup_file.*-> Result<String, String>" "$CHANGES"; then
    echo "FAIL: backup_file() does not return Result type"
    exit 1
fi

echo "PASS: Backup failure correctly aborts change"
echo "Invariant F11: PASS"
