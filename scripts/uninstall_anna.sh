#!/usr/bin/env bash
# Anna Uninstaller v0.6.9
# Thin wrapper around 'annactl uninstall' for backward compatibility.

set -euo pipefail

# Find annactl (check system and user install locations)
ANNACTL=""
if command -v annactl >/dev/null 2>&1; then
  ANNACTL="annactl"
elif [[ -x "/usr/local/bin/annactl" ]]; then
  ANNACTL="/usr/local/bin/annactl"
elif [[ -x "$HOME/.local/bin/annactl" ]]; then
  ANNACTL="$HOME/.local/bin/annactl"
fi

if [[ -z "$ANNACTL" ]]; then
  echo "Error: annactl not found in PATH, /usr/local/bin, or ~/.local/bin"
  echo ""
  echo "Anna installation appears to be corrupted or incomplete."
  echo "You can try manual removal:"
  echo "  System mode:"
  echo "    sudo systemctl stop annad"
  echo "    sudo systemctl disable annad"
  echo "    sudo rm -f /etc/systemd/system/annad.service"
  echo "    sudo rm -f /usr/local/sbin/annad /usr/local/bin/annactl"
  echo "    sudo rm -rf /var/lib/anna /etc/anna /run/anna"
  echo ""
  echo "  User mode:"
  echo "    systemctl --user stop annad"
  echo "    systemctl --user disable annad"
  echo "    rm -f ~/.config/systemd/user/annad.service"
  echo "    rm -f ~/.local/bin/{annad,annactl}"
  echo "    rm -rf ~/.anna"
  exit 1
fi

# Map old flags to new flags for backward compatibility
ARGS=()
for arg in "$@"; do
  case "$arg" in
    --purge)
      ARGS+=("--complete")
      ;;
    --keep-data)
      ARGS+=("--keep")
      ;;
    *)
      ARGS+=("$arg")
      ;;
  esac
done

# Call canonical uninstall command
exec "$ANNACTL" uninstall "${ARGS[@]}"
