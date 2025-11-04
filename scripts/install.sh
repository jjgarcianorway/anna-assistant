#!/usr/bin/env bash
# Anna Auto-Installer - Always Works
set -Eeuo pipefail

OWNER="jjgarcianorway"
REPO="anna-assistant"

# Simple output
info() { echo "→ $*"; }
success() { echo "✓ $*"; }
error() { echo "✗ ERROR: $*" >&2; exit 1; }

require() { command -v "$1" >/dev/null 2>&1 || error "Missing: $1"; }
require curl
require tar
require sha256sum

# Must run as root
[[ $EUID -eq 0 ]] || error "Run as root: sudo ./scripts/install.sh"

echo ""
echo "═══════════════════════════════════════════"
echo "  Anna Auto-Installer"
echo "═══════════════════════════════════════════"
echo ""

# Get latest release with assets
info "Finding latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/$OWNER/$REPO/releases" 2>/dev/null | \
         jq -r '.[] | select(.draft==false) | select(.assets[] | .name=="anna-linux-x86_64.tar.gz") | .tag_name' | \
         head -1)

[[ -n "$LATEST" ]] || error "No releases found with assets"

success "Latest release: $LATEST"

# Download with retry
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

ASSET_URL="https://github.com/$OWNER/$REPO/releases/download/$LATEST/anna-linux-x86_64.tar.gz"
CHECKSUM_URL="https://github.com/$OWNER/$REPO/releases/download/$LATEST/anna-linux-x86_64.tar.gz.sha256"

info "Downloading $LATEST..."

MAX_WAIT=300
elapsed=0
while [ $elapsed -lt $MAX_WAIT ]; do
    if curl -fsSL -o "$TMPDIR/anna-linux-x86_64.tar.gz" "$ASSET_URL" 2>/dev/null && \
       curl -fsSL -o "$TMPDIR/anna-linux-x86_64.tar.gz.sha256" "$CHECKSUM_URL" 2>/dev/null; then
        break
    fi

    if [ $elapsed -eq 0 ]; then
        info "Assets not ready, waiting for CI..."
    fi

    sleep 10
    elapsed=$((elapsed + 10))
done

[[ -f "$TMPDIR/anna-linux-x86_64.tar.gz" ]] || error "Download failed after ${MAX_WAIT}s"

success "Downloaded"

# Verify checksum
info "Verifying checksum..."
cd "$TMPDIR"
if sha256sum -c anna-linux-x86_64.tar.gz.sha256 2>&1 | grep -q OK; then
    success "Checksum OK"
else
    error "Checksum verification failed"
fi
cd - >/dev/null

# Extract
info "Extracting..."
tar -xzf "$TMPDIR/anna-linux-x86_64.tar.gz" -C "$TMPDIR"
[[ -f "$TMPDIR/annad" && -f "$TMPDIR/annactl" ]] || error "Tarball missing binaries"

# Install
info "Installing to /usr/local/bin..."
install -m 755 "$TMPDIR/annad" /usr/local/bin/annad
install -m 755 "$TMPDIR/annactl" /usr/local/bin/annactl

# Create directories
info "Creating directories..."
mkdir -p /etc/anna/policies.d
mkdir -p /var/lib/anna/{telemetry,backups}
mkdir -p /var/log/anna
mkdir -p /run/anna

chmod 755 /etc/anna /etc/anna/policies.d
chmod 755 /var/lib/anna /var/lib/anna/telemetry /var/lib/anna/backups
chmod 755 /var/log/anna /run/anna

echo "$LATEST" > /etc/anna/version

# Systemd service
if command -v systemctl >/dev/null 2>&1; then
    info "Installing systemd service..."

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
    systemctl restart annad.service 2>/dev/null || true
fi

# Verify
info "Verifying installation..."
INSTALLED_VER=$(annactl --version 2>/dev/null | awk '{print $NF}')
EXPECTED_VER="${LATEST#v}"

if [[ "$INSTALLED_VER" == "$EXPECTED_VER" ]]; then
    success "Version verified: $INSTALLED_VER"
else
    error "Version mismatch: installed=$INSTALLED_VER, expected=$EXPECTED_VER"
fi

echo ""
echo "══════════════════════════════════════════════"
echo "  ✓ Installation Complete!"
echo "══════════════════════════════════════════════"
echo ""
echo "Version: $LATEST"
echo "Check:   annactl status"
echo ""
