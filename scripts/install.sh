#!/usr/bin/env bash
# Anna Installer — zero-arg
# Always fetch latest GitHub release assets unless --from-local is passed
# Verifies versions, restarts daemon, waits for RPC

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

ORG="jjgarcianorway"
REPO="anna-assistant"
BIN_DIR="/usr/local/bin"
SERVICE="annad"
TMP="$(mktemp -d)"
FROM_LOCAL=false

cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

say() { printf "%s\n" "$*"; }
title() {
  printf "╭─────────────────────────────────────────╮\n"
  printf "│  Anna Assistant Installer              │\n"
  printf "│  Event-Driven Intelligence             │\n"
  printf "╰─────────────────────────────────────────╯\n"
}

# Check for --from-local flag
while [[ $# -gt 0 ]]; do
  case $1 in
    --from-local)
      FROM_LOCAL=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--from-local]"
      exit 1
      ;;
  esac
done

get_latest_tag() {
  curl -sSfL "https://api.github.com/repos/${ORG}/${REPO}/releases/latest" | jq -r '.tag_name'
}

download_release() {
  local tag="$1"
  say "→ Fetching assets for ${tag}…"
  curl -sSfL "https://api.github.com/repos/${ORG}/${REPO}/releases/tags/${tag}" > "$TMP/meta.json"

  local annad_url annactl_url
  annad_url=$(jq -r '.assets[] | select(.name|test("^annad-.*-x86_64$")) | .browser_download_url' "$TMP/meta.json")
  annactl_url=$(jq -r '.assets[] | select(.name|test("^annactl-.*-x86_64$")) | .browser_download_url' "$TMP/meta.json")

  if [[ -z "$annad_url" || -z "$annactl_url" ]]; then
    echo "ERROR: No assets found for ${tag}"
    echo "This usually means the GitHub Actions build hasn't completed yet."
    echo "Wait a few minutes and try again, or build from source with:"
    echo "  cargo build --release"
    echo "  sudo install -m 0755 target/release/annad ${BIN_DIR}/annad"
    echo "  sudo install -m 0755 target/release/annactl ${BIN_DIR}/annactl"
    exit 1
  fi

  say "→ Downloading binaries…"
  curl -sSfL "$annad_url" -o "$TMP/annad"
  curl -sSfL "$annactl_url" -o "$TMP/annactl"
  chmod +x "$TMP/annad" "$TMP/annactl"
}

build_local() {
  say "→ Building from local source…"
  cargo build --release --bin annad --bin annactl
  cp target/release/annad "$TMP/annad"
  cp target/release/annactl "$TMP/annactl"
  chmod +x "$TMP/annad" "$TMP/annactl"
}

install_binaries() {
  say "→ Installing binaries to ${BIN_DIR}…"
  sudo install -m 0755 "$TMP/annad"   "${BIN_DIR}/annad"
  sudo install -m 0755 "$TMP/annactl" "${BIN_DIR}/annactl"
}

ensure_systemd() {
  say "→ Configuring systemd service…"

  # Create service file if it doesn't exist
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

# User/Group (create if needed)
User=anna
Group=anna
UMask=0027

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

  # Create directories
  sudo mkdir -p /var/lib/anna /run/anna
  sudo chown anna:anna /var/lib/anna /run/anna
  sudo chmod 0750 /var/lib/anna /run/anna

  # Add current user to anna group for socket access
  if ! groups | grep -q anna; then
    sudo usermod -aG anna "$USER"
    say "→ Added $USER to anna group (logout/login for effect)"
  fi

  sudo systemctl daemon-reload
  sudo systemctl enable --now "${SERVICE}"
  sudo systemctl restart "${SERVICE}"
  say "→ Service restarted"
}

wait_rpc() {
  say "→ Waiting for RPC socket…"
  # Wait up to 10s for RPC socket to respond via annactl
  for i in {1..20}; do
    if timeout 2 annactl version >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.5
  done
  return 1
}

title

if [[ "$FROM_LOCAL" == "true" ]]; then
  build_local
else
  TAG="$(get_latest_tag)"
  say "→ Latest release: ${TAG}"
  download_release "$TAG"
fi

install_binaries
ensure_systemd

if wait_rpc; then
  say "✓ Daemon responding and CLI reachable"
else
  echo "✗ Daemon active but RPC not responding"
  echo "  Check logs: sudo journalctl -u ${SERVICE} -n 50 --no-pager"
  echo "  Check socket: ls -la /run/anna/annad.sock"
  echo "  Check permissions: groups (should include 'anna')"
  exit 2
fi

echo ""
echo "→ Installed versions:"
echo "   annad:   $(${BIN_DIR}/annad --version 2>/dev/null || echo 'n/a')"
echo "   annactl: $(${BIN_DIR}/annactl version 2>/dev/null || echo 'n/a')"
echo ""
echo "✓ Installation complete"
echo ""
echo "Next steps:"
echo "  annactl status"
echo "  annactl advise --limit 5"
echo "  annactl report"
