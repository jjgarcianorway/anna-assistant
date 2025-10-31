#!/bin/bash
# Anna v0.10.1 Uninstaller

set -e

PURGE=false

if [[ "$1" == "--purge" ]]; then
    PURGE=true
fi

echo "Uninstalling Anna v0.10.1..."

# Stop and disable service
if systemctl is-active annad &>/dev/null; then
    echo "  ✓ Stopping daemon..."
    systemctl stop annad
fi

if systemctl is-enabled annad &>/dev/null; then
    echo "  ✓ Disabling service..."
    systemctl disable annad
fi

# Remove systemd unit
if [[ -f /etc/systemd/system/annad.service ]]; then
    echo "  ✓ Removing systemd unit..."
    rm /etc/systemd/system/annad.service
    systemctl daemon-reload
fi

# Remove binaries
if [[ -f /usr/local/bin/annad ]]; then
    echo "  ✓ Removing daemon binary..."
    rm /usr/local/bin/annad
fi

if [[ -f /usr/local/bin/annactl ]]; then
    echo "  ✓ Removing CLI binary..."
    rm /usr/local/bin/annactl
fi

# Remove runtime dir
if [[ -d /run/anna ]]; then
    echo "  ✓ Removing runtime directory..."
    rm -rf /run/anna
fi

if [[ "$PURGE" == "true" ]]; then
    echo "  ✓ Purging data and logs..."
    rm -rf /var/lib/anna
    rm -rf /var/log/anna

    if id anna &>/dev/null; then
        echo "  ✓ Removing anna user..."
        userdel anna
    fi

    echo "✓ Complete uninstall (data purged)"
else
    echo "✓ Uninstalled (data preserved in /var/lib/anna)"
    echo "  To remove all data, run: $0 --purge"
fi
