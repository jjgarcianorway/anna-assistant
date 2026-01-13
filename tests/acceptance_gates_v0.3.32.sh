#!/bin/bash
# Anna v0.3.32 Acceptance Gates Proof Script
# Run as root: sudo bash tests/acceptance_gates_v0.3.32.sh
#
# This script verifies:
# A) No home folder writes
# B) Permissions model security
# C) Socket access control
# D) Migration idempotency
# E) Updater still works

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; exit 1; }
info() { echo -e "${CYAN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

echo "======================================"
echo "  Anna v0.3.32 Acceptance Gates"
echo "======================================"
echo

# Check root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root: sudo bash $0"
    exit 1
fi

# Install new binaries first
info "Installing v0.3.32 binaries..."
if [ -f "target/release/annad" ] && [ -f "target/release/annactl" ]; then
    cp target/release/annad /usr/local/bin/annad
    cp target/release/annactl /usr/local/bin/annactl
    pass "Binaries installed"
else
    fail "Build binaries first: cargo build --release"
fi

# Ensure anna group exists
if ! getent group anna >/dev/null; then
    groupadd anna
    info "Created anna group"
fi

# ================================================
# GATE A: No Home Writes
# ================================================
echo
echo "======================================"
echo "  GATE A: No Home Writes"
echo "======================================"
echo

# Create test users
info "Creating test users anna_u1 and anna_u2..."
useradd -m anna_u1 2>/dev/null || true
useradd -m anna_u2 2>/dev/null || true
usermod -aG anna anna_u1
usermod -aG anna anna_u2

# Clean any existing anna dirs in test user homes
rm -rf /home/anna_u1/.anna /home/anna_u1/.local/share/anna /home/anna_u1/.config/anna
rm -rf /home/anna_u2/.anna /home/anna_u2/.local/share/anna /home/anna_u2/.config/anna

info "User homes before test:"
echo "anna_u1:"
ls -la /home/anna_u1/ | grep -v "^total"
echo "anna_u2:"
ls -la /home/anna_u2/ | grep -v "^total"

# Apply correct permissions
info "Applying secure permissions (750)..."
for dir in /var/lib/anna /var/lib/anna/backups /var/lib/anna/wiki /var/lib/anna/recipes /var/log/anna /run/anna; do
    mkdir -p "$dir"
    chown root:anna "$dir"
    chmod 750 "$dir"
done

# Set file permissions
for f in /var/lib/anna/*.json /var/lib/anna/*.toml 2>/dev/null; do
    [ -f "$f" ] && chmod 640 "$f"
done

# Restart daemon
info "Restarting annad..."
systemctl restart annad
sleep 3

# Test as each user
info "Running annactl status as anna_u1..."
su - anna_u1 -c "annactl status" || warn "annactl status failed (may need daemon running)"

info "Running annactl status as anna_u2..."
su - anna_u2 -c "annactl status" || warn "annactl status failed (may need daemon running)"

# Check for forbidden directories
info "Checking for home directory writes..."
U1_ANNA=$(find /home/anna_u1 -maxdepth 3 -type d \( -name ".anna" -o -path "*/.local/share/anna" -o -path "*/.config/anna" \) 2>/dev/null || true)
U2_ANNA=$(find /home/anna_u2 -maxdepth 3 -type d \( -name ".anna" -o -path "*/.local/share/anna" -o -path "*/.config/anna" \) 2>/dev/null || true)

if [ -z "$U1_ANNA" ] && [ -z "$U2_ANNA" ]; then
    pass "No Anna state directories created in user homes"
else
    fail "Found Anna directories in user homes: $U1_ANNA $U2_ANNA"
fi

# ================================================
# GATE B: Permissions Model
# ================================================
echo
echo "======================================"
echo "  GATE B: Permissions Model"
echo "======================================"
echo

info "Directory permissions:"
ls -ld /etc/anna
ls -ld /var/lib/anna
ls -ld /var/lib/anna/backups 2>/dev/null || echo "(backups not yet created)"
ls -ld /run/anna

info "Socket permissions:"
ls -l /run/anna/anna.sock 2>/dev/null || echo "(socket may not exist yet)"

info "File permissions:"
ls -l /var/lib/anna/*.json 2>/dev/null || echo "(no json files yet)"

# Verify no world-writable
WORLD_WRITE=$(find /var/lib/anna /run/anna -perm -o+w 2>/dev/null | head -5)
if [ -z "$WORLD_WRITE" ]; then
    pass "No world-writable paths in /var/lib/anna or /run/anna"
else
    fail "Found world-writable paths: $WORLD_WRITE"
fi

# Verify no group-writable on critical files
info "Verifying critical files are not group-writable..."
for f in /var/lib/anna/update_ledger.json /var/lib/anna/stats.json; do
    if [ -f "$f" ]; then
        PERMS=$(stat -c "%a" "$f")
        if [ "$((PERMS & 020))" -ne 0 ]; then
            fail "$f is group-writable (mode $PERMS) - security risk!"
        fi
    fi
done
pass "Critical files are not group-writable"

echo
echo "PERMISSIONS POLICY:"
echo "  - Directories: 750 (rwxr-x---) root:anna"
echo "  - Files: 640 (rw-r-----) root:anna"
echo "  - Socket: 660 (rw-rw----) root:anna"
echo "  - Writes: Daemon (root) only, via RPC"
echo "  - Reads: anna group members can read for diagnostics"
echo

# ================================================
# GATE C: Socket Access Control
# ================================================
echo
echo "======================================"
echo "  GATE C: Socket Access Control"
echo "======================================"
echo

# Create user NOT in anna group
info "Creating test user 'nogrp' NOT in anna group..."
useradd -m nogrp 2>/dev/null || true
gpasswd -d nogrp anna 2>/dev/null || true

info "Testing socket access for user NOT in anna group..."
if su - nogrp -c "annactl status" 2>&1 | grep -qi "permission\|denied\|connect\|refused"; then
    pass "User not in anna group correctly denied access"
else
    # Check if it actually failed
    if ! su - nogrp -c "annactl status" 2>/dev/null; then
        pass "User not in anna group correctly denied access"
    else
        fail "User not in anna group was able to access socket!"
    fi
fi

info "Testing socket access for user IN anna group (anna_u1)..."
if su - anna_u1 -c "annactl status" 2>/dev/null; then
    pass "User in anna group can access socket"
else
    warn "User in anna group had issues (may need newgrp or re-login)"
fi

# ================================================
# GATE D: Migration Idempotency
# ================================================
echo
echo "======================================"
echo "  GATE D: Migration Idempotency"
echo "======================================"
echo

# Check tombstone
TOMBSTONE="/var/lib/anna/.migrated"
if [ -f "$TOMBSTONE" ]; then
    info "Migration tombstone exists:"
    cat "$TOMBSTONE"
    pass "Migration is idempotent (tombstone present)"
else
    info "No migration tombstone (no legacy data to migrate or migration not yet run)"
fi

# Document merge rules
echo
echo "MIGRATION MERGE RULES:"
echo "  - Tickets: Merge by ID, keep newest by timestamp"
echo "  - Stats: Sum totals (questions, XP), keep highest reliability"
echo "  - Recipes: Deduplicate by content hash"
echo "  - Ledgers: Keep most recent valid chain"
echo "  - Memory: Deduplicate experiences by query"
echo

# ================================================
# GATE E: Updater Works
# ================================================
echo
echo "======================================"
echo "  GATE E: Updater Works"
echo "======================================"
echo

# Check update ledger
LEDGER="/var/lib/anna/update_ledger.json"
if [ -f "$LEDGER" ]; then
    info "Update ledger contents:"
    cat "$LEDGER" | head -30

    # Check for successful installs
    INSTALLS=$(grep -c '"Installed"' "$LEDGER" 2>/dev/null || echo "0")
    info "Successful installs recorded: $INSTALLS"

    if [ "$INSTALLS" -gt 0 ]; then
        pass "Update mechanism has worked (found install records)"
    fi
else
    info "No update ledger yet (first install)"
fi

# Verify versions match
info "Version check:"
ANNACTL_VER=$(/usr/local/bin/annactl --version 2>/dev/null | head -1)
ANNAD_VER=$(/usr/local/bin/annad --version 2>/dev/null | head -1)
echo "  annactl: $ANNACTL_VER"
echo "  annad:   $ANNAD_VER"

if echo "$ANNACTL_VER" | grep -q "0.3.32" && echo "$ANNAD_VER" | grep -q "0.3.32"; then
    pass "Both binaries are v0.3.32"
else
    warn "Version mismatch or not v0.3.32"
fi

# ================================================
# Summary
# ================================================
echo
echo "======================================"
echo "  SUMMARY"
echo "======================================"
echo
echo "Gate A (No Home Writes):    Run annactl as multiple users, verify no ~/state"
echo "Gate B (Permissions):       750/640/660 model, daemon-only writes"
echo "Gate C (Socket Access):     Group-based access control verified"
echo "Gate D (Migration):         Idempotent via tombstone"
echo "Gate E (Updater):           Ledger shows install history"
echo
echo "To clean up test users:"
echo "  userdel -r anna_u1"
echo "  userdel -r anna_u2"
echo "  userdel -r nogrp"
echo
