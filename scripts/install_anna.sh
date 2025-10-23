#!/usr/bin/env bash
set -euo pipefail

ANNAD="/usr/local/bin/annad"
ANNACTL="/usr/local/bin/annactl"
UNIT="/etc/systemd/system/annad.service"
DATA="/var/lib/anna"
CONF="/etc/anna"

# Colors (auto-off if not a TTY)
if [[ -t 1 ]]; then
  b=$'\033[1m'; dim=$'\033[2m'; blue=$'\033[34m'; green=$'\033[32m'
  yellow=$'\033[33m'; red=$'\033[31m'; reset=$'\033[0m'
else
  b=""; dim=""; blue=""; green=""; yellow=""; red=""; reset=""
fi
hr(){ printf "%s\n" "${dim}-----------------------------------------------${reset}"; }
step(){ printf "› %s\n" "$*"; }
ok(){ printf "  ${green}✔${reset} %s\n" "$*"; }
note(){ printf "  ${yellow}• %s${reset}\n" "$*"; }
fail(){ printf "  ${red}✖ %s${reset}\n" "$*"; }

# Intro
if [[ -f "scripts/installer_intro.sh" ]]; then
  bash scripts/installer_intro.sh
else
  printf "${b}Anna – system assistant installer${reset}\n"; hr
fi

# Preflight
printf "${b}Preflight${reset}\n"
HAVE_UNIT=0; [[ -f "$UNIT" ]] && HAVE_UNIT=1
ACTIVE=0; systemctl is-active --quiet annad 2>/dev/null && ACTIVE=1 || true
HAVE_AD=0; [[ -x "$ANNAD" ]] && HAVE_AD=1
HAVE_CTL=0; [[ -x "$ANNACTL" ]] && HAVE_CTL=1
HAVE_DATA=0; [[ -d "$DATA" ]] && HAVE_DATA=1
HAVE_CONF=0; [[ -d "$CONF" ]] && HAVE_CONF=1

printf "  annad binary:       %s\n" $([[ $HAVE_AD -eq 1 ]] && echo "present" || echo "missing")
printf "  annactl binary:     %s\n" $([[ $HAVE_CTL -eq 1 ]] && echo "present" || echo "missing")
printf "  service file:       %s\n" $([[ $HAVE_UNIT -eq 1 ]] && echo "present" || echo "missing")
printf "  service active:     %s\n" $([[ $ACTIVE -eq 1 ]] && echo "yes" || echo "no")
printf "  data dir:           %s\n" $([[ $HAVE_DATA -eq 1 ]] && echo "$DATA (present)" || echo "$DATA (missing)")
printf "  conf dir:           %s\n" $([[ $HAVE_CONF -eq 1 ]] && echo "$CONF (present)" || echo "$CONF (missing)")
hr

# Build
step "Building ${b}annad${reset} and ${b}annactl${reset} (release)…"
cargo build --release --quiet
AD_SRC="target/release/annad"
CTL_SRC="target/release/annactl"
if [[ ! -x "$AD_SRC" || ! -x "$CTL_SRC" ]]; then
  fail "Build artifacts not found after cargo build."
  printf "\nSearched: target/release/{annad,annactl}\n"
  exit 1
fi
ok "Build ready"

# Helper: copy only if src newer than dst (mtime compare)
install_if_newer() {
  local src="$1" dst="$2" changed=0
  if [[ ! -e "$dst" ]]; then
    sudo install -Dm755 "$src" "$dst"
    echo "installed"
    return 0
  fi
  local sm dm
  sm=$(stat -c %Y "$src")
  dm=$(stat -c %Y "$dst")
  if [[ "$sm" -gt "$dm" ]]; then
    sudo install -Dm755 "$src" "$dst"
    echo "updated"
  else
    echo "up-to-date"
    return 1
  fi
}

# Privileges once
step "Requesting privileges to install system files…"
sudo -v

# Binaries (mtime-aware)
step "Checking binaries in /usr/local/bin"
CHANGED=0
res=$(install_if_newer "$AD_SRC" "$ANNAD") || true
case "$res" in
  installed) ok "annad installed"; CHANGED=1;;
  updated)   ok "annad updated";   CHANGED=1;;
  up-to-date) note "annad already up-to-date";;
  *) ;;
esac
res=$(install_if_newer "$CTL_SRC" "$ANNACTL") || true
case "$res" in
  installed) ok "annactl installed"; CHANGED=1;;
  updated)   ok "annactl updated";   CHANGED=1;;
  up-to-date) note "annactl already up-to-date";;
  *) ;;
esac

# Data/config
step "Ensuring data directories exist"
sudo install -d -m755 "$CONF"
sudo install -d -m755 "$DATA"
ok "Directories ready"

# Unit
if [[ $HAVE_UNIT -eq 0 ]]; then
  step "Installing systemd unit ${b}$UNIT${reset}"
  sudo tee "$UNIT" >/dev/null <<'UNIT'
[Unit]
Description=Anna - System Assistant Daemon v0.1.0
After=network.target

[Service]
ExecStart=/usr/local/bin/annad
Restart=always
RestartSec=3
WorkingDirectory=/var/lib/anna
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
UNIT
  sudo systemctl daemon-reload
  ok "Unit installed"
else
  note "Unit already present"
fi

# Service control
if systemctl is-enabled --quiet annad 2>/dev/null; then
  if [[ $CHANGED -eq 1 ]]; then
    step "Restarting annad (binaries updated)…"
    sudo systemctl restart annad
    ok "Service restarted"
  else
    ok "Service already enabled; no restart needed"
  fi
else
  step "Enabling and starting annad"
  sudo systemctl enable --now annad
  ok "Service enabled and started"
fi

hr
printf "${b}Installation complete.${reset}\n"
printf "• Status: %s\n" "$(systemctl is-active annad || true)"
printf "• Enabled: %s\n" "$(systemctl is-enabled annad || true)"
printf "• Data dir: %s\n" "$DATA"
hr

read -r -p "Run a quick health check now? [Y/n] " run_qhc
if [[ -z "$run_qhc" || "$run_qhc" =~ ^[Yy]$ ]]; then
  step "Running quick health check…"
  if sudo "$ANNACTL" quickscan; then
    ok "Quick health check complete"
    note "Review recommendations with 'annactl advice list'"
  else
    fail "Quick health check failed"
  fi
  hr
fi

printf "Tip: see runtime info with ${b}annactl status${reset}\n"
