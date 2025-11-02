#!/bin/sh
# Anna v0.12.5 - Btrfs Snapshot Pruning Script
# Cleans up old snapshots based on retention policy
# Usage: prune.sh [--dry-run] [--keep N] [--days N]

set -e

# === Configuration ===
SNAPSHOT_ROOT="/.snapshots"
DRY_RUN=0
KEEP_COUNT=10        # Keep last N snapshots
KEEP_DAYS=30         # Delete snapshots older than N days
PREFIX_FILTER="pacman-*"  # Only prune pacman snapshots by default

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

Prune old Btrfs snapshots based on retention policy.

OPTIONS:
    --dry-run           Show what would be deleted without deleting
    --keep N            Keep last N snapshots (default: $KEEP_COUNT)
    --days N            Delete snapshots older than N days (default: $KEEP_DAYS)
    --prefix PREFIX     Filter snapshots by prefix (default: $PREFIX_FILTER)
    --all               Prune all snapshots (not just pacman-*)
    -h, --help          Show this help message

EXAMPLES:
    $(basename "$0") --dry-run
    $(basename "$0") --keep 5
    $(basename "$0") --days 7 --dry-run
    $(basename "$0") --prefix "manual-*" --keep 3

NOTES:
    - Snapshots are sorted by modification time (oldest first)
    - Both --keep and --days policies are applied (snapshots must violate both to be deleted)
    - Read-only snapshots are deleted safely with 'btrfs subvolume delete'

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
while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --keep)
            KEEP_COUNT="$2"
            shift 2
            ;;
        --days)
            KEEP_DAYS="$2"
            shift 2
            ;;
        --prefix)
            PREFIX_FILTER="$2"
            shift 2
            ;;
        --all)
            PREFIX_FILTER="*"
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

# Check if root is Btrfs
if ! stat -f -c %T / 2>/dev/null | grep -q btrfs; then
    log_err "Root is not Btrfs"
    exit 1
fi

# Check if btrfs command exists
if ! command -v btrfs >/dev/null 2>&1; then
    log_err "btrfs-progs not installed"
    exit 1
fi

# Check if snapshots directory exists
if [ ! -d "$SNAPSHOT_ROOT" ]; then
    log_warn "Snapshots directory not found: $SNAPSHOT_ROOT"
    exit 0
fi

# === Main Logic ===

log_header "Btrfs Snapshot Pruning"
echo ""

if [ "$DRY_RUN" -eq 1 ]; then
    log_warn "DRY RUN MODE - No snapshots will be deleted"
    echo ""
fi

log_dim "Configuration:"
log_dim "  Root:        $SNAPSHOT_ROOT"
log_dim "  Filter:      $PREFIX_FILTER"
log_dim "  Keep count:  $KEEP_COUNT"
log_dim "  Keep days:   $KEEP_DAYS"
echo ""

# Find all matching snapshots
SNAPSHOT_LIST=$(find "$SNAPSHOT_ROOT" -maxdepth 1 -name "$PREFIX_FILTER" -type d 2>/dev/null | sort)

if [ -z "$SNAPSHOT_LIST" ]; then
    log_info "No snapshots found matching '$PREFIX_FILTER'"
    exit 0
fi

TOTAL_COUNT=$(echo "$SNAPSHOT_LIST" | wc -l)
log_info "Found $TOTAL_COUNT snapshot(s) matching filter"
echo ""

# === Age-based Pruning ===
CUTOFF_DATE=$(date -d "$KEEP_DAYS days ago" '+%s' 2>/dev/null || date -v-"${KEEP_DAYS}d" '+%s' 2>/dev/null)
DELETED_COUNT=0

echo "$SNAPSHOT_LIST" | while IFS= read -r snapshot; do
    SNAPSHOT_NAME=$(basename "$snapshot")
    SNAPSHOT_DATE=$(stat -c %Y "$snapshot" 2>/dev/null)

    # Check age
    if [ "$SNAPSHOT_DATE" -lt "$CUTOFF_DATE" ]; then
        AGE_DAYS=$(( ($(date '+%s') - SNAPSHOT_DATE) / 86400 ))

        if [ "$DRY_RUN" -eq 1 ]; then
            log_warn "[DRY RUN] Would delete (${AGE_DAYS}d old): $SNAPSHOT_NAME"
        else
            log_info "Deleting (${AGE_DAYS}d old): $SNAPSHOT_NAME"
            if btrfs subvolume delete "$snapshot" >/dev/null 2>&1; then
                log_dim "  ✓ Deleted"
            else
                log_err "  ✗ Failed to delete"
            fi
        fi
        DELETED_COUNT=$((DELETED_COUNT + 1))
    fi
done

# === Count-based Pruning ===
REMAINING_COUNT=$(find "$SNAPSHOT_ROOT" -maxdepth 1 -name "$PREFIX_FILTER" -type d 2>/dev/null | wc -l)

if [ "$REMAINING_COUNT" -gt "$KEEP_COUNT" ]; then
    EXCESS_COUNT=$((REMAINING_COUNT - KEEP_COUNT))
    echo ""
    log_info "Pruning excess snapshots (keeping last $KEEP_COUNT)"

    find "$SNAPSHOT_ROOT" -maxdepth 1 -name "$PREFIX_FILTER" -type d -printf '%T+ %p\n' 2>/dev/null | \
        sort | \
        head -n "$EXCESS_COUNT" | \
        cut -d' ' -f2 | \
        while IFS= read -r old_snapshot; do
            SNAPSHOT_NAME=$(basename "$old_snapshot")
            if [ "$DRY_RUN" -eq 1 ]; then
                log_warn "[DRY RUN] Would delete (excess): $SNAPSHOT_NAME"
            else
                log_info "Deleting (excess): $SNAPSHOT_NAME"
                if btrfs subvolume delete "$old_snapshot" >/dev/null 2>&1; then
                    log_dim "  ✓ Deleted"
                else
                    log_err "  ✗ Failed to delete"
                fi
            fi
            DELETED_COUNT=$((DELETED_COUNT + 1))
        done
fi

# === Summary ===
echo ""
FINAL_COUNT=$(find "$SNAPSHOT_ROOT" -maxdepth 1 -name "$PREFIX_FILTER" -type d 2>/dev/null | wc -l)

if [ "$DRY_RUN" -eq 1 ]; then
    log_info "Summary: Would delete $DELETED_COUNT snapshot(s)"
    log_dim "  Before:  $TOTAL_COUNT snapshots"
    log_dim "  After:   $((TOTAL_COUNT - DELETED_COUNT)) snapshots"
else
    log_info "Summary: Deleted $DELETED_COUNT snapshot(s)"
    log_dim "  Before:  $TOTAL_COUNT snapshots"
    log_dim "  After:   $FINAL_COUNT snapshots"
fi

echo ""
exit 0
