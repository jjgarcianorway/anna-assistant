#!/usr/bin/env bash
set -euo pipefail

PURGE=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --purge) PURGE=1 ;;
    *) echo "Unknown flag: $1"; exit 2 ;;
  esac; shift
done

ANNAD="/usr/local/bin/annad"
ANNACTL="/usr/local/bin/annactl"
UNIT="/etc/systemd/system/annad.service"
DATA="/var/lib/anna"
CONF="/etc/anna"

# Colors
if [[ -t 1 ]]; then
  green=$'\033[32m'; yellow=$'\033[33m'; red=$'\033[31m'; reset=$'\033[0m'
else
  green=""; yellow=""; red=""; reset=""
fi
hr(){ printf "%s\n" "----------------------------------"; }

echo "Anna – Uninstaller"
hr
echo "Check:"
service_registered=no; [[ -f "$UNIT" ]] && service_registered=yes
service_active=no; systemctl is-active --quiet annad 2>/dev/null && service_active=yes || true
unit_file=$([[ -f "$UNIT" ]] && echo present || echo missing)
bin_ad=$([[ -x "$ANNAD" ]] && echo present || echo missing)
bin_ctl=$([[ -x "$ANNACTL" ]] && echo present || echo missing)
data_conf=$([[ -d "$CONF" ]] && echo present || echo missing)
data_var=$([[ -d "$DATA" ]] && echo present || echo missing)

echo "  • service registered: $service_registered"
echo "  • service active:     $service_active"
echo "  • unit file:          $unit_file"
echo "  • binary annad:       $bin_ad"
echo "  • binary annactl:     $bin_ctl"
echo "  • data /etc/anna:     $data_conf"
echo "  • data /var/lib/anna: $data_var"
hr

any_change=0
need_sudo() { [[ $EUID -ne 0 ]] && echo sudo || true; }
cmd=$(need_sudo)

# Only elevate if something to change
if [[ "$service_active" == "yes" || "$service_registered" == "yes" || -x "$ANNAD" || -x "$ANNACTL" || -f "$UNIT" || ($PURGE -eq 1 && ( -d "$CONF" || -d "$DATA" )) ]]; then
  $cmd -v
fi

# Stop/disable/unit/binaries
if [[ "$service_active" == "yes" ]]; then $cmd systemctl stop annad.service || true; any_change=1; fi
if [[ "$service_registered" == "yes" ]]; then $cmd systemctl disable annad.service || true; any_change=1; fi
if [[ -f "$UNIT" ]]; then $cmd rm -f "$UNIT"; $cmd systemctl daemon-reload; any_change=1; fi
if [[ -x "$ANNAD" ]]; then $cmd rm -f "$ANNAD"; any_change=1; fi
if [[ -x "$ANNACTL" ]]; then $cmd rm -f "$ANNACTL"; any_change=1; fi

# Data
if [[ $PURGE -eq 1 ]]; then
  if [[ -d "$CONF" ]]; then $cmd rm -rf "$CONF"; any_change=1; fi
  if [[ -d "$DATA" ]]; then $cmd rm -rf "$DATA"; any_change=1; fi
else
  # Just report; don't change
  :
fi

hr
# Post-check
after_active=$(systemctl is-active annad 2>/dev/null || true)
after_unit=$([[ -f "$UNIT" ]] && echo present || echo missing)
after_bins=$([[ -x "$ANNAD" || -x "$ANNACTL" ]] && echo present || echo missing)
after_data=$([[ -d "$CONF" || -d "$DATA" ]] && echo present || echo missing)

echo "Check (after):"
echo "  • service active:     $([[ "$after_active" == active ]] && echo yes || echo no)"
echo "  • unit file:          $after_unit"
echo "  • binaries present:   $after_bins"
echo "  • data present:       $after_data"
hr

# Outcome classification
if [[ $any_change -eq 0 ]]; then
  if [[ "$after_bins" == missing && "$after_unit" == missing && "$after_active" != active && "$after_data" == present ]]; then
    echo -e "${yellow}No changes; program files already removed; data preserved.${reset}"
    exit 0
  elif [[ "$after_bins" == missing && "$after_unit" == missing && "$after_active" != active && "$after_data" == missing ]]; then
    echo -e "${yellow}No changes; everything already removed previously.${reset}"
    exit 0
  else
    echo -e "${yellow}No changes performed.${reset}"
    exit 0
  fi
else
  if [[ "$after_bins" == missing && "$after_unit" == missing && "$after_active" != active ]]; then
    if [[ "$after_data" == missing ]]; then
      echo -e "${green}Uninstall completed successfully (all components removed).${reset}"
      exit 0
    else
      echo -e "${yellow}Uninstall completed; program files removed, data preserved.${reset}"
      exit 0
    fi
  else
    echo -e "${red}Uninstall incomplete (some components remain).${reset}"
    exit 1
  fi
fi
