# Root Cause Correlation Matrix
**Anna Assistant - Proactive Engine**

**Purpose**: Complete mapping of telemetry signals to root-cause diagnoses

This document defines every correlation rule used by the proactive engine to deduce root causes from multiple telemetry signals.

---

## Matrix Structure

Each rule follows this pattern:
```
RULE: [Name]
SIGNALS: List of required/optional signals
LOGIC: Boolean expression combining signals
OUTPUT: RootCause enum variant
SEVERITY: Critical | Warning | Info
CONFIDENCE: 0-100 (based on signal strength)
REMEDIATION: Commands to fix root cause
```

---

## Network Root Causes

### Rule: NET-001 - Routing Conflict Detection

**Signals Required**:
- Brain insight: `duplicate_default_routes`
- Network monitoring: Multiple interfaces with default gateway

**Signals Optional** (increase confidence):
- Systemd journal: Connection timeout errors
- Network monitoring: Packet loss > 5%
- Network monitoring: DNS resolution failures

**Logic**:
```
IF:
  duplicate_default_routes.present
  AND default_route_count >= 2
THEN:
  RootCause = NetworkRoutingConflict
```

**Severity Calculation**:
- Critical: IF packet_loss > 10% OR connection_failures > 5/min
- Warning: OTHERWISE

**Confidence**:
- Base: 80%
- +10% if packet loss present
- +10% if connection timeouts present

**Remediation**:
```bash
# View current routing table
ip route

# Identify interfaces with default routes
ip route | grep default

# Remove duplicate (replace <gateway> and <interface>)
sudo ip route del default via <gateway> dev <interface>

# Restart NetworkManager to rebuild routes
sudo systemctl restart NetworkManager

# Verify single default route
ip route | grep default
```

---

### Rule: NET-002 - Priority Mismatch Correlation

**Signals Required**:
- Brain insight: `network_priority_mismatch`
- Network monitoring: Slow interface has default route
- Network monitoring: Fast interface exists but unused

**Logic**:
```
IF:
  network_priority_mismatch.present
  AND slow_interface.has_default_route
  AND fast_interface.speed > slow_interface.speed * 2
THEN:
  RootCause = NetworkPriorityMismatch
```

**Severity**: Warning (impacts performance, not availability)

**Confidence**:
- Base: 90% (brain already confirmed mismatch)

**Remediation**:
```bash
# Check current network configuration
nmcli connection show

# Option 1: Disconnect slower interface
nmcli connection down <slow_interface>

# Option 2: Adjust route metrics (lower = higher priority)
nmcli connection modify <fast_interface> ipv4.route-metric 100
nmcli connection modify <slow_interface> ipv4.route-metric 200

# Verify WiFi is now default route
ip route
```

---

### Rule: NET-003 - Quality Degradation

**Signals Required**:
- Network monitoring: packet_loss > 5%
  OR latency > 200ms
  OR interface errors > 100/min

**Signals Optional**:
- Brain insight: `high_packet_loss` or `high_latency`
- Systemd journal: Network-related errors

**Logic**:
```
IF:
  (packet_loss > 5% OR latency > 200ms OR errors > 100/min)
  AND NOT duplicate_default_routes
  AND NOT network_priority_mismatch
THEN:
  RootCause = NetworkQualityDegradation
```

**Severity Calculation**:
- Critical: packet_loss > 20% OR latency > 500ms
- Warning: OTHERWISE

**Confidence**:
- packet_loss only: 70%
- latency only: 60%
- errors only: 50%
- Multiple signals: 90%

**Remediation**:
```bash
# Test packet loss to gateway
ping -c 20 $(ip route | grep default | awk '{print $3}')

# Test packet loss to internet
ping -c 20 1.1.1.1

# Check WiFi signal strength
nmcli device wifi

# Check interface statistics
ip -s link show

# Restart NetworkManager
sudo systemctl restart NetworkManager
```

---

## Disk Root Causes

### Rule: DISK-001 - Disk Pressure Detection

**Signals Required**:
- Brain insight: `disk_space_critical` OR `disk_space_warning`

**Signals Optional**:
- Filesystem: inode usage > 90%
- Historian: Disk usage trending up
- Systemd journal: "No space left on device" errors

**Logic**:
```
IF:
  (disk_space_critical OR disk_space_warning)
  AND usage_percent >= 85%
THEN:
  RootCause = DiskPressure
```

**Severity Calculation**:
- Critical: usage >= 95% OR inodes >= 95% OR ENOSPC errors present
- Warning: usage >= 85% OR inodes >= 85%

**Confidence**:
- Disk usage only: 90%
- +5% if inode exhaustion
- +5% if ENOSPC errors

**Inode Exhaustion Detection**:
```rust
if inode_usage > 90% && disk_usage < 90% {
    // Many small files (common in /var/log, /tmp)
    inode_exhaustion = true
}
```

**Remediation**:
```bash
# Check disk usage
df -h <mountpoint>

# Check inode usage
df -i <mountpoint>

# Find large directories
du -h <mountpoint> | sort -h | tail -20

# Clean package cache (Arch Linux)
sudo pacman -Sc

# Rotate logs
sudo journalctl --vacuum-size=100M

# Find and remove large files
find <mountpoint> -type f -size +100M -exec ls -lh {} \;
```

---

### Rule: DISK-002 - Log Growth Correlation

**Signals Required**:
- Disk usage increasing (> 1GB/hour)
- `/var/log` size increasing

**Signals Optional**:
- Systemd journal: Service logging excessively
- Historian: Disk usage spike after specific event

**Logic**:
```
IF:
  disk_usage_increasing
  AND var_log_growth > 100_MB_per_hour
  AND specific_service.log_rate > 10_MB_per_hour
THEN:
  RootCause = DiskLogGrowth { service, growth_rate }
```

**Severity**: Warning (predictable, preventable)

**Confidence**:
- Base: 80%
- +10% if specific service identified
- +10% if log rate is extreme (>1GB/hour)

**Remediation**:
```bash
# Find large log files
du -sh /var/log/*

# Check log growth rate
journalctl --disk-usage

# Rotate logs immediately
sudo journalctl --vacuum-size=100M

# Reduce log verbosity for specific service
sudo systemctl edit <service>
# Add: Environment="LOG_LEVEL=WARNING"

# Configure log rotation
sudo vim /etc/systemd/journald.conf
# Set: SystemMaxUse=500M
```

---

## Service Root Causes

### Rule: SVC-001 - Service Flapping Detection

**Signals Required**:
- Systemd journal: Service restarted >= 3 times in 1 hour

**Signals Optional**:
- Systemd journal: Service exit codes
- Brain insight: `degraded_services`
- Historian: Service restart history

**Logic**:
```
IF:
  service.restart_count >= 3 within 1_hour
  AND service.current_state == "running"
THEN:
  RootCause = ServiceFlapping { service, restart_count }
```

**Severity**: Warning (service recovers automatically, but unstable)

**Confidence**:
- 3 restarts: 80%
- 5+ restarts: 95%
- +5% if exit codes indicate crash (not clean shutdown)

**Remediation**:
```bash
# Check service status
systemctl status <service>

# View recent logs
journalctl -u <service> -n 100

# Check for dependency failures
systemctl list-dependencies <service>

# Review configuration
systemctl cat <service>

# Check for resource limits
systemctl show <service> | grep -E 'Memory|CPU|Limit'

# Disable auto-restart temporarily to debug
sudo systemctl edit <service>
# Add:
# [Service]
# Restart=no
```

---

### Rule: SVC-002 - Service Under Load

**Signals Required**:
- Process monitoring: Service CPU > 80% OR memory growth trend

**Signals Optional**:
- Systemd journal: Service responding slowly
- Network monitoring: Service connection queue building

**Logic**:
```
IF:
  (service.cpu_percent > 80% OR service.memory_growth_rate > 10_MB_per_min)
  AND service.current_state == "running"
THEN:
  RootCause = ServiceUnderLoad { service, cpu, memory }
```

**Severity Calculation**:
- Critical: CPU > 95% OR memory > 90% of limit
- Warning: OTHERWISE

**Confidence**:
- CPU spike only: 70%
- Memory growth only: 75%
- Both: 90%
- +10% if connection queue present

**Remediation**:
```bash
# Identify resource usage
top -p $(pgrep <service>)

# Check memory map
pmap -x $(pgrep <service>)

# Review service limits
systemctl show <service> | grep -E 'Memory|CPU|Tasks'

# Increase limits if needed
sudo systemctl edit <service>
# Add:
# [Service]
# MemoryMax=2G
# CPUQuota=200%

# Scale service (if supports multiple instances)
sudo systemctl start <service>@2

# Restart service as temporary relief
sudo systemctl restart <service>
```

---

### Rule: SVC-003 - Service Configuration Error

**Signals Required**:
- Brain insight: `failed_services`
- Systemd journal: Exit code 1, 2, or 78 (config error)

**Signals Optional**:
- Systemd journal: Specific error message mentioning "config"
- Historian: Recent package update or manual config change

**Logic**:
```
IF:
  service.state == "failed"
  AND service.exit_code IN [1, 2, 78]
  AND (error_message.contains("config") OR error_message.contains("syntax"))
  AND NOT dependency_failed
THEN:
  RootCause = ServiceConfigError { service, error }
```

**Severity**: Critical (service non-functional)

**Confidence**:
- Exit code + error message: 95%
- Exit code only: 75%
- +5% if recent config change in historian

**Remediation**:
```bash
# Check service status
systemctl status <service>

# View full logs
journalctl -u <service> -n 50 --no-pager

# Validate configuration (if service supports it)
<service> -t  # Example: nginx -t

# Compare with backup config
diff /etc/<service>/<config> /etc/<service>/<config>.bak

# Review recent changes
sudo systemctl cat <service>
git log /etc/<service>/  # If using etckeeper

# Restore working configuration
sudo cp /etc/<service>/<config>.bak /etc/<service>/<config>
sudo systemctl restart <service>
```

---

## Resource Root Causes

### Rule: RES-001 - Memory Pressure Correlation

**Signals Required**:
- Brain insight: `memory_pressure_critical` OR `memory_pressure_warning`

**Signals Optional**:
- System monitoring: Swap usage > 50%
- Systemd journal: OOM killer events
- Process monitoring: Specific process with memory leak

**Logic**:
```
IF:
  (memory_pressure_critical OR memory_pressure_warning)
  AND ram_usage_percent >= 85%
THEN:
  RootCause = MemoryPressure { ram_percent, swap_percent }
```

**Severity Calculation**:
- Critical: ram >= 95% OR OOM events OR swap >= 90%
- Warning: ram >= 85% OR swap >= 50%

**Confidence**:
- RAM usage only: 85%
- +5% if swap usage high
- +10% if OOM events present

**Remediation**:
```bash
# Check memory usage
free -h

# Check swap usage
swapon --show

# Identify memory hogs
ps aux --sort=-%mem | head -10

# Check for memory leaks
watch -n 5 'ps aux | grep <process>'

# Add swap if none exists
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Kill runaway process (LAST RESORT)
sudo kill -9 <pid>
```

---

### Rule: RES-002 - CPU Overload Correlation

**Signals Required**:
- Brain insight: `cpu_overload_critical` OR `cpu_high_load`
- System monitoring: load_per_core >= 2.0

**Signals Optional**:
- Process monitoring: Specific process > 80% CPU
- Historian: CPU load trend increasing

**Logic**:
```
IF:
  (cpu_overload_critical OR cpu_high_load)
  AND load_per_core >= 2.0
THEN:
  RootCause = CpuOverload { load_per_core, runaway_process }
```

**Severity Calculation**:
- Critical: load_per_core >= 3.0
- Warning: load_per_core >= 2.0

**Confidence**:
- Load only: 80%
- +15% if specific runaway process identified
- +5% if sustained for >5 minutes

**Remediation**:
```bash
# Check current load
uptime

# Identify CPU hogs
top -o %CPU

# Check per-process CPU usage
ps aux --sort=-%cpu | head -10

# Nice down CPU-intensive process
sudo renice -n 10 -p <pid>

# Limit CPU usage for service
sudo systemctl set-property <service> CPUQuota=50%

# Kill runaway process (if identified)
sudo kill <pid>
```

---

## System Root Causes

### Rule: SYS-001 - Kernel Regression Detection

**Signals Required**:
- Historian: Recent kernel package update
- Systemd journal: Boot errors increased after update

**Signals Optional**:
- Systemd journal: Driver failure messages
- Hardware monitoring: Device initialization failures

**Logic**:
```
IF:
  recent_kernel_update within 7_days
  AND boot_error_count_after > boot_error_count_before * 2
THEN:
  RootCause = KernelRegression { boot_errors, driver_failures }
```

**Severity**: Critical (system stability impacted)

**Confidence**:
- Kernel update + boot errors: 85%
- +10% if specific driver failures
- +5% if hardware errors

**Remediation**:
```bash
# List installed kernels
ls /boot/vmlinuz-*

# Boot previous kernel
# (Reboot and select from GRUB menu)

# Check boot errors
journalctl -b -p err

# Check dmesg for driver failures
dmesg | grep -i error

# Downgrade kernel
sudo pacman -U /var/cache/pacman/pkg/linux-<previous-version>.pkg.tar.zst

# Report bug
uname -r > kernel_version.txt
journalctl -b -p err > boot_errors.log
# Submit to kernel bugzilla or Arch bug tracker
```

---

### Rule: SYS-002 - Device Hotplug Correlation

**Signals Required**:
- Systemd journal: USB/PCI device add/remove event

**Signals Optional**:
- Network monitoring: Interface added/removed
- Systemd journal: Routing table changed after event

**Logic**:
```
IF:
  device_hotplug_event within 5_minutes
  AND (network_interface_changed OR routing_changed)
THEN:
  RootCause = DeviceHotplug { added, removed }
```

**Severity**: Info (context, not necessarily problem)

**Confidence**: 100% (direct causation)

**Remediation**: (Contextual explanation, not fix)
```
Your network routing changed because:
- Device connected: <device_name>
- Interface added: <interface_name>
- Default route updated: <new_route>

This is expected behavior when plugging in USB Ethernet adapters
or docking stations.

To prevent routing changes:
# Disable auto-configuration for specific device
nmcli connection modify <connection> ipv4.never-default true
```

---

## Correlation Conflict Resolution

When multiple root causes could explain the same symptoms:

### Priority Order:
1. **Direct causation** (hotplug → routing change)
2. **Configuration errors** (service failed with config exit code)
3. **Resource exhaustion** (OOM → service crashed)
4. **Quality degradation** (packet loss → slow response)
5. **Trend projections** (escalating load → future failure)

### Example Conflict:
```
Symptom: Service crashed
Possible causes:
  - ServiceConfigError (exit code 78)
  - MemoryPressure (OOM event at same time)

Resolution:
  IF exit_code == 78 AND error_message.contains("config"):
    RootCause = ServiceConfigError  (direct evidence)
  ELSE IF oom_event.timestamp within 10_seconds:
    RootCause = MemoryPressure      (temporal proximity)
```

---

## Signal Weighting

Signals have different reliability and should be weighted:

### High Weight (0.9-1.0):
- Brain insights (already analyzed and validated)
- Exit codes (direct evidence)
- OOM killer events (kernel-level truth)
- Hardware errors (device-level truth)

### Medium Weight (0.6-0.8):
- Systemd journal errors (may be transient)
- Network statistics (subject to momentary spikes)
- Process CPU/memory (can fluctuate)

### Low Weight (0.3-0.5):
- Trend projections (predictive, not current)
- User reports (subjective)
- Heuristics (pattern matching)

### Confidence Calculation:
```rust
confidence = (
    signal1.weight * signal1.strength +
    signal2.weight * signal2.strength +
    ...
) / total_signals

Clamped to [0, 100]
```

---

## Temporal Correlation

Some root causes require temporal sequence analysis:

### Rule: Service Crash After Package Update
```
IF:
  package_updated.timestamp < service_crashed.timestamp < package_updated.timestamp + 1_hour
THEN:
  RootCause likely = PackageRegression
  Confidence += 20%
```

### Rule: Disk Full After Log Spike
```
IF:
  log_growth_detected.timestamp < disk_full.timestamp
  AND log_size_increase >= disk_space_decrease
THEN:
  RootCause = DiskLogGrowth
  Confidence = 95%
```

### Rule: Network Issue After Device Hotplug
```
IF:
  device_connected.timestamp < routing_conflict.timestamp < device_connected.timestamp + 5_min
THEN:
  RootCause = DeviceHotplug (context)
  Confidence = 100%
```

---

## Deduplication Rules

Multiple signals may report the same root cause:

### Example: Disk Pressure
```
Signals:
  - Brain: disk_space_critical on /var
  - Monitoring: /var usage 96%
  - Journal: "No space left on device"

Deduplication:
  Create single CorrelatedIssue with:
    - root_cause = DiskPressure { mountpoint: "/var", usage: 96% }
    - contributing_signals = [brain, monitoring, journal]
    - confidence = 100% (multiple confirming signals)
```

### Deduplication Logic:
```rust
fn deduplicate(issues: Vec<CorrelatedIssue>) -> Vec<CorrelatedIssue> {
    let mut deduplicated = Vec::new();

    for issue in issues {
        if let Some(existing) = find_matching_root_cause(&issue, &deduplicated) {
            // Merge signals
            existing.contributing_signals.extend(issue.contributing_signals);
            // Update confidence (weighted average)
            existing.confidence = max(existing.confidence, issue.confidence);
            // Update timestamps
            existing.last_seen = max(existing.last_seen, issue.last_seen);
        } else {
            deduplicated.push(issue);
        }
    }

    deduplicated
}
```

---

## Extension Points

Future correlation rules to add:

### Beta.273+:
- **Package Regression Detection**: Service failure after package update
- **Configuration Drift Detection**: Config differs from known-good state
- **Security Event Correlation**: Failed SSH + suspicious processes
- **Performance Regression**: Response time degraded after change
- **Backup Failure Correlation**: Disk full + backup service failed

### Long-term:
- **Machine Learning Anomaly Detection** (still deterministic, trained offline)
- **Multi-node Correlation** (collective health)
- **User Behavior Pattern Analysis** (detect unusual activity)

---

**End of Correlation Matrix**
