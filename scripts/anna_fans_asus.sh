#!/usr/bin/env bash
# Anna ASUS Fan Control - Quiet, steady behavior
# Manages fan curves for ASUS laptops via asusctl

set -euo pipefail

echo "[ANNA-FANS] Applying quiet profile for ASUS laptop"

# Set quiet profile (reduces noise and power)
if command -v asusctl &>/dev/null; then
    asusctl profile -P quiet && echo "[ANNA-FANS] Quiet profile applied" || echo "[ANNA-FANS] Failed to set quiet profile"
else
    echo "[ANNA-FANS] asusctl not found, skipping ASUS profile"
    exit 0
fi

# Apply custom fan curve if supported
# Lower ramp to avoid sudden whoosh
# Format: temp:percent (temperature in C : fan speed percentage)
if asusctl fan-curve --help &>/dev/null; then
    echo "[ANNA-FANS] Applying custom fan curve"

    asusctl fan-curve --apply <<'EOF' || {
        echo "[ANNA-FANS] Fan curve not supported on this model, using profile only"
    }
# device mode  temp:percent  (lower ramp to avoid sudden whoosh)
cpu  quiet  45:0 55:20 65:35 75:55 85:75 92:100
gpu  quiet  45:0 55:20 65:35 75:55 85:75 92:100
EOF
else
    echo "[ANNA-FANS] Fan curve control not available, using profile only"
fi

echo "[ANNA-FANS] Configuration complete"
