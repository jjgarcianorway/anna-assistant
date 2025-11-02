#!/bin/sh
# Anna v0.12.5 - Btrfs Pre-Transaction Snapshot Hook
# Creates a snapshot before pacman package operations
# Called by pacman hook: /etc/pacman.d/hooks/90-btrfs-autosnap.hook

set -e

# === Configuration ===
SNAPSHOT_ROOT="/.snapshots"
SUBVOL_ROOT="/"
MAX_SNAPSHOTS=10
TIMESTAMP=$(date '+%Y-%m-%d_%H%M%S')
SNAPSHOT_DESC="pacman-pre-$(date '+%Y-%m-%d %H:%M:%S')"

# === Styling (respects NO_COLOR) ===
if [ -t 1 ] && [ -z "${NO_COLOR}" ]; then
    C_GREEN='\033[38;5;120m'
    C_YELLOW='\033[38;5;228m'
    C_GRAY='\033[38;5;240m'
    C_RESET='\033[0m'
else
    C_GREEN=''
    C_YELLOW=''
    C_GRAY=''
    C_RESET=''
fi

# === Helper Functions ===
log_info() {
    printf "${C_GREEN}▶${C_RESET} %s\n" "$1" >&2
}

log_warn() {
    printf "${C_YELLOW}!${C_RESET} %s\n" "$1" >&2
}

log_dim() {
    printf "${C_GRAY}%s${C_RESET}\n" "$1" >&2
}

# === Checks ===

# Check if root is Btrfs
if ! stat -f -c %T / 2>/dev/null | grep -q btrfs; then
    log_dim "Root is not Btrfs, skipping snapshot"
    exit 0
fi

# Check if btrfs command exists
if ! command -v btrfs >/dev/null 2>&1; then
    log_warn "btrfs-progs not installed, skipping snapshot"
    exit 0
fi

# Check if snapshots directory exists, create if missing
if [ ! -d "$SNAPSHOT_ROOT" ]; then
    log_info "Creating snapshots directory: $SNAPSHOT_ROOT"
    if ! mkdir -p "$SNAPSHOT_ROOT" 2>/dev/null; then
        log_warn "Cannot create $SNAPSHOT_ROOT, skipping snapshot"
        exit 0
    fi
fi

# === Snapshot Creation ===

SNAPSHOT_PATH="${SNAPSHOT_ROOT}/pacman-${TIMESTAMP}"

if [ -t 1 ]; then
    # Interactive mode - show output
    log_info "Creating pre-transaction snapshot"
    log_dim "  Source: $SUBVOL_ROOT"
    log_dim "  Target: $SNAPSHOT_PATH"
fi

# Create read-only snapshot
if btrfs subvolume snapshot -r "$SUBVOL_ROOT" "$SNAPSHOT_PATH" >/dev/null 2>&1; then
    if [ -t 1 ]; then
        log_info "Snapshot created: pacman-${TIMESTAMP}"
    fi
else
    log_warn "Failed to create snapshot"
    exit 1
fi

# === Cleanup Old Snapshots ===

# Count pacman snapshots
SNAPSHOT_COUNT=$(find "$SNAPSHOT_ROOT" -maxdepth 1 -name "pacman-*" -type d 2>/dev/null | wc -l)

if [ "$SNAPSHOT_COUNT" -gt "$MAX_SNAPSHOTS" ]; then
    SNAPSHOTS_TO_DELETE=$((SNAPSHOT_COUNT - MAX_SNAPSHOTS))

    if [ -t 1 ]; then
        log_info "Pruning old snapshots (keeping last $MAX_SNAPSHOTS)"
    fi

    # Delete oldest snapshots
    find "$SNAPSHOT_ROOT" -maxdepth 1 -name "pacman-*" -type d -printf '%T+ %p\n' 2>/dev/null | \
        sort | \
        head -n "$SNAPSHOTS_TO_DELETE" | \
        cut -d' ' -f2 | \
        while IFS= read -r old_snapshot; do
            if [ -t 1 ]; then
                log_dim "  Deleting: $(basename "$old_snapshot")"
            fi
            btrfs subvolume delete "$old_snapshot" >/dev/null 2>&1 || true
        done
fi

# === Success ===
exit 0
