#!/bin/bash
# 20 Crazy Questions - Multi-step, Complex Scenarios
# Tests Anna's ability to handle real user problems

ANNACTL="./target/release/annactl"

# Check if annactl exists
if [[ ! -x "$ANNACTL" ]]; then
    echo "Error: annactl not found at $ANNACTL"
    exit 1
fi

# Timeout for each question (seconds)
TIMEOUT=120

# Questions - Complex, multi-step, real user scenarios
QUESTIONS=(
    # Hardware awareness questions (should use profile)
    "My WiFi keeps disconnecting randomly, what could be wrong and how do I check the logs?"
    "I have an NVIDIA GPU and the screen tears when watching videos, what are my options?"
    "My laptop fan runs at full speed all the time even when idle, how do I diagnose this?"
    "How do I check if my SSD is healthy and what SMART values should I worry about?"

    # Multi-step troubleshooting
    "Pacman says database is locked, what happened and how do I safely fix it?"
    "My system takes 2 minutes to boot, how do I find out what's slowing it down?"
    "Audio stopped working after an update, how do I troubleshoot pipewire vs pulseaudio?"
    "systemd-journald is using 2GB of disk space, how do I manage this properly?"

    # Configuration challenges
    "I want to run docker containers as my regular user, what do I need to set up?"
    "How do I set up automatic login on TTY but keep GDM on Wayland for graphical sessions?"
    "I need to mount a CIFS/SMB share at boot with credentials, whats the secure way?"
    "How do I make my USB DAC the default audio output device and persist it across reboots?"

    # Advanced scenarios
    "I want to dual boot Windows but keep my existing Arch install, what are the steps?"
    "How do I set up wireguard VPN that starts automatically and routes all traffic?"
    "I need to run Steam games on my AMD GPU while my NVIDIA handles displays, is that possible?"
    "How do I create a btrfs snapshot before every pacman upgrade automatically?"

    # System administration
    "I want to limit how much RAM and CPU a specific user can use, how do I do that with cgroups?"
    "How do I set up SSH with key-only auth, custom port, and fail2ban protection?"
    "My /var is almost full, how do I identify whats using space and safely clean it?"
    "I accidentally rm -rf'd my /usr/lib folder, what are my recovery options?"
)

# Evaluator - uses brief heuristics to judge response quality
evaluate_response() {
    local question="$1"
    local response="$2"
    local timeout_flag="$3"

    # Check for timeout
    if [[ "$timeout_flag" == "TIMEOUT" ]]; then
        echo "FAIL:TIMEOUT"
        return
    fi

    # Check for empty response
    if [[ -z "$response" || "$response" == *"I don't"* || "$response" == *"I cannot"* ]]; then
        echo "FAIL:EMPTY_OR_REFUSAL"
        return
    fi

    # Check response length (complex questions need detailed answers)
    local word_count=$(echo "$response" | wc -w)
    if [[ $word_count -lt 30 ]]; then
        echo "WEAK:TOO_SHORT"
        return
    fi

    # Check for actionable content (commands, steps, paths)
    local has_commands=false
    local has_paths=false
    local has_steps=false

    if echo "$response" | grep -qE '`[^`]+`|^\$|^#'; then
        has_commands=true
    fi

    if echo "$response" | grep -qE '/[a-z]+/|/etc/|/var/|/usr/'; then
        has_paths=true
    fi

    if echo "$response" | grep -qEi '\b(first|then|next|step|1\.|2\.|3\.)\b'; then
        has_steps=true
    fi

    # Score based on content
    local score=0
    $has_commands && ((score++))
    $has_paths && ((score++))
    $has_steps && ((score++))

    if [[ $score -ge 2 ]]; then
        echo "GOOD:DETAILED"
    elif [[ $score -eq 1 ]]; then
        echo "OK:PARTIAL"
    else
        echo "WEAK:GENERIC"
    fi
}

# Run tests
echo "=========================================="
echo "  CRAZY QUESTIONS TEST - 20 Complex Ones"
echo "=========================================="
echo ""

PASS=0
WEAK=0
FAIL=0

for i in "${!QUESTIONS[@]}"; do
    q="${QUESTIONS[$i]}"
    num=$((i + 1))

    echo "[$num/20] $q"
    echo "---"

    # Run with timeout
    response=$(timeout $TIMEOUT $ANNACTL "$q" 2>&1)
    exit_code=$?

    if [[ $exit_code -eq 124 ]]; then
        result=$(evaluate_response "$q" "" "TIMEOUT")
    else
        result=$(evaluate_response "$q" "$response" "")
    fi

    status="${result%%:*}"
    detail="${result##*:}"

    # Print first 200 chars of response
    preview=$(echo "$response" | head -c 200 | tr '\n' ' ')
    echo "Response: ${preview}..."
    echo "Result: $result"
    echo ""

    case "$status" in
        GOOD|OK)
            ((PASS++))
            ;;
        WEAK)
            ((WEAK++))
            ;;
        FAIL)
            ((FAIL++))
            ;;
    esac
done

echo "=========================================="
echo "  RESULTS"
echo "=========================================="
echo "PASS (Good/OK): $PASS/20"
echo "WEAK: $WEAK/20"
echo "FAIL: $FAIL/20"
echo ""
echo "Success Rate: $(( (PASS * 100) / 20 ))%"
echo "=========================================="
