#!/bin/sh
# Anna v0.12.5 - systemd-boot Snapshot Entry Generator
# Generates systemd-boot entries for Btrfs snapshots
# Usage: sdboot-gen.sh [--dry-run] [--limit N]

set -e

# === Configuration ===
SNAPSHOT_ROOT="/.snapshots"
BOOT_ENTRIES_DIR="/boot/loader/entries"
SNAPSHOT_PREFIX="pacman-"
LIMIT=5  # Generate entries for last N snapshots
DRY_RUN=0

# === Styling (respects NO_COLOR) ===
if [ -t 1 ] && [ -z "${NO_COLOR}" ]; then
    C_GREEN='\033[38;5;120m'
    C_YELLOW='\033[38;5;228m'
    C_RED='\033[38;5;210m'
    C_CYAN='\033[38;5;87m'
    C_GRAY='\033[38;5;240m'
    C_RESET='\033[0m'
else
    C_GREEN=''
    C_YELLOW=''
    C_RED=''
    C_CYAN=''
    C_GRAY=''
    C_RESET=''
fi

# === Helper Functions ===
usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Generate systemd-boot entries for Btrfs snapshots.

OPTIONS:
    --dry-run           Show what would be created without creating
    --limit N           Generate entries for last N snapshots (default: $LIMIT)
    --clean             Remove existing snapshot entries before generating
    -h, --help          Show this help message

EXAMPLES:
    $(basename "$0") --dry-run
    $(basename "$0") --limit 3
    $(basename "$0") --clean --limit 5

NOTES:
    - Requires systemd-boot bootloader
    - Generates entries in $BOOT_ENTRIES_DIR
    - Entry files named: anna-snapshot-TIMESTAMP.conf
    - Snapshots must be read-only
    - Automatically detects kernel and initramfs from main entry

PREREQUISITES:
    - systemd-boot installed and configured
    - Btrfs root filesystem with snapshots in $SNAPSHOT_ROOT
    - Existing boot entry to use as template

EOF
}

log_header() {
    printf "${C_CYAN}╭─ %s ${C_RESET}\n" "$1"
}

log_info() {
    printf "${C_GREEN}▶${C_RESET} %s\n" "$1"
}

log_warn() {
    printf "${C_YELLOW}!${C_RESET} %s\n" "$1"
}

log_err() {
    printf "${C_RED}✗${C_RESET} %s\n" "$1"
}

log_dim() {
    printf "${C_GRAY}%s${C_RESET}\n" "$1"
}

# === Parse Arguments ===
CLEAN_MODE=0

while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --limit)
            LIMIT="$2"
            shift 2
            ;;
        --clean)
            CLEAN_MODE=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            exit 1
            ;;
    esac
done

# === Checks ===

# Check if systemd-boot is installed
if [ ! -d "$BOOT_ENTRIES_DIR" ]; then
    log_err "systemd-boot entries directory not found: $BOOT_ENTRIES_DIR"
    log_dim "This tool requires systemd-boot. For GRUB, use grub-btrfs instead."
    exit 1
fi

# Check if root is Btrfs
if ! stat -f -c %T / 2>/dev/null | grep -q btrfs; then
    log_err "Root is not Btrfs"
    exit 1
fi

# Check if snapshots directory exists
if [ ! -d "$SNAPSHOT_ROOT" ]; then
    log_err "Snapshots directory not found: $SNAPSHOT_ROOT"
    exit 1
fi

# === Main Logic ===

log_header "systemd-boot Snapshot Entry Generator"
echo ""

if [ "$DRY_RUN" -eq 1 ]; then
    log_warn "DRY RUN MODE - No entries will be created"
    echo ""
fi

# === Clean Existing Entries (if --clean) ===

if [ "$CLEAN_MODE" -eq 1 ]; then
    log_info "Cleaning existing snapshot entries"

    EXISTING_ENTRIES=$(find "$BOOT_ENTRIES_DIR" -name "anna-snapshot-*.conf" 2>/dev/null || true)
    if [ -n "$EXISTING_ENTRIES" ]; then
        echo "$EXISTING_ENTRIES" | while IFS= read -r entry; do
            if [ "$DRY_RUN" -eq 1 ]; then
                log_dim "[DRY RUN] Would delete: $(basename "$entry")"
            else
                rm -f "$entry"
                log_dim "  Deleted: $(basename "$entry")"
            fi
        done
    else
        log_dim "  No existing entries found"
    fi
    echo ""
fi

# === Find Template Entry ===

# Look for main arch entry
TEMPLATE_ENTRY=$(find "$BOOT_ENTRIES_DIR" -name "arch*.conf" ! -name "anna-snapshot-*" -type f 2>/dev/null | head -1)

if [ -z "$TEMPLATE_ENTRY" ]; then
    log_err "No template entry found (looking for arch*.conf in $BOOT_ENTRIES_DIR)"
    log_dim "Cannot generate snapshot entries without a base entry to copy settings from"
    exit 1
fi

log_info "Using template: $(basename "$TEMPLATE_ENTRY")"
log_dim "  Path: $TEMPLATE_ENTRY"
echo ""

# Extract kernel and initrd from template
LINUX_LINE=$(grep -E '^linux' "$TEMPLATE_ENTRY" || true)
INITRD_LINE=$(grep -E '^initrd' "$TEMPLATE_ENTRY" || true)
OPTIONS_LINE=$(grep -E '^options' "$TEMPLATE_ENTRY" || true)

if [ -z "$LINUX_LINE" ] || [ -z "$INITRD_LINE" ]; then
    log_err "Template entry incomplete (missing linux or initrd)"
    exit 1
fi

log_dim "Template configuration:"
log_dim "  $LINUX_LINE"
log_dim "  $INITRD_LINE"
echo ""

# === Find Snapshots ===

SNAPSHOTS=$(find "$SNAPSHOT_ROOT" -maxdepth 1 -name "${SNAPSHOT_PREFIX}*" -type d 2>/dev/null | sort -r | head -n "$LIMIT")

if [ -z "$SNAPSHOTS" ]; then
    log_warn "No snapshots found matching '${SNAPSHOT_PREFIX}*'"
    exit 0
fi

SNAPSHOT_COUNT=$(echo "$SNAPSHOTS" | wc -l)
log_info "Found $SNAPSHOT_COUNT snapshot(s) (generating entries for last $LIMIT)"
echo ""

# === Generate Entries ===

GENERATED_COUNT=0

echo "$SNAPSHOTS" | while IFS= read -r snapshot; do
    SNAPSHOT_NAME=$(basename "$snapshot")
    SNAPSHOT_TIMESTAMP=$(echo "$SNAPSHOT_NAME" | sed "s/${SNAPSHOT_PREFIX}//")

    # Extract subvolume ID
    SUBVOL_ID=$(btrfs subvolume show "$snapshot" 2>/dev/null | grep -E '^\s+Subvolume ID:' | awk '{print $3}' || echo "")

    if [ -z "$SUBVOL_ID" ]; then
        log_warn "Cannot determine subvolume ID for $SNAPSHOT_NAME, skipping"
        continue
    fi

    # Create entry filename
    ENTRY_FILE="${BOOT_ENTRIES_DIR}/anna-snapshot-${SNAPSHOT_TIMESTAMP}.conf"

    # Check if snapshot is read-only
    if ! btrfs property get "$snapshot" ro 2>/dev/null | grep -q "ro=true"; then
        log_warn "Snapshot not read-only: $SNAPSHOT_NAME, skipping"
        continue
    fi

    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "[DRY RUN] Would create: anna-snapshot-${SNAPSHOT_TIMESTAMP}.conf"
        log_dim "  Snapshot: $SNAPSHOT_NAME"
        log_dim "  Subvol ID: $SUBVOL_ID"
    else
        log_info "Creating entry: anna-snapshot-${SNAPSHOT_TIMESTAMP}.conf"
        log_dim "  Snapshot: $SNAPSHOT_NAME"
        log_dim "  Subvol ID: $SUBVOL_ID"

        # Generate entry content
        {
            echo "title   Arch Linux (snapshot $SNAPSHOT_TIMESTAMP)"
            echo "$LINUX_LINE"
            echo "$INITRD_LINE"

            # Modify options to use snapshot subvolume
            if echo "$OPTIONS_LINE" | grep -q "rootflags=subvol="; then
                # Replace existing subvol with subvolid
                MODIFIED_OPTIONS=$(echo "$OPTIONS_LINE" | sed "s|rootflags=subvol=[^ ]*|rootflags=subvolid=$SUBVOL_ID|")
            else
                # Add rootflags=subvolid
                MODIFIED_OPTIONS=$(echo "$OPTIONS_LINE" | sed "s|options|options rootflags=subvolid=$SUBVOL_ID|")
            fi

            echo "$MODIFIED_OPTIONS"
        } > "$ENTRY_FILE"

        log_dim "  ✓ Created"
    fi

    GENERATED_COUNT=$((GENERATED_COUNT + 1))
done

# === Summary ===

echo ""
if [ "$DRY_RUN" -eq 1 ]; then
    log_info "Summary: Would create $GENERATED_COUNT entry/entries"
else
    log_info "Summary: Created $GENERATED_COUNT entry/entries"
    log_dim "Boot entries are now available in systemd-boot menu"
fi

echo ""
log_dim "To verify entries:"
log_dim "  ls -lh $BOOT_ENTRIES_DIR/anna-snapshot-*.conf"
echo ""

exit 0
