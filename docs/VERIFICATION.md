# Anna Verification Guide

This document provides exact commands to verify Anna v0.0.16+ is working correctly.

## Prerequisites

```bash
# Ensure anna group membership
groups | grep -q anna || sudo usermod -aG anna $USER && newgrp anna

# Verify daemon is running
systemctl is-active annad
```

## 1. Release Asset Verification

Verify release assets exist before deployment:

```bash
VERSION="0.0.16"
BASE="https://github.com/jjgarcianorway/anna-assistant/releases/download/v${VERSION}"

# Check assets exist (should return 302 or 200)
curl -sI "${BASE}/annad-linux-x86_64" | head -1
curl -sI "${BASE}/annactl-linux-x86_64" | head -1
curl -sI "${BASE}/SHA256SUMS" | head -1
```

Expected: `HTTP/2 302` (GitHub redirect) or `HTTP/2 200`

## 2. Binary Verification

```bash
# Version matches
annactl -V | grep -E "^annactl 0\.0\.[0-9]+$"

# Binary exists and is executable
test -x /usr/local/bin/annactl && echo "OK: annactl executable"
test -x /usr/local/bin/annad && echo "OK: annad executable"
```

## 3. Status Command Verification

```bash
# Basic status
annactl status

# Debug status with latency stats
annactl status --debug
```

Expected output includes:
- daemon: Ready (pid NNN)
- debug_mode: ON or OFF
- version: 0.0.16
- llm: Ready
- models: translator/specialist/supervisor roles

With `--debug`:
- Latency Stats section with translator/probes/specialist/total avg/p95

## 4. Deterministic Output Verification

### 4.1 CPU Query
```bash
annactl "what cpu do i have"
```
Expected: Shows CPU model, core count from hardware snapshot.

### 4.2 RAM Query
```bash
annactl "how much ram"
```
Expected: Shows RAM in GB.

### 4.3 Top Memory Processes
```bash
annactl "top memory processes"
```
Expected output must contain:
- Header: `PID`, `COMMAND`, `%MEM`, `RSS`, `USER`
- At least 5 process rows
- RSS in human format (e.g., `12M`, `1.2G`)

Verify with:
```bash
annactl "top memory" 2>&1 | grep -E "^[0-9]+" | head -5
# Should show 5 lines starting with PID numbers
```

### 4.4 Disk Usage
```bash
annactl "disk space"
```
Expected output must contain:
- Mount points with usage percentages
- CRITICAL (>= 95%) or WARNING (>= 85%) labels when applicable

Verify critical/warning detection:
```bash
annactl "disk space" 2>&1 | grep -E "(CRITICAL|WARNING|Filesystem)"
```

### 4.5 Network Interfaces
```bash
annactl "network interfaces"
```
Expected output must contain:
- Active interface at top (e.g., "Active: Wi-Fi (wlan0) 192.168.x.x")
- Interface type detection (WiFi/Ethernet/Loopback)

Verify active first:
```bash
annactl "network" 2>&1 | head -5
# First substantive line should show "Active:"
```

## 5. Timeout Verification

Global request timeout (20s default):
```bash
# In config.toml, request_timeout_secs controls this
grep request_timeout_secs /etc/anna/config.toml
```

## 6. Test Suite Verification

```bash
cd /path/to/anna-assistant
cargo test --release 2>&1 | tail -20
```

Expected: All tests pass (151+ tests as of v0.0.16)

Key test files:
- `deterministic_tests.rs`: PID column, CRITICAL warnings, state display
- `router_tests.rs`: Domain routing, scoring validation
- `triage_tests.rs`: Confidence thresholds, probe caps

## 7. Latency Stats Verification

After running a few queries:
```bash
annactl status --debug | grep -A10 "Latency Stats"
```

Expected:
- translator: avg Nms, p95 Nms
- probes: avg Nms, p95 Nms
- specialist: avg Nms, p95 Nms
- total: avg Nms, p95 Nms
- samples: N

## 8. Debug Mode Toggle

Check current mode:
```bash
annactl status | grep debug_mode
```

Toggle via config:
```bash
# /etc/anna/config.toml
[daemon]
debug_mode = true  # or false
```

Restart daemon after config change:
```bash
sudo systemctl restart annad
```

## 9. Scoring Verification

Deterministic answers should show:
- `grounded=true` only when parsed data count > 0
- Reliability score reflects actual probe success

```bash
annactl "top memory" 2>&1 | grep -i reliability
# Should show score > 60 for successful deterministic answers
```

## 10. REPL Mode Verification

```bash
annactl
# Enter REPL
# Type: help
# Type: exit (or quit, bye, q, :q, :wq)
```

Expected:
- Spinner while waiting (debug OFF)
- Stage transitions shown (debug ON)
- Clean exit on Ctrl-D

## Quick Smoke Test

```bash
# All-in-one verification
annactl status && \
annactl "what cpu" && \
annactl "top memory" 2>&1 | grep -q PID && echo "PID column: OK" && \
annactl "disk space" 2>&1 | grep -qE "(Filesystem|CRITICAL|WARNING)" && echo "Disk status: OK" && \
annactl "network" 2>&1 | head -5 | grep -qi "active\|interface" && echo "Network: OK"
```

## Troubleshooting

### Daemon not running
```bash
sudo systemctl status annad
sudo journalctl -u annad -n 50
```

### Socket permission denied
```bash
ls -la /run/anna/anna.sock
# Should be: srwxrwx--- 1 root anna
sudo usermod -aG anna $USER && newgrp anna
```

### LLM not ready
```bash
annactl status | grep llm
# If bootstrapping, wait for model pull
```

## Version History

| Version | Key Changes |
|---------|-------------|
| 0.0.16  | Global timeout, latency stats, PID in process output |
| 0.0.15  | Triage router, probe summarizer, evidence redaction |
| 0.0.14  | Deterministic router, help command |
| 0.0.13  | Per-stage model selection, config timeouts |
| 0.0.12  | Deterministic answerer fallback |
