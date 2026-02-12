#!/bin/bash
# Anna Comprehensive Stress Test & Analysis
# Goal: Push Anna to 100% reliability, discover everything

set -euo pipefail

REPORT_DIR="/tmp/anna_stress_test_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$REPORT_DIR"

echo "=== Anna Comprehensive Stress Test ===" | tee "$REPORT_DIR/summary.txt"
echo "Started: $(date)" | tee -a "$REPORT_DIR/summary.txt"
echo "" | tee -a "$REPORT_DIR/summary.txt"

# Phase 1: System Forensics
echo "Phase 1: System Forensics" | tee -a "$REPORT_DIR/summary.txt"
echo "Analyzing logs, patterns, anomalies..." | tee -a "$REPORT_DIR/summary.txt"

# Collect all relevant logs
echo "  - Collecting system logs..." | tee -a "$REPORT_DIR/summary.txt"
journalctl --since "7 days ago" > "$REPORT_DIR/system_logs_7days.txt" 2>&1 || true
journalctl -u annad --since "7 days ago" > "$REPORT_DIR/anna_logs_7days.txt" 2>&1 || true
tail -1000 /var/log/pacman.log > "$REPORT_DIR/pacman.log" 2>&1 || true

echo "  - Analyzing boot times..." | tee -a "$REPORT_DIR/summary.txt"
systemd-analyze > "$REPORT_DIR/boot_analysis.txt" 2>&1 || true
systemd-analyze blame > "$REPORT_DIR/boot_blame.txt" 2>&1 || true

echo "  - Memory and disk usage patterns..." | tee -a "$REPORT_DIR/summary.txt"
free -h > "$REPORT_DIR/memory.txt"
df -h > "$REPORT_DIR/disk.txt"
du -sh /home/* > "$REPORT_DIR/home_usage.txt" 2>&1 || true

echo "  - Failed services..." | tee -a "$REPORT_DIR/summary.txt"
systemctl --failed > "$REPORT_DIR/failed_services.txt" 2>&1 || true

echo "  - Recent package operations..." | tee -a "$REPORT_DIR/summary.txt"
grep -E "(installed|upgraded|removed)" /var/log/pacman.log | tail -100 > "$REPORT_DIR/recent_packages.txt" 2>&1 || true

# Phase 2: Anna State Analysis
echo "" | tee -a "$REPORT_DIR/summary.txt"
echo "Phase 2: Anna State Analysis" | tee -a "$REPORT_DIR/summary.txt"
echo "Examining Anna's internal state..." | tee -a "$REPORT_DIR/summary.txt"

if [ -d "/var/lib/anna" ]; then
    ls -lah /var/lib/anna/ > "$REPORT_DIR/anna_state_files.txt"

    # Copy important state files
    for file in learning.json stats.json memory.json baseline.json; do
        if [ -f "/var/lib/anna/$file" ]; then
            cp "/var/lib/anna/$file" "$REPORT_DIR/" 2>&1 || true
        fi
    done
fi

echo "Report directory: $REPORT_DIR"
echo "Phase 1 & 2 complete." | tee -a "$REPORT_DIR/summary.txt"
