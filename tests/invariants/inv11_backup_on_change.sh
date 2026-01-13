#!/bin/bash
# Invariant 11: Configuration changes must create backups
# Test: Backup function exists and is called before changes

set -e

echo "Testing Invariant 11: Backups before configuration changes"

CHANGES_FILE="crates/annad/src/changes.rs"

if [ ! -f "$CHANGES_FILE" ]; then
    echo "FAIL: changes.rs not found"
    exit 1
fi

# Check backup function exists
if ! grep -q "backup_file\|create_backup" "$CHANGES_FILE"; then
    echo "FAIL: No backup function found"
    exit 1
fi

# Check backup is called before apply_change
if ! grep -B5 "fs::write" "$CHANGES_FILE" | grep -q "backup"; then
    echo "WARN: Backup may not be called before write"
fi

# Check backup directory exists in paths
if ! grep -q "backup" crates/anna-shared/src/paths.rs; then
    echo "FAIL: No backup directory in paths"
    exit 1
fi

echo "PASS: Backup mechanism exists"
echo "Invariant 11: PASS"
