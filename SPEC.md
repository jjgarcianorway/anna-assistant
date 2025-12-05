# Anna Specification v0.0.18

This document is the authoritative specification for Anna. All implementation
must conform to this spec. If code and spec conflict, update spec first, then code.

## Overview

Anna is a local AI assistant for Linux systems. It consists of two components:
- **annad**: A root-level systemd service that manages system state, Ollama, and models
- **annactl**: A user-facing CLI that communicates with annad over a Unix socket

## Architecture

```
┌─────────────┐     Unix Socket     ┌─────────────┐
│   annactl   │ ◄─────────────────► │    annad    │
│  (user CLI) │    JSON-RPC 2.0     │  (root svc) │
└─────────────┘                     └─────────────┘
                                          │
                                          ▼
                                   ┌─────────────┐
                                   │   Ollama    │
                                   │  (managed)  │
                                   └─────────────┘
```

## Component Specifications

### annad (Daemon)

**Runs as**: root (systemd service)
**Socket**: `/run/anna/anna.sock`
**Config**: `/etc/anna/config.toml`
**State directory**: `/var/lib/anna/`
**Log**: systemd journal

**Responsibilities**:
1. Install and manage Ollama
2. Probe hardware (CPU, RAM, GPU) and select appropriate model
3. Pull and manage models based on hardware capabilities
4. Maintain installation ledger at `/var/lib/anna/ledger.json`
5. Provide RPC interface for annactl
6. Run update check every 60 seconds, auto-update if enabled
7. Self-healing: repair permissions, restart services, re-pull models
8. Run system probes for grounded LLM responses
9. Execute service desk pipeline for all requests
10. Deterministic fallback when LLM times out
11. Track per-stage latency statistics (v0.0.16)

### annactl (CLI)

**Runs as**: Current user
**Connects to**: `/run/anna/anna.sock`

**Commands** (locked CLI surface - no additions allowed):
- `annactl <request>` - Send request to Anna
- `annactl` (no args) - Enter REPL mode
- `annactl status` - Show system status
- `annactl status --debug` - Show status with latency stats (v0.0.16)
- `annactl reset` - Reset learned data
- `annactl uninstall` - Trigger safe uninstall
- `annactl -V` / `annactl --version` - Show version

## Configuration (v0.0.13+)

`/etc/anna/config.toml`:
```toml
[daemon]
debug_mode = true
auto_update = true
update_interval = 600

[llm]
provider = "ollama"
translator_model = "qwen2.5:1.5b-instruct"
specialist_model = "qwen2.5:7b-instruct"
supervisor_model = "qwen2.5:1.5b-instruct"
translator_timeout_secs = 4
specialist_timeout_secs = 12
supervisor_timeout_secs = 6
probe_timeout_secs = 4
request_timeout_secs = 20  # v0.0.16: Global request timeout
```

## Service Desk Architecture

### Internal Roles

1. **Translator** (LLM-based): Converts user text to structured JSON ticket
2. **Dispatcher**: Routes to appropriate specialist, runs probes
3. **Specialist**: Domain expert (System/Network/Storage/Security/Packages)
4. **Supervisor**: Quality control with deterministic scoring
5. **Deterministic Answerer**: Fallback when LLM unavailable

### Pipeline Flow

```
User Query
    │
    ▼
┌─────────────────┐
│  Deterministic  │──── Known query? ────► Direct Answer
│     Router      │     (help, cpu, ram)
└────────┬────────┘
         │ Unknown
         ▼
┌─────────────────┐
│   Translator    │──── Confidence < 0.7 ────► Clarification
│    (LLM)        │
└────────┬────────┘
         │ High confidence
         ▼
┌─────────────────┐
│   Dispatcher    │──► Run probes (max 3)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Specialist    │──── Timeout? ────► Deterministic Fallback
│    (LLM)        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Supervisor    │──► Score reliability
└─────────────────┘
```

### Deterministic Router (v0.0.14+)

Overrides LLM for known query classes:

| Pattern | Action | Domain |
|---------|--------|--------|
| "help" | Return help text | - |
| cpu/processor | Hardware snapshot | System |
| ram/memory amount | Hardware snapshot | System |
| gpu/graphics | Hardware snapshot | System |
| "top memory/processes" | Run top_memory probe | System |
| "top cpu" | Run top_cpu probe | System |
| disk/storage/space | Run disk_usage probe | Storage |
| network/interface/ip | Run network_addrs probe | Network |
| slow/sluggish | Multi-probe diagnostic | System |

### Triage Rules (v0.0.15+)

- **Confidence threshold**: < 0.7 triggers clarification
- **Probe cap**: Maximum 3 probes per query
- **Clarification max reliability**: Capped at 40%

### Deterministic Answerer

**Supported Query Types**:

| Query Type | Data Sources | Output Format |
|------------|--------------|---------------|
| CpuInfo | Hardware snapshot | Model, cores |
| RamInfo | Hardware snapshot | Total GB |
| GpuInfo | Hardware snapshot | Model, VRAM |
| TopMemoryProcesses | ps aux --sort=-%mem | PID, COMMAND, %MEM, RSS, USER (10 rows) |
| DiskSpace | df -h | Mount, usage %, CRITICAL/WARNING flags |
| NetworkInterfaces | ip addr show | Active first, type detection |

**Output Requirements** (v0.0.16):
- Process tables include PID column
- RSS formatted human-readable (12M, 1.2G)
- Disk shows CRITICAL (>=95%) and WARNING (>=85%)
- Network shows active interface first with type (WiFi/Ethernet)

### Reliability Scoring

| Signal | Condition | Points |
|--------|-----------|--------|
| translator_confident | confidence >= 0.7 AND no timeout | 20 |
| probe_coverage | all requested probes succeeded | 20 |
| answer_grounded | deterministic OR answer references data | 20 |
| no_invention | deterministic OR no hedging words | 20 |
| clarification_not_needed | answer is not empty | 20 |

**Scoring Rules**:
- `grounded=true` only if parsed data count > 0
- Empty parser result = clarification needed
- Coverage based on actual probe success

### Timeout Handling

| Stage | Timeout | On Timeout |
|-------|---------|------------|
| Global request | 20s (configurable) | Graceful timeout response |
| Translator | 4s | Use deterministic router |
| Probe (each) | 4s | Skip, mark as failed |
| Specialist | 12s | Try deterministic answerer |
| Supervisor | 6s | Deterministic scoring |

### Probe Allowlist

| Probe ID | Command |
|----------|---------|
| top_memory | `ps aux --sort=-%mem` |
| top_cpu | `ps aux --sort=-%cpu` |
| cpu_info | `lscpu` |
| memory_info | `free -h` |
| disk_usage | `df -h` |
| block_devices | `lsblk` |
| network_addrs | `ip addr show` |
| network_routes | `ip route` |
| listening_ports | `ss -tulpn` |
| failed_services | `systemctl --failed` |
| system_logs | `journalctl -p warning..alert -n 200 --no-pager` |

### Evidence Redaction (v0.0.15+)

Automatic removal of sensitive patterns:
- Private keys (BEGIN...PRIVATE KEY)
- Password hashes (/etc/shadow format)
- AWS keys (AKIA...)
- API tokens (Bearer, api_key, etc.)

Applied even in debug mode for security.

## Latency Statistics (v0.0.16)

Per-stage latency tracking for last 20 requests:

- **translator**: LLM translation time
- **probes**: Total probe execution time
- **specialist**: LLM specialist time
- **total**: End-to-end request time

Exposed via `annactl status --debug`:
```
Latency Stats (last 20 requests):
translator      avg 120ms, p95 250ms
probes          avg 80ms, p95 150ms
specialist      avg 1200ms, p95 2500ms
total           avg 1500ms, p95 3000ms
samples         20
```

## Display Modes

**debug OFF** (fly-on-the-wall):
```
anna v0.0.16
──────────────────────────────────────
[you]
what cpu do i have?

[anna]
AMD Ryzen 7 5800X (8 cores)

reliability 100% | domain system
──────────────────────────────────────
```

**debug ON** (troubleshooting):
```
[anna->translator] starting (timeout 4s) [0.0s]
[anna] translator complete [0.12s]
[anna->probe] running top_memory [0.12s]
[anna] probe top_memory ok (45ms) [0.17s]
...
```

## Constraints

1. **400-line limit**: No source file may exceed 400 lines
2. **CLI surface locked**: Only listed commands allowed
3. **LLM optional for answers**: Deterministic fallback when LLM unavailable
4. **Grounding mandatory**: All responses must be grounded in data
5. **No invented facts**: Never claim facts not in context
6. **Probe allowlist**: Only read-only commands allowed
7. **Max 3 probes**: Per query limit enforced

## Version

- Version: 0.0.18
- Status: UX hardening - fixed duplicate output, CLI help handling, deterministic stage display
