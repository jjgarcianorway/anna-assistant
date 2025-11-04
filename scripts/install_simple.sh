#!/usr/bin/env bash
# Anna Simple Installer - ALWAYS WORKS
# Priority: Local binaries > GitHub release > Fail with clear error

set -Eeuo pipefail

echo ""
echo "═══════════════════════════════════════════"
echo "  Anna Installer"
echo "═══════════════════════════════════════════"
echo ""

# Must run as root
if [[ $EUID -ne 0 ]]; then
    echo "✗ ERROR: Must run as root"
    echo "  Run: sudo $0"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Function to install binaries
install_binaries() {
    local ANNAD_SRC="$1"
    local ANNACTL_SRC="$2"
    local VERSION="$3"

    echo "→ Stopping annad service (if running)..."
    systemctl stop annad 2>/dev/null || true

    echo "→ Installing binaries..."
    install -m 755 "$ANNAD_SRC" /usr/local/bin/annad
    install -m 755 "$ANNACTL_SRC" /usr/local/bin/annactl

    echo "→ Creating directories..."
    mkdir -p /etc/anna/policies.d
    mkdir -p /var/lib/anna/{telemetry,backups}
    mkdir -p /var/log/anna
    mkdir -p /run/anna

    chmod 755 /etc/anna /etc/anna/policies.d
    chmod 755 /var/lib/anna /var/lib/anna/telemetry /var/lib/anna/backups
    chmod 755 /var/log/anna /run/anna

    echo "$VERSION" > /etc/anna/version

    echo "→ Installing systemd service..."
    cat > /etc/systemd/system/annad.service <<'EOF'
[Unit]
Description=Anna Assistant Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/annad
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable annad.service 2>/dev/null || true
    systemctl start annad.service 2>/dev/null || true

    echo ""
    echo "══════════════════════════════════════════════"
    echo "  ✓ Installation Complete!"
    echo "══════════════════════════════════════════════"
    echo ""
    echo "Version: $VERSION"
    echo "Check:   annactl status"
    echo ""
}

# Try 1: Local binaries
if [[ -f "$REPO_ROOT/target/release/annad" && -f "$REPO_ROOT/target/release/annactl" ]]; then
    echo "✓ Found local binaries"

    LOCAL_VERSION=$("$REPO_ROOT/target/release/annactl" --version 2>/dev/null | awk '{print $NF}')

    # Verify socket fix is in binary
    if strings "$REPO_ROOT/target/release/annad" | grep -q "Socket group ownership set to anna"; then
        echo "✓ Socket ownership fix verified in binary"
    else
        echo "✗ ERROR: Local binary missing socket fix!"
        echo "  Rebuild with: cargo build --release"
        exit 1
    fi

    install_binaries "$REPO_ROOT/target/release/annad" "$REPO_ROOT/target/release/annactl" "v$LOCAL_VERSION"
    exit 0
fi

# Try 2: GitHub release
echo "→ No local binaries, downloading from GitHub..."

if ! command -v curl >/dev/null 2>&1; then
    echo "✗ ERROR: curl not found"
    exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
    echo "✗ ERROR: tar not found"
    exit 1
fi

TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

echo "→ Finding latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/jjgarcianorway/anna-assistant/releases/latest" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [[ -z "$LATEST" ]]; then
    echo "✗ ERROR: Could not find GitHub release"
    echo ""
    echo "To install from source:"
    echo "  1. cd $REPO_ROOT"
    echo "  2. cargo build --release"
    echo "  3. sudo $0"
    exit 1
fi

echo "✓ Latest release: $LATEST"

ASSET_URL="https://github.com/jjgarcianorway/anna-assistant/releases/download/$LATEST/anna-linux-x86_64.tar.gz"

echo "→ Downloading $LATEST..."
if ! curl -fsSL -o "$TMPDIR/anna.tar.gz" "$ASSET_URL" 2>/dev/null; then
    echo "✗ ERROR: Download failed"
    exit 1
fi

echo "→ Extracting..."
tar -xzf "$TMPDIR/anna.tar.gz" -C "$TMPDIR"

if [[ ! -f "$TMPDIR/annad" || ! -f "$TMPDIR/annactl" ]]; then
    echo "✗ ERROR: Tarball missing binaries"
    exit 1
fi

# Verify socket fix in downloaded binary
if strings "$TMPDIR/annad" | grep -q "Socket group ownership set to anna"; then
    echo "✓ Socket ownership fix verified in downloaded binary"
else
    echo "✗ ERROR: Downloaded binary missing socket fix!"
    echo "  Please report this issue on GitHub"
    exit 1
fi

install_binaries "$TMPDIR/annad" "$TMPDIR/annactl" "$LATEST"
