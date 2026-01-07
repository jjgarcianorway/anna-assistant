#!/bin/bash
# tools/snapshot.sh — Quick "pre-work" snapshot branch creator
set -euo pipefail

TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
BRANCH_NAME="snapshot-$TIMESTAMP"

git checkout -b "$BRANCH_NAME"

echo "Created snapshot branch: $BRANCH_NAME"
echo "To return to your previous branch, use: git checkout -"
