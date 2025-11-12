#!/bin/bash
# Anna Assistant System Setup Script
# Creates user, group, directories, and READMEs
# Idempotent - safe to run multiple times

set -e

ANNA_USER="anna"
ANNA_GROUP="anna"
STATE_DIR="/var/lib/anna"
LOG_DIR="/var/log/anna"

echo "=== Anna Assistant System Setup ==="
echo

# Create user and group if they don't exist
if ! getent group "$ANNA_GROUP" >/dev/null 2>&1; then
    echo "Creating group: $ANNA_GROUP"
    groupadd --system "$ANNA_GROUP"
else
    echo "Group $ANNA_GROUP already exists"
fi

if ! getent passwd "$ANNA_USER" >/dev/null 2>&1; then
    echo "Creating user: $ANNA_USER"
    useradd --system \
        --gid "$ANNA_GROUP" \
        --home-dir "$STATE_DIR" \
        --shell /usr/sbin/nologin \
        --comment "Anna Assistant Daemon" \
        "$ANNA_USER"
else
    echo "User $ANNA_USER already exists"
fi

# Create directories with proper permissions
echo
echo "Creating directories..."
for dir in "$STATE_DIR" "$LOG_DIR"; do
    if [ ! -d "$dir" ]; then
        echo "  Creating: $dir"
        mkdir -p "$dir"
    else
        echo "  Exists: $dir"
    fi

    echo "  Setting permissions: 750"
    chmod 750 "$dir"

    echo "  Setting ownership: $ANNA_USER:$ANNA_GROUP"
    chown "$ANNA_USER:$ANNA_GROUP" "$dir"
done

# Create README in state directory
echo
echo "Creating README files..."
cat > "$STATE_DIR/README" <<'EOF'
Anna Assistant State Directory
===============================

This directory contains persistent state for the Anna daemon:

- mirror_audit/state.json       Mirror audit state (TIS history, bias findings)
- chronos/chronicle.json         Forecast history and outcomes
- collective_mind/state.json     Collective knowledge state
- mirror_protocol/state.json     Metacognitive reflection state
- keys/                          Ed25519 keys for distributed consensus (Phase 1.7+)

All files are JSON formatted. State is updated atomically.

BACKUP RECOMMENDATION:
    sudo tar czf anna-state-$(date +%Y%m%d).tar.gz /var/lib/anna

PERMISSIONS:
    Owner: anna:anna
    Mode: 750 (rwxr-x---)

WARNING: Do not manually edit state files while daemon is running.

Citation: [archwiki:System_maintenance]
EOF

cat > "$LOG_DIR/README" <<'EOF'
Anna Assistant Log Directory
=============================

This directory contains append-only logs for the Anna daemon:

- mirror-audit.jsonl    Temporal integrity audit trail (JSONL format)
- daemon.log            General daemon logs (if file logging enabled)

LOG ROTATION:
    Managed by logrotate at /etc/logrotate.d/anna
    - Size threshold: 10MB
    - Retention: 14 rotations
    - Compression: yes
    - Missing ok: yes

AUDIT LOG FORMAT:
    Each line is a JSON object containing:
    - forecast_id
    - predicted (SystemMetrics)
    - actual (SystemMetrics)
    - errors (MAE, RMSE)
    - temporal_integrity_score (TIS components)
    - bias_findings
    - adjustment_plan (advisory only)

MONITORING:
    tail -f /var/log/anna/mirror-audit.jsonl | jq .
    annactl mirror audit-forecast --json | jq '.average_temporal_integrity'

PERMISSIONS:
    Owner: anna:anna
    Mode: 750 (rwxr-x---)

Citation: [archwiki:System_maintenance]
EOF

chown "$ANNA_USER:$ANNA_GROUP" "$STATE_DIR/README" "$LOG_DIR/README"
chmod 640 "$STATE_DIR/README" "$LOG_DIR/README"

echo
echo "=== Setup Complete ==="
echo
echo "User:        $ANNA_USER (system account)"
echo "Group:       $ANNA_GROUP"
echo "State dir:   $STATE_DIR (750)"
echo "Log dir:     $LOG_DIR (750)"
echo
echo "Next steps:"
echo "  1. Install Anna binaries to /usr/bin/"
echo "  2. Copy systemd service: cp systemd/anna-daemon.service /etc/systemd/system/"
echo "  3. Copy logrotate config: cp logrotate/anna /etc/logrotate.d/"
echo "  4. Reload systemd: systemctl daemon-reload"
echo "  5. Enable service: systemctl enable anna-daemon"
echo "  6. Start service: systemctl start anna-daemon"
echo "  7. Check status: systemctl status anna-daemon"
echo
