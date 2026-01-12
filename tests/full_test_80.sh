#!/bin/bash
# Full test of 80 questions for Anna v0.2.3

ANNACTL="./target/release/annactl"
RESULTS_DIR="tests/results_v023_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

questions=(
"How much RAM do I have and how much is currently free"
"Show detailed memory usage including buffers and cache"
"What CPU model am I running and how many cores are available"
"Is CPU frequency scaling enabled and which governor is active"
"Show current system load and explain if it is healthy"
"What kernel version am I running and when was it released"
"Check if my kernel is out of date compared to Arch stable"
"List all mounted filesystems and their usage"
"Which filesystem types am I using and why"
"Check disk health using SMART and summarize the results"
"How much free space do I have on my root partition"
"Detect which GPU I am using and which driver is loaded"
"Is hardware acceleration working for my GPU"
"Show current Wayland or X11 session details"
"Which display manager is running and how it was started"
"Show all active systemd services and highlight failures"
"Why did my last boot take so long"
"Analyze boot time and list the slowest units"
"Is my system clock synchronized correctly"
"Show NTP status and recent time drift"
"Check battery health and current charge cycles"
"Estimate remaining battery lifespan based on usage"
"Show current power profile and recommendations"
"Detect laptop lid close behavior and suspend configuration"
"Which network interfaces are available and which is active"
"Test current network connectivity and latency"
"Diagnose slow internet connection causes"
"Show DNS configuration and detect misconfiguration"
"Is NetworkManager working correctly"
"Check WiFi signal strength and driver stability"
"List all installed packages sorted by size"
"Is package linux-firmware installed and which version"
"Check for partially upgraded packages"
"Detect orphaned packages and explain risks"
"Remove orphaned packages safely"
"Update my system fully and report changes"
"Simulate a full system upgrade without applying it"
"Check pacman database integrity"
"Clear pacman cache and show reclaimed space"
"Which AUR helper is installed if any"
"Is yay installed and which version"
"Build and install an AUR package safely"
"Check for failed AUR builds in the past"
"Show recent system errors from journalctl"
"Explain the most critical error from last boot"
"Detect recurring warnings in system logs"
"Is my system affected by known Arch security advisories"
"Check if secure boot is enabled"
"Verify microcode updates for my CPU"
"Detect filesystem errors and recommend actions"
"Check swap usage and swappiness value"
"Should I be using zram on this system"
"Detect thermal throttling and overheating"
"Show fan control status and temperatures"
"Is my audio stack using PipeWire or PulseAudio"
"Diagnose no sound output issue"
"List all audio devices and profiles"
"Check Bluetooth service health and adapters"
"Why does Bluetooth fail after suspend"
"Show USB devices and power states"
"Detect problematic USB devices"
"Is my printer correctly configured"
"Show CUPS status and queued jobs"
"Check firewall status and active rules"
"Is ufw or firewalld installed and running"
"Detect open listening ports"
"Check SSH service status and security settings"
"Audit my system for basic hardening issues"
"Check sudo configuration for safety problems"
"Is my home directory encrypted"
"Verify backup configuration and last backup time"
"Detect missing backups and warn me"
"Analyze disk IO performance"
"Is my SSD using optimal mount options"
"Check TRIM status and schedule"
"Detect laptop power drain causes"
"Compare idle power usage against expected baseline"
"Show running processes sorted by CPU usage"
"Identify memory leaks or runaway processes"
"Kill a frozen application safely"
"Restart a crashed systemd service"
"Explain why my system froze earlier today"
"Generate a full system health report suitable for my boss"
)

total=${#questions[@]}
echo "Running $total questions..."
echo "Results will be saved to $RESULTS_DIR"
echo ""

passed=0
failed=0
timeout_count=0

for i in "${!questions[@]}"; do
    q="${questions[$i]}"
    num=$((i + 1))
    echo -n "[$num/$total] Testing: ${q:0:50}... "

    # Run with 45 second timeout
    output=$(timeout 45 $ANNACTL "$q" 2>&1)
    exit_code=$?

    # Save output
    safe_name=$(echo "$q" | tr ' ' '_' | tr -cd '[:alnum:]_' | cut -c1-40)
    echo "$output" > "$RESULTS_DIR/q${num}_${safe_name}.txt"

    if [ $exit_code -eq 124 ]; then
        echo "TIMEOUT"
        ((timeout_count++))
        ((failed++))
    elif echo "$output" | grep -q "ANSWER:"; then
        echo "OK"
        ((passed++))
    elif echo "$output" | grep -q "Anna:"; then
        echo "OK"
        ((passed++))
    else
        echo "FAILED"
        ((failed++))
    fi
done

echo ""
echo "================================"
echo "RESULTS SUMMARY"
echo "================================"
echo "Total:    $total"
echo "Passed:   $passed"
echo "Failed:   $failed"
echo "Timeouts: $timeout_count"
echo "Success:  $(( passed * 100 / total ))%"
echo ""
echo "Results saved to: $RESULTS_DIR"
