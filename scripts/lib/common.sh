#!/usr/bin/env bash
# shellcheck shell=bash
set -euo pipefail

# -------- colors (safe even if no TTY) --------
if [[ -t 1 ]] && command -v tput >/dev/null 2>&1; then
  bold=$(tput bold); dim=$(tput dim); reset=$(tput sgr0)
  red=$(tput setaf 1); green=$(tput setaf 2); yellow=$(tput setaf 3)
  blue=$(tput setaf 4); magenta=$(tput setaf 5); cyan=$(tput setaf 6)
else
  bold=""; dim=""; reset=""; red=""; green=""; yellow=""; blue=""; magenta=""; cyan=""
fi

say()  { printf "%b%s%b\n" "$dim" "$*" "$reset"; }
info() { printf "%b• %s%b\n" "$blue" "$*" "$reset"; }
ok()   { printf "%b✔ %s%b\n" "$green" "$*" "$reset"; }
warn() { printf "%b• %s%b\n" "$yellow" "$*" "$reset"; }
bad()  { printf "%b✖ %s%b\n" "$magenta" "$*" "$reset"; }

# -------- paths (real vs FAKE_ROOT) --------
FAKE_ROOT_DEFAULT=""
FAKE_ROOT="${FAKE_ROOT:-$FAKE_ROOT_DEFAULT}"
_root() { [[ -n "$FAKE_ROOT" ]] && printf "%s" "$FAKE_ROOT" || printf "" ; }

BIN_DIR()      { [[ -n "$FAKE_ROOT" ]] && printf "%s/usr/local/bin" "$FAKE_ROOT" || printf "/usr/local/bin"; }
SYSTEMD_DIR()  { [[ -n "$FAKE_ROOT" ]] && printf "%s/etc/systemd/system" "$FAKE_ROOT" || printf "/etc/systemd/system"; }
ETC_ANNA()     { [[ -n "$FAKE_ROOT" ]] && printf "%s/etc/anna" "$FAKE_ROOT" || printf "/etc/anna"; }
VARLIB_ANNA()  { [[ -n "$FAKE_ROOT" ]] && printf "%s/var/lib/anna" "$FAKE_ROOT" || printf "/var/lib/anna"; }

# -------- privileged ops (sudo unless FAKE_ROOT) --------
asroot() {
  if [[ -n "$FAKE_ROOT" ]]; then
    # shellcheck disable=SC2312
    "$@"
  else
    sudo "$@"
  fi
}
need_sudo_once() {
  [[ -n "$FAKE_ROOT" ]] && return 0
  info "Elevating once to remove system files (binaries, unit file, data)."
  sudo -v
}
