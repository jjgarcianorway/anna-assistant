#!/usr/bin/env bash
set -euo pipefail

# Anna Installer v0.6.9 - Defaults to system mode; supports --user for dev/test.
FORCE_MODE="" REPAIR=0 NO_COMPILE=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --system)     FORCE_MODE="system"; shift;;
    --user)       FORCE_MODE="user"; shift;;
    --repair)     REPAIR=1; shift;;
    --no-compile) NO_COMPILE=1; shift;;
    --help)
      cat <<HELP
Anna Installer v0.6.9
Usage: $0 [OPTIONS]
Options:
  --system        Force system mode (default)
  --user          Force user mode (dev/test only)
  --repair        Re-run setup without clobbering data
  --no-compile    Skip cargo build if binaries already present
  --help          Show this help message
HELP
      exit 0;;
    *) echo "Unknown option: $1 (try --help)"; exit 1;;
  esac
done

detect_mode() {
  [[ -n "$FORCE_MODE" ]] && { echo "$FORCE_MODE"; return; }
  [[ -f /etc/systemd/system/annad.service || -d /var/lib/anna ]] && { echo "system"; return; }
  [[ -f ~/.config/systemd/user/annad.service || -d ~/.anna ]] && { echo "user"; return; }
  [[ "$EUID" -eq 0 || -n "${SUDO_USER:-}" ]] && { echo "system"; return; }
  echo "system"
}
INSTALL_MODE=$(detect_mode)

# Determine effective user (handle sudo case)
if [[ -n "${SUDO_USER:-}" && "${SUDO_USER}" != "root" ]]; then
  REAL_USER="$SUDO_USER"; REAL_UID=$(id -u "$SUDO_USER")
else
  REAL_USER="$USER"; REAL_UID=$(id -u)
fi

# Set paths based on install mode
if [[ "$INSTALL_MODE" == "system" ]]; then
  ANNAD="/usr/local/sbin/annad" ANNACTL="/usr/local/bin/annactl"
  UNIT="/etc/systemd/system/annad.service" DATA="/var/lib/anna" CONF="/etc/anna"
  SOCKET_DIR="/run/anna" SOCKET_PATH="$SOCKET_DIR/annad.sock" SYSTEMCTL_PREFIX=""
else
  ANNAD="$HOME/.local/bin/annad" ANNACTL="$HOME/.local/bin/annactl"
  UNIT="$HOME/.config/systemd/user/annad.service" DATA="$HOME/.anna/data" CONF="$HOME/.anna/config"
  SOCKET_DIR="${XDG_RUNTIME_DIR:-$HOME/.anna/run}/anna"
  SOCKET_PATH="$SOCKET_DIR/annad.sock" SYSTEMCTL_PREFIX="--user "
fi

# Colors (auto-off if not a TTY or NO_COLOR set)
if [[ -t 1 && -z "${NO_COLOR:-}" ]]; then
  b=$'\033[1m' dim=$'\033[2m' green=$'\033[32m' yellow=$'\033[33m' red=$'\033[31m' cyan=$'\033[36m' reset=$'\033[0m'
else
  b="" dim="" green="" yellow="" red="" cyan="" reset=""
fi
hr(){ printf "%s\n" "${dim}────────────────────────────────────────────────${reset}"; }
step(){ printf "› %s\n" "$*"; }
ok(){ printf "  ${green}+${reset} %s\n" "$*"; }
note(){ printf "  ${yellow}*${reset} %s\n" "$*"; }
info(){ printf "  ${cyan}i${reset}  %s\n" "$*"; }
fail(){ printf "  ${red}x${reset} %s\n" "$*"; }

printf "${b}Anna System Assistant - Installer v0.6.9${reset}\n"; hr
if [[ "$INSTALL_MODE" == "system" ]]; then
  info "Install mode: ${b}system${reset} (system-wide, /usr/local, systemd)"
  info "Effective user: ${REAL_USER} (UID ${REAL_UID})"
else
  info "Install mode: ${b}user${reset} (dev/test, ~/.anna, user systemd)"
  printf "${yellow}!  DEV USER MODE - Use --system for production${reset}\n"
fi
[[ $REPAIR -eq 1 ]] && note "Repair mode enabled (will not clobber existing data)"
hr

get_installed_version() { [[ -x "$ANNACTL" ]] && "$ANNACTL" --version 2>/dev/null | grep -oP 'annactl \K[0-9.]+' || echo "none"; }
get_source_version() { grep -oP '^version = "\K[^"]+' cmd/annactl/Cargo.toml 2>/dev/null || echo "unknown"; }
INSTALLED_VERSION=$(get_installed_version); SOURCE_VERSION=$(get_source_version)

# Preflight check
printf "${b}Preflight Check${reset}\n"
HAVE_UNIT=0; [[ -f "$UNIT" ]] && HAVE_UNIT=1
if [[ "$INSTALL_MODE" == "system" ]]; then
  ACTIVE=0; systemctl is-active --quiet annad 2>/dev/null && ACTIVE=1 || true
else
  ACTIVE=0; systemctl --user is-active --quiet annad 2>/dev/null && ACTIVE=1 || true
fi
HAVE_AD=0; [[ -x "$ANNAD" ]] && HAVE_AD=1
HAVE_CTL=0; [[ -x "$ANNACTL" ]] && HAVE_CTL=1
HAVE_DATA=0; [[ -d "$DATA" ]] && HAVE_DATA=1
HAVE_CONF=0; [[ -d "$CONF" ]] && HAVE_CONF=1

printf "  installed: %s | source: %s\n" "$INSTALLED_VERSION" "$SOURCE_VERSION"
printf "  annad: %s | annactl: %s | unit: %s | active: %s\n" \
  $([[ $HAVE_AD -eq 1 ]] && echo "yes" || echo "no") \
  $([[ $HAVE_CTL -eq 1 ]] && echo "yes" || echo "no") \
  $([[ $HAVE_UNIT -eq 1 ]] && echo "yes" || echo "no") \
  $([[ $ACTIVE -eq 1 ]] && echo "yes" || echo "no")
printf "  data: %s (%s) | conf: %s (%s)\n" "$DATA" $([[ $HAVE_DATA -eq 1 ]] && echo "ok" || echo "missing") \
  "$CONF" $([[ $HAVE_CONF -eq 1 ]] && echo "ok" || echo "missing")
hr

# Build logic
NEED_BUILD=1
if [[ $NO_COMPILE -eq 1 && -x "target/release/annad" && -x "target/release/annactl" ]]; then
  NEED_BUILD=0; note "Skipping build (--no-compile, binaries present)"
elif [[ $NO_COMPILE -eq 1 ]]; then
  note "--no-compile requested but binaries missing, building anyway"
fi

if [[ $NEED_BUILD -eq 1 ]]; then
  if [[ "$INSTALLED_VERSION" == "$SOURCE_VERSION" && $REPAIR -eq 0 ]]; then
    info "Already up to date (v${SOURCE_VERSION}). Skipping rebuild. Use --repair to force."
  else
    step "Building Anna v${SOURCE_VERSION}..."
    cargo build --release --quiet 2>&1 && ok "Build successful" || { fail "Build failed"; exit 1; }
  fi
fi

AD_SRC="target/release/annad"; CTL_SRC="target/release/annactl"
[[ -x "$AD_SRC" && -x "$CTL_SRC" ]] || { fail "Build artifacts not found"; exit 1; }

# Privileges and group management (system mode only)
if [[ "$INSTALL_MODE" == "system" ]]; then
  step "Requesting sudo privileges..."; sudo -v
  step "Ensuring system group 'anna' exists"
  getent group anna >/dev/null && note "Group 'anna' already exists" || { sudo groupadd anna; ok "Group 'anna' created"; }
  if id -nG "$REAL_USER" | tr ' ' '\n' | grep -qx "anna"; then
    note "User ${REAL_USER} already in group 'anna'"
  else
    sudo usermod -aG anna "$REAL_USER"; ok "Added ${REAL_USER} to group 'anna'"
    printf "${yellow}!  Re-login required for group membership to take effect${reset}\n"
  fi
fi

# Install binaries
step "Installing binaries"
if [[ "$INSTALL_MODE" == "system" ]]; then
  sudo install -Dm755 "$AD_SRC" "$ANNAD"; ok "annad -> $ANNAD"
  sudo install -Dm755 "$CTL_SRC" "$ANNACTL"; ok "annactl -> $ANNACTL"
else
  mkdir -p "$(dirname "$ANNAD")" "$(dirname "$ANNACTL")"
  install -Dm755 "$AD_SRC" "$ANNAD"; ok "annad -> $ANNAD"
  install -Dm755 "$CTL_SRC" "$ANNACTL"; ok "annactl -> $ANNACTL"
fi

# Setup directories and permissions
step "Setting up directories"
if [[ "$INSTALL_MODE" == "system" ]]; then
  sudo install -d -m755 "$CONF"; ok "$CONF (0755)"
  sudo install -d -m755 "$CONF/policy.d"; ok "$CONF/policy.d (0755)"
  sudo install -d -m755 "$DATA"; ok "$DATA (0755)"
  USER_DATA="$DATA/users/$REAL_UID"
  sudo install -d -o root -g anna -m2770 "$USER_DATA"; ok "$USER_DATA (2770 root:anna)"
  for subdir in reports advice persona signals profiles; do
    sudo install -d -o root -g anna -m2770 "$USER_DATA/$subdir"
  done; ok "Per-user subdirectories created"
  POLICY_FILE="$CONF/policy.d/${REAL_UID}.toml"
  if [[ ! -f "$POLICY_FILE" ]]; then
    sudo tee "$POLICY_FILE" >/dev/null <<'POLICY'
[level]
auto_apply = 1
[approval]
confirm_dangerous = true
prompt_style = "interactive"
POLICY
    sudo chmod 644 "$POLICY_FILE"; ok "Created default policy: $POLICY_FILE"
  else note "Policy already exists: $POLICY_FILE"; fi
else
  mkdir -p "$CONF" "$DATA" "$SOCKET_DIR" "$DATA"/{reports,advice,persona,signals,profiles}
  chmod 755 "$CONF" "$DATA" "$SOCKET_DIR"
  chmod 770 "$DATA"/{reports,advice,persona,signals,profiles}
  ok "User directories created at ~/.anna"
fi

# Install systemd unit
step "Installing systemd service"
if [[ "$INSTALL_MODE" == "system" ]]; then
  sudo tee "$UNIT" >/dev/null <<UNIT
[Unit]
Description=Anna System Assistant Daemon v0.6.7
After=network.target
[Service]
Type=simple
User=root
Group=anna
ExecStart=$ANNAD
Restart=always
RestartSec=3
RuntimeDirectory=anna
RuntimeDirectoryMode=0770
WorkingDirectory=$DATA
StandardOutput=journal
StandardError=journal
NoNewPrivileges=false
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=$DATA /run/anna
[Install]
WantedBy=multi-user.target
UNIT
  sudo systemctl daemon-reload; ok "System service installed: $UNIT"
else
  mkdir -p "$(dirname "$UNIT")"
  tee "$UNIT" >/dev/null <<UNIT
[Unit]
Description=Anna System Assistant Daemon v0.6.7 (User Mode)
After=default.target
[Service]
Type=simple
ExecStart=$ANNAD
Restart=always
RestartSec=3
WorkingDirectory=$DATA
Environment="ANNA_MODE=user"
Environment="XDG_RUNTIME_DIR=%t"
StandardOutput=journal
StandardError=journal
[Install]
WantedBy=default.target
UNIT
  systemctl --user daemon-reload; ok "User service installed: $UNIT"
fi

# Enable and start service
step "Enabling and starting service"
if [[ "$INSTALL_MODE" == "system" ]]; then
  if systemctl is-enabled --quiet annad 2>/dev/null; then
    sudo systemctl restart annad; ok "Service restarted"
  else sudo systemctl enable --now annad; ok "Service enabled and started"; fi
  sleep 2
  [[ -S "$SOCKET_PATH" ]] && ok "Socket created: $SOCKET_PATH" || { fail "Socket not found at $SOCKET_PATH"; note "Check: systemctl status annad"; }
else
  systemctl --user daemon-reload
  if systemctl --user is-enabled --quiet annad 2>/dev/null; then
    systemctl --user restart annad; ok "Service restarted"
  else systemctl --user enable --now annad; ok "Service enabled and started"; fi
  info "Waiting for socket..."
  SOCKET_FOUND=0
  for _ in {1..10}; do [[ -S "$SOCKET_PATH" ]] && { SOCKET_FOUND=1; break; }; sleep 0.5; done
  [[ $SOCKET_FOUND -eq 1 ]] && ok "Socket created: $SOCKET_PATH" || {
    fail "Socket not found at $SOCKET_PATH"
    systemctl --user status annad --no-pager -n 5 || true
    [[ -z "${XDG_RUNTIME_DIR:-}" ]] && note "XDG_RUNTIME_DIR is not set - check ~/.anna/run/annad.sock"
  }
fi

hr; printf "${b}Installation Complete${reset}\n"
printf "  Install mode: %s | Version: %s | Socket: %s\n" "$INSTALL_MODE" "$SOURCE_VERSION" "$SOCKET_PATH"
[[ "$INSTALL_MODE" == "system" ]] && printf "  User data: %s/users/%s | Policy: %s/policy.d/%s.toml\n" "$DATA" "$REAL_UID" "$CONF" "$REAL_UID"
hr

# Post-install quickcheck
step "Running post-install checks..."
"$ANNACTL" status >/dev/null 2>&1 && ok "annactl status" || fail "annactl status failed"
"$ANNACTL" doctor perms >/dev/null 2>&1 && ok "annactl doctor perms" || { fail "annactl doctor perms found issues"; note "Run: annactl doctor perms"; }

if [[ "$INSTALL_MODE" == "system" ]]; then
  if timeout 5s "$ANNACTL" quickscan >/dev/null 2>&1; then ok "annactl quickscan"
  else
    note "annactl quickscan timed out or failed"
    id -nG "$REAL_USER" | tr ' ' '\n' | grep -qx "anna" || {
      printf "${yellow}!  User ${REAL_USER} was just added to 'anna' group. Re-login to apply.${reset}\n"
    }
  fi
fi

hr; printf "${green}Installation successful!${reset}\n\n"
printf "Next steps:\n"
if [[ "$INSTALL_MODE" == "system" ]]; then
  printf "  1. If group was just added: log out and log back in\n"
  printf "  2. annactl status | 3. annactl quickscan | 4. annactl advice list\n"
else
  printf "  1. annactl status | 2. annactl quickscan | 3. annactl advice list\n"
fi
printf "\nDocumentation: docs/installer.md\n"
