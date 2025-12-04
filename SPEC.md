# Anna Specification v0.0.12

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
10. **Deterministic fallback**: Answer from data when LLM times out (v0.0.12)

### annactl (CLI)

**Runs as**: Current user
**Connects to**: `/run/anna/anna.sock`

**Commands** (locked CLI surface - no additions allowed):
- `annactl <request>` - Send request to Anna
- `annactl` (no args) - Enter REPL mode
- `annactl status` - Show system status
- `annactl reset` - Reset learned data
- `annactl uninstall` - Trigger safe uninstall
- `annactl -V` / `annactl --version` - Show version

## Service Desk Architecture (v0.0.12)

### Internal Roles

1. **Translator** (LLM-based): Converts user text to structured JSON ticket
2. **Dispatcher**: Routes to appropriate specialist, runs probes
3. **Specialist**: Domain expert (System/Network/Storage/Security/Packages)
4. **Supervisor**: Quality control with deterministic scoring
5. **Deterministic Answerer** (NEW in v0.0.12): Fallback when LLM unavailable

### Deterministic Answerer (v0.0.12)

When specialist LLM times out or errors, the deterministic answerer can produce
answers for common queries by parsing hardware snapshots and probe outputs.

**Supported Query Types**:

| Query Type | Data Sources | Example Queries |
|------------|--------------|-----------------|
| CpuInfo | Hardware snapshot, lscpu | "what cpu do i have?" |
| RamInfo | Hardware snapshot, free -h | "how much ram do i have?" |
| GpuInfo | Hardware snapshot | "what gpu do i have?" |
| TopMemoryProcesses | ps aux --sort=-%mem | "top 5 memory hogs" |
| DiskSpace | df -h | "how much disk space is free?" |
| NetworkInterfaces | ip addr show | "what are my network interfaces?" |

**Rules**:
- Never invent facts. If parsing fails, say what was missing.
- Must produce a clean final answer, not ask for clarification.
- Deterministic answers are always grounded and never invent.

### Reliability Scoring (v0.0.12)

The reliability score is calculated from 5 boolean signals:

| Signal | Condition | Points |
|--------|-----------|--------|
| translator_confident | confidence >= 0.7 AND no timeout | 20 |
| probe_coverage | all requested probes succeeded | 20 |
| answer_grounded | deterministic OR answer references data | 20 |
| no_invention | deterministic OR no hedging words | 20 |
| clarification_not_needed | answer is not empty | 20 |

**Key Change in v0.0.12**: Deterministic answers automatically get `answer_grounded=true`
and `no_invention=true` because they parse real data.

**Scoring Examples**:
- LLM success, all probes: 100
- Translator timeout + deterministic + probes: 80 (no translator_confident)
- Full timeout, no data: 20 (only no_invention)

### Timeout Handling (v0.0.12)

| Stage | Timeout | On Timeout |
|-------|---------|------------|
| Translator | 8s | Use keyword fallback routing |
| Probe (each) | 4s | Skip, mark as failed |
| Probes (total) | 10s | Return timeout response |
| Specialist | 12s | **Try deterministic answerer first** |
| Supervisor | 8s | Deterministic scoring |

**Critical Change**: Specialist timeout no longer returns clarification. Instead:
1. Try deterministic answerer with available context
2. Only if deterministic fails AND no probes succeeded: ask clarification

### Domain Consistency (v0.0.12)

The `ServiceDeskResult.domain` now always reflects the classified domain from
the translator (or fallback routing), not a default value. This ensures the
domain shown in summary matches the actual routing decision.

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

### Response Format

Every response includes:
- `request_id`: Unique request ID for tracking
- `answer`: The response text (LLM or deterministic)
- `reliability_score`: 0-100 deterministic score
- `reliability_signals`: The 5 boolean signals
- `domain`: Which specialist domain (matches routing)
- `evidence`: Hardware fields, probes, translator ticket
- `needs_clarification`: Only true if no answer possible
- `clarification_question`: Question if clarification needed
- `transcript`: Full pipeline event transcript

### Transcript Events (v0.0.11+)

Pipeline events for debugging/visibility:
- **StageStart/StageEnd**: Track stage timings
- **ProbeStart/ProbeEnd**: Individual probe execution
- **Message**: Actor-to-actor communication
- **Note**: Internal annotations

**Render Modes**:
- debug OFF: Human-readable format showing key messages
- debug ON: Full troubleshooting view with timings

## Constraints

1. **400-line limit**: No source file may exceed 400 lines
2. **CLI surface locked**: Only listed commands allowed
3. **LLM optional for answers**: Deterministic fallback when LLM unavailable
4. **Grounding mandatory**: All responses must be grounded in data
5. **No invented facts**: Never claim facts not in context
6. **Probe allowlist**: Only read-only commands allowed

## Version

- Version: 0.0.12
- Status: Deterministic fallback answerer, domain consistency, improved scoring
