#!/usr/bin/env bash
# Anna v0.11.0 - Debug Information Collector
# Quick one-shot collector for CI and user troubleshooting

set -euo pipefail

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_DIR="/tmp/anna_diag_${TIMESTAMP}"

mkdir -p "$OUTPUT_DIR"

echo "Anna Debug Collector"
echo "Collecting system state to: $OUTPUT_DIR"
echo ""

# System info
uname -a > "$OUTPUT_DIR/uname.txt" 2>&1
cat /etc/os-release > "$OUTPUT_DIR/os-release.txt" 2>&1 || true

# Anna binaries
/usr/local/bin/annad --version > "$OUTPUT_DIR/annad_version.txt" 2>&1 || echo "annad not found" > "$OUTPUT_DIR/annad_version.txt"
/usr/local/bin/annactl version > "$OUTPUT_DIR/annactl_version.txt" 2>&1 || echo "annactl not found" > "$OUTPUT_DIR/annactl_version.txt"

# Systemd status
systemctl status annad --no-pager > "$OUTPUT_DIR/systemd_status.txt" 2>&1 || true
systemctl show annad > "$OUTPUT_DIR/systemd_show.txt" 2>&1 || true

# Journal logs
journalctl -u annad -n 100 --no-pager > "$OUTPUT_DIR/journal.log" 2>&1 || true

# Directory listings
ls -laR /var/lib/anna > "$OUTPUT_DIR/dir_var_lib_anna.txt" 2>&1 || true
ls -laR /var/log/anna > "$OUTPUT_DIR/dir_var_log_anna.txt" 2>&1 || true
ls -laR /run/anna > "$OUTPUT_DIR/dir_run_anna.txt" 2>&1 || true
ls -laR /etc/anna > "$OUTPUT_DIR/dir_etc_anna.txt" 2>&1 || true
ls -laR /usr/lib/anna > "$OUTPUT_DIR/dir_usr_lib_anna.txt" 2>&1 || true

# Configuration files
cp /etc/systemd/system/annad.service "$OUTPUT_DIR/annad.service" 2>/dev/null || true
cp /usr/lib/anna/CAPABILITIES.toml "$OUTPUT_DIR/CAPABILITIES.toml" 2>/dev/null || true
cp /etc/anna/config.toml "$OUTPUT_DIR/config.toml" 2>/dev/null || true

# annactl outputs
annactl status > "$OUTPUT_DIR/annactl_status.txt" 2>&1 || true
annactl events --limit 10 > "$OUTPUT_DIR/annactl_events.txt" 2>&1 || true
annactl capabilities > "$OUTPUT_DIR/annactl_capabilities.txt" 2>&1 || true

echo "✓ Collection complete"
echo ""
echo "Output directory: $OUTPUT_DIR"
echo ""
echo "To create a tarball:"
echo "  tar -czf anna_diag_${TIMESTAMP}.tar.gz -C /tmp anna_diag_${TIMESTAMP}"
echo ""
