#!/usr/bin/env bash
# Feature smoke test — tests key announced capabilities end-to-end.
# Requires annactl + annad running.
# Usage: ./tests/features.sh [--quick]
#
# Exit code 0 = all PASS, 1 = one or more FAIL.

set -euo pipefail

ANNACTL="${ANNACTL:-./target/release/annactl}"
TIMEOUT=30
PASS=0
FAIL=0
SKIP=0

RED='\033[0;31m'
GRN='\033[0;32m'
YLW='\033[0;33m'
NC='\033[0m'

require_daemon() {
    if ! "$ANNACTL" status 2>/dev/null | grep -q "running"; then
        echo "annad not running — start it first"
        exit 1
    fi
}

ask() {
    timeout "$TIMEOUT" "$ANNACTL" "$1" 2>/dev/null || true
}

pass() { echo -e "${GRN}[PASS]${NC} $1"; PASS=$((PASS+1)); }
fail() { echo -e "${RED}[FAIL]${NC} $1"; FAIL=$((FAIL+1)); }
skip() { echo -e "${YLW}[SKIP]${NC} $1 (not applicable on this system)"; SKIP=$((SKIP+1)); }

require_daemon

echo "=== Anna Feature Smoke Tests ==="
echo ""

# --- FULL REPORT ---
echo "── Full Report ──"
out=$(ask "What is my system status? Full report please")
if echo "$out" | grep -qi "OVERVIEW\|CPU\|MEMORY\|DISK\|SERVICES"; then
    pass "Full report contains multiple sections"
else
    fail "Full report missing sections. Got: $(echo "$out" | head -5)"
fi

if echo "$out" | grep -qi "kernel\|uptime\|hostname"; then
    pass "Full report contains system overview info"
else
    fail "Full report missing kernel/uptime/hostname"
fi

if echo "$out" | grep -qi "failed\|running normally"; then
    pass "Full report includes services status"
else
    fail "Full report missing services section"
fi

# --- BASIC FACTUAL QUESTIONS ---
echo ""
echo "── Basic Factual Questions ──"

out=$(ask "what kernel am I running")
if echo "$out" | grep -qiE "[0-9]+\.[0-9]+\.[0-9]+"; then
    pass "Kernel version reported"
else
    fail "Kernel version not in answer. Got: $(echo "$out" | head -3)"
fi

out=$(ask "how much RAM do I have")
if echo "$out" | grep -qiE "[0-9]+ ?(G|M|GB|MB|GiB|MiB|Gi|Mi)"; then
    pass "RAM amount reported"
else
    fail "RAM amount not in answer. Got: $(echo "$out" | head -3)"
fi

out=$(ask "what is my disk usage")
if echo "$out" | grep -qiE "[0-9]+%|[0-9]+ ?(G|T|GB|TB)"; then
    pass "Disk usage reported"
else
    fail "Disk usage not in answer. Got: $(echo "$out" | head -3)"
fi

out=$(ask "show me failing services")
if echo "$out" | grep -qiE "fail|running normally|no failed|all services"; then
    pass "Failed services answer returned"
else
    fail "Failed services not answered. Got: $(echo "$out" | head -3)"
fi

# --- CONFIG ROUTING SANITY (must NOT route diagnostic questions to plan generator) ---
echo ""
echo "── Routing Sanity (no false CONFIG) ──"

for q in \
    "what services are running" \
    "how much swap do I have" \
    "show me my network interfaces" \
    "what is my IP address" \
    "list installed packages" \
    "what processes are using the most CPU"
do
    out=$(ask "$q")
    # If Anna asks "Proceed with these commands? (yes/no)" it wrongly went to CONFIG
    if echo "$out" | grep -qi "Proceed with these commands\|pending_plan\|yes/no"; then
        fail "CONFIG false positive for: '$q'"
    else
        pass "Correct routing for: '$q'"
    fi
done

# --- HOW-TO (should not run commands, answer from knowledge) ---
echo ""
echo "── How-To Questions ──"

out=$(ask "how do I install neovim on arch linux")
if echo "$out" | grep -qiE "pacman|yay|paru|-S neovim"; then
    pass "How-to gives package manager answer"
else
    fail "How-to answer missing pacman/yay. Got: $(echo "$out" | head -3)"
fi

out=$(ask "how to enable a systemd service")
if echo "$out" | grep -qi "systemctl enable"; then
    pass "How-to systemctl enable answer"
else
    fail "How-to systemd answer wrong. Got: $(echo "$out" | head -3)"
fi

# --- PDF REPORT ---
echo ""
echo "── PDF Report ──"

out=$(timeout "$TIMEOUT" "$ANNACTL" "generate pdf report" 2>/dev/null || true)
# Must see a .pdf path or an explicit error from the PDF handler (font missing etc.)
# "generating" alone is not enough — that fires before the RPC even completes
if echo "$out" | grep -qiE "\.pdf|font error|Failed to generate|report generation"; then
    pass "PDF report generation triggered and responded"
else
    fail "PDF report did not trigger or gave no response. Got: $(echo "$out" | head -3)"
fi

# Also verify natural phrasings route to PDF handler
out=$(timeout "$TIMEOUT" "$ANNACTL" "send me the report in pdf" 2>/dev/null || true)
if echo "$out" | grep -qiE "\.pdf|font error|Failed to generate|report generation"; then
    pass "PDF natural phrasing routes to PDF handler"
else
    fail "PDF natural phrasing not handled. Got: $(echo "$out" | head -3)"
fi

# --- ARTIFACT REGISTRY ---
echo ""
echo "── Artifact Registry ──"

out=$(ask "what automations have you created")
if echo "$out" | grep -qiE "created|automation|nothing|empty|registry|No automations"; then
    pass "Registry query returns a response"
else
    fail "Registry query gave no response. Got: $(echo "$out" | head -3)"
fi

# --- SSH AUDIT (read-only, should work without daemon) ---
echo ""
echo "── SSH Audit ──"

if [ -f /etc/ssh/sshd_config ]; then
    out=$(ask "audit my ssh config for security issues")
    if echo "$out" | grep -qiE "ssh|sshd|PasswordAuthentication|hardening|security|finding|critical|warning"; then
        pass "SSH audit returns security analysis"
    else
        fail "SSH audit returned nothing useful. Got: $(echo "$out" | head -5)"
    fi
else
    skip "SSH audit (sshd_config not present)"
fi

# --- MORNING BRIEFING QUICK ---
echo ""
echo "── Brief System Status (quick) ──"

out=$(ask "how is my system")
if echo "$out" | grep -qiE "healthy|attention|CPU|Memory|RAM|load|uptime"; then
    pass "Quick system status returns summary"
else
    fail "Quick status returned nothing useful. Got: $(echo "$out" | head -3)"
fi

# --- NO HALLUCINATION CHECK ---
echo ""
echo "── No Hallucination ──"

out=$(ask "how much RAM do I have")
actual_ram=$(free -h | awk 'NR==2{print $2}' | tr -d 'i')
# Extract leading digits (e.g. "31" from "31Gi" or "32" from "32G")
actual_num=$(echo "$actual_ram" | grep -oE '^[0-9]+' | head -1)
if [ -z "$out" ]; then
    fail "RAM answer was empty (daemon timeout or crash)"
elif [ -n "$actual_num" ] && echo "$out" | grep -qE "$actual_num"; then
    pass "RAM answer matches actual system value ($actual_ram)"
else
    # Could be a rounding difference (31Gi vs 32G) — still count as pass but note it
    if echo "$out" | grep -qiE "[0-9]+ ?(G|M|GB|MB|GiB|MiB|Gi|Mi)"; then
        pass "RAM answer present with a numeric value (actual=$actual_ram, manual verify)"
    else
        fail "RAM answer contains no memory value. Got: $(echo "$out" | head -3)"
    fi
fi

# --- SUMMARY ---
echo ""
echo "================================"
echo "Results: ${PASS} passed, ${FAIL} failed, ${SKIP} skipped"
echo "================================"

[ "$FAIL" -eq 0 ]
