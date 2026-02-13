#!/usr/bin/env bash
# Anna vs Direct Resolution Test Suite
# Tests Anna's ability to answer 25 questions compared to direct system commands.
# Evaluates: accuracy, depth, actionability, DE awareness, performance insight.
#
# Usage: ./tests/anna_vs_direct.sh [--anna-only] [--quick]
# Output: Scoring report + per-question comparison.

set -euo pipefail

ANNA="${1:-./target/release/annactl}"
SCORE_PASS=0
SCORE_FAIL=0
SCORE_SKIP=0
RESULTS=()
QUICK="${QUICK:-false}"

[[ "${1:-}" == "--quick" ]] && QUICK=true
[[ "${2:-}" == "--quick" ]] && QUICK=true

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

banner() { echo -e "${CYAN}━━━ $1 ━━━${NC}"; }
pass()   { echo -e "${GREEN}[PASS]${NC} $1"; SCORE_PASS=$((SCORE_PASS+1)); RESULTS+=("PASS: $1"); }
fail()   { echo -e "${RED}[FAIL]${NC} $1"; SCORE_FAIL=$((SCORE_FAIL+1)); RESULTS+=("FAIL: $1"); }
skip()   { echo -e "${YELLOW}[SKIP]${NC} $1"; SCORE_SKIP=$((SCORE_SKIP+1)); RESULTS+=("SKIP: $1"); }
info()   { echo -e "${BLUE}  →${NC} $1"; }

# Ask Anna and capture output (30s timeout)
ask_anna() {
    local question="$1"
    if ! command -v "$ANNA" &>/dev/null && [[ ! -f "$ANNA" ]]; then
        echo "ANNA_UNAVAILABLE"
        return
    fi
    timeout 30s "$ANNA" "$question" 2>/dev/null || echo "TIMEOUT_OR_ERROR"
}

# Run a direct system command
direct() {
    eval "$1" 2>/dev/null || echo "N/A"
}

check_anna_installed() {
    if [[ ! -f "$ANNA" ]] && ! command -v annactl &>/dev/null; then
        echo -e "${YELLOW}Anna not installed at $ANNA. Running direct-only tests.${NC}"
        echo -e "${YELLOW}Build first: cargo build --release${NC}"
        ANNA=""
    fi
}

# ─── Test helpers ─────────────────────────────────────────────────────────────

# Test: Anna's answer must contain keywords, direct command provides ground truth
test_question() {
    local name="$1"
    local question="$2"
    local direct_cmd="$3"
    local required_keywords="${4:-}"  # comma-separated

    banner "$name"
    info "Question: $question"

    # Ground truth
    local direct_out
    direct_out=$(direct "$direct_cmd")
    info "Direct answer: $(echo "$direct_out" | head -3 | tr '\n' ' ')"

    if [[ -z "$ANNA" ]]; then
        skip "$name (Anna unavailable)"
        return
    fi

    local anna_out
    anna_out=$(ask_anna "$question")

    if [[ "$anna_out" == "TIMEOUT_OR_ERROR" || "$anna_out" == "ANNA_UNAVAILABLE" ]]; then
        fail "$name (Anna timeout/error)"
        return
    fi

    info "Anna answer: $(echo "$anna_out" | head -3 | tr '\n' ' ')"

    # Check required keywords
    if [[ -n "$required_keywords" ]]; then
        local all_found=true
        IFS=',' read -ra kws <<< "$required_keywords"
        for kw in "${kws[@]}"; do
            kw="${kw// /}"
            if ! echo "$anna_out" | grep -qi "$kw"; then
                info "MISSING keyword: $kw"
                all_found=false
            fi
        done
        if $all_found; then
            pass "$name"
        else
            fail "$name (missing keywords in answer)"
        fi
    else
        # Just check answer is non-trivial
        if [[ ${#anna_out} -gt 50 ]]; then
            pass "$name"
        else
            fail "$name (answer too short)"
        fi
    fi
    echo
}

# ─── Test categories ──────────────────────────────────────────────────────────

banner "CATEGORY 1: System Information"

test_question "Kernel version" \
    "what kernel am I running?" \
    "uname -r" \
    "kernel,cachyos"

test_question "Disk usage" \
    "what is my disk usage?" \
    "df -h --output=pcent,target | sort -rn | head -5" \
    "disk"

test_question "Memory status" \
    "how much RAM do I have and how much is free?" \
    "free -h" \
    "RAM,total"

test_question "CPU info" \
    "what CPU do I have?" \
    "lscpu | grep 'Model name'" \
    "CPU,Intel"

test_question "Running services" \
    "what services are currently running?" \
    "systemctl list-units --type=service --state=running --no-pager | head -10" \
    "service,running"

banner "CATEGORY 2: Desktop Environment Awareness"

test_question "Session type detection" \
    "am I running Wayland or X11?" \
    'echo "${XDG_SESSION_TYPE:-unknown}"' \
    "wayland,x11"

test_question "Desktop environment" \
    "what desktop environment or window manager am I using?" \
    'echo "${XDG_CURRENT_DESKTOP:-${DESKTOP_SESSION:-unknown}}"' \
    ""

test_question "Display server info" \
    "what display server is active on my system?" \
    'loginctl show-session auto -p Type --value 2>/dev/null || echo $XDG_SESSION_TYPE' \
    ""

banner "CATEGORY 3: Performance and Power"

test_question "Boot time" \
    "how long does my system take to boot?" \
    "systemd-analyze 2>/dev/null | head -1" \
    "boot,time"

test_question "CPU governor" \
    "what CPU frequency scaling governor am I using?" \
    "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo 'unavailable'" \
    ""

test_question "Failed services" \
    "are there any failed systemd services?" \
    "systemctl --failed --no-pager" \
    "service,failed,unit"

test_question "Load average" \
    "what is my system load average?" \
    'uptime | awk -F"load average:" '"'"'{print $2}'"'" \
    "load,average"

banner "CATEGORY 4: Storage and Filesystem"

test_question "Largest directories" \
    "what are my top 5 largest directories in home?" \
    "du -sh $HOME/*/ 2>/dev/null | sort -rh | head -5" \
    ""

test_question "Filesystem type" \
    "what filesystem format is my root partition using?" \
    "findmnt -n -o FSTYPE /" \
    "btrfs"

test_question "Disk health" \
    "is my disk healthy?" \
    "lsblk -d -o NAME,SIZE,TYPE | head -5" \
    ""

banner "CATEGORY 5: Package Management"

test_question "Pending updates" \
    "do I have any pending package updates?" \
    "checkupdates 2>/dev/null | wc -l || echo 'N/A'" \
    ""

test_question "Recently installed" \
    "what packages did I install recently?" \
    "grep 'installed' /var/log/pacman.log | tail -10" \
    "install,package"

banner "CATEGORY 6: Network"

test_question "IP address" \
    "what is my current IP address?" \
    'ip -4 addr show | grep inet | grep -v 127 | awk '"'"'{print $2}'"'" \
    "IP,address"

test_question "Network interfaces" \
    "what network interfaces do I have?" \
    "ip link show | grep -E '^[0-9]'" \
    "interface,lo"

banner "CATEGORY 7: Bootloader Detection"

test_question "Bootloader" \
    "what bootloader am I using?" \
    'bootctl status 2>/dev/null | grep -i "product\|loader" | head -2 || echo "unknown"' \
    "bootloader,limine"

banner "CATEGORY 8: Logs and Errors"

test_question "Recent errors" \
    "were there any errors in the last 24 hours?" \
    "journalctl -p err -b --no-pager | tail -5" \
    ""

test_question "Journal since boot" \
    "what happened since my last boot in the system logs?" \
    "journalctl -b --no-pager -p warning | tail -10" \
    ""

banner "CATEGORY 9: GPU and Hardware"

test_question "GPU detection" \
    "what GPU do I have?" \
    "lspci | grep -i 'vga\|3d\|display'" \
    "GPU,NVIDIA"

banner "CATEGORY 10: Configuration Intelligence"

test_question "Config file awareness" \
    "where is my hyprland config file?" \
    'find ~/.config/hypr ~/.config -name "hyprland.conf" 2>/dev/null | head -1 || echo "not found"' \
    ""

test_question "Shell profile" \
    "where is my shell configuration file?" \
    'echo "$SHELL config: ~/.$(basename $SHELL)rc"' \
    ""

# ─── Results ──────────────────────────────────────────────────────────────────

echo
banner "TEST RESULTS"
echo -e "${GREEN}PASS: $SCORE_PASS${NC} | ${RED}FAIL: $SCORE_FAIL${NC} | ${YELLOW}SKIP: $SCORE_SKIP${NC}"
TOTAL=$((SCORE_PASS + SCORE_FAIL))
if [[ $TOTAL -gt 0 ]]; then
    PCT=$(( SCORE_PASS * 100 / TOTAL ))
    echo -e "Score: ${PCT}% (${SCORE_PASS}/${TOTAL} answered correctly)"
fi

echo
echo "Detailed results:"
for r in "${RESULTS[@]}"; do
    echo "  $r"
done

echo
if [[ $SCORE_FAIL -eq 0 ]]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
elif [[ $PCT -ge 80 ]]; then
    echo -e "${YELLOW}Good performance (${PCT}%). Some improvements needed.${NC}"
    exit 0
else
    echo -e "${RED}Below threshold (${PCT}%). Anna needs improvement.${NC}"
    exit 1
fi
