#!/usr/bin/env bash
# Anna Installer — zero-arg
# Downloads tarball from GitHub releases (including prereleases), verifies checksum
# Falls back to local build with Rust toolchain auto-install

set -Eeuo pipefail

OWNER="jjgarcianorway"
REPO="anna-assistant"
BIN_DIR="/usr/local/bin"
SERVICE="annad"
TMPDIR="$(mktemp -d)"

cleanup() { rm -rf "$TMPDIR"; }
trap cleanup EXIT

ensure_pkg() {
  local pkg="$1"
  if ! command -v "$pkg" >/dev/null 2>&1; then
    echo "→ Installing missing dependency: $pkg"
    if command -v pacman >/dev/null 2>&1; then
      sudo pacman -Sy --noconfirm "$pkg"
    elif command -v apt >/dev/null 2>&1; then
      sudo apt update && sudo apt install -y "$pkg"
    elif command -v dnf >/dev/null 2>&1; then
      sudo dnf install -y "$pkg"
    elif command -v zypper >/dev/null 2>&1; then
      sudo zypper install -y "$pkg"
    else
      echo "✗ Unsupported package manager; install '$pkg' manually"
      exit 1
    fi
  fi
}

ensure_rust_toolchain() {
  if command -v cargo >/dev/null 2>&1; then
    return 0
  fi

  echo "→ Installing Rust toolchain for fallback build"
  if command -v pacman >/dev/null 2>&1; then
    ensure_pkg base-devel
    # Install rust package which provides cargo (don't install rustup, they conflict)
    ensure_pkg rust
  elif command -v apt >/dev/null 2>&1; then
    sudo apt update
    sudo apt install -y build-essential cargo rustc
  elif command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y cargo rust
  elif command -v zypper >/dev/null 2>&1; then
    sudo zypper install -y cargo rust
  else
    echo "✗ Cannot install Rust toolchain on this distro"
    exit 1
  fi
}

# Required tools - auto-install if missing
for dep in curl jq sudo systemctl; do
  ensure_pkg "$dep"
done

say() { printf "%s\n" "$*"; }
title() {
  printf "╭─────────────────────────────────────────╮\n"
  printf "│  Anna Assistant Installer              │\n"
  printf "│  Event-Driven Intelligence             │\n"
  printf "╰─────────────────────────────────────────╯\n"
}

select_release() {
  local api="https://api.github.com/repos/$OWNER/$REPO/releases?per_page=15"
  # Get latest release OR prerelease (exclude only drafts)
  # Sort by version number to get true latest
  curl -fsSL "$api" | jq -r '.[] | select(.draft==false) | .tag_name' | sort -V | tail -n1
}

download_and_verify_tarball() {
  local tag="$1"
  local api="https://api.github.com/repos/$OWNER/$REPO/releases/tags/$tag"
  local tmp="$TMPDIR/anna"
  mkdir -p "$tmp"

  say "→ Fetching assets for $tag…"
  local assets
  assets=$(curl -fsSL "$api")

  local tar_url checksum_url
  tar_url=$(echo "$assets" | jq -r '.assets[] | select(.name=="anna-linux-x86_64.tar.gz") | .browser_download_url')
  checksum_url=$(echo "$assets" | jq -r '.assets[] | select(.name=="anna-linux-x86_64.tar.gz.sha256") | .browser_download_url')

  if [[ -z "$tar_url" || "$tar_url" == "null" || -z "$checksum_url" || "$checksum_url" == "null" ]]; then
    say "✗ Tarball assets not found for $tag"
    return 1
  fi

  say "→ Downloading tarball and checksum…"
  if ! curl -fsSL "$tar_url" -o "$tmp/anna-linux-x86_64.tar.gz"; then
    say "✗ Failed to download tarball"
    return 1
  fi

  if ! curl -fsSL "$checksum_url" -o "$tmp/anna-linux-x86_64.tar.gz.sha256"; then
    say "✗ Failed to download checksum"
    return 1
  fi

  # Verify files were actually downloaded
  if [[ ! -s "$tmp/anna-linux-x86_64.tar.gz" || ! -s "$tmp/anna-linux-x86_64.tar.gz.sha256" ]]; then
    say "✗ Downloaded files are empty"
    return 1
  fi

  say "→ Verifying checksum…"
  if ! (cd "$tmp" && sha256sum -c anna-linux-x86_64.tar.gz.sha256 2>&1); then
    say "✗ Checksum verification failed"
    say "  Checksum file contents:"
    cat "$tmp/anna-linux-x86_64.tar.gz.sha256" | head -3
    return 1
  fi

  say "→ Extracting tarball…"
  tar -xzf "$tmp/anna-linux-x86_64.tar.gz" -C "$tmp"

  if [[ ! -f "$tmp/annad" || ! -f "$tmp/annactl" ]]; then
    say "✗ Binaries not found in tarball"
    return 1
  fi

  sudo install -m 0755 "$tmp/annad" "$BIN_DIR/annad"
  sudo install -m 0755 "$tmp/annactl" "$BIN_DIR/annactl"
  say "→ Installed binaries from GitHub release $tag"

  echo "$tag" > "$TMPDIR/installed_tag"
  return 0
}

build_local() {
  say "→ Building from source (fallback)"
  ensure_rust_toolchain

  cargo build --release --bin annad --bin annactl
  sudo install -m 0755 target/release/annad "$BIN_DIR/annad"
  sudo install -m 0755 target/release/annactl "$BIN_DIR/annactl"
  say "→ Installed binaries from local build"

  # Record version from Cargo.toml
  local version
  version=$(grep -m1 '^version = "' Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')
  echo "v$version" > "$TMPDIR/installed_tag"
}

configure_systemd() {
  say "→ Configuring systemd service…"

  if [[ ! -f /etc/systemd/system/annad.service ]]; then
    sudo tee /etc/systemd/system/annad.service > /dev/null <<'EOF'
[Unit]
Description=Anna Assistant Daemon
Documentation=https://github.com/jjgarcianorway/anna-assistant
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/annad
Restart=on-failure
RestartSec=10s

# Runtime directory
RuntimeDirectory=anna
RuntimeDirectoryMode=0750

# User/Group
User=anna
Group=anna
UMask=0000

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/anna

# Resource limits
MemoryMax=100M
TasksMax=100

[Install]
WantedBy=multi-user.target
EOF
    say "→ Created service file"
  fi

  # Create anna user/group if needed
  if ! id anna &>/dev/null; then
    sudo useradd -r -s /usr/bin/nologin anna
    say "→ Created anna user"
  fi

  # Create directories with correct permissions
  sudo mkdir -p /var/lib/anna /run/anna
  sudo chown anna:anna /var/lib/anna /run/anna
  sudo chmod 0770 /var/lib/anna /run/anna

  # Add current user to anna group for socket access
  if ! groups | grep -q anna; then
    sudo usermod -aG anna "$USER"
    say "→ Added $USER to anna group (logout/login for effect)"
  fi

  # Restart daemon
  sudo systemctl daemon-reload
  sudo systemctl enable --now "$SERVICE"
  sudo systemctl restart "$SERVICE"
  say "→ Service restarted"
}

wait_rpc() {
  say "→ Waiting for RPC socket…"
  for i in {1..20}; do
    if timeout 2 "$BIN_DIR/annactl" version >/dev/null 2>&1; then
      say "✓ Daemon responding and CLI reachable"
      return 0
    fi
    sleep 0.5
  done

  echo "✗ Daemon active but RPC not responding"
  echo "  Check logs: sudo journalctl -u $SERVICE -n 50 --no-pager"
  echo "  Check socket: ls -la /run/anna/annad.sock"
  echo "  Check permissions: groups (should include 'anna')"
  return 1
}

verify_versions() {
  local expected_tag="$1"
  local annactl_ver

  say "→ Verifying installed versions…"

  # Extract version from annactl (talks to running daemon via RPC)
  annactl_ver=$("$BIN_DIR/annactl" version 2>/dev/null | head -1 | grep -oP 'v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?' || echo "unknown")

  echo ""
  echo "→ Installed version: $annactl_ver"
  echo "   Expected:         $expected_tag"

  if [[ "$annactl_ver" == "$expected_tag" ]]; then
    say "✓ Version verification passed"
    return 0
  else
    echo "✗ Version mismatch detected"
    echo "  Expected $expected_tag but got $annactl_ver"
    echo "  This may indicate a build or release issue"
    echo "  Continuing anyway, but please report this"
    return 0  # Don't fail install, just warn
  fi
}

# Main installation flow
title

TAG=$(select_release)
if [[ -z "$TAG" || "$TAG" == "null" ]]; then
  echo "✗ No releases found on GitHub"
  build_local
else
  say "→ Latest release: $TAG"
  if ! download_and_verify_tarball "$TAG"; then
    say "→ Tarball download failed, falling back to local build"
    build_local
  fi
fi

configure_systemd

if ! wait_rpc; then
  exit 2
fi

# Get the tag we installed
INSTALLED_TAG=$(cat "$TMPDIR/installed_tag" 2>/dev/null || echo "unknown")
verify_versions "$INSTALLED_TAG"

echo ""
echo "✓ Installation complete"
echo ""
echo "Next steps:"
echo "  annactl status"
echo "  annactl advise --limit 5"
echo "  annactl report"
