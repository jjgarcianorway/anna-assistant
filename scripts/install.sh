#!/usr/bin/env bash
# Anna Installer — zero-arg
# Fetches newest release including prereleases, falls back to local build
# Self-healing dependencies, 0770 socket permissions

set -Eeuo pipefail

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
      echo "✗ Unsupported package manager; install '$pkg' manually."
      exit 1
    fi
  fi
}

# Required tools - auto-install if missing
for dep in curl jq sudo systemctl; do
  ensure_pkg "$dep"
done

OWNER="jjgarcianorway"
REPO="anna-assistant"
BIN_DIR="/usr/local/bin"
SERVICE="annad"
DO_LOCAL_BUILD=""

say() { printf "%s\n" "$*"; }
title() {
  printf "╭─────────────────────────────────────────╮\n"
  printf "│  Anna Assistant Installer              │\n"
  printf "│  Event-Driven Intelligence             │\n"
  printf "╰─────────────────────────────────────────╯\n"
}

title

# Resolve newest release including prereleases (non-draft)
LATEST_JSON="$(curl -fsSL "https://api.github.com/repos/$OWNER/$REPO/releases?per_page=15")" || {
  echo "ERROR: unable to query releases; falling back to local build"
  DO_LOCAL_BUILD=1
}

if [ -z "$DO_LOCAL_BUILD" ]; then
  TAG="$(echo "$LATEST_JSON" | jq -r '[.[] | select(.draft==false)][0].tag_name')"
  ASSETS_URL="$(echo "$LATEST_JSON" | jq -r '[.[] | select(.draft==false)][0].assets_url')"
fi

if [ -z "$TAG" ] || [ "$TAG" = "null" ] || [ -z "$ASSETS_URL" ] || [ "$ASSETS_URL" = "null" ]; then
  echo "ERROR: no suitable release found; falling back to local build"
  DO_LOCAL_BUILD=1
fi

if [ -z "$DO_LOCAL_BUILD" ]; then
  echo "→ Latest release: $TAG"
  ASSETS="$(curl -fsSL "$ASSETS_URL")" || ASSETS=""
  URL_ANNAD="$(echo "$ASSETS" | jq -r '.[] | select(.name=="annad") | .browser_download_url')"
  URL_ANNACTL="$(echo "$ASSETS" | jq -r '.[] | select(.name=="annactl") | .browser_download_url')"

  if [ -z "$URL_ANNAD" ] || [ "$URL_ANNAD" = "null" ] || [ -z "$URL_ANNACTL" ] || [ "$URL_ANNACTL" = "null" ]; then
    echo "ERROR: no assets found for $TAG; falling back to local build"
    DO_LOCAL_BUILD=1
  fi
fi

if [ -z "$DO_LOCAL_BUILD" ]; then
  say "→ Downloading binaries from $TAG…"
  tmpdir="$(mktemp -d)"
  curl -fsSL "$URL_ANNAD"   -o "$tmpdir/annad"   || DO_LOCAL_BUILD=1
  curl -fsSL "$URL_ANNACTL" -o "$tmpdir/annactl" || DO_LOCAL_BUILD=1

  if [ -z "$DO_LOCAL_BUILD" ]; then
    sudo install -m 0755 "$tmpdir/annad"   "$BIN_DIR/annad"
    sudo install -m 0755 "$tmpdir/annactl" "$BIN_DIR/annactl"
    rm -rf "$tmpdir"
    say "→ Installed binaries from GitHub release"
  else
    rm -rf "$tmpdir"
  fi
fi

if [ -n "$DO_LOCAL_BUILD" ]; then
  say "→ Building from source (fallback)"
  cargo build --release --bin annad --bin annactl
  sudo install -m 0755 target/release/annad   "$BIN_DIR/annad"
  sudo install -m 0755 target/release/annactl "$BIN_DIR/annactl"
  say "→ Installed binaries from local build"
fi

# Configure systemd service
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

# Wait for RPC socket (10s timeout)
say "→ Waiting for RPC socket…"
for i in {1..20}; do
  if timeout 2 "$BIN_DIR/annactl" version >/dev/null 2>&1; then
    say "✓ Daemon responding and CLI reachable"
    break
  fi
  sleep 0.5
  if [ $i -eq 20 ]; then
    echo "✗ Daemon active but RPC not responding"
    echo "  Check logs: sudo journalctl -u $SERVICE -n 50 --no-pager"
    echo "  Check socket: ls -la /run/anna/annad.sock"
    echo "  Check permissions: groups (should include 'anna')"
    exit 2
  fi
done

echo ""
echo "→ Installed versions:"
echo "   annad:   $("$BIN_DIR/annad" --version 2>/dev/null || echo 'n/a')"
echo "   annactl: $("$BIN_DIR/annactl" version 2>/dev/null || echo 'n/a')"
echo ""
echo "✓ Installation complete"
echo ""
echo "Next steps:"
echo "  annactl status"
echo "  annactl advise --limit 5"
echo "  annactl report"
