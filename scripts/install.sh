#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant Installer
# Run as user: ./scripts/install.sh

echo "═══════════════════════════════════════"
echo "  Anna Assistant Installer"
echo "  Version: 0.9.6-alpha.5"
echo "═══════════════════════════════════════"
echo ""

# Check we're in project root
if [[ ! -f Cargo.toml ]]; then
    echo "ERROR: Please run from anna-assistant project root"
    exit 1
fi

echo "→ Building binaries..."
if cargo build --release --quiet; then
    echo "  ✓ Build complete"
else
    echo "  ✗ Build failed"
    exit 1
fi
echo ""

echo "→ Installing to /usr/local/bin..."
echo "  (will ask for password)"
echo ""

if sudo install -m 755 target/release/annad /usr/local/bin/ && \
   sudo install -m 755 target/release/annactl /usr/local/bin/; then
    echo "  ✓ annad installed"
    echo "  ✓ annactl installed"
else
    echo "  ✗ Installation failed"
    exit 1
fi
echo ""

# Verify
if command -v annactl &>/dev/null; then
    echo "═══════════════════════════════════════"
    echo "  ✓ Installation successful!"
    echo "═══════════════════════════════════════"
    echo ""
    echo "Try these commands:"
    echo "  annactl --help"
    echo "  annactl config list"
    echo "  annactl persona list"
    echo "  annactl profile show"
    echo ""
else
    echo "WARNING: annactl not found in PATH"
    echo "You may need to add /usr/local/bin to your PATH"
    exit 1
fi
