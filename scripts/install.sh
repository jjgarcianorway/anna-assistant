#!/usr/bin/env bash
# Anna Assistant Installer
# Downloads and installs the latest pre-compiled binaries

set -euo pipefail

GITHUB_REPO="jjgarcianorway/anna-assistant"

echo "╭─────────────────────────────────────────╮"
echo "│  Anna Assistant Installer              │"
echo "│  Event-Driven Intelligence             │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Find project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        ARTIFACT_NAME="anna-linux-x86_64"
        ;;
    aarch64)
        ARTIFACT_NAME="anna-linux-aarch64"
        ;;
    *)
        echo "✗ Unsupported architecture: $ARCH"
        echo "  Supported: x86_64, aarch64"
        exit 1
        ;;
esac

# Parse command line arguments
if [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
    echo "Usage: $0"
    echo ""
    echo "Downloads and installs the latest Anna Assistant release."
    echo ""
    echo "This installer will:"
    echo "  1. Download pre-compiled binaries from GitHub"
    echo "  2. Create system user and group"
    echo "  3. Install binaries and configuration"
    echo "  4. Set up systemd service"
    echo ""
    exit 0
fi

# Function to fetch latest release and download binaries
download_binaries() {
    echo "→ Fetching latest release from GitHub..."

    # Check for curl or wget
    if ! command -v curl &>/dev/null && ! command -v wget &>/dev/null; then
        echo "✗ Need curl or wget to download binaries"
        echo "  Install: sudo pacman -S curl"
        return 1
    fi

    # Fetch latest release info
    local api_url="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
    local tmp_json=$(mktemp)

    if command -v curl &>/dev/null; then
        if ! curl -s -f -L "$api_url" -o "$tmp_json" 2>/dev/null; then
            rm -f "$tmp_json"
            echo "✗ No releases found on GitHub yet"
            echo ""
            echo "  The project maintainer needs to create a release first."
            echo "  Releases are created automatically when version tags are pushed."
            echo ""
            echo "  Check: https://github.com/${GITHUB_REPO}/releases"
            return 1
        fi
    else
        if ! wget -q -O "$tmp_json" "$api_url" 2>/dev/null; then
            rm -f "$tmp_json"
            echo "✗ No releases found on GitHub yet"
            return 1
        fi
    fi

    # Extract version tag
    local version_tag=$(grep -o '"tag_name"[^,]*' "$tmp_json" | grep -o 'v[0-9.]*' | head -1)
    rm -f "$tmp_json"

    if [ -z "$version_tag" ]; then
        echo "✗ Could not parse release version"
        return 1
    fi

    echo "✓ Latest release: $version_tag"
    echo ""
    echo "→ Downloading binaries for $ARCH..."

    # Download URL
    local download_url="https://github.com/${GITHUB_REPO}/releases/download/${version_tag}/${ARTIFACT_NAME}.tar.gz"
    local tmp_dir=$(mktemp -d)

    if command -v curl &>/dev/null; then
        if ! curl -L -f -o "$tmp_dir/binaries.tar.gz" "$download_url" 2>/dev/null; then
            echo "✗ Download failed"
            echo "  URL: $download_url"
            rm -rf "$tmp_dir"
            return 1
        fi
    else
        if ! wget -q -O "$tmp_dir/binaries.tar.gz" "$download_url"; then
            echo "✗ Download failed"
            rm -rf "$tmp_dir"
            return 1
        fi
    fi

    echo "✓ Downloaded $ARTIFACT_NAME.tar.gz"
    echo ""
    echo "→ Extracting..."

    if ! tar -xzf "$tmp_dir/binaries.tar.gz" -C "$tmp_dir" 2>/dev/null; then
        echo "✗ Extraction failed"
        rm -rf "$tmp_dir"
        return 1
    fi

    # Verify binaries exist
    if [[ ! -f "$tmp_dir/annad" || ! -f "$tmp_dir/annactl" ]]; then
        echo "✗ Binaries not found in archive"
        rm -rf "$tmp_dir"
        return 1
    fi

    echo "✓ Extracted binaries"

    # Move to bin directory
    mkdir -p bin
    mv "$tmp_dir/annad" "$tmp_dir/annactl" bin/
    chmod +x bin/annad bin/annactl
    rm -rf "$tmp_dir"

    echo "✓ Binaries ready for installation"
    echo ""
    return 0
}

# Main installation flow
if [ ! -d "bin" ] || [ ! -f "bin/annad" ] || [ ! -f "bin/annactl" ]; then
    if ! download_binaries; then
        echo ""
        echo "╭─────────────────────────────────────────╮"
        echo "│  Installation Failed                    │"
        echo "╰─────────────────────────────────────────╯"
        echo ""
        echo "No pre-compiled binaries available yet."
        echo ""
        echo "The project needs to create its first release."
        echo "Check: https://github.com/${GITHUB_REPO}/releases"
        exit 1
    fi
else
    echo "→ Using existing binaries from ./bin/"
    echo ""
fi

# Require root for installation
echo "The following steps require elevated privileges:"
echo "  • Create system user and group 'anna'"
echo "  • Install binaries to /usr/local/bin"
echo "  • Install systemd service"
echo "  • Create directories and set permissions"
echo ""
read -p "Proceed? [Y/n] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]?$ ]]; then
    echo "Installation cancelled."
    exit 0
fi
echo ""

# Create anna user and group
echo "→ Creating system user and group..."
if ! getent group anna &>/dev/null; then
    sudo groupadd --system anna
    echo "✓ Created group 'anna'"
else
    echo "✓ Group 'anna' exists"
fi

if ! id -u anna &>/dev/null; then
    sudo useradd --system --no-create-home --shell /usr/sbin/nologin -g anna anna
    echo "✓ Created user 'anna'"
else
    echo "✓ User 'anna' exists"
fi

# Add current user to anna group
echo ""
echo "→ Adding current user to anna group..."
if ! groups | grep -q anna; then
    sudo usermod -aG anna "$USER"
    echo "✓ Added (log out and back in for this to take effect)"
else
    echo "✓ Already in anna group"
fi

# Check if daemon is already running (upgrade scenario)
DAEMON_WAS_RUNNING=false
if systemctl is-active --quiet annad 2>/dev/null; then
    DAEMON_WAS_RUNNING=true
    OLD_VERSION=$(annactl --version 2>/dev/null | awk '{print $2}' || echo "unknown")
    echo "→ Detected running daemon (v${OLD_VERSION})"
    echo "→ Will restart after installing new binaries"
fi

# Install binaries
echo ""
echo "→ Installing binaries..."
sudo install -m 755 bin/annad /usr/local/bin/
sudo install -m 755 bin/annactl /usr/local/bin/
echo "✓ Installed to /usr/local/bin"

# Create directories
echo ""
echo "→ Creating directories..."
sudo install -d -m 0750 -o anna -g anna /var/lib/anna
sudo install -d -m 0750 -o anna -g anna /var/log/anna
sudo install -d -m 0755 -o root -g root /usr/lib/anna
sudo install -d -m 0755 -o root -g root /etc/anna
sudo install -d -m 0755 -o root -g root /etc/anna/policies.d
sudo install -d -m 0755 -o root -g root /etc/anna/personas.d
echo "✓ Directories created"

# Install default configuration
echo ""
echo "→ Installing configuration..."
if [ ! -f /etc/anna/config.toml ]; then
    cat <<'EOF' | sudo tee /etc/anna/config.toml >/dev/null
[autonomy]
level = "low"

[ui]
emojis = false
color = true

[telemetry]
enabled = false
collection_interval_sec = 60

[persona]
active = "dev"
EOF
    echo "✓ Default config installed"
else
    echo "✓ Config already exists"
fi

# Install policy configuration
if [ ! -f /etc/anna/policy.toml ] && [ -f etc/policy.toml ]; then
    sudo install -m 0644 etc/policy.toml /etc/anna/
    echo "✓ Policy config installed"
elif [ -f /etc/anna/policy.toml ]; then
    echo "✓ Policy config already exists"
else
    echo "⚠ policy.toml not found in source (etc/policy.toml)"
fi

# Install capability registry
echo ""
echo "→ Installing capability registry..."
if [ -f etc/CAPABILITIES.toml ]; then
    sudo install -m 0644 etc/CAPABILITIES.toml /usr/lib/anna/
    echo "✓ Capabilities registered"
else
    echo "⚠ CAPABILITIES.toml not found"
fi

# Install systemd service
echo ""
echo "→ Installing systemd service..."
if [ -f etc/systemd/annad.service ]; then
    sudo install -m 0644 etc/systemd/annad.service /etc/systemd/system/
    sudo systemctl daemon-reload
    echo "✓ Service installed"
else
    echo "⚠ Service file not found: etc/systemd/annad.service"
fi

# Show installed version
echo ""
echo "→ Checking installed version..."
if [ -f bin/annactl ]; then
    INSTALLED_VERSION=$(./bin/annactl --version 2>/dev/null | awk '{print $2}' || echo "unknown")
    echo "✓ Installing version: $INSTALLED_VERSION"
else
    echo "⚠ Could not determine version"
fi

# Enable and start/restart service
echo ""
if [ "$DAEMON_WAS_RUNNING" = true ]; then
    echo "→ Restarting Anna with new binaries..."
    if ! sudo systemctl restart annad 2>/dev/null; then
        echo "✗ Restart failed"
        echo "  Check: systemctl status annad"
        exit 1
    fi
    echo "✓ Daemon restarted"
else
    echo "→ Starting Anna..."
    if ! sudo systemctl enable --now annad 2>/dev/null; then
        echo "⚠ Could not start service"
        echo "  Check: systemctl status annad"
        exit 1
    fi
    echo "✓ Daemon started"
fi

echo "→ Waiting for daemon to initialize..."

# Wait up to 10 seconds for socket to appear
WAITED=0
MAX_WAIT=10
while [ $WAITED -lt $MAX_WAIT ]; do
    if [ -S /run/anna/annad.sock ]; then
        break
    fi
    sleep 1
    WAITED=$((WAITED + 1))
done

if [ ! -S /run/anna/annad.sock ]; then
    echo "✗ Socket not created after ${MAX_WAIT}s"
    echo ""
    echo "Diagnostics:"
    sudo systemctl status annad --no-pager -l | head -20
    echo ""
    echo "Check logs: sudo journalctl -u annad -n 30"
    exit 1
fi

# Verify using annactl with timeout
echo "→ Testing daemon response..."
if timeout 5 annactl status &>/dev/null; then
    RUNNING_VERSION=$(annactl --version 2>/dev/null | awk '{print $2}' || echo "unknown")
    echo "✓ Anna is running and responding"
    echo "✓ Daemon version: $RUNNING_VERSION"

    # Validate version match (if we can determine installed version)
    if [ -n "$INSTALLED_VERSION" ] && [ "$INSTALLED_VERSION" != "unknown" ]; then
        if [ "$RUNNING_VERSION" = "$INSTALLED_VERSION" ]; then
            echo "✓ Version verified: $RUNNING_VERSION"
        else
            echo "⚠ Version mismatch: running=$RUNNING_VERSION, installed=$INSTALLED_VERSION"
            echo "  This may indicate a daemon restart issue"
        fi
    fi
else
    echo "✗ Daemon not responding (timeout after 5s)"
    echo ""
    echo "Diagnostics:"
    systemctl is-active annad && echo "  • Process: active" || echo "  • Process: inactive"
    [ -S /run/anna/annad.sock ] && echo "  • Socket: exists ($(stat -c '%U:%G %a' /run/anna/annad.sock))" || echo "  • Socket: missing"
    echo ""
    echo "Recent logs:"
    sudo journalctl -u annad -n 15 --no-pager
    exit 1
fi

echo ""
echo "╭─────────────────────────────────────────╮"
echo "│  ✓ Installation Complete!               │"
echo "╰─────────────────────────────────────────╯"
echo ""
echo "Next steps:"
echo ""
echo "  1. Log out and back in (for group permissions)"
echo "  2. Check status: annactl status"
echo "  3. View help: annactl --help"
echo ""
