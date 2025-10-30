#!/usr/bin/env bash
set -Eeuo pipefail

# Simple, working Anna Assistant installer
# Run as user: ./scripts/install_simple.sh

echo "Anna Assistant Installer"
echo "========================"
echo ""

# Check we're in project root
if [[ ! -f Cargo.toml ]]; then
    echo "ERROR: Run from anna-assistant project root"
    exit 1
fi

# Build
echo "Building binaries..."
cargo build --release || { echo "Build failed"; exit 1; }
echo "✓ Build complete"
echo ""

# Install function
install_with_sudo() {
    echo "$1"
    echo "This requires administrator privileges."
    sudo "$@" || { echo "Installation failed"; exit 1; }
}

# Install binaries
echo "Installing to /usr/local/bin..."
install_with_sudo "Installing annad..." install -m 755 target/release/annad /usr/local/bin/
install_with_sudo "Installing annactl..." install -m 755 target/release/annactl /usr/local/bin/
echo "✓ Binaries installed"
echo ""

# Verify
if command -v annactl &>/dev/null; then
    echo "✓ Installation successful!"
    echo ""
    echo "Try: annactl --help"
else
    echo "ERROR: annactl not in PATH"
    exit 1
fi
