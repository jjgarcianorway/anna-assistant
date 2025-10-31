#!/usr/bin/env bash
# Anna Assistant Installer
# Smart installer: Downloads binaries → Falls back to source build
# Runs as user, escalates only when needed

set -euo pipefail

GITHUB_REPO="jjgarcianorway/anna-assistant"
BUILD_MODE=""
LATEST_VERSION=""  # Will be fetched from GitHub

echo "╭─────────────────────────────────────────╮"
echo "│  Anna Assistant Installer              │"
echo "│  Event-Driven Intelligence             │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Find project root and cd to it
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Verify we're in project root
if [[ ! -f Cargo.toml ]]; then
    echo "✗ Could not find project root (expected Cargo.toml)"
    exit 1
fi

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
        echo "⚠ Unsupported architecture: $ARCH"
        echo "  Only x86_64 and aarch64 are supported for binary downloads"
        echo "  Will attempt to build from source..."
        BUILD_MODE="source"
        ;;
esac

# Parse command line arguments
FORCE_BUILD=false
for arg in "$@"; do
    case "$arg" in
        --build|--source)
            FORCE_BUILD=true
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --build, --source    Force building from source (requires Rust)"
            echo "  --help, -h           Show this help message"
            echo ""
            echo "Installation methods (tried in order):"
            echo "  1. Use binaries from ./bin/ directory"
            echo "  2. Download pre-compiled binaries from GitHub releases"
            echo "  3. Build from source (requires Rust/Cargo)"
            exit 0
            ;;
    esac
done

# Function to check if binaries are available
check_binaries() {
    [[ -f "$1/annad" && -f "$1/annactl" ]]
}

# Function to verify binary integrity (basic check)
verify_binaries() {
    local dir="$1"
    if ! file "$dir/annad" | grep -q "ELF.*executable"; then
        echo "✗ Invalid binary: annad"
        return 1
    fi
    if ! file "$dir/annactl" | grep -q "ELF.*executable"; then
        echo "✗ Invalid binary: annactl"
        return 1
    fi
    return 0
}

# Function to fetch latest release info from GitHub
fetch_latest_release() {
    echo "→ Fetching latest release information..."

    # Check for required tools
    if ! command -v curl &>/dev/null && ! command -v wget &>/dev/null; then
        echo "✗ Neither curl nor wget found (required for downloading)"
        return 1
    fi

    local api_url="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
    local tmp_file=$(mktemp)

    # Fetch release info
    if command -v curl &>/dev/null; then
        if ! curl -s -f -L "$api_url" -o "$tmp_file"; then
            echo "✗ Could not fetch release information from GitHub"
            rm -f "$tmp_file"
            return 1
        fi
    else
        if ! wget -q -O "$tmp_file" "$api_url"; then
            echo "✗ Could not fetch release information from GitHub"
            rm -f "$tmp_file"
            return 1
        fi
    fi

    # Parse JSON to get tag_name (works without jq)
    # Look for "tag_name": "v0.11.1" pattern
    LATEST_VERSION=$(grep -o '"tag_name"[^,]*' "$tmp_file" | grep -o 'v[0-9.]*' | head -1)
    rm -f "$tmp_file"

    if [ -z "$LATEST_VERSION" ]; then
        echo "✗ Could not determine latest version"
        return 1
    fi

    echo "✓ Latest release: $LATEST_VERSION"
    return 0
}

# Function to download pre-compiled binaries
download_binaries() {
    echo "→ Downloading pre-compiled binaries for $ARCH..."

    # Fetch latest release info
    if ! fetch_latest_release; then
        return 1
    fi

    local url="https://github.com/${GITHUB_REPO}/releases/download/${LATEST_VERSION}/${ARTIFACT_NAME}.tar.gz"
    local tmp_dir=$(mktemp -d)

    echo "  Downloading: ${ARTIFACT_NAME}.tar.gz (${LATEST_VERSION})"

    # Download using curl or wget
    if command -v curl &>/dev/null; then
        if ! curl -L -f -o "$tmp_dir/binaries.tar.gz" "$url" 2>&1 | grep -E "error|failed" | head -3; then
            if [ ${PIPESTATUS[0]} -ne 0 ]; then
                echo "✗ Download failed"
                echo "  URL: $url"
                rm -rf "$tmp_dir"
                return 1
            fi
        fi
    else
        if ! wget -q -O "$tmp_dir/binaries.tar.gz" "$url"; then
            echo "✗ Download failed"
            echo "  URL: $url"
            rm -rf "$tmp_dir"
            return 1
        fi
    fi

    # Extract
    echo "  Extracting..."
    if ! tar -xzf "$tmp_dir/binaries.tar.gz" -C "$tmp_dir"; then
        echo "✗ Extraction failed"
        rm -rf "$tmp_dir"
        return 1
    fi

    # Verify extracted binaries
    if ! verify_binaries "$tmp_dir"; then
        echo "✗ Downloaded binaries failed verification"
        rm -rf "$tmp_dir"
        return 1
    fi

    # Create bin directory and move binaries
    mkdir -p bin
    mv "$tmp_dir/annad" "$tmp_dir/annactl" bin/
    chmod +x bin/annad bin/annactl
    rm -rf "$tmp_dir"

    echo "✓ Downloaded and verified binaries"
    BUILD_MODE="downloaded"
    return 0
}

# Function to build from source
build_from_source() {
    echo "→ Building from source..."

    # Check for Rust/Cargo
    if ! command -v cargo &>/dev/null; then
        echo "✗ Cargo not found (required for building from source)"
        echo ""
        echo "To install Rust, run one of:"
        echo "  • System-wide: sudo pacman -S rust"
        echo "  • User-level:  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo ""
        return 1
    fi

    echo "  This may take 2-5 minutes on first build..."
    if cargo build --release 2>&1 | grep -E "^error" | head -5; then
        if [ ${PIPESTATUS[0]} -ne 0 ]; then
            echo "✗ Build failed"
            return 1
        fi
    fi

    # Verify built binaries exist
    if ! check_binaries "target/release"; then
        echo "✗ Build succeeded but binaries not found"
        return 1
    fi

    echo "✓ Build complete"
    BUILD_MODE="source"
    return 0
}

# Main binary acquisition logic
if [ "$FORCE_BUILD" = true ]; then
    echo "→ Forced source build mode"
    if ! build_from_source; then
        exit 1
    fi
elif check_binaries "bin"; then
    echo "→ Using pre-existing binaries from ./bin/"
    if verify_binaries "bin"; then
        BUILD_MODE="preexisting"
    else
        echo "✗ Binaries in ./bin/ failed verification"
        exit 1
    fi
elif [ -n "$ARTIFACT_NAME" ] && download_binaries; then
    # Downloaded successfully
    :
elif build_from_source; then
    # Built from source successfully
    :
else
    echo ""
    echo "✗ All binary acquisition methods failed:"
    echo "  1. No pre-existing binaries in ./bin/"
    echo "  2. Could not download from GitHub releases"
    echo "  3. Could not build from source"
    echo ""
    echo "Please either:"
    echo "  • Install Rust and try again: sudo pacman -S rust"
    echo "  • Download binaries manually from: https://github.com/${GITHUB_REPO}/releases"
    echo "  • Place binaries in ./bin/ directory"
    exit 1
fi

# Determine binary source directory
if [ "$BUILD_MODE" = "source" ]; then
    BIN_SOURCE="target/release"
else
    BIN_SOURCE="bin"
fi

echo ""
echo "The following steps require elevated privileges:"
echo "  • Create anna system user and group"
echo "  • Install binaries to /usr/local/bin"
echo "  • Install systemd service"
echo "  • Create directories and set permissions"
echo ""
read -p "Proceed? [Y/n] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]?$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo ""
echo "→ Creating anna system user and group..."

# Check if group exists first
if ! getent group anna &>/dev/null; then
    GROUP_EXISTS=false
else
    GROUP_EXISTS=true
    echo "✓ Group 'anna' exists"
fi

# Create user (handle group existence)
if ! id -u anna &>/dev/null; then
    if [ "$GROUP_EXISTS" = true ]; then
        # Group exists, so use it explicitly
        sudo useradd --system --no-create-home --shell /usr/sbin/nologin -g anna anna
        echo "✓ User 'anna' created (using existing group)"
    else
        # Neither exists, let useradd create both
        sudo useradd --system --no-create-home --shell /usr/sbin/nologin anna
        echo "✓ User 'anna' created"
        echo "✓ Group 'anna' created"
    fi
else
    echo "✓ User 'anna' exists"
fi

echo ""
echo "→ Adding current user to anna group..."
if ! groups | grep -q anna; then
    sudo usermod -aG anna "$USER"
    echo "✓ Added to group (log out/in for changes to take effect)"
else
    echo "✓ Already in anna group"
fi

echo ""
echo "→ Installing binaries..."
sudo install -m 755 "$BIN_SOURCE/annad" /usr/local/bin/
sudo install -m 755 "$BIN_SOURCE/annactl" /usr/local/bin/
echo "✓ Binaries installed to /usr/local/bin"

echo ""
echo "→ Creating directories..."
# Note: systemd StateDirectory, LogsDirectory, RuntimeDirectory will auto-create
# /var/lib/anna, /var/log/anna, /run/anna with correct ownership on daemon start.
# We create them here for consistency but fix ownership explicitly.

sudo install -d -m 0750 -o anna -g anna /var/lib/anna
sudo install -d -m 0750 -o anna -g anna /var/log/anna
sudo install -d -m 0755 -o root -g root /usr/lib/anna
sudo install -d -m 0755 -o root -g root /etc/anna
sudo install -d -m 0755 -o root -g root /etc/anna/policies.d
sudo install -d -m 0755 -o root -g root /etc/anna/personas.d

# Ensure ownership is correct (defensive fix)
sudo chown -R anna:anna /var/lib/anna /var/log/anna
sudo chmod 0750 /var/lib/anna /var/log/anna
echo "✓ Directories created with correct ownership"

echo ""
echo "→ Installing configuration..."
if [ ! -f /etc/anna/config.toml ]; then
    cat <<'EOF' | sudo tee /etc/anna/config.toml >/dev/null
# Hi, I'm Anna! Please use 'annactl config set' to change settings.
# Don't edit me by hand - I track who changed what and why.

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
    echo "✓ Config exists"
fi

echo ""
echo "→ Installing policies..."

# v0.11.0 Event Auto-Repair Policy
if [ ! -f /etc/anna/policy.toml ]; then
    if [ -f etc/policy.toml ]; then
        sudo cp etc/policy.toml /etc/anna/
        echo "✓ Event auto-repair policy installed"
    else
        echo "⚠ etc/policy.toml not found, skipping"
    fi
else
    echo "✓ Event policy exists"
fi

if [ ! -f /etc/anna/policies.d/00-bootstrap.yaml ]; then
    cat <<'EOF' | sudo tee /etc/anna/policies.d/00-bootstrap.yaml >/dev/null
# Hi, I'm Anna! Use 'annactl policy' to manage these rules.

- when: "telemetry.cpu_usage > 90"
  then: "alert"
  message: "High CPU usage"
  enabled: true

- when: "telemetry.mem_usage > 95"
  then: "alert"
  message: "Critical memory"
  enabled: true

- when: "always"
  then: "log"
  message: "Policy engine ready"
  enabled: true
EOF
    echo "✓ Bootstrap policies installed"
else
    echo "✓ Bootstrap policies exist"
fi

# Install thermal management policies
if [ ! -f /etc/anna/policies.d/thermal.yaml ]; then
    if [ -f etc/policies.d/thermal.yaml ]; then
        sudo cp etc/policies.d/thermal.yaml /etc/anna/policies.d/
        echo "✓ Thermal management policies installed"
    else
        echo "○ Thermal policies not found, skipping"
    fi
else
    echo "✓ Thermal policies exist"
fi

echo ""
echo "→ Installing capability registry..."
if [ -f etc/CAPABILITIES.toml ]; then
    sudo install -m 0644 -o root -g root etc/CAPABILITIES.toml /usr/lib/anna/CAPABILITIES.toml
    echo "✓ CAPABILITIES.toml installed"
elif [ ! -f /usr/lib/anna/CAPABILITIES.toml ]; then
    echo "⚠ etc/CAPABILITIES.toml not found in repo, creating minimal version..."
    cat <<'EOF' | sudo tee /usr/lib/anna/CAPABILITIES.toml >/dev/null
[meta]
version = "0.11.0"
description = "Anna telemetry capability registry"
EOF
    sudo chmod 0644 /usr/lib/anna/CAPABILITIES.toml
    sudo chown root:root /usr/lib/anna/CAPABILITIES.toml
    echo "✓ Minimal CAPABILITIES.toml created"
else
    echo "✓ CAPABILITIES.toml already exists"
fi

echo ""
echo "→ Detecting hardware..."

# Detect ASUS hardware
IS_ASUS=false
if [ -d /sys/devices/platform/asus-nb-wmi ] || [ -d /sys/devices/platform/asus_wmi ] || grep -qi asus /sys/class/dmi/id/board_vendor 2>/dev/null; then
    IS_ASUS=true
    echo "✓ ASUS hardware detected"
else
    echo "✓ Generic system detected"
fi

# Install essential sensors
if ! command -v sensors &>/dev/null; then
    echo "→ Installing lm-sensors..."
    if command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm lm_sensors >/dev/null 2>&1 && echo "✓ Sensors installed" || echo "⚠ Could not install sensors"
    fi
else
    echo "✓ Sensors already installed"
fi

echo ""
echo "→ Installing systemd services..."
if [ -f etc/systemd/annad.service ]; then
    sudo install -m 0644 -o root -g root etc/systemd/annad.service /etc/systemd/system/
    if [ -f etc/systemd/anna-fans.service ]; then
        sudo install -m 0644 -o root -g root etc/systemd/anna-fans.service /etc/systemd/system/
    fi
    sudo systemctl daemon-reload
    echo "✓ Services installed and systemd reloaded"
else
    echo "✗ Service files not found in etc/systemd/"
    echo "  Working directory: $(pwd)"
    exit 1
fi

# Install thermal management scripts
if [ "$IS_ASUS" = true ]; then
    sudo mkdir -p /usr/local/share/anna
    if [ -f scripts/anna_fans_asus.sh ]; then
        sudo cp scripts/anna_fans_asus.sh /usr/local/share/anna/
        sudo chmod +x /usr/local/share/anna/anna_fans_asus.sh
        echo "✓ Thermal management scripts installed"
    fi
fi

echo ""
echo "→ Enabling and starting daemon..."
if sudo systemctl enable --now annad; then
    sleep 3
    if systemctl is-active --quiet annad; then
        echo "✓ Daemon enabled and started successfully"
    else
        echo "⚠ Daemon may still be starting..."
        echo "  Check status: systemctl status annad"
        echo "  Check logs: journalctl -u annad -n 20"
    fi
else
    echo "✗ Failed to enable/start daemon"
    echo "  Check logs: journalctl -u annad -n 20"
    exit 1
fi

echo ""
echo "╭─────────────────────────────────────────╮"
echo "│  ✓ Anna Installed Successfully          │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Show installation method
case "$BUILD_MODE" in
    downloaded)
        echo "Installation method: Pre-compiled binaries (downloaded)"
        ;;
    preexisting)
        echo "Installation method: Pre-compiled binaries (from ./bin/)"
        ;;
    source)
        echo "Installation method: Built from source"
        ;;
esac

echo ""
echo "Next steps:"
echo ""
echo "  1. Run system health check:"
echo "     annactl doctor check"
echo ""
echo "  2. Let Anna help with system setup:"
echo "     annactl doctor setup"
echo ""
echo "  3. Check current status:"
echo "     annactl status"
echo ""
echo "Note: Log out and back in for full permissions"
echo ""
