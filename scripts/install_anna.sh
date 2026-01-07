#!/usr/bin/env bash
set -euo pipefail

# Anna Installer v0.6.9
# Defaults to system mode; supports --user for dev/test.

# Parse arguments
FORCE_MODE=""
REPAIR=0
NO_COMPILE=0
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

Default behavior:
  - Installs system-wide (/usr/local/bin, /etc/systemd/system)
  - Creates 'anna' group and adds current user
  - Sets up per-user data under /var/lib/anna/users/<uid>
  - Creates default policy at /etc/anna/policy.d/<uid>.toml
  - Starts and enables systemd service

HELP
      exit 0
      ;;
    *) echo "Unknown option: $1 (try --help)"; exit 1;;
  esac
done

# Detect install mode
detect_mode() {
  # If force flag set, use that
  if [[ -n "$FORCE_MODE" ]]; then
    echo "$FORCE_MODE"
    return
  fi

  # Check for existing installation
  if [[ -f /etc/systemd/system/annad.service ]] || [[ -d /var/lib/anna ]]; then
    echo "system"
    return
  fi

  if [[ -f ~/.config/systemd/user/annad.service ]] || [[ -d ~/.anna ]]; then
    echo "user"
    return
  fi

  # Default to system mode if running with sudo/root
  if [[ "$EUID" -eq 0 || -n "${SUDO_USER:-}" ]]; then
    echo "system"
    return
  fi

  # Default to system mode (will prompt for sudo when needed)
  echo "system"
}

INSTALL_MODE=$(detect_mode)

# Determine effective user (handle sudo case)
if [[ -n "${SUDO_USER:-}" && "${SUDO_USER}" != "root" ]]; then
  REAL_USER="$SUDO_USER"
  REAL_UID=$(id -u "$SUDO_USER")
else
  REAL_USER="$USER"
  REAL_UID=$(id -u)
fi

# Set paths based on install mode
if [[ "$INSTALL_MODE" == "system" ]]; then
  ANNAD="/usr/local/sbin/annad"
  ANNACTL="/usr/local/bin/annactl"
  UNIT="/etc/systemd/system/annad.service"
  DATA="/var/lib/anna"
  CONF="/etc/anna"
  SOCKET_DIR="/run/anna"
  SOCKET_PATH="$SOCKET_DIR/annad.sock"
  SYSTEMCTL_PREFIX=""
else
  ANNAD="$HOME/.local/bin/annad"
  ANNACTL="$HOME/.local/bin/annactl"
  UNIT="$HOME/.config/systemd/user/annad.service"
  DATA="$HOME/.anna/data"
  CONF="$HOME/.anna/config"
  # Prefer XDG_RUNTIME_DIR, fallback to ~/.anna/run
  if [[ -n "${XDG_RUNTIME_DIR:-}" ]]; then
    SOCKET_DIR="$XDG_RUNTIME_DIR/anna"
  else
    SOCKET_DIR="$HOME/.anna/run"
  fi
  SOCKET_PATH="$SOCKET_DIR/annad.sock"
  SYSTEMCTL_PREFIX="--user "
fi

# Colors (auto-off if not a TTY or NO_COLOR set)
if [[ -t 1 && -z "${NO_COLOR:-}" ]]; then
  b=$'\033[1m'; dim=$'\033[2m'; blue=$'\033[34m'; cyan=$'\033[36m'; green=$'\033[32m'
  yellow=$'\033[33m'; red=$'\033[31m'; reset=$'\033[0m'
else
  b=""; dim=""; blue=""; cyan=""; green=""; yellow=""; red=""; reset=""
fi
hr(){ printf "%s\n" "${dim}────────────────────────────────────────────────${reset}"; }
step(){ printf "› %s\n" "$*"; }
ok(){ printf "  ${green}✔${reset} %s\n" "$*"; }
note(){ printf "  ${yellow}•${reset} %s\n" "$*"; }
info(){ printf "  ${cyan}ℹ${reset}  %s\n" "$*"; }
fail(){ printf "  ${red}✖${reset} %s\n" "$*"; }

# Banner
printf "${b}Anna System Assistant — Installer v0.6.9${reset}\n"
hr

# Show install mode
if [[ "$INSTALL_MODE" == "system" ]]; then
  info "Install mode: ${b}system${reset} (system-wide, /usr/local, systemd)"
  info "Effective user: ${REAL_USER} (UID ${REAL_UID})"
else
  info "Install mode: ${b}user${reset} (dev/test, ~/.anna, user systemd)"
  printf "${yellow}⚠  DEV USER MODE — Use --system for production${reset}\n"
fi

if [[ $REPAIR -eq 1 ]]; then
  note "Repair mode enabled (will not clobber existing data)"
fi

hr

# Version detection
get_installed_version() {
  if [[ -x "$ANNACTL" ]]; then
    "$ANNACTL" --version 2>/dev/null | grep -oP 'annactl \K[0-9.]+' || echo "unknown"
  else
    echo "none"
  fi
}

get_source_version() {
  grep -oP '^version = "\K[^"]+' cmd/annactl/Cargo.toml 2>/dev/null || echo "unknown"
}

INSTALLED_VERSION=$(get_installed_version)
SOURCE_VERSION=$(get_source_version)

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

printf "  installed version:  %s\n" "$INSTALLED_VERSION"
printf "  source version:     %s\n" "$SOURCE_VERSION"
printf "  annad binary:       %s\n" $([[ $HAVE_AD -eq 1 ]] && echo "present" || echo "missing")
printf "  annactl binary:     %s\n" $([[ $HAVE_CTL -eq 1 ]] && echo "present" || echo "missing")
printf "  service file:       %s\n" $([[ $HAVE_UNIT -eq 1 ]] && echo "present" || echo "missing")
printf "  service active:     %s\n" $([[ $ACTIVE -eq 1 ]] && echo "yes" || echo "no")
printf "  data dir:           %s %s\n" "$DATA" $([[ $HAVE_DATA -eq 1 ]] && echo "(present)" || echo "(missing)")
printf "  conf dir:           %s %s\n" "$CONF" $([[ $HAVE_CONF -eq 1 ]] && echo "(present)" || echo "(missing)")
hr

# Skip build if --no-compile and binaries exist
NEED_BUILD=1
if [[ $NO_COMPILE -eq 1 ]]; then
  if [[ -x "target/release/annad" && -x "target/release/annactl" ]]; then
    NEED_BUILD=0
    note "Skipping build (--no-compile, binaries present)"
  else
    note "--no-compile requested but binaries missing, building anyway"
  fi
fi

# Build if needed
if [[ $NEED_BUILD -eq 1 ]]; then
  if [[ "$INSTALLED_VERSION" == "$SOURCE_VERSION" && $REPAIR -eq 0 ]]; then
    info "Already up to date (v${SOURCE_VERSION}). Skipping rebuild."
    info "Use --repair to force reinstall."
  else
    step "Building Anna v${SOURCE_VERSION}…"
    if cargo build --release --quiet 2>&1; then
      ok "Build successful"
    else
      fail "Build failed"
      exit 1
    fi
  fi
fi

AD_SRC="target/release/annad"
CTL_SRC="target/release/annactl"
if [[ ! -x "$AD_SRC" || ! -x "$CTL_SRC" ]]; then
  fail "Build artifacts not found at target/release/{annad,annactl}"
  exit 1
fi

# Privileges and group management (system mode only)
if [[ "$INSTALL_MODE" == "system" ]]; then
  step "Requesting sudo privileges…"
  sudo -v

  step "Ensuring system group 'anna' exists"
  if getent group anna >/dev/null; then
    note "Group 'anna' already exists"
  else
    sudo groupadd anna
    ok "Group 'anna' created"
  fi

  # Add effective user to anna group
  if id -nG "$REAL_USER" | tr ' ' '\n' | grep -qx "anna"; then
    note "User ${REAL_USER} already in group 'anna'"
  else
    sudo usermod -aG anna "$REAL_USER"
    ok "Added ${REAL_USER} to group 'anna'"
    printf "${yellow}⚠  Re-login required for group membership to take effect${reset}\n"
  fi
fi

# Install binaries
step "Installing binaries"
if [[ "$INSTALL_MODE" == "system" ]]; then
  sudo install -Dm755 "$AD_SRC" "$ANNAD"
  ok "annad → $ANNAD"
  sudo install -Dm755 "$CTL_SRC" "$ANNACTL"
  ok "annactl → $ANNACTL"
else
  mkdir -p "$(dirname "$ANNAD")" "$(dirname "$ANNACTL")"
  install -Dm755 "$AD_SRC" "$ANNAD"
  ok "annad → $ANNAD"
  install -Dm755 "$CTL_SRC" "$ANNACTL"
  ok "annactl → $ANNACTL"
fi

# Setup directories and permissions
step "Setting up directories"
if [[ "$INSTALL_MODE" == "system" ]]; then
  # System config directory
  sudo install -d -m755 "$CONF"
  ok "$CONF (0755)"

  # Policy directory
  sudo install -d -m755 "$CONF/policy.d"
  ok "$CONF/policy.d (0755)"

  # System data root
  sudo install -d -m755 "$DATA"
  ok "$DATA (0755)"

  # Per-user data directory
  USER_DATA="$DATA/users/$REAL_UID"
  sudo install -d -o root -g anna -m2770 "$USER_DATA"
  ok "$USER_DATA (2770 root:anna)"

  # Subdirectories for user
  for subdir in reports advice persona signals profiles; do
    sudo install -d -o root -g anna -m2770 "$USER_DATA/$subdir"
  done
  ok "Per-user subdirectories created"

  # Create default policy if missing
  POLICY_FILE="$CONF/policy.d/${REAL_UID}.toml"
  if [[ ! -f "$POLICY_FILE" ]]; then
    sudo tee "$POLICY_FILE" >/dev/null <<'POLICY'
# Anna Policy Configuration
# Policy level: 0=Manual, 1=SafeMaintenance, 2=SafeModerate, 3=FullAutonomy

[level]
auto_apply = 1  # SafeMaintenance (cache cleanup, orphan packages, journal trim)

[approval]
confirm_dangerous = true       # Always confirm dangerous operations
prompt_style = "interactive"   # interactive | silent | requiresudo
POLICY
    sudo chmod 644 "$POLICY_FILE"
    ok "Created default policy: $POLICY_FILE"
  else
    note "Policy already exists: $POLICY_FILE"
  fi
else
  # User mode directories
  mkdir -p "$CONF" "$DATA" "$SOCKET_DIR"
  mkdir -p "$DATA"/{reports,advice,persona,signals,profiles}
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

# Security restrictions
NoNewPrivileges=false
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=$DATA /run/anna

[Install]
WantedBy=multi-user.target
UNIT
  sudo systemctl daemon-reload
  ok "System service installed: $UNIT"
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
  systemctl --user daemon-reload
  ok "User service installed: $UNIT"
fi

# Enable and start service
step "Enabling and starting service"
if [[ "$INSTALL_MODE" == "system" ]]; then
  if systemctl is-enabled --quiet annad 2>/dev/null; then
    sudo systemctl restart annad
    ok "Service restarted"
  else
    sudo systemctl enable --now annad
    ok "Service enabled and started"
  fi

  # Wait for socket to appear
  sleep 2
  if [[ -S "$SOCKET_PATH" ]]; then
    ok "Socket created: $SOCKET_PATH"
  else
    fail "Socket not found at $SOCKET_PATH"
    note "Check: systemctl status annad"
    note "Check: journalctl -u annad -n 50"
  fi
else
  # User mode - ensure daemon-reload happens first
  systemctl --user daemon-reload

  if systemctl --user is-enabled --quiet annad 2>/dev/null; then
    systemctl --user restart annad
    ok "Service restarted"
  else
    systemctl --user enable --now annad
    ok "Service enabled and started"
  fi

  # Poll for socket (up to 5 seconds)
  info "Waiting for socket to appear..."
  SOCKET_FOUND=0
  for i in {1..10}; do
    if [[ -S "$SOCKET_PATH" ]]; then
      SOCKET_FOUND=1
      break
    fi
    sleep 0.5
  done

  if [[ $SOCKET_FOUND -eq 1 ]]; then
    ok "Socket created: $SOCKET_PATH"
  else
    fail "Socket not found at $SOCKET_PATH"
    note "Service status:"
    systemctl --user status annad --no-pager -n 5 || true
    if [[ -z "${XDG_RUNTIME_DIR:-}" ]]; then
      note "⚠ XDG_RUNTIME_DIR is not set - socket may be at fallback location"
      note "Check: ~/.anna/run/annad.sock"
    fi
    note "Check logs: journalctl --user -u annad -n 50"
    note "Check session: loginctl show-user \$USER | grep State"
  fi
fi

hr
printf "${b}✅ Installation Complete${reset}\n"
printf "  Install mode:    %s\n" "$INSTALL_MODE"
printf "  Version:         %s\n" "$SOURCE_VERSION"
printf "  Socket:          %s\n" "$SOCKET_PATH"
if [[ "$INSTALL_MODE" == "system" ]]; then
  printf "  User data:       %s/users/%s\n" "$DATA" "$REAL_UID"
  printf "  Policy:          %s/policy.d/%s.toml\n" "$CONF" "$REAL_UID"
else
  printf "  User data:       %s\n" "$DATA"
fi
hr

# Post-install quickcheck
step "Running post-install checks…"

# Check 1: annactl status
if "$ANNACTL" status >/dev/null 2>&1; then
  ok "annactl status"
else
  fail "annactl status failed"
fi

# Check 2: annactl doctor perms
if "$ANNACTL" doctor perms >/dev/null 2>&1; then
  ok "annactl doctor perms"
else
  fail "annactl doctor perms found issues"
  note "Run: annactl doctor perms (for details)"
fi

# Check 3: Socket accessibility
if [[ "$INSTALL_MODE" == "system" ]]; then
  # Check if we can access the socket (might fail if user not in anna group yet)
  if timeout 5s "$ANNACTL" quickscan >/dev/null 2>&1; then
    ok "annactl quickscan (via RPC, no sudo)"
  else
    note "annactl quickscan timed out or failed"
    if ! id -nG "$REAL_USER" | tr ' ' '\n' | grep -qx "anna"; then
      printf "${yellow}⚠  User ${REAL_USER} was just added to 'anna' group${reset}\n"
      printf "${yellow}   Re-login (or run 'newgrp anna') to apply group membership${reset}\n"
    else
      note "Check daemon: systemctl status annad"
      note "Check logs: journalctl -u annad -n 200"
    fi
  fi
fi

hr
printf "${green}Installation successful!${reset}\n\n"
printf "Next steps:\n"
if [[ "$INSTALL_MODE" == "system" ]]; then
  printf "  1. If group was just added: log out and log back in\n"
  printf "  2. Check status:            annactl status\n"
  printf "  3. Run health check:        annactl quickscan\n"
  printf "  4. View recommendations:    annactl advice list\n"
  printf "  5. Apply an action:         annactl advice apply <id>\n"
else
  printf "  1. Check status:            annactl status\n"
  printf "  2. Run health check:        annactl quickscan\n"
  printf "  3. View recommendations:    annactl advice list\n"
fi
printf "\nDocumentation: docs/installer.md\n"
