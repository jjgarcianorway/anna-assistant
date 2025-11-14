#!/bin/bash
# Anna Assistant - Uninstaller
# Safely removes Anna with optional data backup

set -e

INSTALL_DIR="/usr/local/bin"
DATA_DIR="/var/lib/anna"
LOG_DIR="/var/log/anna"
CONFIG_DIR="/etc/anna"

# Colors
if [ -t 1 ] && command -v tput >/dev/null 2>&1 && [ "$(tput colors)" -ge 256 ]; then
    BLUE='\033[38;5;117m'
    GREEN='\033[38;5;120m'
    YELLOW='\033[38;5;228m'
    RED='\033[38;5;210m'
    CYAN='\033[38;5;159m'
    GRAY='\033[38;5;250m'
    RESET='\033[0m'
    BOLD='\033[1m'
    CHECK="✓"; CROSS="✗"; ARROW="→"
else
    BLUE=''; GREEN=''; YELLOW=''; RED=''; CYAN=''; GRAY=''; RESET=''; BOLD=''
    CHECK="[OK]"; CROSS="[X]"; ARROW="->"
fi

error_exit() {
    echo -e "${RED}${CROSS} $1${RESET}" >&2
    exit 1
}

# Header
echo
echo -e "${BOLD}${CYAN}====================================================${RESET}"
echo -e "${BOLD}${BLUE}    Anna Assistant Uninstaller${RESET}"
echo -e "${BOLD}${CYAN}====================================================${RESET}"
echo

# Check if Anna is installed
if [ ! -f "$INSTALL_DIR/annactl" ] && [ ! -f "$INSTALL_DIR/annad" ]; then
    echo -e "${YELLOW}${ARROW}${RESET} Anna doesn't appear to be installed."
    echo
    exit 0
fi

# Get current version
VERSION=""
if command -v annactl >/dev/null 2>&1; then
    VERSION=$(annactl --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?' || echo "unknown")
fi

echo "This will remove Anna Assistant from your system."
echo
if [ -n "$VERSION" ] && [ "$VERSION" != "unknown" ]; then
    echo -e "${GRAY}Current version: ${CYAN}v${VERSION}${RESET}"
    echo
fi

# Check sudo early
command -v sudo >/dev/null 2>&1 || error_exit "sudo required"

# Stop daemon first
echo -e "${CYAN}${ARROW}${RESET} Stopping Anna daemon..."
if systemctl is-active --quiet annad 2>/dev/null; then
    sudo systemctl stop annad
    echo -e "${GREEN}${CHECK}${RESET} Daemon stopped"
else
    echo -e "${GRAY}  Daemon not running${RESET}"
fi

# Ask about data
echo
echo -e "${BOLD}${YELLOW}Data Management${RESET}"
echo -e "${GRAY}────────────────────────────────────────────────────${RESET}"
echo

DATA_SIZE="0"
if [ -d "$DATA_DIR" ]; then
    DATA_SIZE=$(du -sh "$DATA_DIR" 2>/dev/null | cut -f1 || echo "unknown")
fi

echo "Anna has stored data and knowledge about your system:"
if [ -d "$DATA_DIR" ]; then
    echo -e "  ${CYAN}${DATA_DIR}${RESET} (${DATA_SIZE})"
fi
if [ -d "$LOG_DIR" ]; then
    echo -e "  ${CYAN}${LOG_DIR}${RESET}"
fi
echo

echo -e "${BOLD}Do you want to delete Anna's data? [y/N]:${RESET} "
read -r DELETE_DATA < /dev/tty
echo

BACKUP_CREATED=false
BACKUP_PATH=""

if [[ $DELETE_DATA =~ ^[Yy]$ ]]; then
    # Offer backup
    echo -e "${BOLD}Would you like to create a backup before deleting? [Y/n]:${RESET} "
    read -r CREATE_BACKUP < /dev/tty
    echo

    if [[ ! $CREATE_BACKUP =~ ^[Nn]$ ]]; then
        # Create backup
        BACKUP_DIR="$HOME/anna-backups"
        mkdir -p "$BACKUP_DIR"

        TIMESTAMP=$(date +%Y%m%d-%H%M%S)
        if [ -n "$VERSION" ] && [ "$VERSION" != "unknown" ]; then
            BACKUP_NAME="anna-backup-v${VERSION}-${TIMESTAMP}.tar.gz"
        else
            BACKUP_NAME="anna-backup-${TIMESTAMP}.tar.gz"
        fi
        BACKUP_PATH="$BACKUP_DIR/$BACKUP_NAME"

        echo -e "${CYAN}${ARROW}${RESET} Creating backup..."
        echo -e "${GRAY}Backup location: ${BACKUP_PATH}${RESET}"

        # Create tarball
        BACKUP_DIRS=""
        [ -d "$DATA_DIR" ] && BACKUP_DIRS="$BACKUP_DIRS $DATA_DIR"
        [ -d "$LOG_DIR" ] && BACKUP_DIRS="$BACKUP_DIRS $LOG_DIR"
        [ -d "$CONFIG_DIR" ] && BACKUP_DIRS="$BACKUP_DIRS $CONFIG_DIR"

        if [ -n "$BACKUP_DIRS" ]; then
            if sudo tar czf "$BACKUP_PATH" $BACKUP_DIRS 2>/dev/null; then
                sudo chown "$USER:$(id -gn)" "$BACKUP_PATH"
                BACKUP_SIZE=$(du -sh "$BACKUP_PATH" 2>/dev/null | cut -f1 || echo "unknown")
                echo -e "${GREEN}${CHECK}${RESET} Backup created: ${BACKUP_PATH} (${BACKUP_SIZE})"
                BACKUP_CREATED=true
            else
                echo -e "${RED}${CROSS}${RESET} Backup failed"
            fi
        else
            echo -e "${GRAY}  No data to backup${RESET}"
        fi
        echo
    fi

    # Delete data
    echo -e "${CYAN}${ARROW}${RESET} Deleting Anna's data..."

    [ -d "$DATA_DIR" ] && sudo rm -rf "$DATA_DIR"
    [ -d "$LOG_DIR" ] && sudo rm -rf "$LOG_DIR"
    [ -d "$CONFIG_DIR" ] && sudo rm -rf "$CONFIG_DIR"

    echo -e "${GREEN}${CHECK}${RESET} Data deleted"
else
    echo -e "${GRAY}Keeping Anna's data (can be reused on reinstall)${RESET}"
fi

echo

# Remove binaries
echo -e "${CYAN}${ARROW}${RESET} Removing binaries..."
sudo rm -f "$INSTALL_DIR/annad" "$INSTALL_DIR/annactl"
echo -e "${GREEN}${CHECK}${RESET} Binaries removed"

# Remove systemd service
echo -e "${CYAN}${ARROW}${RESET} Removing systemd service..."
sudo systemctl disable annad 2>/dev/null || true
sudo rm -f /etc/systemd/system/annad.service
sudo systemctl daemon-reload
echo -e "${GREEN}${CHECK}${RESET} Service removed"

# Remove shell completions
echo -e "${CYAN}${ARROW}${RESET} Removing shell completions..."
sudo rm -f /usr/share/bash-completion/completions/annactl
sudo rm -f /usr/share/zsh/site-functions/_annactl
sudo rm -f /usr/share/fish/vendor_completions.d/annactl.fish
echo -e "${GREEN}${CHECK}${RESET} Completions removed"

# Success message
echo
echo -e "${BOLD}${GREEN}====================================================${RESET}"
echo -e "${BOLD}${GREEN}  ✓ Anna Assistant Uninstalled${RESET}"
echo -e "${BOLD}${GREEN}====================================================${RESET}"
echo

if [ "$BACKUP_CREATED" = true ]; then
    echo -e "${BOLD}${CYAN}Backup Information:${RESET}"
    echo
    echo -e "Your Anna data has been backed up to:"
    echo -e "  ${CYAN}${BACKUP_PATH}${RESET}"
    echo
    echo "To restore this backup on a future installation:"
    echo -e "  1. Install Anna: ${GRAY}curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash${RESET}"
    echo -e "  2. Stop daemon: ${GRAY}sudo systemctl stop annad${RESET}"
    echo -e "  3. Restore backup: ${GRAY}sudo tar xzf ${BACKUP_PATH} -C /${RESET}"
    echo -e "  4. Restart daemon: ${GRAY}sudo systemctl start annad${RESET}"
    echo
fi

if [[ ! $DELETE_DATA =~ ^[Yy]$ ]]; then
    echo -e "${BOLD}${YELLOW}Note:${RESET} Anna's data was preserved"
    echo
    echo "Data location: ${CYAN}${DATA_DIR}${RESET}"
    echo
    echo "To remove it manually later:"
    echo -e "  ${GRAY}sudo rm -rf ${DATA_DIR} ${LOG_DIR}${RESET}"
    echo
fi

echo "Thank you for trying Anna!"
echo
